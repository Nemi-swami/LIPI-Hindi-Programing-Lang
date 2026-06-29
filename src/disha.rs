//! भारत.दिशा — navigation & geodesy (Phase 18 H4). Pure Rust.
//! Great-circle distance, bearing, destination, and ECEF transforms. All lat/lon
//! inputs/outputs in degrees. Earth treated as a sphere (WGS-84 radius) for
//! haversine; ECEF uses the WGS-84 ellipsoid.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

const R_KM: f64 = 6371.0088;          // mean Earth radius (km)
const WGS_A: f64 = 6_378_137.0;       // semi-major axis (m)
const WGS_E2: f64 = 0.006_694_379_990_14; // first eccentricity squared

fn num(v: Option<&Value>, f: &str) -> Result<f64, String> {
    match v { Some(Value::Number(n)) => Ok(*n), _ => Err(format!("{f}(): संख्या अपेक्षित")) }
}

fn mahavritta_doori(args: Vec<Value>) -> Result<Value, String> {
    let lat1 = num(args.first(), "महावृत्त_दूरी")?.to_radians();
    let lon1 = num(args.get(1), "महावृत्त_दूरी")?.to_radians();
    let lat2 = num(args.get(2), "महावृत्त_दूरी")?.to_radians();
    let lon2 = num(args.get(3), "महावृत्त_दूरी")?.to_radians();
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    Ok(Value::Number(R_KM * c))
}

fn disha_kon(args: Vec<Value>) -> Result<Value, String> {
    let lat1 = num(args.first(), "दिशा_कोण")?.to_radians();
    let lon1 = num(args.get(1), "दिशा_कोण")?.to_radians();
    let lat2 = num(args.get(2), "दिशा_कोण")?.to_radians();
    let lon2 = num(args.get(3), "दिशा_कोण")?.to_radians();
    let dlon = lon2 - lon1;
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
    let brng = y.atan2(x).to_degrees();
    Ok(Value::Number((brng + 360.0) % 360.0))
}

fn gantavya(args: Vec<Value>) -> Result<Value, String> {
    let lat = num(args.first(), "गंतव्य")?.to_radians();
    let lon = num(args.get(1), "गंतव्य")?.to_radians();
    let brng = num(args.get(2), "गंतव्य")?.to_radians();
    let dist_km = num(args.get(3), "गंतव्य")?;
    let dr = dist_km / R_KM;
    let lat2 = (lat.sin() * dr.cos() + lat.cos() * dr.sin() * brng.cos()).asin();
    let lon2 = lon + (brng.sin() * dr.sin() * lat.cos()).atan2(dr.cos() - lat.sin() * lat2.sin());
    Ok(Value::List(vec![Value::Number(lat2.to_degrees()), Value::Number(lon2.to_degrees())]))
}

fn ecef(args: Vec<Value>) -> Result<Value, String> {
    let lat = num(args.first(), "ईसीईएफ")?.to_radians();
    let lon = num(args.get(1), "ईसीईएफ")?.to_radians();
    let alt = num(args.get(2), "ईसीईएफ")?; // metres
    let n = WGS_A / (1.0 - WGS_E2 * lat.sin().powi(2)).sqrt();
    let x = (n + alt) * lat.cos() * lon.cos();
    let y = (n + alt) * lat.cos() * lon.sin();
    let z = (n * (1.0 - WGS_E2) + alt) * lat.sin();
    Ok(Value::List(vec![Value::Number(x), Value::Number(y), Value::Number(z)]))
}

pub fn disha_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("महावृत्त_दूरी", mahavritta_doori),
        ("दिशा_कोण", disha_kon),
        ("गंतव्य", gantavya),
        ("ईसीईएफ", ecef),
    ];
    list
}
