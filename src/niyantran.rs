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

// ── N-dimensional Kalman filter ──────────────────────────────────────────────
// State carried in a Dict: x = state vector (List), P/F/H/Q/R = matrices
// (List of Lists). All linear-algebra helpers are private to this file.

type Mat = Vec<Vec<f64>>;
type Vec1 = Vec<f64>;

fn as_vec(v: &Value, f: &str) -> Result<Vec1, String> {
    match v {
        Value::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for it in items {
                match it { Value::Number(n) => out.push(*n), _ => return Err(format!("{f}(): सदिश में संख्या अपेक्षित")) }
            }
            Ok(out)
        }
        _ => Err(format!("{f}(): सूची (सदिश) अपेक्षित")),
    }
}

fn as_mat(v: &Value, f: &str) -> Result<Mat, String> {
    match v {
        Value::List(rows) => {
            let mut out = Vec::with_capacity(rows.len());
            for r in rows { out.push(as_vec(r, f)?); }
            let cols = out.first().map(|r| r.len()).unwrap_or(0);
            if out.iter().any(|r| r.len() != cols) { return Err(format!("{f}(): आव्यूह की पंक्तियाँ असमान")); }
            Ok(out)
        }
        _ => Err(format!("{f}(): सूची-की-सूची (आव्यूह) अपेक्षित")),
    }
}

fn vec_to_val(v: &[f64]) -> Value { Value::List(v.iter().map(|&n| Value::Number(n)).collect()) }
fn mat_to_val(m: &Mat) -> Value { Value::List(m.iter().map(|r| vec_to_val(r)).collect()) }

fn mat_mul(a: &Mat, b: &Mat) -> Result<Mat, String> {
    let (ar, ac, br, bc) = (a.len(), a.first().map(|r| r.len()).unwrap_or(0), b.len(), b.first().map(|r| r.len()).unwrap_or(0));
    if ac != br { return Err("आव्यूह गुणन: आयाम बेमेल".into()); }
    let mut out = vec![vec![0.0; bc]; ar];
    for i in 0..ar {
        for k in 0..ac {
            let aik = a[i][k];
            for j in 0..bc { out[i][j] += aik * b[k][j]; }
        }
    }
    Ok(out)
}

fn mat_vec(a: &Mat, x: &[f64]) -> Result<Vec1, String> {
    let ac = a.first().map(|r| r.len()).unwrap_or(0);
    if ac != x.len() { return Err("आव्यूह×सदिश: आयाम बेमेल".into()); }
    Ok(a.iter().map(|row| row.iter().zip(x).map(|(&v, &xv)| v * xv).sum()).collect())
}

fn transpose(a: &Mat) -> Mat {
    let (r, c) = (a.len(), a.first().map(|r| r.len()).unwrap_or(0));
    let mut out = vec![vec![0.0; r]; c];
    for i in 0..r { for j in 0..c { out[j][i] = a[i][j]; } }
    out
}

fn mat_add(a: &Mat, b: &Mat) -> Mat {
    a.iter().zip(b).map(|(ra, rb)| ra.iter().zip(rb).map(|(&x, &y)| x + y).collect()).collect()
}

fn mat_sub(a: &Mat, b: &Mat) -> Mat {
    a.iter().zip(b).map(|(ra, rb)| ra.iter().zip(rb).map(|(&x, &y)| x - y).collect()).collect()
}

fn identity(n: usize) -> Mat {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n { m[i][i] = 1.0; }
    m
}

fn inverse(a: &Mat) -> Result<Mat, String> {
    let n = a.len();
    if n == 0 || a.iter().any(|r| r.len() != n) { return Err("व्युत्क्रम: वर्ग आव्यूह अपेक्षित".into()); }
    let mut aug = a.clone();
    let mut inv = identity(n);
    for col in 0..n {
        // partial pivoting for numerical stability
        let mut piv = col;
        for r in (col + 1)..n { if aug[r][col].abs() > aug[piv][col].abs() { piv = r; } }
        if aug[piv][col].abs() < 1e-12 { return Err("व्युत्क्रम: एकवचन आव्यूह (अव्युत्क्रमणीय)".into()); }
        aug.swap(col, piv);
        inv.swap(col, piv);
        let d = aug[col][col];
        for j in 0..n { aug[col][j] /= d; inv[col][j] /= d; }
        for r in 0..n {
            if r == col { continue; }
            let factor = aug[r][col];
            if factor == 0.0 { continue; }
            for j in 0..n { aug[r][j] -= factor * aug[col][j]; inv[r][j] -= factor * inv[col][j]; }
        }
    }
    Ok(inv)
}

