//! Minimal Language Server Protocol implementation for LIPI — pure Rust.
//!
//! `lipi lsp` speaks LSP over stdio (Content-Length framed JSON-RPC). It provides:
//!   - initialize / shutdown / exit lifecycle
//!   - textDocument/didOpen + didChange → publishDiagnostics (parse errors)
//!   - textDocument/hover     → keyword / builtin descriptions
//!   - textDocument/completion→ LIPI keywords + builtins
//!   - textDocument/documentSymbol → विधि / वर्ग definitions
//!
//! A tiny self-contained JSON value type is used so no external crate is needed.

use std::collections::HashMap;
use std::io::{Read, Write};

// ── Minimal JSON ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    fn get(&self, key: &str) -> Option<&J> {
        if let J::Obj(m) = self { m.iter().find(|(k, _)| k == key).map(|(_, v)| v) } else { None }
    }
    fn as_str(&self) -> Option<&str> { if let J::Str(s) = self { Some(s) } else { None } }
    fn as_num(&self) -> Option<f64> { if let J::Num(n) = self { Some(*n) } else { None } }

    fn write(&self, out: &mut String) {
        match self {
            J::Null => out.push_str("null"),
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            J::Num(n) => { if n.fract() == 0.0 { out.push_str(&format!("{}", *n as i64)); } else { out.push_str(&format!("{n}")); } }
            J::Str(s) => write_json_str(s, out),
            J::Arr(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() { if i > 0 { out.push(','); } v.write(out); }
                out.push(']');
            }
            J::Obj(m) => {
                out.push('{');
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    write_json_str(k, out); out.push(':'); v.write(out);
                }
                out.push('}');
            }
        }
    }
    fn to_string(&self) -> String { let mut s = String::new(); self.write(&mut s); s }
}

fn write_json_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

struct JParser { chars: Vec<char>, pos: usize }
impl JParser {
    fn new(s: &str) -> Self { JParser { chars: s.chars().collect(), pos: 0 } }
    fn ws(&mut self) { while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() { self.pos += 1; } }
    fn parse(&mut self) -> Result<J, String> {
        self.ws();
        if self.pos >= self.chars.len() { return Err("JSON: समय से पहले अंत".into()); }
        match self.chars[self.pos] {
            '{' => self.obj(),
            '[' => self.arr(),
            '"' => Ok(J::Str(self.string()?)),
            't' => { self.lit("true")?; Ok(J::Bool(true)) }
            'f' => { self.lit("false")?; Ok(J::Bool(false)) }
            'n' => { self.lit("null")?; Ok(J::Null) }
            _ => self.num(),
        }
    }
    fn lit(&mut self, s: &str) -> Result<(), String> {
        for c in s.chars() {
            if self.pos >= self.chars.len() || self.chars[self.pos] != c { return Err(format!("JSON: '{s}' अपेक्षित")); }
            self.pos += 1;
        }
        Ok(())
    }
    fn num(&mut self) -> Result<J, String> {
        let start = self.pos;
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_digit() || "-+.eE".contains(self.chars[self.pos])) { self.pos += 1; }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().map(J::Num).map_err(|_| "JSON: अमान्य संख्या".into())
    }
    fn string(&mut self) -> Result<String, String> {
        self.pos += 1; // opening quote
        let mut s = String::new();
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos]; self.pos += 1;
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let e = self.chars.get(self.pos).copied().unwrap_or('"'); self.pos += 1;
                    match e {
                        '"' => s.push('"'), '\\' => s.push('\\'), '/' => s.push('/'),
                        'n' => s.push('\n'), 'r' => s.push('\r'), 't' => s.push('\t'),
                        'b' => s.push('\u{08}'), 'f' => s.push('\u{0c}'),
                        'u' => {
                            let hex: String = self.chars[self.pos..(self.pos + 4).min(self.chars.len())].iter().collect();
                            self.pos += 4;
                            if let Ok(n) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(n) { s.push(ch); }
                            }
                        }
                        o => s.push(o),
                    }
                }
                c => s.push(c),
            }
        }
        Err("JSON: अधूरा वाक्य".into())
    }
    fn arr(&mut self) -> Result<J, String> {
        self.pos += 1; let mut v = Vec::new(); self.ws();
        if self.pos < self.chars.len() && self.chars[self.pos] == ']' { self.pos += 1; return Ok(J::Arr(v)); }
        loop {
            v.push(self.parse()?); self.ws();
            match self.chars.get(self.pos) { Some(',') => { self.pos += 1; } Some(']') => { self.pos += 1; break; } _ => return Err("JSON: ',' या ']' अपेक्षित".into()) }
        }
        Ok(J::Arr(v))
    }
    fn obj(&mut self) -> Result<J, String> {
        self.pos += 1; let mut m = Vec::new(); self.ws();
        if self.pos < self.chars.len() && self.chars[self.pos] == '}' { self.pos += 1; return Ok(J::Obj(m)); }
        loop {
            self.ws();
            let k = self.string()?; self.ws();
            if self.chars.get(self.pos) != Some(&':') { return Err("JSON: ':' अपेक्षित".into()); }
            self.pos += 1;
            let val = self.parse()?; m.push((k, val)); self.ws();
            match self.chars.get(self.pos) { Some(',') => { self.pos += 1; } Some('}') => { self.pos += 1; break; } _ => return Err("JSON: ',' या '}' अपेक्षित".into()) }
        }
        Ok(J::Obj(m))
    }
}

