//! भारत.एक्सएमएल — a minimal XML / well-formed-HTML reader (Phase 19 F8).
//!
//! Pure Rust, no external crates. Parses elements, attributes, nested children,
//! and text into a tree of Dicts. Each element is:
//!   { "नाम": tag, "गुण": {attrs}, "बच्चे": [child elements], "पाठ": text }
//! Skips comments `<!-- -->`, the `<?xml ?>` declaration, and `<!DOCTYPE>`.
//!
//!   xml_पढ़ो(पाठ)   → root element Dict
//!
//! Requires well-formed markup (every tag closed). HTML works if it is XHTML-ish;
//! unclosed void tags (`<br>`) are not auto-closed.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

struct Cur<'a> { s: &'a [u8], i: usize, src: &'a str }

impl<'a> Cur<'a> {
    fn peek(&self) -> u8 { if self.i < self.s.len() { self.s[self.i] } else { 0 } }
    fn starts(&self, p: &str) -> bool { self.src[self.i..].starts_with(p) }
    fn skip_ws(&mut self) { while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() { self.i += 1; } }

    fn skip_junk(&mut self) {
        loop {
            self.skip_ws();
            if self.starts("<!--") {
                if let Some(end) = self.src[self.i..].find("-->") { self.i += end + 3; } else { self.i = self.s.len(); }
            } else if self.starts("<?") {
                if let Some(end) = self.src[self.i..].find("?>") { self.i += end + 2; } else { self.i = self.s.len(); }
            } else if self.starts("<!") {
                if let Some(end) = self.src[self.i..].find('>') { self.i += end + 1; } else { self.i = self.s.len(); }
            } else {
                break;
            }
        }
    }

    fn read_name(&mut self) -> String {
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if c.is_ascii_whitespace() || c == b'>' || c == b'/' || c == b'=' { break; }
            self.i += 1;
        }
        self.src[start..self.i].to_string()
    }

    fn read_attrs(&mut self) -> HashMap<String, Value> {
        let mut attrs = HashMap::new();
        loop {
            self.skip_ws();
            let c = self.peek();
            if c == b'>' || c == b'/' || c == 0 { break; }
            let name = self.read_name();
            if name.is_empty() { break; }
            self.skip_ws();
            let mut val = String::new();
            if self.peek() == b'=' {
                self.i += 1;
                self.skip_ws();
                let q = self.peek();
                if q == b'"' || q == b'\'' {
                    self.i += 1;
                    let start = self.i;
                    while self.i < self.s.len() && self.s[self.i] != q { self.i += 1; }
                    val = decode(&self.src[start..self.i]);
                    self.i += 1;
                }
            }
            attrs.insert(name, Value::Str(val));
        }
        attrs
    }

    fn parse_element(&mut self) -> Result<Value, String> {
        self.skip_junk();
        if self.peek() != b'<' { return Err("एक्सएमएल: '<' अपेक्षित".to_string()); }
        self.i += 1;
        let name = self.read_name();
        if name.is_empty() { return Err("एक्सएमएल: टैग नाम खाली".to_string()); }
        let attrs = self.read_attrs();
        self.skip_ws();

        let mut children = Vec::new();
        let mut text = String::new();

        if self.peek() == b'/' {
            self.i += 1;
            if self.peek() == b'>' { self.i += 1; }
            return Ok(make_elem(name, attrs, children, text));
        }
        if self.peek() == b'>' { self.i += 1; }

        loop {
            if self.i >= self.s.len() { return Err(format!("एक्सएमएल: <{name}> बंद नहीं हुआ")); }
            if self.starts("</") {
                self.i += 2;
                let _close = self.read_name();
                self.skip_ws();
                if self.peek() == b'>' { self.i += 1; }
                break;
            }
            if self.starts("<!--") {
                if let Some(end) = self.src[self.i..].find("-->") { self.i += end + 3; continue; }
            }
            if self.peek() == b'<' {
                children.push(self.parse_element()?);
            } else {
                let start = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'<' { self.i += 1; }
                let t = decode(self.src[start..self.i].trim());
                if !t.is_empty() { if !text.is_empty() { text.push(' '); } text.push_str(&t); }
            }
        }
        Ok(make_elem(name, attrs, children, text))
    }
}

fn decode(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"")
        .replace("&apos;", "'").replace("&amp;", "&")
}

fn make_elem(name: String, attrs: HashMap<String, Value>, children: Vec<Value>, text: String) -> Value {
    let mut m = HashMap::new();
    m.insert("नाम".to_string(), Value::Str(name));
    m.insert("गुण".to_string(), Value::Dict(attrs));
    m.insert("बच्चे".to_string(), Value::List(children));
    m.insert("पाठ".to_string(), Value::Str(text));
    Value::Dict(m)
}

fn xml_read(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("xml_पढ़ो(): पाठ (वाक्य) अपेक्षित".to_string()),
    };
    let mut cur = Cur { s: text.as_bytes(), i: 0, src: &text };
    cur.parse_element()
}

pub fn xml_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("xml_पढ़ो", xml_read),
    ];
    list
}
