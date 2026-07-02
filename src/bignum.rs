//! Arbitrary-precision integers for LIPI — pure Rust, no external crates.
//!
//! Exposed as the stdlib module `भारत.बड़ी`. Values are passed as decimal strings
//! (LIPI's f64 Number only stays exact up to 2^53, so big integers live as Str and
//! are operated on by these functions). Base-1e9 little-endian limbs.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

const BASE: u64 = 1_000_000_000;
const BASE_DIGITS: usize = 9;

/// Sign-magnitude big integer. `mag` is little-endian base-1e9, no leading zeros.
/// Zero is represented as an empty `mag` with `neg == false`.
#[derive(Clone, PartialEq, Eq)]
pub struct BigInt {
    neg: bool,
    mag: Vec<u32>, // each < BASE
}

impl BigInt {
    fn zero() -> Self { BigInt { neg: false, mag: Vec::new() } }

    fn is_zero(&self) -> bool { self.mag.is_empty() }

    fn trim(mut self) -> Self {
        while self.mag.last() == Some(&0) { self.mag.pop(); }
        if self.mag.is_empty() { self.neg = false; }
        self
    }

    /// Parse a decimal string (optional leading +/-). Errors on invalid input.
    pub fn parse(s: &str) -> Result<BigInt, String> {
        let s = s.trim();
        if s.is_empty() { return Err("बड़ी संख्या: खाली वाक्य".to_string()); }
        let (neg, digits) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("बड़ी संख्या: अमान्य अंक '{s}'"));
        }
        let bytes = digits.as_bytes();
        let mut mag = Vec::new();
        let mut i = bytes.len();
        while i > 0 {
            let start = i.saturating_sub(BASE_DIGITS);
            let chunk = std::str::from_utf8(&bytes[start..i]).unwrap();
            mag.push(chunk.parse::<u32>().unwrap());
            i = start;
        }
        Ok(BigInt { neg, mag }.trim())
    }

    pub fn to_decimal(&self) -> String {
        if self.is_zero() { return "0".to_string(); }
        let mut s = String::new();
        if self.neg { s.push('-'); }
        let n = self.mag.len();
        s.push_str(&self.mag[n - 1].to_string());
        for i in (0..n - 1).rev() {
            s.push_str(&format!("{:09}", self.mag[i]));
        }
        s
    }

    fn from_u64(mut v: u64) -> BigInt {
        let mut mag = Vec::new();
        while v > 0 { mag.push((v % BASE) as u32); v /= BASE; }
        BigInt { neg: false, mag }
    }

    /// Compare magnitudes only: Ordering of |self| vs |other|.
    fn cmp_mag(a: &[u32], b: &[u32]) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        if a.len() != b.len() { return a.len().cmp(&b.len()); }
        for i in (0..a.len()).rev() {
            match a[i].cmp(&b[i]) { Equal => continue, o => return o }
        }
        Equal
    }

    /// Signed comparison → -1 / 0 / 1.
    pub fn cmp(&self, other: &BigInt) -> i32 {
        use std::cmp::Ordering::*;
        match (self.neg, other.neg) {
            (false, true) => 1,
            (true, false) => -1,
            (false, false) => match Self::cmp_mag(&self.mag, &other.mag) { Less => -1, Equal => 0, Greater => 1 },
            (true, true)  => match Self::cmp_mag(&self.mag, &other.mag) { Less => 1, Equal => 0, Greater => -1 },
        }
    }

    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let x = *a.get(i).unwrap_or(&0) as u64;
            let y = *b.get(i).unwrap_or(&0) as u64;
            let s = x + y + carry;
            out.push((s % BASE) as u32);
            carry = s / BASE;
        }
        if carry > 0 { out.push(carry as u32); }
        out
    }

    /// Subtract magnitudes, assuming |a| >= |b|.
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for i in 0..a.len() {
            let x = a[i] as i64;
            let y = *b.get(i).unwrap_or(&0) as i64;
            let mut d = x - y - borrow;
            if d < 0 { d += BASE as i64; borrow = 1; } else { borrow = 0; }
            out.push(d as u32);
        }
        out
    }

    pub fn add(&self, other: &BigInt) -> BigInt {
        if self.neg == other.neg {
            BigInt { neg: self.neg, mag: Self::add_mag(&self.mag, &other.mag) }.trim()
        } else {
            // different signs → subtract smaller magnitude from larger
            match Self::cmp_mag(&self.mag, &other.mag) {
                std::cmp::Ordering::Equal => BigInt::zero(),
                std::cmp::Ordering::Greater => BigInt { neg: self.neg, mag: Self::sub_mag(&self.mag, &other.mag) }.trim(),
                std::cmp::Ordering::Less => BigInt { neg: other.neg, mag: Self::sub_mag(&other.mag, &self.mag) }.trim(),
            }
        }
    }

    pub fn neg(&self) -> BigInt {
        if self.is_zero() { BigInt::zero() } else { BigInt { neg: !self.neg, mag: self.mag.clone() } }
    }

    pub fn sub(&self, other: &BigInt) -> BigInt { self.add(&other.neg()) }

    pub fn mul(&self, other: &BigInt) -> BigInt {
        if self.is_zero() || other.is_zero() { return BigInt::zero(); }
        let mut out = vec![0u64; self.mag.len() + other.mag.len()];
        for (i, &x) in self.mag.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &y) in other.mag.iter().enumerate() {
                let cur = out[i + j] + x as u64 * y as u64 + carry;
                out[i + j] = cur % BASE;
                carry = cur / BASE;
            }
            out[i + other.mag.len()] += carry;
        }
        let mag: Vec<u32> = out.into_iter().map(|v| v as u32).collect();
        BigInt { neg: self.neg != other.neg, mag }.trim()
    }

    /// Truncated division toward zero → (quotient, remainder). Remainder has the
    /// sign of the dividend. Errors on divide-by-zero.
    pub fn divmod(&self, other: &BigInt) -> Result<(BigInt, BigInt), String> {
        if other.is_zero() { return Err("बड़ी संख्या: शून्य से भाग".to_string()); }
        if Self::cmp_mag(&self.mag, &other.mag) == std::cmp::Ordering::Less {
            return Ok((BigInt::zero(), self.clone()));
        }
        // Long division in base 1e9 with binary search per quotient limb.
        let divisor = BigInt { neg: false, mag: other.mag.clone() };
        let mut rem = BigInt::zero();
        let mut quot = vec![0u32; self.mag.len()];
        for i in (0..self.mag.len()).rev() {
            // rem = rem * BASE + self.mag[i]
            rem = rem.mul(&BigInt::from_u64(BASE)).add(&BigInt::from_u64(self.mag[i] as u64));
            // find largest q in [0, BASE) with divisor*q <= rem
            let (mut lo, mut hi) = (0u64, BASE - 1);
            while lo < hi {
                let mid = (lo + hi + 1) / 2;
                let prod = divisor.mul(&BigInt::from_u64(mid));
                if prod.cmp(&rem) <= 0 { lo = mid; } else { hi = mid - 1; }
            }
            quot[i] = lo as u32;
            rem = rem.sub(&divisor.mul(&BigInt::from_u64(lo)));
        }
        let q = BigInt { neg: self.neg != other.neg, mag: quot }.trim();
        let r = BigInt { neg: self.neg, mag: rem.mag }.trim();
        Ok((q, r))
    }

    /// self ^ exp for non-negative exp (binary exponentiation).
    pub fn pow(&self, mut exp: u64) -> BigInt {
        let mut result = BigInt::from_u64(1);
        let mut base = self.clone();
        while exp > 0 {
            if exp & 1 == 1 { result = result.mul(&base); }
            exp >>= 1;
            if exp > 0 { base = base.mul(&base); }
        }
        result
    }
}