// ── LSP server ───────────────────────────────────────────────────────────────

const KEYWORDS: &[(&str, &str)] = &[
    ("बताओ", "मान को नई पंक्ति के साथ छापें (print)"),
    ("लिखो", "मान को बिना नई पंक्ति के छापें (print inline)"),
    ("है", "असाइनमेंट: नाम है मान"),
    ("विधि", "फलन परिभाषा (function): विधि नाम(पैरामीटर):"),
    ("फल", "फलन से मान लौटाएँ (return)"),
    ("उत्पन्न", "जनरेटर से मान yield करें"),
    ("यदि", "शर्त (if): यदि स्थिति:"),
    ("अन्यथा", "अन्यथा (else)"),
    ("के", "लूप: के लिए … में"),
    ("जब", "जब तक (while loop)"),
    ("बार", "N बार करो — दोहराव लूप"),
    ("वर्ग", "क्लास परिभाषा (class)"),
    ("कोशिश", "त्रुटि प्रबंधन (try)"),
    ("पकड़ो", "त्रुटि पकड़ें (catch)"),
    ("फेंको", "त्रुटि फेंकें (throw)"),
    ("आयात", "मॉड्यूल या फ़ाइल आयात करें (import)"),
    ("सत्य", "बूलियन सत्य (true)"),
    ("असत्य", "बूलियन असत्य (false)"),
    ("और", "तार्किक AND"),
    ("या", "तार्किक OR"),
    ("नहीं", "तार्किक NOT"),
    ("लाम्डा", "अनाम फलन (lambda)"),
    ("जाँचो", "assert — स्थिति सत्य होनी चाहिए"),
    ("स्थिर", "अपरिवर्तनीय स्थिरांक (const)"),
];

const BUILTINS: &[&str] = &[
    "लम्बाई", "पूर्णांक", "वाक्य", "पढ़ो", "यादृच्छिक", "निरपेक्ष", "घात", "वर्गमूल", "गोल",
    "मानचित्र", "छानो", "मोड़ो", "प्रकार", "स्वरूप", "यूआईडी", "युग्म", "गणना", "श्रृंखला",
    "स्मरण", "आंशिक", "संयोजित", "सामान्यीकृत", "पूर्ण_है", "क्रमित_कोश",
];

