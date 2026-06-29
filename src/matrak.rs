//! भारत.मात्रक — units of measure with dimensional analysis (Phase 18 H1).
//! Pure Rust, no external crates.
//!
//! A quantity carries its value (converted to SI) plus a 7-vector of SI base
//! dimension exponents [L, M, T, I, Θ, N, J]. Arithmetic checks dimensions and
//! errors on a mismatch — the class of bug that destroyed the Mars Climate
//! Orbiter (pound-force used where newton was expected, $327M lost).
//!
//!   मात्रा(मान, "इकाई")      → quantity (value stored in SI)
//!   जोड़_मात्रा / घटा_मात्रा  → add/subtract (same dimension required)
//!   गुणा_मात्रा / भाग_मात्रा  → multiply/divide (dimensions combine)
//!   मान_में(q, "इकाई")       → numeric value in a given unit
//!   मात्रा_वाक्य(q)          → "5 मीटर", "9.81 मीटर/सेकंड²"
//!   विमा_बराबर(a, b)        → same dimensions? (Bool)

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

type Dim = [i32; 7];
const DIMLESS: Dim = [0; 7];

const BASE_NAMES: [&str; 7] = ["मीटर", "किग्रा", "सेकंड", "एम्पियर", "केल्विन", "मोल", "कैंडेला"];

fn unit_lookup(name: &str) -> Option<(f64, Dim)> {
    let l: Dim = [1, 0, 0, 0, 0, 0, 0];
    let m: Dim = [0, 1, 0, 0, 0, 0, 0];
    let t: Dim = [0, 0, 1, 0, 0, 0, 0];
    let i: Dim = [0, 0, 0, 1, 0, 0, 0];
    let th: Dim = [0, 0, 0, 0, 1, 0, 0];
    let force: Dim = [1, 1, -2, 0, 0, 0, 0];
    let energy: Dim = [2, 1, -2, 0, 0, 0, 0];
    let power: Dim = [2, 1, -3, 0, 0, 0, 0];
    let pressure: Dim = [-1, 1, -2, 0, 0, 0, 0];
    let vel: Dim = [1, 0, -1, 0, 0, 0, 0];
    let accel: Dim = [1, 0, -2, 0, 0, 0, 0];
    let area: Dim = [2, 0, 0, 0, 0, 0, 0];
    let vol: Dim = [3, 0, 0, 0, 0, 0, 0];
    Some(match name {
        // length
        "मीटर" | "m" => (1.0, l),
        "किमी" | "किलोमीटर" => (1000.0, l),
        "सेमी" => (0.01, l),
        "मिमी" => (0.001, l),
        "फुट" | "ft" => (0.3048, l),
        "इंच" | "in" => (0.0254, l),
        "मील" => (1609.344, l),
        "गज" | "yard" => (0.9144, l),
        // mass
        "किग्रा" | "किलोग्राम" | "kg" => (1.0, m),
        "ग्राम" | "g" => (0.001, m),
        "पाउंड" | "lb" => (0.45359237, m),
        "औंस" => (0.0283495, m),
        "टन" => (1000.0, m),
        // time
        "सेकंड" | "s" => (1.0, t),
        "मिनट" => (60.0, t),
        "घंटा" => (3600.0, t),
        "दिन" => (86400.0, t),
        // current / temperature
        "एम्पियर" | "A" => (1.0, i),
        "केल्विन" | "K" => (1.0, th),
        // force — पाउंड_बल is the Mars Climate Orbiter culprit
        "न्यूटन" | "N" => (1.0, force),
        "पाउंड_बल" | "lbf" => (4.4482216152605, force),
        "डाइन" => (1e-5, force),
        // energy / power / pressure
        "जूल" | "J" => (1.0, energy),
        "किलोजूल" => (1000.0, energy),
        "कैलोरी" => (4.184, energy),
        "वाट" | "W" => (1.0, power),
        "किलोवाट" => (1000.0, power),
        "पास्कल" | "Pa" => (1.0, pressure),
        "बार" => (1e5, pressure),
        // kinematics
        "मीटर/सेकंड" => (1.0, vel),
        "किमी/घंटा" => (1000.0 / 3600.0, vel),
        "मीटर/सेकंड२" => (1.0, accel),
        // area / volume
        "वर्गमीटर" => (1.0, area),
        "हेक्टेयर" => (10000.0, area),
        "घनमीटर" => (1.0, vol),
        "लीटर" => (0.001, vol),
        _ => return None,
    })
}

fn dim_to_string(d: &Dim) -> String {
    let mut num: Vec<String> = Vec::new();
    let mut den: Vec<String> = Vec::new();
    let sup = |e: i32| -> String {
        if e.abs() == 1 { String::new() } else { to_superscript(e.abs()) }
    };
    for (idx, &e) in d.iter().enumerate() {
        if e > 0 { num.push(format!("{}{}", BASE_NAMES[idx], sup(e))); }
        else if e < 0 { den.push(format!("{}{}", BASE_NAMES[idx], sup(e))); }
    }
    let n = if num.is_empty() { "1".to_string() } else { num.join("·") };
    if den.is_empty() { n } else { format!("{}/{}", n, den.join("·")) }
}

fn to_superscript(n: i32) -> String {
    n.to_string().chars().map(|c| match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴',
        '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', '9' => '⁹', _ => c,
    }).collect()
}

fn make_qty(si_value: f64, dim: Dim) -> Value {
    let mut m = HashMap::new();
    m.insert("__मात्रा__".to_string(), Value::Bool(true));
    m.insert("मान".to_string(), Value::Number(si_value));
    m.insert("विमा".to_string(), Value::List(dim.iter().map(|&e| Value::Number(e as f64)).collect()));
    Value::Dict(m)
}

