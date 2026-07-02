mod karaka;
mod types;
mod lexer;
mod ast;
mod parser;
mod interpreter;
mod bharat_stdlib;
mod regex_engine;
mod bignum;
mod net;
mod zip;
mod sql;
mod server;
mod threads;
mod matrak;
mod rekhiy;
mod niyantran;
mod disha;
mod suraksha;
mod antaral;
mod ffi;
mod tantra;
mod cbthunk;
mod https;
mod lognaad;
mod tomlparse;
mod yaml;
mod xmlparse;
mod argparse;
mod daak;
mod jit;
mod opcode;
mod compiler;
mod lvm;
mod formatter;
mod roman;
mod phonetic;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Pre-load stdin lines for पढ़ो() — must be called before run_source() in web mode.
/// Each line in `input` is consumed in order by each पढ़ो() call.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn set_stdin(input: &str) {
    let lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();
    lvm::set_stdin_buffer(lines);
}

/// Run a LIPI source string and return all output as a single String.
/// Called directly from the WASM playground; also usable in native integration tests.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn run_source(src: &str) -> String {
    let tokens = lexer::tokenize(src);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new_capturing();
            match vm.run(&program) {
                Ok(()) => vm.output,
                Err(e) => format!("LVM त्रुटि: {e}"),
            }
        }
        Err(e) => format!("व्याकरण त्रुटि: {e}"),
    }
}

/// Run the program and return a JSON step trace for the in-IDE debugger:
/// {"steps":[{"line":N,"depth":D,"vars":{name:"val",…}}],"error?":"…"}.
/// On a parse error returns {"steps":[],"error":"…"}.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_trace(src: &str) -> String {
    let tokens = lexer::tokenize(src);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new_capturing();
            vm.run_trace(&program)
        }
        Err(e) => format!("{{\"steps\":[],\"error\":{}}}", json_str(&e)),
    }
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Parse the source and return diagnostics as JSON: [{"line":N,"message":"..."}].
/// Empty array `[]` means no parse errors.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_diagnostics(src: &str) -> String {
    let tokens = lexer::tokenize(src);
    match parser::parse(tokens) {
        Ok(_) => "[]".to_string(),
        Err(e) => {
            let mut line = 1i64;
            if let Some(p) = e.find("(line ") {
                let rest = &e[p + 6..];
                if let Some(end) = rest.find(')') { line = rest[..end].trim().parse().unwrap_or(1); }
            }
            format!("[{{\"line\":{},\"message\":{}}}]", line, json_str(&e))
        }
    }
}

/// Extract विधि/वर्ग definitions as JSON: [{"name":"..","kind":"function|class","line":N}].
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_symbols(src: &str) -> String {
    let mut items = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let t = raw.trim_start();
        let (kind, kw) = if t.starts_with("विधि ") || t.starts_with("शुद्ध विधि ") || t.starts_with("साझा विधि ") {
            ("function", "विधि")
        } else if t.starts_with("वर्ग ") || t.starts_with("सार वर्ग ") {
            ("class", "वर्ग")
        } else { continue; };
        if let Some(rest) = t.split(kw).nth(1) {
            let name: String = rest.trim().chars().take_while(|c| !"(:".contains(*c)).collect();
            let name = name.trim().to_string();
            if name.is_empty() { continue; }
            items.push(format!("{{\"name\":{},\"kind\":{},\"line\":{}}}", json_str(&name), json_str(kind), i + 1));
        }
    }
    format!("[{}]", items.join(","))
}

const KW_DOCS: &[(&str, &str)] = &[
    ("बताओ", "print with newline"), ("लिखो", "print inline"), ("विधि", "function definition"),
    ("फल", "return"), ("उत्पन्न", "yield (generator)"), ("प्रतीक्षा", "await (async)"),
    ("यदि", "if"), ("अन्यथा", "else"), ("जब", "जब तक — while"), ("बार", "N बार करो — repeat"),
    ("वर्ग", "class"), ("कोशिश", "try"), ("पकड़ो", "catch"), ("फेंको", "throw"),
    ("आयात", "import"), ("लाम्डा", "lambda"), ("जाँचो", "assert"), ("स्थिर", "const"),
    ("सत्य", "true"), ("असत्य", "false"), ("और", "and"), ("या", "or"), ("नहीं", "not"),
];
const BUILTINS: &[&str] = &[
    "लम्बाई", "पूर्णांक", "वाक्य", "पढ़ो", "यादृच्छिक", "निरपेक्ष", "घात", "वर्गमूल", "गोल",
    "मानचित्र", "छानो", "मोड़ो", "प्रकार", "स्वरूप", "यूआईडी", "स्मरण", "आंशिक", "संयोजित",
    "आगे", "सूची_में", "सोओ", "चलाओ", "इकट्ठा", "सामान्यीकृत", "पूर्ण_है", "क्रमित_कोश",
];

/// Completion items as JSON: [{"label":"..","detail":"..","kind":"keyword|function"}].
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_completions() -> String {
    let mut items = Vec::new();
    for (kw, doc) in KW_DOCS {
        items.push(format!("{{\"label\":{},\"detail\":{},\"kind\":\"keyword\"}}", json_str(kw), json_str(doc)));
    }
    for b in BUILTINS {
        items.push(format!("{{\"label\":{},\"detail\":\"builtin\",\"kind\":\"function\"}}", json_str(b)));
    }
    format!("[{}]", items.join(","))
}

/// Format LIPI source (behavior-preserving, idempotent) — same as `lipi fmt`.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_format(src: &str) -> String {
    formatter::format_source(src)
}

/// Phonetic transliteration (QWERTY → Devanagari): keywords + identifiers.
/// `batao "x"\nnaam hai 5` → `बताओ "x"\nनाम है 5`. Used by LIPI Studio's live
/// Roman-input mode and the whole-document convert button.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_phonetic(src: &str) -> String {
    phonetic::vani_to_devanagari(src)
}

/// Keyword-only Roman → Devanagari (identifiers/strings kept verbatim).
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_roman(src: &str) -> String {
    roman::roman_to_devanagari(src)
}

/// Hover documentation for a word (keyword or builtin), or "" if unknown.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn lipi_hover(word: &str) -> String {
    if let Some((_, doc)) = KW_DOCS.iter().find(|(k, _)| *k == word) {
        return format!("**{}** — {}", word, doc);
    }
    if BUILTINS.contains(&word) {
        return format!("**{}** — अंतर्निहित फलन (builtin)", word);
    }
    String::new()
}