fn read_message(stdin: &mut impl Read) -> Option<String> {
    // read headers
    let mut header = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        if stdin.read(&mut byte).ok()? == 0 { return None; }
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") { break; }
    }
    let header_str = String::from_utf8_lossy(&header);
    let mut len = 0usize;
    for line in header_str.lines() {
        if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
            len = v.trim().parse().ok()?;
        }
    }
    let mut buf = vec![0u8; len];
    stdin.read_exact(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn send(out: &mut impl Write, msg: &J) {
    let body = msg.to_string();
    let _ = write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = out.flush();
}

fn response(id: J, result: J) -> J {
    J::Obj(vec![("jsonrpc".into(), J::Str("2.0".into())), ("id".into(), id), ("result".into(), result)])
}

fn notification(method: &str, params: J) -> J {
    J::Obj(vec![("jsonrpc".into(), J::Str("2.0".into())), ("method".into(), J::Str(method.into())), ("params".into(), params)])
}

/// Compute diagnostics for a document by running the parser; report parse errors.
fn diagnostics(text: &str) -> Vec<J> {
    let tokens = crate::lexer::tokenize(text);
    match crate::parser::parse(tokens) {
        Ok(_) => Vec::new(),
        Err(e) => {
            // extract "(line N)" if present
            let mut line = 0i64;
            if let Some(p) = e.find("(line ") {
                let rest = &e[p + 6..];
                if let Some(end) = rest.find(')') { line = rest[..end].trim().parse().unwrap_or(1); }
            }
            let lnum = (line.max(1) - 1) as f64;
            let range = J::Obj(vec![
                ("start".into(), J::Obj(vec![("line".into(), J::Num(lnum)), ("character".into(), J::Num(0.0))])),
                ("end".into(), J::Obj(vec![("line".into(), J::Num(lnum)), ("character".into(), J::Num(200.0))])),
            ]);
            vec![J::Obj(vec![
                ("range".into(), range),
                ("severity".into(), J::Num(1.0)), // Error
                ("source".into(), J::Str("lipi".into())),
                ("message".into(), J::Str(e)),
            ])]
        }
    }
}

fn publish_diagnostics(out: &mut impl Write, uri: &str, text: &str) {
    let diags = diagnostics(text);
    let params = J::Obj(vec![
        ("uri".into(), J::Str(uri.to_string())),
        ("diagnostics".into(), J::Arr(diags)),
    ]);
    send(out, &notification("textDocument/publishDiagnostics", params));
}

pub fn run() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    // document store: uri → text
    let mut docs: HashMap<String, String> = HashMap::new();

    while let Some(raw) = read_message(&mut stdin) {
        let msg = match JParser::new(&raw).parse() { Ok(m) => m, Err(_) => continue };
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(J::Null);

        match method.as_str() {
            "initialize" => {
                let caps = J::Obj(vec![
                    ("textDocumentSync".into(), J::Num(1.0)), // full sync
                    ("hoverProvider".into(), J::Bool(true)),
                    ("completionProvider".into(), J::Obj(vec![("resolveProvider".into(), J::Bool(false))])),
                    ("documentSymbolProvider".into(), J::Bool(true)),
                ]);
                let result = J::Obj(vec![
                    ("capabilities".into(), caps),
                    ("serverInfo".into(), J::Obj(vec![("name".into(), J::Str("lipi-lsp".into())), ("version".into(), J::Str("0.1.0".into()))])),
                ]);
                if let Some(id) = id { send(&mut stdout, &response(id, result)); }
            }
            "initialized" => {}
            "textDocument/didOpen" => {
                if let Some(td) = params.get("textDocument") {
                    let uri = td.get("uri").and_then(|u| u.as_str()).unwrap_or("").to_string();
                    let text = td.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    docs.insert(uri.clone(), text.clone());
                    publish_diagnostics(&mut stdout, &uri, &text);
                }
            }
            "textDocument/didChange" => {
                let uri = params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()).unwrap_or("").to_string();
                if let Some(J::Arr(changes)) = params.get("contentChanges") {
                    if let Some(last) = changes.last() {
                        if let Some(text) = last.get("text").and_then(|t| t.as_str()) {
                            docs.insert(uri.clone(), text.to_string());
                            publish_diagnostics(&mut stdout, &uri, text);
                        }
                    }
                }
            }
            "textDocument/hover" => {
                let result = hover(&params, &docs).unwrap_or(J::Null);
                if let Some(id) = id { send(&mut stdout, &response(id, result)); }
            }
            "textDocument/completion" => {
                let mut items = Vec::new();
                for (kw, doc) in KEYWORDS {
                    items.push(J::Obj(vec![
                        ("label".into(), J::Str(kw.to_string())),
                        ("kind".into(), J::Num(14.0)), // Keyword
                        ("detail".into(), J::Str(doc.to_string())),
                    ]));
                }
                for b in BUILTINS {
                    items.push(J::Obj(vec![
                        ("label".into(), J::Str(b.to_string())),
                        ("kind".into(), J::Num(3.0)), // Function
                    ]));
                }
                if let Some(id) = id { send(&mut stdout, &response(id, J::Arr(items))); }
            }
            "textDocument/documentSymbol" => {
                let uri = params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()).unwrap_or("");
                let symbols = document_symbols(docs.get(uri).map(|s| s.as_str()).unwrap_or(""));
                if let Some(id) = id { send(&mut stdout, &response(id, J::Arr(symbols))); }
            }
            "shutdown" => { if let Some(id) = id { send(&mut stdout, &response(id, J::Null)); } }
            "exit" => break,
            _ => {
                // unknown request with id → empty result; notifications ignored
                if let Some(id) = id { send(&mut stdout, &response(id, J::Null)); }
            }
        }
    }
}

