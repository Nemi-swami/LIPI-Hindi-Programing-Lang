//! भारत.नियंत्रण — control systems: PID controller + 1-D Kalman filter (Phase 18 H3).
//! Pure Rust. State lives in a Dict (copy-on-write); step functions return
//! [output, new_state] so the caller reassigns. Used for thruster/rudder/reactor
//! control loops and GPS+IMU sensor-fusion smoothing.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

fn num(v: Option<&Value>, f: &str) -> Result<f64, String> {
    match v { Some(Value::Number(n)) => Ok(*n), _ => Err(format!("{f}(): संख्या अपेक्षित")) }
}

fn dget(m: &HashMap<String, Value>, k: &str) -> f64 {
    match m.get(k) { Some(Value::Number(n)) => *n, _ => 0.0 }
}

// ── PID ──────────────────────────────────────────────────────────────────────

fn pid_banao(args: Vec<Value>) -> Result<Value, String> {
    let kp = num(args.first(), "पीआईडी_बनाओ")?;
    let ki = num(args.get(1), "पीआईडी_बनाओ")?;
    let kd = num(args.get(2), "पीआईडी_बनाओ")?;
    let imax = match args.get(3) { Some(Value::Number(n)) => *n, _ => 1e9 };
    let mut m = HashMap::new();
    m.insert("kp".into(), Value::Number(kp));
    m.insert("ki".into(), Value::Number(ki));
    m.insert("kd".into(), Value::Number(kd));
    m.insert("समाकल".into(), Value::Number(0.0));
    m.insert("पिछली_त्रुटि".into(), Value::Number(0.0));
    m.insert("समाकल_सीमा".into(), Value::Number(imax));
    Ok(Value::Dict(m))
}

fn pid_charan(args: Vec<Value>) -> Result<Value, String> {
    let m = match args.first() { Some(Value::Dict(m)) => m.clone(), _ => return Err("पीआईडी_चरण(): पहला तर्क पीआईडी होना चाहिए".into()) };
    let err = num(args.get(1), "पीआईडी_चरण")?;
    let dt = num(args.get(2), "पीआईडी_चरण")?;
    if dt <= 0.0 { return Err("पीआईडी_चरण(): dt धनात्मक होना चाहिए".into()); }
    let (kp, ki, kd) = (dget(&m, "kp"), dget(&m, "ki"), dget(&m, "kd"));
    let imax = dget(&m, "समाकल_सीमा");
    let mut integral = dget(&m, "समाकल") + err * dt;
    if integral > imax { integral = imax; } else if integral < -imax { integral = -imax; } // anti-windup
    let derivative = (err - dget(&m, "पिछली_त्रुटि")) / dt;
    let output = kp * err + ki * integral + kd * derivative;
    let mut nm = m.clone();
    nm.insert("समाकल".into(), Value::Number(integral));
    nm.insert("पिछली_त्रुटि".into(), Value::Number(err));
    Ok(Value::List(vec![Value::Number(output), Value::Dict(nm)]))
}

// ── 1-D Kalman filter ────────────────────────────────────────────────────────

fn kalman_banao(args: Vec<Value>) -> Result<Value, String> {
    let x = num(args.first(), "कलमैन_बनाओ")?;       // initial estimate
    let p = num(args.get(1), "कलमैन_बनाओ")?;        // initial uncertainty
    let q = num(args.get(2), "कलमैन_बनाओ")?;        // process noise
    let r = num(args.get(3), "कलमैन_बनाओ")?;        // measurement noise
    let mut m = HashMap::new();
    m.insert("x".into(), Value::Number(x));
    m.insert("P".into(), Value::Number(p));
    m.insert("Q".into(), Value::Number(q));
    m.insert("R".into(), Value::Number(r));
    Ok(Value::Dict(m))
}

fn kalman_charan(args: Vec<Value>) -> Result<Value, String> {
    let m = match args.first() { Some(Value::Dict(m)) => m.clone(), _ => return Err("कलमैन_चरण(): पहला तर्क कलमैन होना चाहिए".into()) };
    let z = num(args.get(1), "कलमैन_चरण")?; // measurement
    let (mut x, mut p) = (dget(&m, "x"), dget(&m, "P"));
    let (q, r) = (dget(&m, "Q"), dget(&m, "R"));
    // predict
    p += q;
    // update
    let k = p / (p + r);
    x += k * (z - x);
    p *= 1.0 - k;
    let mut nm = m.clone();
    nm.insert("x".into(), Value::Number(x));
    nm.insert("P".into(), Value::Number(p));
    Ok(Value::List(vec![Value::Number(x), Value::Dict(nm)]))
}

pub fn niyantran_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("पीआईडी_बनाओ", pid_banao),
        ("पीआईडी_चरण", pid_charan),
        ("कलमैन_बनाओ", kalman_banao),
        ("कलमैन_चरण", kalman_charan),
    ];
    list
}
