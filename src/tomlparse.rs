//! भारत.टोमल — a practical TOML config reader (Phase 19 F6).
//!
//! Pure Rust, no external crates. Parses a useful subset of TOML into a nested
//! Dict: bare/quoted keys, `[table]` and `[a.b.c]` nested tables, `# comments`,
//! and values — basic & literal strings, integers, floats, booleans, and arrays
//! (including arrays of the above). Enough to read `lipi.toml` and typical config.
//!
//!   toml_पढ़ो(पाठ)   → Dict  (tables become nested Dicts)
//!
//! Not supported (kept out to stay small): `[[array-of-tables]]`, datetimes
//! (returned as their string form), multi-line strings. Malformed input is a
//! catchable error naming the line.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

fn parse_value(raw: &str, line: usize) -> Result<Value, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err(format!("टोमल: पंक्ति {line} — मान खाली है"));
    }
    // strings
    if let Some(inner) = s.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        return Ok(Value::Str(unescape_basic(inner)));
    }
    if let Some(inner) = s.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return Ok(Value::Str(inner.to_string())); // literal string — verbatim
    }
    // booleans
    if s == "true" { return Ok(Value::Bool(true)); }
    if s == "false" { return Ok(Value::Bool(false)); }
    // arrays
    if s.starts_with('[') && s.ends_with(']') {
        let items = split_array(&s[1..s.len() - 1], line)?;
        let mut out = Vec::new();
        for it in items {
            out.push(parse_value(&it, line)?);
        }
        return Ok(Value::List(out));
    }
    // numbers (allow underscores as TOML digit separators)
    let cleaned: String = s.chars().filter(|c| *c != '_').collect();
    if let Ok(i) = cleaned.parse::<i64>() {
        return Ok(Value::Number(i as f64));
    }
    if let Ok(f) = cleaned.parse::<f64>() {
        return Ok(Value::Number(f));
    }
    // fall back: treat as a bare string (dates, etc.)
    Ok(Value::Str(s.to_string()))
}

fn unescape_basic(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => { out.push('\\'); out.push(other); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Split an array body on top-level commas (respecting quotes and nested `[]`).
fn split_array(body: &str, line: usize) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_str: Option<char> = None;
    for c in body.chars() {
        match in_str {
            Some(q) => {
                cur.push(c);
                if c == q { in_str = None; }
            }
            None => match c {
                '"' | '\'' => { in_str = Some(c); cur.push(c); }
                '[' => { depth += 1; cur.push(c); }
                ']' => { depth -= 1; cur.push(c); }
                ',' if depth == 0 => {
                    if !cur.trim().is_empty() { items.push(cur.trim().to_string()); }
                    cur.clear();
                }
                _ => cur.push(c),
            },
        }
    }
    if in_str.is_some() { return Err(format!("टोमल: पंक्ति {line} — सरणी में अधूरा वाक्य")); }
    if !cur.trim().is_empty() { items.push(cur.trim().to_string()); }
    Ok(items)
}

/// Strip a `#` comment that is not inside a string literal.
fn strip_comment(line: &str) -> &str {
    let mut in_str: Option<char> = None;
    for (i, c) in line.char_indices() {
        match in_str {
            Some(q) => { if c == q { in_str = None; } }
            None => match c {
                '"' | '\'' => in_str = Some(c),
                '#' => return &line[..i],
                _ => {}
            },
        }
    }
    line
}

fn split_key_path(key: &str) -> Vec<String> {
    key.split('.').map(|k| k.trim().trim_matches(|c| c == '"' || c == '\'').to_string()).collect()
}

/// Insert `val` at the given key path into the nested-Dict tree rooted at `root`.
fn insert_path(root: &mut HashMap<String, Value>, path: &[String], val: Value) {
    if path.len() == 1 {
        root.insert(path[0].clone(), val);
        return;
    }
    let entry = root.entry(path[0].clone()).or_insert_with(|| Value::Dict(HashMap::new()));
    if let Value::Dict(inner) = entry {
        insert_path(inner, &path[1..], val);
    } else {
        // overwrite a scalar that a deeper table now needs
        let mut inner = HashMap::new();
        insert_path(&mut inner, &path[1..], val);
        *entry = Value::Dict(inner);
    }
}

fn toml_read(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("toml_पढ़ो(): पाठ (वाक्य) अपेक्षित".to_string()),
    };
    let mut root: HashMap<String, Value> = HashMap::new();
    let mut cur_table: Vec<String> = Vec::new();

    for (i, raw_line) in text.lines().enumerate() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() { continue; }

        if line.starts_with('[') && line.ends_with(']') {
            let name = &line[1..line.len() - 1];
            if name.starts_with('[') {
                return Err(format!("टोमल: पंक्ति {} — [[array-of-tables]] समर्थित नहीं", i + 1));
            }
            cur_table = split_key_path(name);
            // ensure the table exists even if empty
            if !cur_table.is_empty() {
                insert_path(&mut root, &cur_table, Value::Dict(HashMap::new()));
            }
            continue;
        }

        let eq = line.find('=').ok_or_else(|| format!("टोमल: पंक्ति {} — '=' नहीं मिला", i + 1))?;
        let key = line[..eq].trim();
        let val = parse_value(&line[eq + 1..], i + 1)?;
        let mut path = cur_table.clone();
        path.extend(split_key_path(key));
        insert_path(&mut root, &path, val);
    }
    Ok(Value::Dict(root))
}

/// INI reader — `[section]` groups, `key = value` or `key: value`, `#`/`;`
/// comments. Sections become nested Dicts; keys before any section go in root.
/// All values stay Str.
fn ini_read(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("ini_पढ़ो(): पाठ (वाक्य) अपेक्षित".to_string()),
    };
    let mut root: HashMap<String, Value> = HashMap::new();
    let mut section: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') { continue; }
        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim().to_string();
            root.entry(name.clone()).or_insert_with(|| Value::Dict(HashMap::new()));
            section = Some(name);
            continue;
        }
        let sep = line.find('=').or_else(|| line.find(':'));
        let sep = match sep { Some(p) => p, None => continue };
        let key = line[..sep].trim().to_string();
        let val = Value::Str(line[sep + 1..].trim().trim_matches('"').to_string());
        match &section {
            Some(sec) => {
                if let Some(Value::Dict(d)) = root.get_mut(sec) { d.insert(key, val); }
            }
            None => { root.insert(key, val); }
        }
    }
    Ok(Value::Dict(root))
}

pub fn toml_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("toml_पढ़ो", toml_read),
        ("ini_पढ़ो", ini_read),
    ];
    list
}
