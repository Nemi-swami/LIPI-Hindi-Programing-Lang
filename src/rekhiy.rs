//! भारत.रेखीय — linear algebra: vectors, matrices, quaternions (Phase 18 H2).
//! Pure Rust. Vectors are Lists of numbers; matrices are Lists of Lists (rows);
//! quaternions are 4-element Lists [w, x, y, z]. Used for spacecraft/vessel
//! attitude (quaternions avoid gimbal lock), navigation, and sensor fusion.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

fn vec_of(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
    match v {
        Value::List(items) => items.iter().map(|x| match x {
            Value::Number(n) => Ok(*n),
            other => Err(format!("{fname}(): सदिश में संख्या अपेक्षित, मिला: {other}")),
        }).collect(),
        _ => Err(format!("{fname}(): सदिश (सूची) अपेक्षित")),
    }
}

fn mat_of(v: &Value, fname: &str) -> Result<Vec<Vec<f64>>, String> {
    match v {
        Value::List(rows) => rows.iter().map(|r| vec_of(r, fname)).collect(),
        _ => Err(format!("{fname}(): आव्यूह (सूचियों की सूची) अपेक्षित")),
    }
}

fn list(v: Vec<f64>) -> Value { Value::List(v.into_iter().map(Value::Number).collect()) }
fn matrix(m: Vec<Vec<f64>>) -> Value { Value::List(m.into_iter().map(list).collect()) }

fn arg<'a>(args: &'a [Value], i: usize, f: &str) -> Result<&'a Value, String> {
    args.get(i).ok_or_else(|| format!("{f}(): पर्याप्त तर्क नहीं"))
}

// ── Vectors ──────────────────────────────────────────────────────────────────

fn sadish_yog(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "सदिश_योग")?, "सदिश_योग")?;
    let b = vec_of(arg(&args, 1, "सदिश_योग")?, "सदिश_योग")?;
    if a.len() != b.len() { return Err("सदिश_योग(): विमाएँ असमान".into()); }
    Ok(list(a.iter().zip(&b).map(|(x, y)| x + y).collect()))
}

fn sadish_ghata(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "सदिश_घटा")?, "सदिश_घटा")?;
    let b = vec_of(arg(&args, 1, "सदिश_घटा")?, "सदिश_घटा")?;
    if a.len() != b.len() { return Err("सदिश_घटा(): विमाएँ असमान".into()); }
    Ok(list(a.iter().zip(&b).map(|(x, y)| x - y).collect()))
}

fn adish_gunan(args: Vec<Value>) -> Result<Value, String> {
    let s = match arg(&args, 0, "अदिश_गुणन")? { Value::Number(n) => *n, _ => return Err("अदिश_गुणन(): पहला तर्क संख्या".into()) };
    let a = vec_of(arg(&args, 1, "अदिश_गुणन")?, "अदिश_गुणन")?;
    Ok(list(a.iter().map(|x| s * x).collect()))
}

fn bindu_gunan(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "बिंदु_गुणन")?, "बिंदु_गुणन")?;
    let b = vec_of(arg(&args, 1, "बिंदु_गुणन")?, "बिंदु_गुणन")?;
    if a.len() != b.len() { return Err("बिंदु_गुणन(): विमाएँ असमान".into()); }
    Ok(Value::Number(a.iter().zip(&b).map(|(x, y)| x * y).sum()))
}

fn kon_gunan(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "कोण_गुणन")?, "कोण_गुणन")?;
    let b = vec_of(arg(&args, 1, "कोण_गुणन")?, "कोण_गुणन")?;
    if a.len() != 3 || b.len() != 3 { return Err("कोण_गुणन(): दोनों 3D सदिश होने चाहिए".into()); }
    Ok(list(vec![
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]))
}

fn parimaan(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "परिमाण")?, "परिमाण")?;
    Ok(Value::Number(a.iter().map(|x| x * x).sum::<f64>().sqrt()))
}

fn samanya(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "सामान्य")?, "सामान्य")?;
    let n = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    if n == 0.0 { return Err("सामान्य(): शून्य सदिश को सामान्य नहीं कर सकते".into()); }
    Ok(list(a.iter().map(|x| x / n).collect()))
}

fn doori(args: Vec<Value>) -> Result<Value, String> {
    let a = vec_of(arg(&args, 0, "दूरी")?, "दूरी")?;
    let b = vec_of(arg(&args, 1, "दूरी")?, "दूरी")?;
    if a.len() != b.len() { return Err("दूरी(): विमाएँ असमान".into()); }
    Ok(Value::Number(a.iter().zip(&b).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()))
}

// ── Matrices ─────────────────────────────────────────────────────────────────