fn parse_qty(v: &Value, fname: &str) -> Result<(f64, Dim), String> {
    let m = match v {
        Value::Dict(m) if m.contains_key("__मात्रा__") => m,
        _ => return Err(format!("{fname}(): मात्रा अपेक्षित — पहले मात्रा(मान, इकाई) बनाएँ")),
    };
    let val = match m.get("मान") { Some(Value::Number(n)) => *n, _ => return Err(format!("{fname}(): अमान्य मात्रा")) };
    let dim = match m.get("विमा") {
        Some(Value::List(v)) if v.len() == 7 => {
            let mut d = DIMLESS;
            for (i, e) in v.iter().enumerate() {
                d[i] = match e { Value::Number(n) => *n as i32, _ => return Err(format!("{fname}(): अमान्य विमा")) };
            }
            d
        }
        _ => return Err(format!("{fname}(): अमान्य विमा")),
    };
    Ok((val, dim))
}

fn matra(args: Vec<Value>) -> Result<Value, String> {
    let val = match args.first() { Some(Value::Number(n)) => *n, _ => return Err("मात्रा(): पहला तर्क संख्या होना चाहिए".to_string()) };
    let unit = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("मात्रा(): दूसरा तर्क इकाई (वाक्य) होना चाहिए".to_string()) };
    let (factor, dim) = unit_lookup(&unit).ok_or_else(|| format!("मात्रा(): अज्ञात इकाई '{unit}'"))?;
    Ok(make_qty(val * factor, dim))
}

fn jod_matra(args: Vec<Value>) -> Result<Value, String> {
    let (a, da) = parse_qty(args.first().unwrap_or(&Value::Nil), "जोड़_मात्रा")?;
    let (b, db) = parse_qty(args.get(1).unwrap_or(&Value::Nil), "जोड़_मात्रा")?;
    if da != db { return Err(format!("जोड़_मात्रा(): विमा बेमेल — {} बनाम {} (जैसे न्यूटन में पाउंड-बल नहीं जोड़ सकते बिना रूपांतर)", dim_to_string(&da), dim_to_string(&db))); }
    Ok(make_qty(a + b, da))
}

fn ghata_matra(args: Vec<Value>) -> Result<Value, String> {
    let (a, da) = parse_qty(args.first().unwrap_or(&Value::Nil), "घटा_मात्रा")?;
    let (b, db) = parse_qty(args.get(1).unwrap_or(&Value::Nil), "घटा_मात्रा")?;
    if da != db { return Err(format!("घटा_मात्रा(): विमा बेमेल — {} बनाम {}", dim_to_string(&da), dim_to_string(&db))); }
    Ok(make_qty(a - b, da))
}

fn guna_matra(args: Vec<Value>) -> Result<Value, String> {
    let (a, da) = parse_qty(args.first().unwrap_or(&Value::Nil), "गुणा_मात्रा")?;
    let (b, db) = parse_qty(args.get(1).unwrap_or(&Value::Nil), "गुणा_मात्रा")?;
    let mut d = DIMLESS;
    for i in 0..7 { d[i] = da[i] + db[i]; }
    Ok(make_qty(a * b, d))
}

fn bhag_matra(args: Vec<Value>) -> Result<Value, String> {
    let (a, da) = parse_qty(args.first().unwrap_or(&Value::Nil), "भाग_मात्रा")?;
    let (b, db) = parse_qty(args.get(1).unwrap_or(&Value::Nil), "भाग_मात्रा")?;
    if b == 0.0 { return Err("भाग_मात्रा(): शून्य से भाग".to_string()); }
    let mut d = DIMLESS;
    for i in 0..7 { d[i] = da[i] - db[i]; }
    Ok(make_qty(a / b, d))
}

fn maan_mein(args: Vec<Value>) -> Result<Value, String> {
    let (val, dim) = parse_qty(args.first().unwrap_or(&Value::Nil), "मान_में")?;
    let unit = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("मान_में(): इकाई (वाक्य) अपेक्षित".to_string()) };
    let (factor, udim) = unit_lookup(&unit).ok_or_else(|| format!("मान_में(): अज्ञात इकाई '{unit}'"))?;
    if udim != dim { return Err(format!("मान_में(): विमा बेमेल — मात्रा {} है, इकाई '{}' {}", dim_to_string(&dim), unit, dim_to_string(&udim))); }
    Ok(Value::Number(val / factor))
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{n}") }
}

fn matra_vakya(args: Vec<Value>) -> Result<Value, String> {
    let (val, dim) = parse_qty(args.first().unwrap_or(&Value::Nil), "मात्रा_वाक्य")?;
    let num = fmt_num(val);
    if dim == DIMLESS { Ok(Value::Str(num)) }
    else { Ok(Value::Str(format!("{} {}", num, dim_to_string(&dim)))) }
}

fn vima_barabar(args: Vec<Value>) -> Result<Value, String> {
    let (_, da) = parse_qty(args.first().unwrap_or(&Value::Nil), "विमा_बराबर")?;
    let (_, db) = parse_qty(args.get(1).unwrap_or(&Value::Nil), "विमा_बराबर")?;
    Ok(Value::Bool(da == db))
}

pub fn matrak_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("मात्रा", matra),
        ("जोड़_मात्रा", jod_matra),
        ("घटा_मात्रा", ghata_matra),
        ("गुणा_मात्रा", guna_matra),
        ("भाग_मात्रा", bhag_matra),
        ("मान_में", maan_mein),
        ("मात्रा_वाक्य", matra_vakya),
        ("विमा_बराबर", vima_barabar),
    ];
    list
}
