//! भारत.सुरक्षा — fault tolerance (Phase 18 H5). Pure Rust.
//! Hamming(7,4) ECC corrects single bit-flips (cosmic-ray SEUs in space),
//! CRC-32 integrity, triple-modular-redundancy majority voting, and a deadline
//! checker. Bits are Lists of 0/1.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

fn bits_of(v: &Value, n: usize, f: &str) -> Result<Vec<u8>, String> {
    match v {
        Value::List(items) if items.len() == n => items.iter().map(|x| match x {
            Value::Number(b) if *b == 0.0 => Ok(0u8),
            Value::Number(b) if *b == 1.0 => Ok(1u8),
            _ => Err(format!("{f}(): बिट 0 या 1 होना चाहिए")),
        }).collect(),
        _ => Err(format!("{f}(): {n} बिट की सूची अपेक्षित")),
    }
}

fn to_bit_list(bits: &[u8]) -> Value {
    Value::List(bits.iter().map(|b| Value::Number(*b as f64)).collect())
}

// Hamming(7,4): data d0..d3 → codeword [p1,p2,d0,p3,d1,d2,d3] (positions 1..7)
fn hamming_koot(args: Vec<Value>) -> Result<Value, String> {
    let d = bits_of(args.first().unwrap_or(&Value::Nil), 4, "हैमिंग_कूट")?;
    let p1 = d[0] ^ d[1] ^ d[3];
    let p2 = d[0] ^ d[2] ^ d[3];
    let p3 = d[1] ^ d[2] ^ d[3];
    Ok(to_bit_list(&[p1, p2, d[0], p3, d[1], d[2], d[3]]))
}

// Decode 7-bit codeword: locate + correct one flipped bit, return the 4 data bits.
fn hamming_vikod(args: Vec<Value>) -> Result<Value, String> {
    let mut c = bits_of(args.first().unwrap_or(&Value::Nil), 7, "हैमिंग_विकोड")?;
    // syndrome (1-based positions)
    let s1 = c[0] ^ c[2] ^ c[4] ^ c[6];
    let s2 = c[1] ^ c[2] ^ c[5] ^ c[6];
    let s3 = c[3] ^ c[4] ^ c[5] ^ c[6];
    let pos = (s1 as usize) | ((s2 as usize) << 1) | ((s3 as usize) << 2);
    if pos != 0 && pos <= 7 { c[pos - 1] ^= 1; } // flip the erroneous bit back
    // data bits are at positions 3,5,6,7 (0-indexed 2,4,5,6)
    Ok(to_bit_list(&[c[2], c[4], c[5], c[6]]))
}

fn crc32(args: Vec<Value>) -> Result<Value, String> {
    let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("सीआरसी32(): वाक्य अपेक्षित".into()) };
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in s.as_bytes() {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    Ok(Value::Number((!crc) as f64))
}

// Triple-modular redundancy: majority of three values (by display equality).
fn bahumat(args: Vec<Value>) -> Result<Value, String> {
    let a = args.first().ok_or("बहुमत(): तीन तर्क आवश्यक")?;
    let b = args.get(1).ok_or("बहुमत(): तीन तर्क आवश्यक")?;
    let c = args.get(2).ok_or("बहुमत(): तीन तर्क आवश्यक")?;
    let (sa, sb, sc) = (format!("{a:?}"), format!("{b:?}"), format!("{c:?}"));
    if sa == sb || sa == sc { Ok(a.clone()) }
    else if sb == sc { Ok(b.clone()) }
    else { Err("बहुमत(): तीनों मान भिन्न — कोई बहुमत नहीं".into()) }
}

fn samay_seema_jaanch(args: Vec<Value>) -> Result<Value, String> {
    let elapsed = match args.first() { Some(Value::Number(n)) => *n, _ => return Err("समय_सीमा_जाँच(): बीता_समय (संख्या)".into()) };
    let limit = match args.get(1) { Some(Value::Number(n)) => *n, _ => return Err("समय_सीमा_जाँच(): सीमा (संख्या)".into()) };
    Ok(Value::Bool(elapsed <= limit))
}

pub fn suraksha_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("हैमिंग_कूट", hamming_koot),
        ("हैमिंग_विकोड", hamming_vikod),
        ("सीआरसी32", crc32),
        ("बहुमत", bahumat),
        ("समय_सीमा_जाँच", samay_seema_jaanch),
    ];
    list
}