fn aavyuh_gunan(args: Vec<Value>) -> Result<Value, String> {
    let a = mat_of(arg(&args, 0, "आव्यूह_गुणन")?, "आव्यूह_गुणन")?;
    let b = mat_of(arg(&args, 1, "आव्यूह_गुणन")?, "आव्यूह_गुणन")?;
    let (ra, ca) = (a.len(), a.first().map_or(0, |r| r.len()));
    let (rb, cb) = (b.len(), b.first().map_or(0, |r| r.len()));
    if ca != rb { return Err(format!("आव्यूह_गुणन(): {ra}×{ca} और {rb}×{cb} गुणा नहीं हो सकते")); }
    let mut out = vec![vec![0.0; cb]; ra];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..ca).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    Ok(matrix(out))
}

fn parivart(args: Vec<Value>) -> Result<Value, String> {
    let a = mat_of(arg(&args, 0, "परिवर्त")?, "परिवर्त")?;
    let (r, c) = (a.len(), a.first().map_or(0, |x| x.len()));
    let mut out = vec![vec![0.0; r]; c];
    for i in 0..r { for j in 0..c { out[j][i] = a[i][j]; } }
    Ok(matrix(out))
}

fn aavyuh_sadish(args: Vec<Value>) -> Result<Value, String> {
    let a = mat_of(arg(&args, 0, "आव्यूह_सदिश")?, "आव्यूह_सदिश")?;
    let v = vec_of(arg(&args, 1, "आव्यूह_सदिश")?, "आव्यूह_सदिश")?;
    Ok(list(a.iter().map(|row| {
        if row.len() != v.len() { return f64::NAN; }
        row.iter().zip(&v).map(|(x, y)| x * y).sum()
    }).collect()))
}

fn tatsamak(args: Vec<Value>) -> Result<Value, String> {
    let n = match arg(&args, 0, "तत्समक")? { Value::Number(n) => *n as usize, _ => return Err("तत्समक(): आकार (संख्या) अपेक्षित".into()) };
    let mut m = vec![vec![0.0; n]; n];
    for (i, row) in m.iter_mut().enumerate() { row[i] = 1.0; }
    Ok(matrix(m))
}

fn lu_det_inv(a: &[Vec<f64>], want_inv: bool) -> Result<(f64, Option<Vec<Vec<f64>>>), String> {
    let n = a.len();
    if n == 0 || a.iter().any(|r| r.len() != n) { return Err("वर्ग आव्यूह आवश्यक".into()); }
    let mut m: Vec<Vec<f64>> = a.to_vec();
    let mut inv: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();
    let mut det = 1.0;
    for col in 0..n {
        let mut piv = col;
        for r in (col + 1)..n { if m[r][col].abs() > m[piv][col].abs() { piv = r; } }
        if m[piv][col].abs() < 1e-12 { return Ok((0.0, None)); }
        if piv != col { m.swap(piv, col); inv.swap(piv, col); det = -det; }
        let d = m[col][col];
        det *= d;
        for j in 0..n { m[col][j] /= d; inv[col][j] /= d; }
        for r in 0..n {
            if r == col { continue; }
            let f = m[r][col];
            if f != 0.0 {
                for j in 0..n { m[r][j] -= f * m[col][j]; inv[r][j] -= f * inv[col][j]; }
            }
        }
    }
    Ok((det, if want_inv { Some(inv) } else { None }))
}

fn saaranik(args: Vec<Value>) -> Result<Value, String> {
    let a = mat_of(arg(&args, 0, "सारणिक")?, "सारणिक")?;
    let (det, _) = lu_det_inv(&a, false)?;
    Ok(Value::Number(det))
}

fn pratilom(args: Vec<Value>) -> Result<Value, String> {
    let a = mat_of(arg(&args, 0, "प्रतिलोम")?, "प्रतिलोम")?;
    let (det, inv) = lu_det_inv(&a, true)?;
    match inv {
        Some(m) if det != 0.0 => Ok(matrix(m)),
        _ => Err("प्रतिलोम(): आव्यूह व्युत्क्रमणीय नहीं (सारणिक शून्य)".into()),
    }
}

// ── Quaternions [w, x, y, z] ─────────────────────────────────────────────────

fn quat_of(v: &Value, f: &str) -> Result<[f64; 4], String> {
    let q = vec_of(v, f)?;
    if q.len() != 4 { return Err(format!("{f}(): चतुष्क [w,x,y,z] (4 तत्व) अपेक्षित")); }
    Ok([q[0], q[1], q[2], q[3]])
}

fn chatushk_gunan(args: Vec<Value>) -> Result<Value, String> {
    let a = quat_of(arg(&args, 0, "चतुष्क_गुणन")?, "चतुष्क_गुणन")?;
    let b = quat_of(arg(&args, 1, "चतुष्क_गुणन")?, "चतुष्क_गुणन")?;
    Ok(list(vec![
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]))
}

fn chatushk_samanya(args: Vec<Value>) -> Result<Value, String> {
    let q = quat_of(arg(&args, 0, "चतुष्क_सामान्य")?, "चतुष्क_सामान्य")?;
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n == 0.0 { return Err("चतुष्क_सामान्य(): शून्य चतुष्क".into()); }
    Ok(list(q.iter().map(|x| x / n).collect()))
}