// ── Stdlib registry ──────────────────────────────────────────────────────────

fn arg_str(args: &[Value], idx: usize, fname: &str) -> Result<String, String> {
    match args.get(idx) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(format!("{}", *n as i64)),
        Some(other) => Err(format!("{fname}(): वाक्य/संख्या अपेक्षित, मिला: {other}")),
        None => Err(format!("{fname}(): पर्याप्त तर्क नहीं")),
    }
}

fn parse2(args: &[Value], fname: &str) -> Result<(BigInt, BigInt), String> {
    let a = BigInt::parse(&arg_str(args, 0, fname)?)?;
    let b = BigInt::parse(&arg_str(args, 1, fname)?)?;
    Ok((a, b))
}

fn maha_jod(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_जोड़")?;
    Ok(Value::Str(a.add(&b).to_decimal()))
}
fn maha_ghata(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_घटा")?;
    Ok(Value::Str(a.sub(&b).to_decimal()))
}
fn maha_guna(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_गुणा")?;
    Ok(Value::Str(a.mul(&b).to_decimal()))
}
fn maha_bhag(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_भाग")?;
    Ok(Value::Str(a.divmod(&b)?.0.to_decimal()))
}
fn maha_shesh(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_शेष")?;
    Ok(Value::Str(a.divmod(&b)?.1.to_decimal()))
}
fn maha_tulna(args: Vec<Value>) -> Result<Value, String> {
    let (a, b) = parse2(&args, "महा_तुलना")?;
    Ok(Value::Number(a.cmp(&b) as f64))
}
fn maha_ghat(args: Vec<Value>) -> Result<Value, String> {
    let a = BigInt::parse(&arg_str(&args, 0, "महा_घात")?)?;
    let e = match args.get(1) {
        Some(Value::Number(n)) if *n >= 0.0 => *n as u64,
        Some(Value::Str(s)) => s.trim().parse::<u64>().map_err(|_| "महा_घात(): घातांक ऋणेतर पूर्णांक होना चाहिए".to_string())?,
        _ => return Err("महा_घात(): घातांक ऋणेतर पूर्णांक होना चाहिए".to_string()),
    };
    Ok(Value::Str(a.pow(e).to_decimal()))
}
fn maha_bhajya(args: Vec<Value>) -> Result<Value, String> {
    // factorial of a non-negative integer
    let n = match args.first() {
        Some(Value::Number(n)) if *n >= 0.0 => *n as u64,
        Some(Value::Str(s)) => s.trim().parse::<u64>().map_err(|_| "महा_भाज्य(): ऋणेतर पूर्णांक चाहिए".to_string())?,
        _ => return Err("महा_भाज्य(): ऋणेतर पूर्णांक चाहिए".to_string()),
    };
    let mut acc = BigInt::from_u64(1);
    for i in 2..=n { acc = acc.mul(&BigInt::from_u64(i)); }
    Ok(Value::Str(acc.to_decimal()))
}

pub fn badi_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("महा_जोड़", maha_jod),
        ("महा_घटा", maha_ghata),
        ("महा_गुणा", maha_guna),
        ("महा_भाग", maha_bhag),
        ("महा_शेष", maha_shesh),
        ("महा_तुलना", maha_tulna),
        ("महा_घात", maha_ghat),
        ("महा_भाज्य", maha_bhajya),
    ];
    list
}