fn kalman_nd_banao(args: Vec<Value>) -> Result<Value, String> {
    let nil = Value::Nil;
    let x = as_vec(args.first().unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let p = as_mat(args.get(1).unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let f = as_mat(args.get(2).unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let h = as_mat(args.get(3).unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let q = as_mat(args.get(4).unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let r = as_mat(args.get(5).unwrap_or(&nil), "कलमैन_एनडी_बनाओ")?;
    let mut m = HashMap::new();
    m.insert("x".into(), vec_to_val(&x));
    m.insert("P".into(), mat_to_val(&p));
    m.insert("F".into(), mat_to_val(&f));
    m.insert("H".into(), mat_to_val(&h));
    m.insert("Q".into(), mat_to_val(&q));
    m.insert("R".into(), mat_to_val(&r));
    Ok(Value::Dict(m))
}

fn kalman_nd_charan(args: Vec<Value>) -> Result<Value, String> {
    let m = match args.first() { Some(Value::Dict(m)) => m.clone(), _ => return Err("कलमैन_एनडी_चरण(): पहला तर्क कलमैन (कोश) होना चाहिए".into()) };
    let nil = Value::Nil;
    let z = as_vec(args.get(1).unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let x = as_vec(m.get("x").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let pp = as_mat(m.get("P").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let f = as_mat(m.get("F").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let h = as_mat(m.get("H").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let q = as_mat(m.get("Q").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;
    let r = as_mat(m.get("R").unwrap_or(&nil), "कलमैन_एनडी_चरण")?;

    // predict: x = F·x,  P = F·P·Fᵀ + Q
    let xp = mat_vec(&f, &x)?;
    let ft = transpose(&f);
    let pp1 = mat_add(&mat_mul(&mat_mul(&f, &pp)?, &ft)?, &q);

    // update: y = z − H·x,  S = H·P·Hᵀ + R,  K = P·Hᵀ·S⁻¹
    let hx = mat_vec(&h, &xp)?;
    if hx.len() != z.len() { return Err("कलमैन_एनडी_चरण(): माप सदिश का आयाम बेमेल".into()); }
    let y: Vec1 = z.iter().zip(&hx).map(|(&zi, &hxi)| zi - hxi).collect();
    let ht = transpose(&h);
    let s = mat_add(&mat_mul(&mat_mul(&h, &pp1)?, &ht)?, &r);
    let sinv = inverse(&s)?;
    let pht = mat_mul(&pp1, &ht)?;
    let k = mat_mul(&pht, &sinv)?;
    // x = x + K·y
    let ky = mat_vec(&k, &y)?;
    let xnew: Vec1 = xp.iter().zip(&ky).map(|(&xi, &kyi)| xi + kyi).collect();
    // P = (I − K·H)·P
    let kh = mat_mul(&k, &h)?;
    let ikh = mat_sub(&identity(kh.len()), &kh);
    let pnew = mat_mul(&ikh, &pp1)?;

    let mut nm = m.clone();
    nm.insert("x".into(), vec_to_val(&xnew));
    nm.insert("P".into(), mat_to_val(&pnew));
    Ok(Value::List(vec![vec_to_val(&xnew), Value::Dict(nm)]))
}

pub fn niyantran_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("पीआईडी_बनाओ", pid_banao),
        ("पीआईडी_चरण", pid_charan),
        ("कलमैन_बनाओ", kalman_banao),
        ("कलमैन_चरण", kalman_charan),
        ("कलमैन_एनडी_बनाओ", kalman_nd_banao),
        ("कलमैन_एनडी_चरण", kalman_nd_charan),
    ];
    list
}