fn kon_se_chatushk(args: Vec<Value>) -> Result<Value, String> {
    // (roll, pitch, yaw) radians → quaternion
    let r = match arg(&args, 0, "कोण_से_चतुष्क")? { Value::Number(n) => *n, _ => return Err("कोण_से_चतुष्क(): roll संख्या".into()) };
    let p = match arg(&args, 1, "कोण_से_चतुष्क")? { Value::Number(n) => *n, _ => return Err("कोण_से_चतुष्क(): pitch संख्या".into()) };
    let y = match arg(&args, 2, "कोण_से_चतुष्क")? { Value::Number(n) => *n, _ => return Err("कोण_से_चतुष्क(): yaw संख्या".into()) };
    let (cr, sr) = ((r * 0.5).cos(), (r * 0.5).sin());
    let (cp, sp) = ((p * 0.5).cos(), (p * 0.5).sin());
    let (cy, sy) = ((y * 0.5).cos(), (y * 0.5).sin());
    Ok(list(vec![
        cr * cp * cy + sr * sp * sy,
        sr * cp * cy - cr * sp * sy,
        cr * sp * cy + sr * cp * sy,
        cr * cp * sy - sr * sp * cy,
    ]))
}

fn chatushk_se_kon(args: Vec<Value>) -> Result<Value, String> {
    let q = quat_of(arg(&args, 0, "चतुष्क_से_कोण")?, "चतुष्क_से_कोण")?;
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    let roll = (2.0 * (w * x + y * z)).atan2(1.0 - 2.0 * (x * x + y * y));
    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 { (std::f64::consts::FRAC_PI_2).copysign(sinp) } else { sinp.asin() };
    let yaw = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z));
    Ok(list(vec![roll, pitch, yaw]))
}

fn chatushk_ghumaav(args: Vec<Value>) -> Result<Value, String> {
    let q = quat_of(arg(&args, 0, "चतुष्क_घुमाव")?, "चतुष्क_घुमाव")?;
    let v = vec_of(arg(&args, 1, "चतुष्क_घुमाव")?, "चतुष्क_घुमाव")?;
    if v.len() != 3 { return Err("चतुष्क_घुमाव(): 3D सदिश अपेक्षित".into()); }
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    // v' = q * (0,v) * q^-1, expanded
    let t0 = 2.0 * (y * v[2] - z * v[1]);
    let t1 = 2.0 * (z * v[0] - x * v[2]);
    let t2 = 2.0 * (x * v[1] - y * v[0]);
    Ok(list(vec![
        v[0] + w * t0 + (y * t2 - z * t1),
        v[1] + w * t1 + (z * t0 - x * t2),
        v[2] + w * t2 + (x * t1 - y * t0),
    ]))
}

fn chatushk_prakshep(args: Vec<Value>) -> Result<Value, String> {
    let a = quat_of(arg(&args, 0, "चतुष्क_प्रक्षेप")?, "चतुष्क_प्रक्षेप")?;
    let mut b = quat_of(arg(&args, 1, "चतुष्क_प्रक्षेप")?, "चतुष्क_प्रक्षेप")?;
    let t = match arg(&args, 2, "चतुष्क_प्रक्षेप")? { Value::Number(n) => *n, _ => return Err("चतुष्क_प्रक्षेप(): t संख्या".into()) };
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    if dot < 0.0 { dot = -dot; for x in b.iter_mut() { *x = -*x; } }
    if dot > 0.9995 {
        let r: Vec<f64> = (0..4).map(|i| a[i] + t * (b[i] - a[i])).collect();
        let n = r.iter().map(|x| x * x).sum::<f64>().sqrt();
        return Ok(list(r.iter().map(|x| x / n).collect()));
    }
    let theta0 = dot.acos();
    let theta = theta0 * t;
    let st0 = theta0.sin();
    let s0 = (theta0 - theta).sin() / st0;
    let s1 = theta.sin() / st0;
    Ok(list((0..4).map(|i| s0 * a[i] + s1 * b[i]).collect()))
}

pub fn rekhiy_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("सदिश_योग", sadish_yog),
        ("सदिश_घटा", sadish_ghata),
        ("अदिश_गुणन", adish_gunan),
        ("बिंदु_गुणन", bindu_gunan),
        ("कोण_गुणन", kon_gunan),
        ("परिमाण", parimaan),
        ("सामान्य", samanya),
        ("दूरी", doori),
        ("आव्यूह_गुणन", aavyuh_gunan),
        ("परिवर्त", parivart),
        ("आव्यूह_सदिश", aavyuh_sadish),
        ("तत्समक", tatsamak),
        ("सारणिक", saaranik),
        ("प्रतिलोम", pratilom),
        ("चतुष्क_गुणन", chatushk_gunan),
        ("चतुष्क_सामान्य", chatushk_samanya),
        ("कोण_से_चतुष्क", kon_se_chatushk),
        ("चतुष्क_से_कोण", chatushk_se_kon),
        ("चतुष्क_घुमाव", chatushk_ghumaav),
        ("चतुष्क_प्रक्षेप", chatushk_prakshep),
    ];
    list
}