fn word_at(text: &str, line: usize, character: usize) -> Option<String> {
    let line_str = text.lines().nth(line)?;
    let chars: Vec<char> = line_str.chars().collect();
    if character > chars.len() { return None; }
    let is_word = |c: char| !c.is_whitespace() && !"()[]{}:,+-*/=<>\"".contains(c);
    let mut start = character.min(chars.len());
    while start > 0 && is_word(chars[start - 1]) { start -= 1; }
    let mut end = character.min(chars.len());
    while end < chars.len() && is_word(chars[end]) { end += 1; }
    if start == end { return None; }
    Some(chars[start..end].iter().collect())
}

fn hover(params: &J, docs: &HashMap<String, String>) -> Option<J> {
    let uri = params.get("textDocument")?.get("uri")?.as_str()?;
    let pos = params.get("position")?;
    let line = pos.get("line")?.as_num()? as usize;
    let ch = pos.get("character")?.as_num()? as usize;
    let text = docs.get(uri)?;
    let word = word_at(text, line, ch)?;
    let desc = KEYWORDS.iter().find(|(k, _)| *k == word).map(|(_, d)| d.to_string())
        .or_else(|| if BUILTINS.contains(&word.as_str()) { Some(format!("अंतर्निहित फलन (builtin): {word}")) } else { None })?;
    Some(J::Obj(vec![
        ("contents".into(), J::Obj(vec![
            ("kind".into(), J::Str("markdown".into())),
            ("value".into(), J::Str(format!("**{word}** — {desc}"))),
        ])),
    ]))
}

fn document_symbols(text: &str) -> Vec<J> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        let (kind, kw) = if t.starts_with("विधि ") || t.starts_with("शुद्ध विधि ") || t.starts_with("साझा विधि ") {
            (12.0, "विधि") // Function
        } else if t.starts_with("वर्ग ") || t.starts_with("सार वर्ग ") {
            (5.0, "वर्ग") // Class
        } else { continue; };
        // extract the name after the keyword up to '(' or ':'
        if let Some(rest) = t.split(kw).nth(1) {
            let name: String = rest.trim().chars().take_while(|c| !"(:".contains(*c)).collect();
            let name = name.trim().to_string();
            if name.is_empty() { continue; }
            let pos = J::Obj(vec![("line".into(), J::Num(i as f64)), ("character".into(), J::Num(0.0))]);
            let range = J::Obj(vec![("start".into(), pos.clone()), ("end".into(), pos)]);
            out.push(J::Obj(vec![
                ("name".into(), J::Str(name)),
                ("kind".into(), J::Num(kind)),
                ("range".into(), range.clone()),
                ("selectionRange".into(), range),
            ]));
        }
    }
    out
}
