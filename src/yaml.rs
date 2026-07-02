//! भारत.यामल — a practical YAML subset reader (Phase 19 F7).
//!
//! Pure Rust, no external crates. Handles the common config shapes: nested
//! mappings (indentation-based), sequences (`- item`), scalars (quoted/bare
//! strings, ints, floats, true/false, null), `# comments`, and blank lines.
//!
//!   यामल_पढ़ो(पाठ)   → Value (mapping→Dict, sequence→List, scalar→Number/Str/Bool/शून्य)
//!
//! Not supported (kept out to stay small): anchors/aliases, multi-line block
//! scalars (`|`, `>`), flow style (`{a: 1}`, `[1, 2]`), multiple documents.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

struct Ln { indent: usize, text: String }

fn scalar(s: &str) -> Value {
    let t = s.trim();
    if t.is_empty() || t == "~" || t == "null" { return Value::Nil; }
    if t == "true" { return Value::Bool(true); }
    if t == "false" { return Value::Bool(false); }
    if let Some(inner) = t.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        return Value::Str(inner.replace("\\\"", "\"").replace("\\n", "\n"));
    }
    if let Some(inner) = t.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return Value::Str(inner.to_string());
    }
    if let Ok(i) = t.parse::<i64>() { return Value::Number(i as f64); }
    if let Ok(f) = t.parse::<f64>() { return Value::Number(f); }
    Value::Str(t.to_string())
}

/// Parse the block of lines[start..end] that are all at indent ≥ `base`, where
/// the first line sits exactly at `base`. Returns a Dict, List, or scalar.
fn parse_block(lines: &[Ln], start: usize, end: usize) -> Result<Value, String> {
    if start >= end { return Ok(Value::Nil); }
    let base = lines[start].indent;
    let is_seq = lines[start].text.starts_with("- ") || lines[start].text == "-";

    if is_seq {
        let mut list = Vec::new();
        let mut i = start;
        while i < end {
            if lines[i].indent != base { i += 1; continue; }
            let item = lines[i].text[1..].trim_start(); // after '-'
            // find the extent of this item (until next line at `base`)
            let mut j = i + 1;
            while j < end && lines[j].indent > base { j += 1; }
            if item.is_empty() {
                list.push(parse_block(lines, i + 1, j)?);
            } else if item.contains(": ") || item.ends_with(':') {
                // "- key: val" — an inline mapping start; reparse this item plus
                // deeper lines as a mapping by synthesizing a virtual line block.
                let mut sub = vec![Ln { indent: 0, text: item.to_string() }];
                for k in (i + 1)..j {
                    sub.push(Ln { indent: lines[k].indent - base + 2, text: lines[k].text.clone() });
                }
                list.push(parse_block(&sub, 0, sub.len())?);
            } else {
                list.push(scalar(item));
            }
            i = j;
        }
        return Ok(Value::List(list));
    }

    // mapping
    let mut map = HashMap::new();
    let mut i = start;
    while i < end {
        if lines[i].indent != base { i += 1; continue; }
        let line = &lines[i].text;
        let colon = line.find(':').ok_or_else(|| format!("यामल: ':' नहीं मिला — '{line}'"))?;
        let key = line[..colon].trim().trim_matches(|c| c == '"' || c == '\'').to_string();
        let rest = line[colon + 1..].trim();
        let mut j = i + 1;
        while j < end && lines[j].indent > base { j += 1; }
        if rest.is_empty() {
            map.insert(key, parse_block(lines, i + 1, j)?);
        } else {
            map.insert(key, scalar(rest));
        }
        i = j;
    }
    Ok(Value::Dict(map))
}

fn yaml_read(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("यामल_पढ़ो(): पाठ (वाक्य) अपेक्षित".to_string()),
    };
    let mut lines: Vec<Ln> = Vec::new();
    for raw in text.lines() {
        // strip a trailing comment that isn't in quotes (simple heuristic)
        let no_comment = match raw.find(" #") { Some(p) if !raw[..p].contains('"') => &raw[..p], _ => raw };
        if no_comment.trim().is_empty() { continue; }
        if no_comment.trim() == "---" { continue; }
        let indent = no_comment.len() - no_comment.trim_start().len();
        lines.push(Ln { indent, text: no_comment.trim_end().trim_start().to_string() });
    }
    if lines.is_empty() { return Ok(Value::Dict(HashMap::new())); }
    parse_block(&lines, 0, lines.len())
}

pub fn yaml_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("यामल_पढ़ो", yaml_read),
    ];
    list
}
