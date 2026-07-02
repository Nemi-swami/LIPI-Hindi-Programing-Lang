//! भारत.लॉग — structured logging (Phase 19 F5).
//!
//! Level-filtered log output to stderr, so program output on stdout stays clean.
//! Levels: डिबग(0) < सूचना(1) < चेतावनी(2) < त्रुटि(3). A message is printed only
//! if its level ≥ the current threshold (default सूचना). Pure Rust; WASM-safe
//! (timestamps are added on native builds only).
//!
//!   लॉग_स्तर("डिबग"|"सूचना"|"चेतावनी"|"त्रुटि")   → set the threshold
//!   लॉग_डिबग(संदेश) / लॉग_सूचना / लॉग_चेतावनी / लॉग_त्रुटि(संदेश)

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::cell::Cell;

thread_local! {
    static LEVEL: Cell<u8> = const { Cell::new(1) }; // default: सूचना
}

fn level_from_name(s: &str) -> Option<u8> {
    match s {
        "डिबग" | "debug" => Some(0),
        "सूचना" | "info" => Some(1),
        "चेतावनी" | "warn" => Some(2),
        "त्रुटि" | "error" => Some(3),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("[{}] ", d.as_secs()),
        Err(_) => String::new(),
    }
}
#[cfg(target_arch = "wasm32")]
fn stamp() -> String { String::new() }

fn msg_of(args: &[Value], fname: &str) -> Result<String, String> {
    match args.first() {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{n}") }),
        Some(Value::Bool(b)) => Ok(if *b { "सत्य".into() } else { "असत्य".into() }),
        _ => Err(format!("{fname}(): संदेश वाक्य होना चाहिए")),
    }
}

fn emit(level: u8, tag: &str, args: Vec<Value>, fname: &str) -> Result<Value, String> {
    let threshold = LEVEL.with(|l| l.get());
    if level >= threshold {
        let m = msg_of(&args, fname)?;
        eprintln!("{}{} {}", stamp(), tag, m);
    }
    Ok(Value::Nil)
}

fn set_level(args: Vec<Value>) -> Result<Value, String> {
    let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("लॉग_स्तर(): स्तर नाम (वाक्य) अपेक्षित".to_string()) };
    let lv = level_from_name(&name).ok_or_else(|| format!("लॉग_स्तर(): अज्ञात स्तर '{name}' (डिबग/सूचना/चेतावनी/त्रुटि)"))?;
    LEVEL.with(|l| l.set(lv));
    Ok(Value::Nil)
}

fn log_debug(a: Vec<Value>) -> Result<Value, String> { emit(0, "डिबग", a, "लॉग_डिबग") }
fn log_info(a: Vec<Value>) -> Result<Value, String> { emit(1, "सूचना", a, "लॉग_सूचना") }
fn log_warn(a: Vec<Value>) -> Result<Value, String> { emit(2, "चेतावनी", a, "लॉग_चेतावनी") }
fn log_error(a: Vec<Value>) -> Result<Value, String> { emit(3, "त्रुटि", a, "लॉग_त्रुटि") }

pub fn log_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("लॉग_स्तर", set_level),
        ("लॉग_डिबग", log_debug),
        ("लॉग_सूचना", log_info),
        ("लॉग_चेतावनी", log_warn),
        ("लॉग_त्रुटि", log_error),
    ];
    list
}
