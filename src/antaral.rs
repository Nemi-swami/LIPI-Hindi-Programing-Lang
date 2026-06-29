//! भारत.अंतराल — interval arithmetic (Phase 18 H6). Pure Rust.
//! An interval [lo, hi] bounds a value with guaranteed lower/upper limits;
//! operations propagate the bounds so the result is a rigorous enclosure
//! (verification & validation of numerical computations). Intervals are
//! 2-element Lists [lo, hi].

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

fn iv_of(v: &Value, f: &str) -> Result<(f64, f64), String> {
    match v {
        Value::List(items) if items.len() == 2 => {
            let lo = match &items[0] { Value::Number(n) => *n, _ => return Err(format!("{f}(): अंतराल में संख्या अपेक्षित")) };
            let hi = match &items[1] { Value::Number(n) => *n, _ => return Err(format!("{f}(): अंतराल में संख्या अपेक्षित")) };
            Ok((lo.min(hi), lo.max(hi)))
        }
        _ => Err(format!("{f}(): अंतराल [निम्न, उच्च] अपेक्षित")),
    }
}

fn iv(lo: f64, hi: f64) -> Value { Value::List(vec![Value::Number(lo), Value::Number(hi)]) }

fn antaral(args: Vec<Value>) -> Result<Value, String> {
    let lo = match args.first() { Some(Value::Number(n)) => *n, _ => return Err("अंतराल(): निम्न (संख्या)".into()) };
    let hi = match args.get(1) { Some(Value::Number(n)) => *n, _ => return Err("अंतराल(): उच्च (संख्या)".into()) };
    Ok(iv(lo.min(hi), lo.max(hi)))
}

fn antaral_yog(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_योग")?;
    let (c, d) = iv_of(args.get(1).unwrap_or(&Value::Nil), "अंतराल_योग")?;
    Ok(iv(a + c, b + d))
}

fn antaral_ghata(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_घटा")?;
    let (c, d) = iv_of(args.get(1).unwrap_or(&Value::Nil), "अंतराल_घटा")?;
    Ok(iv(a - d, b - c))
}

fn antaral_guna(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_गुणा")?;
    let (c, d) = iv_of(args.get(1).unwrap_or(&Value::Nil), "अंतराल_गुणा")?;
    let ps = [a * c, a * d, b * c, b * d];
    let lo = ps.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = ps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Ok(iv(lo, hi))
}

fn antaral_chaudai(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_चौड़ाई")?;
    Ok(Value::Number(b - a))
}

fn antaral_madhya(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_मध्य")?;
    Ok(Value::Number((a + b) / 2.0))
}

fn antaral_mein(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = iv_of(args.first().unwrap_or(&Value::Nil), "अंतराल_में")?;
    let x = match args.get(1) { Some(Value::Number(n)) => *n, _ => return Err("अंतराल_में(): मान (संख्या)".into()) };
    Ok(Value::Bool(x >= a && x <= b))
}

pub fn antaral_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("अंतराल", antaral),
        ("अंतराल_योग", antaral_yog),
        ("अंतराल_घटा", antaral_ghata),
        ("अंतराल_गुणा", antaral_guna),
        ("अंतराल_चौड़ाई", antaral_chaudai),
        ("अंतराल_मध्य", antaral_madhya),
        ("अंतराल_में", antaral_mein),
    ];
    list
}
