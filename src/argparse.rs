//! भारत.तर्कपार्स — command-line argument parsing (Phase 19 F9).
//!
//! Turns a raw arg List (from `तर्क()`) into a Dict. Generic, no schema needed.
//! To stay unambiguous without a schema, a bare `--नाम` is ALWAYS a boolean flag;
//! give a value with `--नाम=मान`:
//!   --नाम=मान      → {"नाम": "मान"}
//!   --झंडा         → {"झंडा": सत्य}   (boolean flag)
//!   -क             → {"क": सत्य}      (short flag, boolean)
//!   बाकी           → appended to the "_स्थिति" (positional) List
//!
//!   तर्क_पार्स(सूची)   → Dict
//!
//! Values are always Str (convert with पूर्णांक() as needed). Repeated options
//! keep the last value.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

fn arg_parse(args: Vec<Value>) -> Result<Value, String> {
    let list = match args.first() {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("तर्क_पार्स(): सूची (List of वाक्य) अपेक्षित".to_string()),
    };
    let toks: Vec<String> = list.iter().map(|v| match v {
        Value::Str(s) => s.clone(),
        Value::Number(n) => if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{n}") },
        other => format!("{other}"),
    }).collect();

    let mut out: HashMap<String, Value> = HashMap::new();
    let mut positional: Vec<Value> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        if let Some(rest) = t.strip_prefix("--") {
            // schema-less: --key=value sets a value; bare --flag is always boolean
            if let Some((k, v)) = rest.split_once('=') {
                out.insert(k.to_string(), Value::Str(v.to_string()));
            } else {
                out.insert(rest.to_string(), Value::Bool(true));
            }
        } else if let Some(short) = t.strip_prefix('-') {
            if !short.is_empty() && short.chars().all(|c| c.is_ascii_alphabetic()) {
                for c in short.chars() {
                    out.insert(c.to_string(), Value::Bool(true));
                }
            } else {
                positional.push(Value::Str(t.clone()));
            }
        } else {
            positional.push(Value::Str(t.clone()));
        }
        i += 1;
    }
    out.insert("_स्थिति".to_string(), Value::List(positional));
    Ok(Value::Dict(out))
}

pub fn argparse_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("तर्क_पार्स", arg_parse),
    ];
    list
}
