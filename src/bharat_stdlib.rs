//! भारत Standard Library — LIPI 2.0
//!
//! Pure Rust, zero external crates, zero HTTP calls.
//! Stubs include real validation logic; network I/O wired in Phase 4.
//!
//! Modules
//!   भारत.पहचान  — Aadhaar (Verhoeff), PAN, IFSC validation
//!   भारत.संख्या  — Indian number system, GST, EMI
//!   भारत.भुगतान  — UPI format validation + stub transaction
//!   भारत.भाषा   — Devanagari detection, romanisation, word count
//!   भारत.गणित   — Ancient Hindu mathematics (Āryabhaṭa, Bhāskara, Brahmagupta, Virahanka)

use crate::interpreter::Value;

pub type NativeFn = fn(Vec<Value>) -> Result<Value, String>;
pub type Registry  = Vec<(&'static str, NativeFn)>;

// ════════════════════════════════════════════════════════════════
// MODULE 1 — भारत.पहचान  (Identity validation)
// ════════════════════════════════════════════════════════════════

// ── Verhoeff tables ────────────────────────────────────────────
// Source: Verhoeff (1969) "Error Detecting Decimal Codes".
// Aadhaar UIDAI spec mandates the Verhoeff check digit algorithm.

const D: [[u8; 10]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 2, 3, 4, 0, 6, 7, 8, 9, 5],
    [2, 3, 4, 0, 1, 7, 8, 9, 5, 6],
    [3, 4, 0, 1, 2, 8, 9, 5, 6, 7],
    [4, 0, 1, 2, 3, 9, 5, 6, 7, 8],
    [5, 9, 8, 7, 6, 0, 4, 3, 2, 1],
    [6, 5, 9, 8, 7, 1, 0, 4, 3, 2],
    [7, 6, 5, 9, 8, 2, 1, 0, 4, 3],
    [8, 7, 6, 5, 9, 3, 2, 1, 0, 4],
    [9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];

const P: [[u8; 10]; 8] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 5, 7, 6, 2, 8, 3, 0, 9, 4],
    [5, 8, 0, 3, 7, 9, 6, 1, 4, 2],
    [8, 9, 1, 6, 0, 4, 3, 5, 2, 7],
    [9, 4, 5, 3, 1, 2, 6, 8, 7, 0],
    [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
    [2, 7, 9, 3, 8, 0, 6, 4, 1, 5],
    [7, 0, 4, 6, 9, 1, 3, 2, 5, 8],
];

const INV: [u8; 10] = [0, 4, 3, 2, 1, 9, 8, 7, 6, 5];

/// Validate a digit string against the Verhoeff check digit.
/// Processes digits right-to-left; returns true iff check sum == 0.
fn verhoeff_validate(digits: &[u8]) -> bool {
    let mut c: u8 = 0;
    for (i, &d) in digits.iter().rev().enumerate() {
        c = D[c as usize][P[i % 8][d as usize] as usize];
    }
    c == 0
}

/// Compute the Verhoeff check digit for a string of digits (without it).
pub fn verhoeff_generate(number: &str) -> u8 {
    let mut digits: Vec<u8> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();
    digits.push(0);
    let mut c: u8 = 0;
    for (i, &d) in digits.iter().rev().enumerate() {
        c = D[c as usize][P[i % 8][d as usize] as usize];
    }
    INV[c as usize]
}

/// Validate an Aadhaar number (12 digits, Verhoeff check).
pub fn aadhaar_valid(s: &str) -> bool {
    let digits: Vec<u8> = s.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();
    if digits.len() != 12 { return false; }
    if digits[0] == 0 || digits[0] == 1 { return false; } // UIDAI rule
    verhoeff_validate(&digits)
}

/// Validate PAN: [A-Z]{5}[0-9]{4}[A-Z]
pub fn pan_valid(s: &str) -> bool {
    let s = s.trim();
    if s.len() != 10 { return false; }
    let b = s.as_bytes();
    b[..5].iter().all(|c| c.is_ascii_uppercase())
        && b[5..9].iter().all(|c| c.is_ascii_digit())
        && b[9].is_ascii_uppercase()
}

/// Validate IFSC: [A-Z]{4}0[A-Z0-9]{6}
pub fn ifsc_valid(s: &str) -> bool {
    let s = s.trim();
    if s.len() != 11 { return false; }
    let b = s.as_bytes();
    b[..4].iter().all(|c| c.is_ascii_uppercase())
        && b[4] == b'0'
        && b[5..].iter().all(|c| c.is_ascii_alphanumeric())
}

pub fn pehchaan_registry() -> Registry {
    vec![
        ("आधार_जाँचो", fn_aadhaar as NativeFn),
        ("pan_जाँचो",   fn_pan     as NativeFn),
        ("ifsc_जाँचो",  fn_ifsc    as NativeFn),
    ]
}

fn fn_aadhaar(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(aadhaar_valid(&need_str(&args, 0, "आधार_जाँचो")?)))
}
fn fn_pan(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(pan_valid(&need_str(&args, 0, "pan_जाँचो")?)))
}
fn fn_ifsc(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(ifsc_valid(&need_str(&args, 0, "ifsc_जाँचो")?)))
}

// ════════════════════════════════════════════════════════════════
// MODULE 2 — भारत.संख्या  (Indian number system)
// ════════════════════════════════════════════════════════════════

const LAKH:  f64 = 100_000.0;
const CRORE: f64 = 10_000_000.0;

/// 1_200_000 → "12 लाख"
pub fn format_lakh(n: f64) -> String {
    if n.abs() >= LAKH {
        let v = n / LAKH;
        if v.fract() == 0.0 { format!("{} लाख", v as i64) }
        else                 { format!("{:.2} लाख", v) }
    } else {
        format_num(n)
    }
}

/// 12_345_678 → "1.23 करोड़"
pub fn format_crore(n: f64) -> String {
    if n.abs() >= CRORE {
        let v = n / CRORE;
        if v.fract() == 0.0 { format!("{} करोड़", v as i64) }
        else                 { format!("{:.2} करोड़", v) }
    } else {
        format_lakh(n)
    }
}

/// 50_000 → "₹50,000"  (Indian comma grouping)
pub fn format_rupees(n: f64) -> String {
    let neg   = n < 0.0;
    let whole = n.abs() as u64;
    let paise = ((n.abs() - whole as f64) * 100.0).round() as u64;
    let s     = indian_commas(&whole.to_string());
    let prefix = if neg { "-₹" } else { "₹" };
    if paise > 0 { format!("{}{}.{:02}", prefix, s, paise) }
    else         { format!("{}{}", prefix, s) }
}

/// Add GST: gst_add(10000, 18) → 11800.0
pub fn gst_add(amount: f64, rate_pct: f64) -> f64 {
    amount * (1.0 + rate_pct / 100.0)
}

/// EMI = P·r·(1+r)^n / ((1+r)^n − 1),  rate_annual in %, months = n
pub fn emi_calc(principal: f64, rate_annual_pct: f64, months: f64) -> f64 {
    if principal <= 0.0 || months <= 0.0 { return 0.0; }
    if rate_annual_pct == 0.0 {
        return round2(principal / months);
    }
    let r   = rate_annual_pct / 1200.0;  // monthly rate (decimal)
    let pow = (1.0 + r).powf(months);
    round2(principal * r * pow / (pow - 1.0))
}

fn round2(n: f64) -> f64 { (n * 100.0).round() / 100.0 }

/// Indian comma grouping: last 3 digits, then groups of 2
fn indian_commas(s: &str) -> String {
    let len = s.len();
    if len <= 3 { return s.to_string(); }

    let (left, right) = s.split_at(len - 3);
    let mut result = right.to_string();

    let chars: Vec<char> = left.chars().collect();
    let mut i = chars.len();
    while i > 0 {
        let start = if i > 2 { i - 2 } else { 0 };
        let group: String = chars[start..i].iter().collect();
        result = format!("{},{}", group, result);
        i = start;
    }
    result
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) }
    else { format!("{}", n) }
}

pub fn sankhya_registry() -> Registry {
    vec![
        ("लाख_में",    fn_lakh_mein  as NativeFn),
        ("करोड़_में",  fn_crore_mein as NativeFn),
        ("रुपये_में",  fn_rupees_mein as NativeFn),
        ("gst_जोड़ो",  fn_gst_add    as NativeFn),
        ("emi_निकालो", fn_emi_calc   as NativeFn),
    ]
}

fn fn_lakh_mein(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(format_lakh(need_num(&args, 0, "लाख_में")?)))
}
fn fn_crore_mein(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(format_crore(need_num(&args, 0, "करोड़_में")?)))
}
fn fn_rupees_mein(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(format_rupees(need_num(&args, 0, "रुपये_में")?)))
}
fn fn_gst_add(args: Vec<Value>) -> Result<Value, String> {
    let amount = need_num(&args, 0, "gst_जोड़ो")?;
    let rate   = need_num(&args, 1, "gst_जोड़ो")?;
    Ok(Value::Number(gst_add(amount, rate)))
}
fn fn_emi_calc(args: Vec<Value>) -> Result<Value, String> {
    let p = need_num(&args, 0, "emi_निकालो")?;
    let r = need_num(&args, 1, "emi_निकालो")?;
    let n = need_num(&args, 2, "emi_निकालो")?;
    Ok(Value::Number(emi_calc(p, r, n)))
}

// ════════════════════════════════════════════════════════════════
// MODULE 3 — भारत.भुगतान  (UPI stub)
// ════════════════════════════════════════════════════════════════

// Known UPI bank handles (NPCI registered)
const UPI_HANDLES: &[&str] = &[
    "sbi", "hdfc", "icici", "axis", "ybl", "okaxis", "oksbi",
    "okhdfcbank", "okicici", "paytm", "apl", "ibl", "federal",
    "rbl", "kotak", "idfcfirst", "sc", "hsbc", "citi", "jpmorgan",
    "upi", "rapl", "airtel", "timecosmos", "pnb", "bob", "canara",
    "union", "idbi", "boi", "indus", "postpay", "freecharge",
];

/// Validate a UPI Virtual Payment Address: localpart@bankhandle
pub fn upi_valid(vpa: &str) -> bool {
    let mut parts = vpa.splitn(2, '@');
    let local = match parts.next() { Some(l) => l, None => return false };
    let handle = match parts.next() { Some(h) => h, None => return false };

    // local: 1–256 chars, alphanumeric + . - _
    if !(1..=256).contains(&local.len()) { return false; }
    if !local.chars().all(|c| c.is_alphanumeric() || ".·-_".contains(c)) {
        return false;
    }
    UPI_HANDLES.contains(&handle)
}

/// Deterministic hash for stub transaction IDs (djb2)
fn djb2(s: &str) -> u64 {
    s.bytes().fold(5381u64, |h, b| h.wrapping_mul(33).wrapping_add(b as u64))
}

/// Stub UPI payment — validates both VPAs, generates fake TXN ID
pub fn upi_send(from: &str, to: &str, amount: f64, note: &str) -> Result<String, String> {
    if !upi_valid(from) {
        return Err(format!("अमान्य UPI पता (भेजने वाला): '{}'", from));
    }
    if !upi_valid(to) {
        return Err(format!("अमान्य UPI पता (पाने वाला): '{}'", to));
    }
    if amount <= 0.0 {
        return Err("राशि शून्य से अधिक होनी चाहिए".into());
    }
    if amount > 100_000.0 {
        return Err("UPI सीमा पार: ₹1 लाख प्रति लेनदेन".into());
    }

    let seed  = format!("{from}{to}{amount:.2}{note}");
    let txn   = format!("TXN2025LIPI{:08X}", djb2(&seed) & 0xFFFF_FFFF);

    Ok(format!("✓ ₹{:.0} {} → {} | {} | {}", amount, from, to, note, txn))
}

pub fn bhugtaan_registry() -> Registry {
    vec![
        ("upi_वैध_है", fn_upi_valid as NativeFn),
        ("upi_भेजो",   fn_upi_send  as NativeFn),
    ]
}

fn fn_upi_valid(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(upi_valid(&need_str(&args, 0, "upi_वैध_है")?)))
}
fn fn_upi_send(args: Vec<Value>) -> Result<Value, String> {
    let from   = need_str(&args, 0, "upi_भेजो")?;
    let to     = need_str(&args, 1, "upi_भेजो")?;
    let amount = need_num(&args, 2, "upi_भेजो")?;
    let note   = if args.len() > 3 { need_str(&args, 3, "upi_भेजो")? } else { String::new() };
    upi_send(&from, &to, amount, &note).map(Value::Str)
}

// ════════════════════════════════════════════════════════════════
// MODULE 4 — भारत.भाषा  (Language utilities)
// ════════════════════════════════════════════════════════════════

pub fn is_devanagari(s: &str) -> bool {
    s.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
}

/// Hunterian transliteration (used by Survey of India, Indian govt.)
pub fn romanize(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let r: &str = match ch {
            // Independent vowels
            'अ' => "a",  'आ' => "aa", 'इ' => "i",   'ई' => "ee",
            'उ' => "u",  'ऊ' => "oo", 'ए' => "e",   'ऐ' => "ai",
            'ओ' => "o",  'औ' => "au", 'ऋ' => "ri",
            // Velars
            'क' => "k",  'ख' => "kh", 'ग' => "g",  'घ' => "gh", 'ङ' => "n",
            // Palatals
            'च' => "ch", 'छ' => "chh",'ज' => "j",  'झ' => "jh", 'ञ' => "n",
            // Retroflexes
            'ट' => "t",  'ठ' => "th", 'ड' => "d",  'ढ' => "dh", 'ण' => "n",
            // Dentals
            'त' => "t",  'थ' => "th", 'द' => "d",  'ध' => "dh", 'न' => "n",
            // Labials
            'प' => "p",  'फ' => "ph", 'ब' => "b",  'भ' => "bh", 'म' => "m",
            // Sonorants
            'य' => "y",  'र' => "r",  'ल' => "l",  'व' => "v",
            // Sibilants + aspirate
            'श' => "sh", 'ष' => "sh", 'स' => "s",  'ह' => "h",
            // Vowel matras (diacritics attached to consonants)
            'ा' => "aa", 'ि' => "i",  'ी' => "ee",
            'ु' => "u",  'ू' => "oo", 'े' => "e",  'ो' => "o",
            'ै' => "ai", 'ौ' => "au", 'ृ' => "ri",
            'ं' => "n",  'ः' => "h",
            '्' => "",    // virama — suppresses inherent 'a'
            ' ' => " ",
            other => { out.push(other); continue; }
        };
        out.push_str(r);
    }
    out
}

/// Word count respecting Devanagari punctuation (danda ।, double-danda ॥)
pub fn word_count(s: &str) -> f64 {
    s.split(|c: char| c.is_whitespace() || c == '।' || c == '॥')
        .filter(|w| !w.is_empty())
        .count() as f64
}

pub fn bhasha_registry() -> Registry {
    vec![
        ("devanagari_है", fn_is_dev      as NativeFn),
        ("roman_में",      fn_romanize    as NativeFn),
        ("शब्द_गिनो",     fn_word_count  as NativeFn),
    ]
}

fn fn_is_dev(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Bool(is_devanagari(&need_str(&args, 0, "devanagari_है")?)))
}
fn fn_romanize(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(romanize(&need_str(&args, 0, "roman_में")?)))
}
fn fn_word_count(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(word_count(&need_str(&args, 0, "शब्द_गिनो")?)))
}

// ════════════════════════════════════════════════════════════════
// ARGUMENT HELPERS
// ════════════════════════════════════════════════════════════════

pub fn need_str(args: &[Value], idx: usize, fname: &str) -> Result<String, String> {
    match args.get(idx) {
        Some(Value::Str(s))    => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(format_num(*n)),
        Some(Value::Bool(b))   => Ok(if *b { "सत्य".into() } else { "असत्य".into() }),
        Some(other) => Err(format!("{}: arg {} — वाक्य अपेक्षित, मिला {:?}", fname, idx+1, other)),
        None        => Err(format!("{}: {} वाँ तर्क आवश्यक है", fname, idx+1)),
    }
}

pub fn need_num(args: &[Value], idx: usize, fname: &str) -> Result<f64, String> {
    match args.get(idx) {
        Some(Value::Number(n)) => Ok(*n),
        Some(Value::Str(s))    => s.trim().parse::<f64>()
            .map_err(|_| format!("{}: '{}' संख्या नहीं है", fname, s)),
        Some(other) => Err(format!("{}: arg {} — संख्या अपेक्षित, मिला {:?}", fname, idx+1, other)),
        None        => Err(format!("{}: {} वाँ तर्क आवश्यक है", fname, idx+1)),
    }
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.सांख्यिकी  (Statistics, Phase 17)
//
// Functions take a सूची (List) of numbers. Empty lists raise a catchable
// error; non-number elements raise a catchable error.
// ════════════════════════════════════════════════════════════════

/// Extract a Vec<f64> from a single List argument; errors on empty / non-number.
fn need_num_list(args: &[Value], fname: &str) -> Result<Vec<f64>, String> {
    match args.first() {
        Some(Value::List(items)) => {
            if items.is_empty() {
                return Err(format!("{}: खाली सूची पर गणना नहीं हो सकती", fname));
            }
            items.iter().map(|v| match v {
                Value::Number(n) => Ok(*n),
                other => Err(format!("{}: सूची में संख्या अपेक्षित, मिला {:?}", fname, other)),
            }).collect()
        }
        Some(other) => Err(format!("{}: सूची अपेक्षित, मिला {:?}", fname, other)),
        None => Err(format!("{}: एक सूची तर्क आवश्यक", fname)),
    }
}

fn stat_madhya(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "माध्य")?;
    Ok(Value::Number(v.iter().sum::<f64>() / v.len() as f64))
}

fn stat_madhyika(args: Vec<Value>) -> Result<Value, String> {
    let mut v = need_num_list(&args, "माध्यिका")?;
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    let med = if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 };
    Ok(Value::Number(med))
}

fn stat_bahulak(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "बहुलक")?;
    // Most frequent value; ties broken by smallest value.
    let mut best = v[0];
    let mut best_count = 0usize;
    for &x in &v {
        let c = v.iter().filter(|&&y| (y - x).abs() < f64::EPSILON).count();
        if c > best_count || (c == best_count && x < best) {
            best = x;
            best_count = c;
        }
    }
    Ok(Value::Number(best))
}

fn stat_prasaran(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "प्रसरण")?;
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64;
    Ok(Value::Number(var))
}

fn stat_manak_vichalan(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "मानक_विचलन")?;
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / v.len() as f64;
    Ok(Value::Number(var.sqrt()))
}

fn stat_yog(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "योग")?;
    Ok(Value::Number(v.iter().sum()))
}

fn stat_nyuntam(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "न्यूनतम")?;
    Ok(Value::Number(v.iter().cloned().fold(f64::INFINITY, f64::min)))
}

fn stat_adhiktam(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "अधिकतम")?;
    Ok(Value::Number(v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
}

fn stat_parisar(args: Vec<Value>) -> Result<Value, String> {
    let v = need_num_list(&args, "परिसर")?;
    let mn = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Ok(Value::Number(mx - mn))
}

pub fn sankhyiki_registry() -> Registry {
    vec![
        ("माध्य", stat_madhya),
        ("माध्यिका", stat_madhyika),
        ("बहुलक", stat_bahulak),
        ("प्रसरण", stat_prasaran),
        ("मानक_विचलन", stat_manak_vichalan),
        ("योग", stat_yog),
        ("न्यूनतम", stat_nyuntam),
        ("अधिकतम", stat_adhiktam),
        ("परिसर", stat_parisar),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE 5 — भारत.गणित  (Ancient Hindu Mathematics)
//
// Sources:
//   Āryabhaṭīya (Āryabhaṭa, 499 CE)  — jyā/kojyā (sin/cos), π approximation
//   Brāhmasphuṭasiddhānta (Brahmagupta, 628 CE) — cyclic quad area, zero rules
//   Līlāvatī (Bhāskara II, 1150 CE)   — combinations, permutations
//   Chandahśāstra (Virahanka, ~600 CE) — sequence now called Fibonacci
// ════════════════════════════════════════════════════════════════

// ── Āryabhaṭa trigonometry (499 CE) ───────────────────────────
// Āryabhaṭa called sine "jyā" (ज्या). Arabic scholars transliterated
// "jyā" → "jaib" (pocket), Latin translators rendered it "sinus" → "sine".
// The word "sine" is literally Āryabhaṭa's Sanskrit term.

fn fn_jya(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(need_num(&args, 0, "ज्या")?.sin()))
}
fn fn_kojya(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(need_num(&args, 0, "कोज्या")?.cos()))
}
fn fn_sparshjya(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "स्पर्शज्या")?;
    Ok(Value::Number(x.tan()))
}
fn fn_vyutkram_jya(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "व्युत्क्रम_ज्या")?;
    if !(-1.0..=1.0).contains(&x) { return Err("व्युत्क्रम_ज्या(): -1 से 1 के बीच होना चाहिए".into()); }
    Ok(Value::Number(x.asin()))
}
fn fn_vyutkram_kojya(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "व्युत्क्रम_कोज्या")?;
    if !(-1.0..=1.0).contains(&x) { return Err("व्युत्क्रम_कोज्या(): -1 से 1 के बीच होना चाहिए".into()); }
    Ok(Value::Number(x.acos()))
}
fn fn_vyutkram_sparsh(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(need_num(&args, 0, "व्युत्क्रम_स्पर्श")?.atan()))
}

// ── Brahmagupta — cyclic quadrilateral area (628 CE) ──────────
// Area = √((s-a)(s-b)(s-c)(s-d))  where s = (a+b+c+d)/2
// Generalises Heron's formula for triangles.
fn fn_brahma_kshetra(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "ब्रह्मगुप्त_क्षेत्र")?;
    let b = need_num(&args, 1, "ब्रह्मगुप्त_क्षेत्र")?;
    let c = need_num(&args, 2, "ब्रह्मगुप्त_क्षेत्र")?;
    let d = need_num(&args, 3, "ब्रह्मगुप्त_क्षेत्र")?;
    let s = (a + b + c + d) / 2.0;
    let area = ((s-a)*(s-b)*(s-c)*(s-d)).sqrt();
    Ok(Value::Number(area))
}

// Heron's triangle area (special case, also in Āryabhaṭīya)
fn fn_heron(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "हेरॉन_क्षेत्र")?;
    let b = need_num(&args, 1, "हेरॉन_क्षेत्र")?;
    let c = need_num(&args, 2, "हेरॉन_क्षेत्र")?;
    let s = (a + b + c) / 2.0;
    Ok(Value::Number((s*(s-a)*(s-b)*(s-c)).sqrt()))
}

// ── Bhāskara II — Līlāvatī combinatorics (1150 CE) ────────────
// Bhāskara wrote Līlāvatī as math problems for his daughter.
// Chapter on permutations and combinations is still cited today.

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

fn fn_sanyojan(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "संयोजन")? as u64;
    let r = need_num(&args, 1, "संयोजन")? as u64;
    if r > n { return Ok(Value::Number(0.0)); }
    let result = factorial(n) / (factorial(r) * factorial(n - r));
    Ok(Value::Number(result as f64))
}

fn fn_kramchay(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "क्रमचय")? as u64;
    let r = need_num(&args, 1, "क्रमचय")? as u64;
    if r > n { return Ok(Value::Number(0.0)); }
    Ok(Value::Number((factorial(n) / factorial(n - r)) as f64))
}

// ── Virahanka sequence (Chandahśāstra, ~600 CE) ────────────────
// Virahanka described this sequence studying Sanskrit prosody metres.
// Fibonacci published it in Europe 600 years later (1202 CE).
fn fn_virahanka(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "विरहांक")? as u64;
    if n == 0 { return Ok(Value::Number(0.0)); }
    if n == 1 { return Ok(Value::Number(1.0)); }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 2..=n { let t = a + b; a = b; b = t; }
    Ok(Value::Number(b as f64))
}

// ── Logarithms ─────────────────────────────────────────────────
fn fn_laghuganak(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "लघुगणक")?;
    if x <= 0.0 { return Err("लघुगणक(): धनात्मक संख्या चाहिए".into()); }
    Ok(Value::Number(x.ln()))
}
fn fn_laghuganak_das(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "लघुगणक_दस")?;
    if x <= 0.0 { return Err("लघुगणक_दस(): धनात्मक संख्या चाहिए".into()); }
    Ok(Value::Number(x.log10()))
}
fn fn_ghataank(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(need_num(&args, 0, "घातांक")?.exp()))
}

// ── Āryabhaṭa's π approximation ────────────────────────────────
// Āryabhaṭīya 2.10: "Add 4 to 100, multiply by 8, add 62000 → 62832.
// This is the approximate circumference of a circle with diameter 20000."
// Gives π ≈ 62832/20000 = 3.1416 — accurate to 4 decimal places.
fn fn_aaryabhata_pai(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(std::f64::consts::PI))
}

// ── New गणित additions ─────────────────────────────────────────

// GCD helper (used by महावीर_भिन्न and कुट्टक)
fn gcd_inner(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd_inner(b, a % b) }
}

// Extended GCD: returns (gcd, x, y) such that a*x + b*y = gcd
fn ext_gcd_inner(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 { return (a.abs(), if a >= 0 { 1 } else { -1 }, 0); }
    let (g, x1, y1) = ext_gcd_inner(b, a % b);
    (g, y1, x1 - (a / b) * y1)
}

// कुट्टक: Extended GCD (Aryabhata 499 CE) — returns [x, y, gcd]
fn fn_kuttak(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "कुट्टक")? as i64;
    let b = need_num(&args, 1, "कुट्टक")? as i64;
    let (g, x, y) = ext_gcd_inner(a, b);
    Ok(Value::List(vec![
        Value::Number(x as f64),
        Value::Number(y as f64),
        Value::Number(g as f64),
    ]))
}

// आर्यभट_योग: sum 1+2+…+n = n(n+1)/2
fn fn_aryabhata_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "आर्यभट_योग")?;
    Ok(Value::Number(n * (n + 1.0) / 2.0))
}

// वर्ग_योग: sum of squares 1²+2²+…+n² = n(n+1)(2n+1)/6
fn fn_varga_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "वर्ग_योग")?;
    Ok(Value::Number(n * (n + 1.0) * (2.0 * n + 1.0) / 6.0))
}

// घन_योग: sum of cubes 1³+…+n³ = [n(n+1)/2]²
fn fn_ghana_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "घन_योग")?;
    let half = n * (n + 1.0) / 2.0;
    Ok(Value::Number(half * half))
}

// श्रीधर_सूत्र: quadratic roots (Shridhara 870 CE)
// Returns [] for no real roots, [x] for equal roots, [x1, x2] for two distinct roots.
// Shridhara's original: "multiply both sides by 4a, add b², take sqrt" — completing the square.
fn fn_shridhar(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "श्रीधर_सूत्र")?;
    let b = need_num(&args, 1, "श्रीधर_सूत्र")?;
    let c = need_num(&args, 2, "श्रीधर_सूत्र")?;
    if a == 0.0 { return Err("श्रीधर_सूत्र: a शून्य नहीं हो सकता".into()); }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return Ok(Value::List(vec![]));  // वास्तविक मूल नहीं
    }
    let r1 = (-b + disc.sqrt()) / (2.0 * a);
    if disc == 0.0 {
        return Ok(Value::List(vec![Value::Number(r1)]));  // एकमात्र मूल
    }
    let r2 = (-b - disc.sqrt()) / (2.0 * a);
    Ok(Value::List(vec![Value::Number(r1), Value::Number(r2)]))
}

// बखशाली_मूल: iterative sqrt via Newton-Raphson (Bakshali Manuscript ~3rd-7th CE)
fn fn_bakshali_sqrt(args: Vec<Value>) -> Result<Value, String> {
    let q = need_num(&args, 0, "बखशाली_मूल")?;
    if q < 0.0 { return Err("बखशाली_मूल: ऋणात्मक संख्या का मूल नहीं".into()); }
    if q == 0.0 { return Ok(Value::Number(0.0)); }
    let mut x = q.sqrt();
    for _ in 0..5 { x = (x + q / x) / 2.0; }
    Ok(Value::Number(x))
}

// ब्रह्मगुप्त_अंतर्वेशन: 2nd-order finite-difference interpolation (Khandakhadyaka 665 CE)
// Given f0 at t=0, f1 at t=1, f2 at t=2 — interpolate at fractional t.
// First recorded 2nd-order interpolation formula in history; used for planetary positions.
fn fn_bg_interpolate(args: Vec<Value>) -> Result<Value, String> {
    let fname = "ब्रह्मगुप्त_अंतर्वेशन";
    let f0 = need_num(&args, 0, fname)?;
    let f1 = need_num(&args, 1, fname)?;
    let f2 = need_num(&args, 2, fname)?;
    let t  = need_num(&args, 3, fname)?;
    let d1 = f1 - f0;
    let d2 = (f2 - f1) - d1;
    Ok(Value::Number(f0 + t * d1 + t * (t - 1.0) / 2.0 * d2))
}

// महावीर_भिन्न: reduce fraction a/b to lowest terms (Mahavira ~850 CE)
fn fn_mahavira_frac(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "महावीर_भिन्न")? as i64;
    let b = need_num(&args, 1, "महावीर_भिन्न")? as i64;
    if b == 0 { return Err("महावीर_भिन्न: हर शून्य नहीं हो सकता".into()); }
    let g = gcd_inner(a.abs(), b.abs());
    let sign = if b < 0 { -1i64 } else { 1i64 };
    Ok(Value::List(vec![
        Value::Number((sign * a / g) as f64),
        Value::Number((sign * b / g) as f64),
    ]))
}

// ब्रह्मगुप्त_गुणन: Brahmagupta–Fibonacci identity (Brahmasphutasiddhanta 628 CE)
// (a²+nb²)(c²+nd²) = (ac−nbd)² + n(ad+bc)²
// Returns [p, q] such that the product = p² + n·q²
// Used for representing numbers as sums of weighted squares — ancestor of complex multiplication.
fn fn_bg_gunana(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "ब्रह्मगुप्त_गुणन")?;
    let b = need_num(&args, 1, "ब्रह्मगुप्त_गुणन")?;
    let c = need_num(&args, 2, "ब्रह्मगुप्त_गुणन")?;
    let d = need_num(&args, 3, "ब्रह्मगुप्त_गुणन")?;
    let n = need_num(&args, 4, "ब्रह्मगुप्त_गुणन")?;
    let p = a * c - n * b * d;
    let q = a * d + b * c;
    Ok(Value::List(vec![Value::Number(p), Value::Number(q)]))
}

// आर्यभट_वर्ग_योग: sum of squares 1²+2²+…+n² = n(n+1)(2n+1)/6  (Aryabhatiya 499 CE)
fn fn_aryabhata_varga_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "आर्यभट_वर्ग_योग")?;
    Ok(Value::Number(n * (n + 1.0) * (2.0 * n + 1.0) / 6.0))
}

// आर्यभट_घन_योग: sum of cubes 1³+…+n³ = [n(n+1)/2]²  (Aryabhatiya 499 CE)
fn fn_aryabhata_ghana_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "आर्यभट_घन_योग")?;
    let half = n * (n + 1.0) / 2.0;
    Ok(Value::Number(half * half))
}

// कटपयादि: Kerala consonant-to-digit cipher (Katapayadi system, pre-8th CE)
// Each Sanskrit consonant encodes a digit (k=1,kh=2,...,n=0); read right-to-left.
// Used by Kerala astronomers to memorize tables as meaningful Sanskrit words.
// Famous: "gopibhagya madhuvrata" encodes π to many digits.
fn fn_katapayadi(args: Vec<Value>) -> Result<Value, String> {
    let text = need_str(&args, 0, "कटपयादि")?;
    let mut digits: Vec<u8> = Vec::new();
    for ch in text.chars() {
        let d: Option<u8> = match ch {
            'क' => Some(1), 'ख' => Some(2), 'ग' => Some(3), 'घ' => Some(4), 'ङ' => Some(5),
            'च' => Some(6), 'छ' => Some(7), 'ज' => Some(8), 'झ' => Some(9), 'ञ' => Some(0),
            'ट' => Some(1), 'ठ' => Some(2), 'ड' => Some(3), 'ढ' => Some(4), 'ण' => Some(5),
            'त' => Some(6), 'थ' => Some(7), 'द' => Some(8), 'ध' => Some(9), 'न' => Some(0),
            'प' => Some(1), 'फ' => Some(2), 'ब' => Some(3), 'भ' => Some(4), 'म' => Some(5),
            'य' => Some(1), 'र' => Some(2), 'ल' => Some(3), 'व' => Some(4),
            'श' => Some(5), 'ष' => Some(6), 'स' => Some(7), 'ह' => Some(8),
            // Vowels, matras, virama, anusvara — ignored in encoding
            'अ'|'आ'|'इ'|'ई'|'उ'|'ऊ'|'ए'|'ऐ'|'ओ'|'औ'|'ऋ' => None,
            'ा'|'ि'|'ी'|'ु'|'ू'|'े'|'ो'|'ै'|'ौ'|'ृ'|'ं'|'ः'|'्'|' ' => None,
            _ => None,
        };
        if let Some(d) = d { digits.push(d); }
    }
    digits.reverse(); // read right-to-left per Katapayadi rules
    if digits.is_empty() { return Ok(Value::Number(0.0)); }
    let s: String = digits.iter().map(|d| char::from_digit(*d as u32, 10).unwrap()).collect();
    Ok(Value::Number(s.parse::<f64>().unwrap_or(0.0)))
}

pub fn ganit_registry() -> Registry {
    vec![
        // Āryabhaṭa trigonometry (499 CE)
        ("ज्या",                fn_jya              as NativeFn),
        ("कोज्या",              fn_kojya             as NativeFn),
        ("स्पर्शज्या",          fn_sparshjya         as NativeFn),
        ("व्युत्क्रम_ज्या",     fn_vyutkram_jya      as NativeFn),
        ("व्युत्क्रम_कोज्या",   fn_vyutkram_kojya    as NativeFn),
        ("व्युत्क्रम_स्पर्श",   fn_vyutkram_sparsh   as NativeFn),
        // Brahmagupta geometry (628 CE)
        ("ब्रह्मगुप्त_क्षेत्र", fn_brahma_kshetra    as NativeFn),
        ("हेरॉन_क्षेत्र",       fn_heron             as NativeFn),
        // Bhāskara II combinatorics (1150 CE)
        ("संयोजन",              fn_sanyojan          as NativeFn),
        ("क्रमचय",              fn_kramchay          as NativeFn),
        // Virahanka sequence (600 CE)
        ("विरहांक",             fn_virahanka         as NativeFn),
        // Logarithms & exponential
        ("लघुगणक",              fn_laghuganak        as NativeFn),
        ("लघुगणक_दस",           fn_laghuganak_das    as NativeFn),
        ("घातांक",              fn_ghataank          as NativeFn),
        // Āryabhaṭa's π
        ("आर्यभट_पाई",          fn_aaryabhata_pai    as NativeFn),
        // Aryabhata algorithms (Aryabhatiya 499 CE)
        ("कुट्टक",                fn_kuttak               as NativeFn),
        ("आर्यभट_योग",            fn_aryabhata_sum        as NativeFn),
        ("आर्यभट_वर्ग_योग",       fn_aryabhata_varga_sum  as NativeFn),
        ("आर्यभट_घन_योग",         fn_aryabhata_ghana_sum  as NativeFn),
        // Shridhara quadratic (Patiganita 870 CE)
        ("श्रीधर_सूत्र",          fn_shridhar             as NativeFn),
        // Bakshali sqrt (Bakshali Manuscript ~3rd-7th CE)
        ("बखशाली_मूल",            fn_bakshali_sqrt        as NativeFn),
        // Brahmagupta (Brahmasphutasiddhanta 628 CE + Khandakhadyaka 665 CE)
        ("ब्रह्मगुप्त_अंतर्वेशन", fn_bg_interpolate       as NativeFn),
        ("ब्रह्मगुप्त_गुणन",      fn_bg_gunana            as NativeFn),
        // Mahavira fraction reduction (Ganitasarasangraha ~850 CE)
        ("महावीर_भिन्न",          fn_mahavira_frac        as NativeFn),
        // Katapayadi cipher (Kerala tradition, pre-8th CE)
        ("कटपयादि",               fn_katapayadi           as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE 6 — भारत.छन्दस् (Pingala's Chandahshastra ~200 BCE)
// Binary encoding: Guru(heavy=1)/Laghu(light=0), Pascal's triangle, binary-meter conversions.
// Pingala's algorithms are mathematically equivalent to binary arithmetic — 1800 years before Leibniz.
// ════════════════════════════════════════════════════════════════

// गुरु_लघु_संकेत: Pingala's Uddhishta — given position (1-indexed) among 2^n meters,
// return the meter as a string of ग (guru/1) and ल (laghu/0). Equivalent to decimal→binary.
fn fn_guru_laghu(args: Vec<Value>) -> Result<Value, String> {
    let n   = need_num(&args, 0, "गुरु_लघु_संकेत")? as u32;
    let pos = need_num(&args, 1, "गुरु_लघु_संकेत")? as u64;
    if n > 30 { return Err("गुरु_लघु_संकेत: n 30 से अधिक नहीं".into()); }
    let total = 1u64 << n;
    if pos < 1 || pos > total {
        return Err(format!("गुरु_लघु_संकेत: स्थान 1–{} के बीच होना चाहिए", total));
    }
    let bits = pos - 1;
    let meter: String = (0..n).map(|i| if (bits >> i) & 1 == 1 { 'ग' } else { 'ल' }).collect();
    Ok(Value::Str(meter))
}

// छन्द_स्थान: Pingala's Nashtam — given a meter string, find its 1-indexed position.
// Inverse of गुरु_लघु_संकेत. Equivalent to binary→decimal.
fn fn_chhand_sthan(args: Vec<Value>) -> Result<Value, String> {
    let meter = need_str(&args, 0, "छन्द_स्थान")?;
    let mut pos = 0u64;
    for (i, ch) in meter.chars().enumerate() {
        let bit = match ch {
            'ग' | 'G' | 'g' => 1u64,
            'ल' | 'L' | 'l' => 0u64,
            _ => return Err(format!("छन्द_स्थान: '{}' मान्य वर्ण नहीं (ग/ल)", ch)),
        };
        pos |= bit << i;
    }
    Ok(Value::Number((pos + 1) as f64))
}

// मात्रा_भार: count syllable weights — returns [guru_count, laghu_count]
fn fn_matra_bhar(args: Vec<Value>) -> Result<Value, String> {
    let meter = need_str(&args, 0, "मात्रा_भार")?;
    let guru  = meter.chars().filter(|&c| c == 'ग' || c == 'G').count();
    let laghu = meter.chars().filter(|&c| c == 'ल' || c == 'L').count();
    Ok(Value::List(vec![Value::Number(guru as f64), Value::Number(laghu as f64)]))
}

pub fn chhandas_registry() -> Registry {
    vec![
        ("मेरु_पंक्ति",    fn_meru_pankti  as NativeFn),
        ("नष्टम",          fn_nashtam      as NativeFn),
        ("उद्दिष्ट",       fn_uddhishta    as NativeFn),
        ("प्रस्तार",       fn_prastara     as NativeFn),
        ("द्विआधार",       fn_dvi_aadhaar  as NativeFn),
        ("गुरु_लघु_संकेत", fn_guru_laghu   as NativeFn),
        ("छन्द_स्थान",     fn_chhand_sthan as NativeFn),
        ("मात्रा_भार",     fn_matra_bhar   as NativeFn),
    ]
}

fn fn_meru_pankti(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "मेरु_पंक्ति")? as usize;
    let mut row: Vec<u64> = vec![1];
    for _ in 0..n {
        let mut next = vec![1u64];
        for i in 0..row.len()-1 { next.push(row[i] + row[i+1]); }
        next.push(1);
        row = next;
    }
    Ok(Value::List(row.iter().map(|&x| Value::Number(x as f64)).collect()))
}

fn fn_nashtam(args: Vec<Value>) -> Result<Value, String> {
    let n   = need_num(&args, 0, "नष्टम")? as u64;
    let pos = need_num(&args, 1, "नष्टम")? as u64;
    if pos < 1 { return Err("नष्टम: pos 1-indexed, minimum 1".into()); }
    let mut result = String::new();
    let mut p = pos - 1;
    for _ in 0..n {
        result.push(if p % 2 == 0 { 'ल' } else { 'ग' });
        p /= 2;
    }
    Ok(Value::Str(result))
}

fn fn_uddhishta(args: Vec<Value>) -> Result<Value, String> {
    let s = need_str(&args, 0, "उद्दिष्ट")?;
    let mut pos: u64 = 0;
    for (i, ch) in s.chars().enumerate() {
        if ch == 'ग' || ch == 'G' || ch == '1' { pos += 1 << i; }
    }
    Ok(Value::Number((pos + 1) as f64))
}

fn fn_prastara(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "प्रस्तार")? as u32;
    if n > 16 { return Err("प्रस्तार: n ≤ 16 होना चाहिए".into()); }
    let total = 1u32 << n;
    let result: Vec<Value> = (0..total).map(|i| {
        let s: String = (0..n).map(|bit| if (i >> bit) & 1 == 0 { 'ल' } else { 'ग' }).collect();
        Value::Str(s)
    }).collect();
    Ok(Value::List(result))
}

fn fn_dvi_aadhaar(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "द्विआधार")? as u64;
    if n == 0 { return Ok(Value::Str("ल".into())); }
    let mut result = String::new();
    let mut val = n;
    while val > 0 { result.push(if val & 1 == 0 { 'ल' } else { 'ग' }); val >>= 1; }
    Ok(Value::Str(result))
}

// ════════════════════════════════════════════════════════════════
// MODULE 7 — भारत.न्याय (Nyaya Sutras ~200 BCE)
// Formal logic: 5-part syllogism, vyapti, hetvabhasa, pramana
// ════════════════════════════════════════════════════════════════

pub fn nyaya_registry() -> Registry {
    vec![
        ("अनुमान",    fn_anuman     as NativeFn),
        ("व्याप्ति",  fn_vyapti     as NativeFn),
        ("हेत्वाभास", fn_hetvabhasa as NativeFn),
        ("प्रमाण",    fn_pramana    as NativeFn),
    ]
}

fn fn_anuman(args: Vec<Value>) -> Result<Value, String> {
    let vyapti = match args.get(2) {
        Some(Value::Bool(b)) => *b,
        _ => return Err("अनुमान: arg 3 व्याप्ति (Bool) आवश्यक".into()),
    };
    let upanaya = match args.get(3) {
        Some(Value::Bool(b)) => *b,
        _ => return Err("अनुमान: arg 4 उपनय (Bool) आवश्यक".into()),
    };
    Ok(Value::Bool(vyapti && upanaya))
}

fn fn_vyapti(args: Vec<Value>) -> Result<Value, String> {
    let obs = match args.get(0) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("व्याप्ति: arg 1 must be List of [Bool, Bool] pairs".into()),
    };
    for pair in &obs {
        if let Value::List(pv) = pair {
            let a = matches!(pv.get(0), Some(Value::Bool(true)));
            let b = matches!(pv.get(1), Some(Value::Bool(true)));
            if a && !b { return Ok(Value::Bool(false)); }
        }
    }
    Ok(Value::Bool(true))
}

// हेत्वाभास: Nyaya fallacy types — accepts number (1-5) OR string name
fn fn_hetvabhasa(args: Vec<Value>) -> Result<Value, String> {
    let desc = match args.get(0) {
        Some(Value::Number(n)) => match *n as u8 {
            1 => "सव्यभिचार — जहाँ साध्य नहीं वहाँ हेतु (irregular/inconsistent evidence)",
            2 => "विरुद्ध — हेतु साध्य के विपरीत सिद्ध करता है (contradictory proof)",
            3 => "प्रकरणसम — हेतु उतना ही अनिश्चित जितना साध्य (question-begging)",
            4 => "साध्यसम — हेतु को स्वयं प्रमाण की जरूरत (ungrounded reason)",
            5 => "कालातीत — हेतु निष्कर्ष के बाद आता है (ill-timed/temporal bug)",
            _ => return Err("हेत्वाभास: 1-5 संख्या या नाम दें".into()),
        },
        Some(Value::Str(s)) => match s.trim() {
            "सव्यभिचार" => "सव्यभिचार — irregular hetu: not always present with sadhya (inconsistent evidence)",
            "विरुद्ध"    => "विरुद्ध — contradictory hetu: proves the opposite (self-defeating argument)",
            "प्रकरणसम"  => "प्रकरणसम — question-begging: hetu is as uncertain as sadhya",
            "साध्यसम"   => "साध्यसम — unproved hetu: reason itself is unproven",
            "कालातीत"   => "कालातीत — ill-timed: hetu arrived after the inference (temporal bug)",
            other => return Err(format!("हेत्वाभास: '{}' अज्ञात। सव्यभिचार/विरुद्ध/प्रकरणसम/साध्यसम/कालातीत या 1-5", other)),
        },
        _ => return Err("हेत्वाभास: संख्या (1-5) या नाम (सव्यभिचार आदि) दें".into()),
    };
    Ok(Value::Str(desc.into()))
}

// प्रमाण: Nyaya knowledge source types — accepts number (1-4) OR string name
fn fn_pramana(args: Vec<Value>) -> Result<Value, String> {
    let desc = match args.get(0) {
        Some(Value::Number(n)) => match *n as u8 {
            1 => "प्रत्यक्ष — प्रत्यक्ष अनुभव (direct perception / sensor data)",
            2 => "अनुमान — तार्किक अनुमान (logical inference / deduction)",
            3 => "उपमान — उपमा/तुलना (comparison / structural similarity matching)",
            4 => "शब्द — विश्वसनीय स्रोत से ज्ञान (testimony / documented specification)",
            _ => return Err("प्रमाण: 1-4 संख्या या नाम दें".into()),
        },
        Some(Value::Str(s)) => match s.trim() {
            "प्रत्यक्ष" => "प्रत्यक्ष — direct perception; sensor input, measurements, observed data",
            "अनुमान"    => "अनुमान — inference; logical deduction from known facts",
            "उपमान"     => "उपमान — comparison; type matching by structural similarity",
            "शब्द"      => "शब्द — testimony; documented specification, trusted source",
            other => return Err(format!("प्रमाण: '{}' अज्ञात। प्रत्यक्ष/अनुमान/उपमान/शब्द या 1-4", other)),
        },
        _ => return Err("प्रमाण: संख्या (1-4) या नाम (प्रत्यक्ष आदि) दें".into()),
    };
    Ok(Value::Str(desc.into()))
}

// ════════════════════════════════════════════════════════════════
// MODULE 8 — भारत.शुल्ब (Baudhayana Sulbasutra ~800 BCE)
// Geometric construction algorithms
// ════════════════════════════════════════════════════════════════

pub fn shulba_registry() -> Registry {
    vec![
        ("कर्ण",          fn_karna         as NativeFn),
        ("शुल्ब_मूल",     fn_shulba_sqrt   as NativeFn),
        ("वृत्त_वर्ग",    fn_circle_square as NativeFn),
        ("वर्ग_वृत्त",    fn_square_circle as NativeFn),
    ]
}

fn fn_karna(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "कर्ण")?;
    let b = need_num(&args, 1, "कर्ण")?;
    Ok(Value::Number((a * a + b * b).sqrt()))
}

fn fn_shulba_sqrt(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "शुल्ब_मूल")?;
    if n < 0.0 { return Err("शुल्ब_मूल: ऋणात्मक संख्या का मूल नहीं".into()); }
    let mut x = n.sqrt();
    for _ in 0..3 { if x != 0.0 { x = (x + n / x) / 2.0; } }
    Ok(Value::Number(x))
}

fn fn_circle_square(args: Vec<Value>) -> Result<Value, String> {
    let r = need_num(&args, 0, "वृत्त_वर्ग")?;
    Ok(Value::Number(r * std::f64::consts::PI.sqrt()))
}

fn fn_square_circle(args: Vec<Value>) -> Result<Value, String> {
    let s = need_num(&args, 0, "वर्ग_वृत्त")?;
    Ok(Value::Number(s / std::f64::consts::PI.sqrt()))
}

// ════════════════════════════════════════════════════════════════
// MODULE 9 — भारत.ज्योतिष (Astronomy — Vedanga Jyotisha + Surya Siddhanta)
// 27 nakshatras, 30 tithis, planetary periods
// ════════════════════════════════════════════════════════════════

const NAKSHATRAS: [&str; 27] = [
    "अश्विनी","भरणी","कृत्तिका","रोहिणी","मृगशिरा","आर्द्रा","पुनर्वसु",
    "पुष्य","आश्लेषा","मघा","पूर्वाफाल्गुनी","उत्तराफाल्गुनी","हस्त",
    "चित्रा","स्वाति","विशाखा","अनुराधा","ज्येष्ठा","मूल","पूर्वाषाढा",
    "उत्तराषाढा","श्रवण","धनिष्ठा","शतभिषा","पूर्वाभाद्रपद",
    "उत्तराभाद्रपद","रेवती",
];

const TITHIS: [&str; 30] = [
    "प्रतिपदा","द्वितीया","तृतीया","चतुर्थी","पंचमी","षष्ठी","सप्तमी",
    "अष्टमी","नवमी","दशमी","एकादशी","द्वादशी","त्रयोदशी","चतुर्दशी",
    "पूर्णिमा","प्रतिपदा","द्वितीया","तृतीया","चतुर्थी","पंचमी","षष्ठी",
    "सप्तमी","अष्टमी","नवमी","दशमी","एकादशी","द्वादशी","त्रयोदशी",
    "चतुर्दशी","अमावस्या",
];

pub fn jyotish_registry() -> Registry {
    vec![
        ("नक्षत्र_नाम",    fn_nakshatra_naam  as NativeFn),
        ("नक्षत्र_क्रम",   fn_nakshatra_kram  as NativeFn),
        ("तिथि_नाम",       fn_tithi_naam      as NativeFn),
        ("युग_नाम",        fn_yug_naam        as NativeFn),
        ("ग्रह_परिक्रमा",  fn_graha_kram      as NativeFn), // proper Aryabhata name: orbital period
        ("ग्रह_क्रम",      fn_graha_kram      as NativeFn), // legacy alias
        ("सभी_नक्षत्र",    fn_sabhi_nakshatra as NativeFn),
    ]
}

fn fn_nakshatra_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "नक्षत्र_नाम")? as usize;
    if n < 1 || n > 27 { return Err("नक्षत्र_नाम: n 1-27 के बीच होना चाहिए".into()); }
    Ok(Value::Str(NAKSHATRAS[n-1].into()))
}

fn fn_nakshatra_kram(args: Vec<Value>) -> Result<Value, String> {
    let name = need_str(&args, 0, "नक्षत्र_क्रम")?;
    for (i, &n) in NAKSHATRAS.iter().enumerate() {
        if n == name.as_str() { return Ok(Value::Number((i + 1) as f64)); }
    }
    Err(format!("नक्षत्र_क्रम: '{}' नहीं मिला", name))
}

fn fn_tithi_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "तिथि_नाम")? as usize;
    if n < 1 || n > 30 { return Err("तिथि_नाम: n 1-30 के बीच होना चाहिए".into()); }
    Ok(Value::Str(TITHIS[n-1].into()))
}

fn fn_yug_naam(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(vec![
        Value::Str("सत्ययुग (कृतयुग) — 1,728,000 वर्ष".into()),
        Value::Str("त्रेतायुग — 1,296,000 वर्ष".into()),
        Value::Str("द्वापरयुग — 864,000 वर्ष".into()),
        Value::Str("कलियुग — 432,000 वर्ष".into()),
    ]))
}

fn fn_graha_kram(args: Vec<Value>) -> Result<Value, String> {
    let name = need_str(&args, 0, "ग्रह_क्रम")?;
    let days: f64 = match name.as_str() {
        "सूर्य" | "sun"      => 365.2563,
        "चन्द्र" | "moon"    => 27.3217,
        "मंगल" | "mars"     => 686.9714,
        "बुध" | "mercury"   => 87.9693,
        "बृहस्पति" | "jupiter" => 4332.589,
        "शुक्र" | "venus"   => 224.701,
        "शनि" | "saturn"   => 10759.22,
        _ => return Err(format!("ग्रह_क्रम: अज्ञात ग्रह '{}'", name)),
    };
    Ok(Value::Number(days))
}

fn fn_sabhi_nakshatra(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(NAKSHATRAS.iter().map(|&n| Value::Str(n.into())).collect()))
}

// ════════════════════════════════════════════════════════════════
// MODULE 10 — भारत.नाट्य (Bharata Muni's Natyashastra ~200 BCE)
// 9 Rasas (aesthetic essences)
// ════════════════════════════════════════════════════════════════

const RASAS: [(&str, &str, &str); 9] = [
    ("शृंगार", "रति",    "love, beauty, romance"),
    ("हास्य",  "हास",    "humor, laughter"),
    ("करुण",   "शोक",   "sorrow, compassion"),
    ("रौद्र",  "क्रोध", "fury, rage"),
    ("वीर",    "उत्साह","heroism, courage"),
    ("भयानक",  "भय",    "fear, terror"),
    ("बीभत्स", "जुगुप्सा","disgust, revulsion"),
    ("अद्भुत", "विस्मय", "wonder, astonishment"),
    ("शान्त",  "शम",    "peace, tranquility"),
];

// रस_सूत्र: Bharata's formula — vibhava × anubhava × sanchari → rasa quality score
// "vibhava-anubhava-vyabhichari-sanyogat rasa-nishpattih" (Natyashastra 6.31)
// Maps: stimulus × response × transient_state → aesthetic output quality (1-9)
fn fn_rasa_sutra(args: Vec<Value>) -> Result<Value, String> {
    let vibhava  = need_str(&args, 0, "रस_सूत्र")?;
    let anubhava = need_str(&args, 1, "रस_सूत्र")?;
    let sanchari = need_str(&args, 2, "रस_सूत्र")?;
    let score = (vibhava.len().wrapping_add(anubhava.len()).wrapping_add(sanchari.len())) % 9 + 1;
    Ok(Value::Number(score as f64))
}

pub fn natya_registry() -> Registry {
    vec![
        ("रस_नाम",   fn_rasa_naam    as NativeFn),
        ("रस_भाव",   fn_rasa_bhav    as NativeFn),
        ("रस_विवरण", fn_rasa_vivaran as NativeFn),
        ("रस_सूत्र", fn_rasa_sutra   as NativeFn),
        ("सभी_रस",   fn_sabhi_rasa   as NativeFn),
    ]
}

fn fn_rasa_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_नाम")? as usize;
    if n < 1 || n > 9 { return Err("रस_नाम: n 1-9 के बीच होना चाहिए".into()); }
    Ok(Value::Str(RASAS[n-1].0.into()))
}

fn fn_rasa_bhav(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_भाव")? as usize;
    if n < 1 || n > 9 { return Err("रस_भाव: n 1-9 के बीच होना चाहिए".into()); }
    Ok(Value::Str(RASAS[n-1].1.into()))
}

fn fn_rasa_vivaran(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_विवरण")? as usize;
    if n < 1 || n > 9 { return Err("रस_विवरण: n 1-9 के बीच होना चाहिए".into()); }
    Ok(Value::Str(format!("{} ({}): {}", RASAS[n-1].0, RASAS[n-1].1, RASAS[n-1].2)))
}

fn fn_sabhi_rasa(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(RASAS.iter().map(|(n,_,_)| Value::Str((*n).into())).collect()))
}

// ════════════════════════════════════════════════════════════════
// MODULE 11 — भारत.तंत्रिका (Neural Networks)
// Pure Rust, no external crates. Tensors = Value::List of Value::Number.
// ════════════════════════════════════════════════════════════════

pub fn tantrika_registry() -> Registry {
    vec![
        ("स्तर_बनाओ",        fn_layer_create    as NativeFn),
        ("आगे_पास",           fn_forward_pass    as NativeFn),
        ("जाल_आगे",           fn_network_forward as NativeFn),
        ("रेलु",              fn_relu            as NativeFn),
        ("सिग्मा",            fn_sigmoid         as NativeFn),
        ("त्रुटि_वर्ग",       fn_mse             as NativeFn),
        ("त्रुटि_वर्ग_अवकल",  fn_mse_grad        as NativeFn),
    ]
}

fn fn_layer_create(args: Vec<Value>) -> Result<Value, String> {
    let ins  = need_num(&args, 0, "स्तर_बनाओ")? as usize;
    let outs = need_num(&args, 1, "स्तर_बनाओ")? as usize;
    let mut seed: u64 = 12345;
    let mut rng = || -> f64 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as f64) / (u32::MAX as f64) * 0.4 - 0.2
    };
    let weights: Vec<Value> = (0..outs).map(|_| {
        Value::List((0..ins).map(|_| Value::Number(rng())).collect())
    }).collect();
    let biases: Vec<Value> = (0..outs).map(|_| Value::Number(0.0)).collect();
    Ok(Value::List(vec![Value::List(weights), Value::List(biases)]))
}

fn fn_forward_pass(args: Vec<Value>) -> Result<Value, String> {
    let layer = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("आगे_पास: arg 1 स्तर (List) आवश्यक".into()) };
    let input = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("आगे_पास: arg 2 निवेश (List) आवश्यक".into()) };
    let weights = match layer.get(0) { Some(Value::List(w)) => w.clone(), _ => return Err("आगे_पास: layer[0] भार-मैट्रिक्स आवश्यक".into()) };
    let biases  = match layer.get(1) { Some(Value::List(b)) => b.clone(), _ => return Err("आगे_पास: layer[1] पक्षपात-सदिश आवश्यक".into()) };
    let in_vals: Vec<f64> = input.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).collect();
    let mut output = Vec::new();
    for (row, bias) in weights.iter().zip(biases.iter()) {
        let w_row = match row { Value::List(r) => r, _ => return Err("आगे_पास: भार पंक्ति List नहीं".into()) };
        let b = if let Value::Number(n) = bias { *n } else { 0.0 };
        let sum: f64 = w_row.iter().zip(in_vals.iter())
            .map(|(w, x)| { let wv = if let Value::Number(n) = w { *n } else { 0.0 }; wv * x })
            .sum::<f64>() + b;
        output.push(Value::Number(sum));
    }
    Ok(Value::List(output))
}

fn fn_network_forward(args: Vec<Value>) -> Result<Value, String> {
    let network = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("जाल_आगे: arg 1 जाल (List of layers) आवश्यक".into()) };
    let mut current = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("जाल_आगे: arg 2 निवेश (List) आवश्यक".into()) };
    for layer in &network {
        let layer_args = vec![layer.clone(), Value::List(current)];
        current = match fn_forward_pass(layer_args)? { Value::List(v) => v, _ => return Err("जाल_आगे: आगे-पास विफल".into()) };
    }
    Ok(Value::List(current))
}

fn fn_relu(args: Vec<Value>) -> Result<Value, String> {
    match args.get(0) {
        Some(Value::Number(n)) => Ok(Value::Number(n.max(0.0))),
        Some(Value::List(v)) => Ok(Value::List(v.iter().map(|val| {
            if let Value::Number(n) = val { Value::Number(n.max(0.0)) } else { val.clone() }
        }).collect())),
        _ => Err("रेलु: संख्या या List आवश्यक".into()),
    }
}

fn fn_sigmoid(args: Vec<Value>) -> Result<Value, String> {
    let sig = |x: f64| 1.0 / (1.0 + (-x).exp());
    match args.get(0) {
        Some(Value::Number(n)) => Ok(Value::Number(sig(*n))),
        Some(Value::List(v)) => Ok(Value::List(v.iter().map(|val| {
            if let Value::Number(n) = val { Value::Number(sig(*n)) } else { val.clone() }
        }).collect())),
        _ => Err("सिग्मा: संख्या या List आवश्यक".into()),
    }
}

fn fn_mse(args: Vec<Value>) -> Result<Value, String> {
    let pred = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग: arg 1 List आवश्यक".into()) };
    let targ = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग: arg 2 List आवश्यक".into()) };
    let n = pred.len().min(targ.len()) as f64;
    if n == 0.0 { return Err("त्रुटि_वर्ग: रिक्त सूची".into()); }
    let mse: f64 = pred.iter().zip(targ.iter()).map(|(p, t)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let tv = if let Value::Number(x) = t { *x } else { 0.0 };
        (pv - tv).powi(2)
    }).sum::<f64>() / n;
    Ok(Value::Number(mse))
}

fn fn_mse_grad(args: Vec<Value>) -> Result<Value, String> {
    let pred = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग_अवकल: arg 1 List आवश्यक".into()) };
    let targ = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग_अवकल: arg 2 List आवश्यक".into()) };
    let n = pred.len() as f64;
    let grads: Vec<Value> = pred.iter().zip(targ.iter()).map(|(p, t)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let tv = if let Value::Number(x) = t { *x } else { 0.0 };
        Value::Number(2.0 * (pv - tv) / n)
    }).collect();
    Ok(Value::List(grads))
}

// ════════════════════════════════════════════════════════════════
// MODULE 12 — भारत.अनुकूलन (Gradient Descent / Optimization)
// ════════════════════════════════════════════════════════════════

pub fn anukooland_registry() -> Registry {
    vec![
        ("ढाल_अवरोह",   fn_gradient_descent as NativeFn),
        ("आदम_चरण",     fn_adam_step        as NativeFn),
        ("ढाल_संख्यिक", fn_numeric_grad     as NativeFn),
        ("सीखने_की_दर",  fn_lr_schedule      as NativeFn),
    ]
}

fn fn_gradient_descent(args: Vec<Value>) -> Result<Value, String> {
    let params = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("ढाल_अवरोह: arg 1 पैरामीटर List आवश्यक".into()) };
    let grads  = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("ढाल_अवरोह: arg 2 ढाल List आवश्यक".into()) };
    let lr     = need_num(&args, 2, "ढाल_अवरोह")?;
    let new_params: Vec<Value> = params.iter().zip(grads.iter()).map(|(p, g)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let gv = if let Value::Number(x) = g { *x } else { 0.0 };
        Value::Number(pv - lr * gv)
    }).collect();
    Ok(Value::List(new_params))
}

fn fn_adam_step(args: Vec<Value>) -> Result<Value, String> {
    let params = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 1 params".into()) };
    let grads  = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 2 grads".into()) };
    let m_prev = match args.get(2) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 3 m (1st moment)".into()) };
    let v_prev = match args.get(3) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 4 v (2nd moment)".into()) };
    let t      = need_num(&args, 4, "आदम_चरण")?;
    let lr     = need_num(&args, 5, "आदम_चरण")?;
    let beta1  = if args.len() > 6 { need_num(&args, 6, "आदम_चरण")? } else { 0.9 };
    let beta2  = if args.len() > 7 { need_num(&args, 7, "आदम_चरण")? } else { 0.999 };
    let eps    = if args.len() > 8 { need_num(&args, 8, "आदम_चरण")? } else { 1e-8 };
    let mut new_params = Vec::new();
    let mut new_m = Vec::new();
    let mut new_v = Vec::new();
    let len = params.len().min(grads.len()).min(m_prev.len()).min(v_prev.len());
    for i in 0..len {
        let p  = if let Some(Value::Number(x)) = params.get(i) { *x } else { 0.0 };
        let g  = if let Some(Value::Number(x)) = grads.get(i)  { *x } else { 0.0 };
        let mi = if let Some(Value::Number(x)) = m_prev.get(i) { *x } else { 0.0 };
        let vi = if let Some(Value::Number(x)) = v_prev.get(i) { *x } else { 0.0 };
        let m_new = beta1 * mi + (1.0 - beta1) * g;
        let v_new = beta2 * vi + (1.0 - beta2) * g * g;
        let m_hat = m_new / (1.0 - beta1.powf(t));
        let v_hat = v_new / (1.0 - beta2.powf(t));
        let p_new = p - lr * m_hat / (v_hat.sqrt() + eps);
        new_params.push(Value::Number(p_new));
        new_m.push(Value::Number(m_new));
        new_v.push(Value::Number(v_new));
    }
    Ok(Value::List(vec![Value::List(new_params), Value::List(new_m), Value::List(new_v)]))
}

fn fn_numeric_grad(args: Vec<Value>) -> Result<Value, String> {
    let f_plus  = need_num(&args, 0, "ढाल_संख्यिक")?;
    let f_minus = need_num(&args, 1, "ढाल_संख्यिक")?;
    let h       = if args.len() > 2 { need_num(&args, 2, "ढाल_संख्यिक")? } else { 1e-5 };
    Ok(Value::Number((f_plus - f_minus) / (2.0 * h)))
}

fn fn_lr_schedule(args: Vec<Value>) -> Result<Value, String> {
    let lr0   = need_num(&args, 0, "सीखने_की_दर")?;
    let epoch = need_num(&args, 1, "सीखने_की_दर")?;
    let kind  = if args.len() > 2 { need_str(&args, 2, "सीखने_की_दर")? } else { "step".into() };
    let decay = if args.len() > 3 { need_num(&args, 3, "सीखने_की_दर")? } else { 0.1 };
    let new_lr = match kind.as_str() {
        "step"    | "चरण"    => lr0 * (1.0 - decay).powf((epoch / 10.0).floor()),
        "exp"     | "घातीय"  => lr0 * (-decay * epoch).exp(),
        "cosine"  | "कोज्या" => lr0 * 0.5 * (1.0 + (std::f64::consts::PI * epoch / 100.0).cos()),
        _                    => lr0,
    };
    Ok(Value::Number(new_lr))
}

// ════════════════════════════════════════════════════════════════
// MODULE 13 — भारत.प्रज्ञा (Statistical Learning)
// ════════════════════════════════════════════════════════════════

pub fn pragya_registry() -> Registry {
    vec![
        ("औसत",              fn_mean              as NativeFn),
        ("विचलन",            fn_std_dev           as NativeFn),
        ("माध्यिका",         fn_median            as NativeFn),
        ("रेखीय_प्रतिगमन",  fn_linear_regression as NativeFn),
        ("भविष्यवाणी",       fn_predict           as NativeFn),
        ("सहसम्बन्ध",        fn_correlation       as NativeFn),
        ("क_साधन",           fn_k_means_step      as NativeFn),
        ("निकटतम_पड़ोसी",    fn_nearest_neighbor  as NativeFn),
    ]
}

fn fn_mean(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("औसत: List आवश्यक".into()) };
    if vals.is_empty() { return Err("औसत: रिक्त सूची".into()); }
    let sum: f64 = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).sum();
    Ok(Value::Number(sum / vals.len() as f64))
}

fn fn_std_dev(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("विचलन: List आवश्यक".into()) };
    if vals.len() < 2 { return Err("विचलन: कम से कम 2 मान चाहिए".into()); }
    let nums: Vec<f64> = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).collect();
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var  = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    Ok(Value::Number(var.sqrt()))
}

fn fn_median(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("माध्यिका: List आवश्यक".into()) };
    if vals.is_empty() { return Err("माध्यिका: रिक्त सूची".into()); }
    let mut nums: Vec<f64> = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).collect();
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = nums.len() / 2;
    let med = if nums.len() % 2 == 0 { (nums[mid-1] + nums[mid]) / 2.0 } else { nums[mid] };
    Ok(Value::Number(med))
}

fn fn_linear_regression(args: Vec<Value>) -> Result<Value, String> {
    let xs = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("रेखीय_प्रतिगमन: arg 1 x-List आवश्यक".into()) };
    let ys = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("रेखीय_प्रतिगमन: arg 2 y-List आवश्यक".into()) };
    let n = xs.len().min(ys.len()) as f64;
    let xv: Vec<f64> = xs.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let yv: Vec<f64> = ys.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let sx: f64  = xv.iter().sum();
    let sy: f64  = yv.iter().sum();
    let sxy: f64 = xv.iter().zip(yv.iter()).map(|(x,y)| x*y).sum();
    let sxx: f64 = xv.iter().map(|x| x*x).sum();
    let denom = n * sxx - sx * sx;
    if denom == 0.0 { return Err("रेखीय_प्रतिगमन: ऊर्ध्वाधर रेखा, ढाल अपरिभाषित".into()); }
    let slope = (n * sxy - sx * sy) / denom;
    let intercept = (sy - slope * sx) / n;
    Ok(Value::List(vec![Value::Number(slope), Value::Number(intercept)]))
}

fn fn_predict(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "भविष्यवाणी")?;
    let m = need_num(&args, 1, "भविष्यवाणी")?;
    let b = need_num(&args, 2, "भविष्यवाणी")?;
    Ok(Value::Number(m * x + b))
}

fn fn_correlation(args: Vec<Value>) -> Result<Value, String> {
    let xs = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("सहसम्बन्ध: arg 1 x-List आवश्यक".into()) };
    let ys = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("सहसम्बन्ध: arg 2 y-List आवश्यक".into()) };
    let xv: Vec<f64> = xs.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let yv: Vec<f64> = ys.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let n = xv.len().min(yv.len()) as f64;
    let mx = xv.iter().sum::<f64>() / n;
    let my = yv.iter().sum::<f64>() / n;
    let cov: f64 = xv.iter().zip(yv.iter()).map(|(x,y)| (x-mx)*(y-my)).sum::<f64>() / n;
    let sx = (xv.iter().map(|x| (x-mx).powi(2)).sum::<f64>() / n).sqrt();
    let sy = (yv.iter().map(|y| (y-my).powi(2)).sum::<f64>() / n).sqrt();
    if sx == 0.0 || sy == 0.0 { return Ok(Value::Number(0.0)); }
    Ok(Value::Number(cov / (sx * sy)))
}

fn fn_k_means_step(args: Vec<Value>) -> Result<Value, String> {
    let points    = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("क_साधन: arg 1 बिन्दु-List आवश्यक".into()) };
    let centroids = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("क_साधन: arg 2 केन्द्रक-List आवश्यक".into()) };
    let k = centroids.len();
    let mut sums: Vec<Vec<f64>>   = vec![vec![0.0, 0.0]; k];
    let mut counts: Vec<f64>       = vec![0.0; k];
    for point in &points {
        if let Value::List(pt) = point {
            let px = if let Some(Value::Number(x)) = pt.get(0) { *x } else { 0.0 };
            let py = if let Some(Value::Number(y)) = pt.get(1) { *y } else { 0.0 };
            let (mut best, mut best_d) = (0, f64::MAX);
            for (ci, c) in centroids.iter().enumerate() {
                if let Value::List(cv) = c {
                    let cx = if let Some(Value::Number(x)) = cv.get(0) { *x } else { 0.0 };
                    let cy = if let Some(Value::Number(y)) = cv.get(1) { *y } else { 0.0 };
                    let d = (px-cx).powi(2) + (py-cy).powi(2);
                    if d < best_d { best_d = d; best = ci; }
                }
            }
            sums[best][0] += px; sums[best][1] += py; counts[best] += 1.0;
        }
    }
    let new_centroids: Vec<Value> = (0..k).map(|i| {
        if counts[i] == 0.0 { centroids[i].clone() }
        else { Value::List(vec![Value::Number(sums[i][0]/counts[i]), Value::Number(sums[i][1]/counts[i])]) }
    }).collect();
    Ok(Value::List(new_centroids))
}

fn fn_nearest_neighbor(args: Vec<Value>) -> Result<Value, String> {
    let query  = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 1 प्रश्न-बिन्दु List आवश्यक".into()) };
    let points = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 2 बिन्दु-List आवश्यक".into()) };
    let labels = match args.get(2) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 3 लेबल-List आवश्यक".into()) };
    let qx = if let Some(Value::Number(x)) = query.get(0) { *x } else { 0.0 };
    let qy = if let Some(Value::Number(y)) = query.get(1) { *y } else { 0.0 };
    let (mut best_idx, mut best_d) = (0, f64::MAX);
    for (i, pt) in points.iter().enumerate() {
        if let Value::List(pv) = pt {
            let px = if let Some(Value::Number(x)) = pv.get(0) { *x } else { 0.0 };
            let py = if let Some(Value::Number(y)) = pv.get(1) { *y } else { 0.0 };
            let d = (qx-px).powi(2) + (qy-py).powi(2);
            if d < best_d { best_d = d; best_idx = i; }
        }
    }
    Ok(labels.get(best_idx).cloned().unwrap_or(Value::Nil))
}

// ════════════════════════════════════════════════════════════════
// MODULE 14 — भारत.तुरिंग (Turing Machine Simulator)
// ════════════════════════════════════════════════════════════════

pub fn turing_registry() -> Registry {
    vec![
        ("तुरिंग_चलाओ", fn_turing_run   as NativeFn),
        ("तुरिंग_सिद्ध", fn_turing_proof as NativeFn),
    ]
}

fn fn_turing_run(args: Vec<Value>) -> Result<Value, String> {
    let tape_in   = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("तुरिंग_चलाओ: arg 1 टेप List आवश्यक".into()) };
    let mut head  = need_num(&args, 1, "तुरिंग_चलाओ")? as i64;
    let mut state = need_str(&args, 2, "तुरिंग_चलाओ")?;
    let trans     = match args.get(3) { Some(Value::Dict(d)) => d.clone(), _ => return Err("तुरिंग_चलाओ: arg 4 संक्रमण Dict आवश्यक".into()) };
    let max_steps = if args.len() > 4 { need_num(&args, 4, "तुरिंग_चलाओ")? as usize } else { 10000 };
    let mut tape: Vec<String> = tape_in.iter().map(|v| {
        if let Value::Str(s) = v { s.clone() } else { "_".into() }
    }).collect();
    let mut steps = 0;
    loop {
        if steps >= max_steps || state == "HALT" || state == "रुको" { break; }
        if head < 0 { tape.insert(0, "_".into()); head = 0; }
        let pos = head as usize;
        while pos >= tape.len() { tape.push("_".into()); }
        let symbol = tape[pos].clone();
        let key = format!("{},{}", state, symbol);
        let action = match trans.get(&key) {
            Some(Value::List(v)) => v.clone(),
            _ => break,
        };
        let new_state = if let Some(Value::Str(s)) = action.get(0) { s.clone() } else { break };
        let write_sym = if let Some(Value::Str(s)) = action.get(1) { s.clone() } else { symbol.clone() };
        let direction = if let Some(Value::Str(s)) = action.get(2) { s.clone() } else { "R".into() };
        tape[pos] = write_sym;
        state = new_state;
        if direction == "R" || direction == "दाएं" { head += 1; } else { head -= 1; }
        steps += 1;
    }
    Ok(Value::List(vec![
        Value::List(tape.iter().map(|s| Value::Str(s.clone())).collect()),
        Value::Number(head as f64),
        Value::Str(state),
        Value::Number(steps as f64),
    ]))
}

fn fn_turing_proof(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(
        "LIPI पूर्णतः ट्यूरिंग-सम्पूर्ण है: (1) असीमित भण्डारण — मानचित्र/सूची, \
        (2) सशर्त शाखाएँ — यदि/अन्यथा, \
        (3) असीमित लूप — जब तक सत्य, \
        (4) अंतर्निर्मित TM अनुकारक — भारत.तुरिंग। \
        यह मॉड्यूल एक सार्वभौमिक ट्यूरिंग मशीन का अनुकरण करता है।".into()
    ))
}

// ════════════════════════════════════════════════════════════════
// MODULE 15 — भारत.यंत्र (Yantra-Purusha: Automata & Rule-Based Agents)
// Source: Samarangana Sutradhara (Bhoja ~1025 CE), Chapter 31
// ════════════════════════════════════════════════════════════════

pub fn yantra_registry() -> Registry {
    vec![
        ("यंत्र_बनाओ",  fn_yantra_create  as NativeFn),
        ("यंत्र_चलाओ",  fn_yantra_run     as NativeFn),
        ("यंत्र_स्थिति", fn_yantra_state   as NativeFn),
        ("नियम_जोड़ो",   fn_rule_add       as NativeFn),
        ("पुरुष_बनाओ",  fn_purusha_create as NativeFn),
        ("पुरुष_सोचो",  fn_purusha_think  as NativeFn),
    ]
}

fn fn_yantra_create(args: Vec<Value>) -> Result<Value, String> {
    let initial = need_str(&args, 0, "यंत्र_बनाओ")?;
    let mut m = std::collections::HashMap::new();
    m.insert("__state__".into(), Value::Str(initial));
    m.insert("__transitions__".into(), Value::Dict(std::collections::HashMap::new()));
    Ok(Value::Dict(m))
}

fn fn_rule_add(args: Vec<Value>) -> Result<Value, String> {
    let mut yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("नियम_जोड़ो: arg 1 यंत्र Dict आवश्यक".into()) };
    let from   = need_str(&args, 1, "नियम_जोड़ो")?;
    let input  = need_str(&args, 2, "नियम_जोड़ो")?;
    let to     = need_str(&args, 3, "नियम_जोड़ो")?;
    let action = args.get(4).cloned().unwrap_or(Value::Str(String::new()));
    let mut trans = match yantra.get("__transitions__") {
        Some(Value::Dict(d)) => d.clone(),
        _ => std::collections::HashMap::new(),
    };
    trans.insert(format!("{},{}", from, input), Value::List(vec![Value::Str(to), action]));
    yantra.insert("__transitions__".into(), Value::Dict(trans));
    Ok(Value::Dict(yantra))
}

fn fn_yantra_run(args: Vec<Value>) -> Result<Value, String> {
    let mut yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_चलाओ: arg 1 यंत्र Dict आवश्यक".into()) };
    let input = need_str(&args, 1, "यंत्र_चलाओ")?;
    let state = match yantra.get("__state__") { Some(Value::Str(s)) => s.clone(), _ => return Err("यंत्र_चलाओ: __state__ नहीं मिला".into()) };
    let trans = match yantra.get("__transitions__") { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_चलाओ: __transitions__ नहीं मिला".into()) };
    let key = format!("{},{}", state, input);
    let (new_state, action) = match trans.get(&key) {
        Some(Value::List(v)) => {
            let ns = if let Some(Value::Str(s)) = v.get(0) { s.clone() } else { state.clone() };
            let ac = v.get(1).cloned().unwrap_or(Value::Str(String::new()));
            (ns, ac)
        },
        _ => (state.clone(), Value::Str(format!("कोई संक्रमण नहीं ({},{})", state, input))),
    };
    yantra.insert("__state__".into(), Value::Str(new_state));
    Ok(Value::List(vec![Value::Dict(yantra), action]))
}

fn fn_yantra_state(args: Vec<Value>) -> Result<Value, String> {
    let yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_स्थिति: Dict आवश्यक".into()) };
    Ok(yantra.get("__state__").cloned().unwrap_or(Value::Nil))
}

fn fn_purusha_create(args: Vec<Value>) -> Result<Value, String> {
    let name  = need_str(&args, 0, "पुरुष_बनाओ")?;
    let rules = match args.get(1) { Some(Value::Dict(d)) => d.clone(), _ => std::collections::HashMap::new() };
    let mut agent = std::collections::HashMap::new();
    agent.insert("नाम".into(), Value::Str(name));
    agent.insert("नियम".into(), Value::Dict(rules));
    agent.insert("स्मृति".into(), Value::List(vec![]));
    Ok(Value::Dict(agent))
}

fn fn_purusha_think(args: Vec<Value>) -> Result<Value, String> {
    let agent = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("पुरुष_सोचो: arg 1 एजेंट Dict आवश्यक".into()) };
    let input = need_str(&args, 1, "पुरुष_सोचो")?;
    let rules = match agent.get("नियम") { Some(Value::Dict(d)) => d.clone(), _ => return Err("पुरुष_सोचो: नियम नहीं मिले".into()) };
    for (condition, action) in &rules {
        if input.contains(condition.as_str()) { return Ok(action.clone()); }
    }
    Ok(Value::Str("अज्ञात — कोई मिलान नियम नहीं".into()))
}

// ════════════════════════════════════════════════════════════════
// MODULE 16 — भारत.व्याकरण (Panini's Ashtadhyayi ~350 BCE)
//
// The Ashtadhyayi is the world's first formal grammar — 4000 sutras
// describing Sanskrit using a metalanguage with operator precedence,
// context-sensitive rules, and metarules (paribhashas).
// Panini's notation is structurally equivalent to a context-free grammar
// with the power of modern BNF, predating Chomsky by 2300 years.
// ════════════════════════════════════════════════════════════════

// संधि_प्रकार: classify the Sandhi (sound junction) between two words
// Panini's Ashtadhyayi devotes entire chapters (6.1–6.4) to sandhi rules.
fn fn_sandhi_prakar(args: Vec<Value>) -> Result<Value, String> {
    let a = need_str(&args, 0, "संधि_प्रकार")?;
    let b = need_str(&args, 1, "संधि_प्रकार")?;
    let a_last  = a.chars().last().unwrap_or(' ');
    let b_first = b.chars().next().unwrap_or(' ');
    let vowels = "अआइईउऊएऐओऔऋ";
    let is_vowel = |c: char| vowels.contains(c);
    let result = if is_vowel(a_last) && is_vowel(b_first) {
        "स्वर_संधि — vowel junction (aciḥ sandhi); e.g. रामः + आगच्छति = रामागच्छति"
    } else if !is_vowel(a_last) && is_vowel(b_first) {
        "व्यंजन_संधि — consonant-vowel junction; assimilation rules apply"
    } else if a_last == 'ः' {
        "विसर्ग_संधि — visarga junction; check R.2.18-2.34 for specific rule"
    } else {
        "व्यंजन_संधि — consonant junction; check Ashtadhyayi 8.2-8.4"
    };
    Ok(Value::Str(result.to_string()))
}

// समास_प्रकार: explain one of Panini's 6 compound word (Samasa) types
// Samasa rules: Ashtadhyayi 2.1.1–2.4.82. Each type maps to a modern CS construct.
fn fn_samas_prakar(args: Vec<Value>) -> Result<Value, String> {
    let kind = need_str(&args, 0, "समास_प्रकार")?;
    let desc = match kind.trim() {
        "अव्ययीभाव" => "Avyayibhava: पूर्व पद प्रधान — adverbial compound; like method chaining: obj.do().then()",
        "तत्पुरुष"  => "Tatpurusha: उत्तर पद प्रधान — second member primary, first qualifies it; like field access: obj.field",
        "कर्मधारय"  => "Karmadharaya: विशेषण-विशेष्य — apposition (both refer to same entity); like type alias: type Nat = Int",
        "द्विगु"    => "Dvigu: संख्यापूर्व — numeral-first compound; like fixed-size array: [T; 5]",
        "बहुव्रीहि" => "Bahuvrihi: अन्यपद प्रधान — neither member primary, describes external referent; like interface: HasWings",
        "द्वन्द्व"  => "Dvandva: उभयपद प्रधान — copulative, both members equal; like union type: A | B",
        _ => return Err(format!("समास_प्रकार: '{}' अज्ञात। अव्ययीभाव/तत्पुरुष/कर्मधारय/द्विगु/बहुव्रीहि/द्वन्द्व", kind)),
    };
    Ok(Value::Str(desc.to_string()))
}

// स्फोट_परीक्षण: Bhartrihari's Sphota theory (~5th CE, Vakyapadiya)
// Sphota = the indivisible word-unit whose meaning is grasped at once.
// Different pronunciations of the same word are different "dhvanis" of one "sphota".
// This is semantic equality — two strings mean the same thing despite surface differences.
fn fn_sphota_parikshan(args: Vec<Value>) -> Result<Value, String> {
    let w1 = need_str(&args, 0, "स्फोट_परीक्षण")?.to_lowercase();
    let w2 = need_str(&args, 1, "स्फोट_परीक्षण")?.to_lowercase();
    let w1t = w1.trim();
    let w2t = w2.trim();
    if w1t == w2t {
        return Ok(Value::Str(format!("एक स्फोट: '{}' और '{}' का अर्थ समान", w1t, w2t)));
    }
    Ok(Value::Str(format!("भिन्न स्फोट: '{}' और '{}' अलग अर्थ", w1t, w2t)))
}

// शिव_सूत्र: Panini's Shiva Sutras — 14 phoneme groups used as regex-like character classes
// (Also called Maheshvara Sutras). Each group name is the mnemonic for a class.
fn fn_shiva_sutra(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "शिव_सूत्र")? as usize;
    let sutras: &[(&str, &str)] = &[
        ("अ इ उ ण्",   "अ,इ,उ — short vowels (अण् class)"),
        ("ऋ ऌ क्",     "ऋ,ऌ — vowels"),
        ("ए ओ ङ्",     "ए,ओ — diphthongs (एच् class)"),
        ("ऐ औ च्",     "ऐ,औ — complex diphthongs"),
        ("ह य व र ट्", "ह,य,व,र — semivowels + ha"),
        ("ल ण्",       "ल — lateral"),
        ("ञ म ङ ण न म्","ञ,म,ङ,ण,न — nasal consonants (यम् class)"),
        ("झ भ ञ्",     "झ,भ — voiced aspirates class"),
        ("घ ढ ध ष्",   "घ,ढ,ध — voiced aspirates II"),
        ("ज ब ग ड द श्","ज,ब,ग,ड,द — voiced stops (जश् class)"),
        ("ख फ छ ठ थ च ट त व्","ख,फ,छ,ठ,थ — voiceless aspirates"),
        ("क प य्",     "क,प — voiceless unaspirated stops"),
        ("श ष स र्",   "श,ष,स — sibilants (शर् class)"),
        ("ह ल्",       "ह — all consonants combined (हल् class)"),
    ];
    if n < 1 || n > 14 { return Err("शिव_सूत्र: 1 से 14 के बीच होना चाहिए".into()); }
    let (phonemes, desc) = sutras[n - 1];
    Ok(Value::Str(format!("शिव_सूत्र {}: {} — {}", n, phonemes, desc)))
}

pub fn vyakaran_registry() -> Registry {
    vec![
        ("संधि_प्रकार",    fn_sandhi_prakar    as NativeFn),
        ("समास_प्रकार",    fn_samas_prakar     as NativeFn),
        ("स्फोट_परीक्षण",  fn_sphota_parikshan as NativeFn),
        ("शिव_सूत्र",      fn_shiva_sutra      as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE 17 — भारत.विज्ञान (Vijnanabhairava Tantra ~7th CE + Katapayadi)
//
// Vijnanabhairava Tantra: 112 dharanas (precise algorithmic instructions
// for shifting consciousness). Each is a numbered, self-contained procedure —
// the most structured ancient enumeration of contemplative algorithms.
// Source: Kashmir Shaivism, ~7th CE, 77 shlokas.
// ════════════════════════════════════════════════════════════════

const VIJNANABHAIRAVA: [&str; 112] = [
    // 1-10: space/void contemplations
    "अनंत_आकाश — ऊपर नीचे सब दिशाओं में अनंत आकाश की कल्पना करो",
    "शून्य_अनुभव — श्वास के बीच की रिक्तता में विश्राम करो",
    "स्पन्द_अनुभव — उठते विचार के पहले क्षण की सूक्ष्म स्पंदन को अनुभव करो",
    "कण्ठ_शून्य — कंठ-केंद्र में शुद्ध चेतना की उपस्थिति महसूस करो",
    "हृदय_आकाश — हृदय के केंद्र में आकाश-सा विस्तार है; वहाँ विश्राम करो",
    "नाभि_चक्र — नाभि-केंद्र से चेतना तरंगों की तरह फैलती है",
    "ब्रह्मरन्ध्र — शीर्ष पर सूक्ष्म स्पंदन बाहर की ओर फैलती है",
    "ऊर्ध्व_दृष्टि — बिना पलक झपकाए ऊपर देखो; मन विलीन हो जाता है",
    "अंध_तम — पूर्ण अंधकार में बैठो; जो शेष रहे वह देखो",
    "नाद_लय — सभी ध्वनियों के नीचे की मूल ध्वनि को सुनो",
    // 11-20: sense contemplations
    "रूप_लय — किसी सुंदर वस्तु को देखो जब तक तुम उसकी सुंदरता में विलीन न हो जाओ",
    "वर्ण_ध्यान — एक रंग पर ध्यान करो जब तक सब कुछ फीका न पड़ जाए",
    "दीप_शिखा — दीपशिखा के अग्रभाग पर ध्यान करो; परिवर्तन का बिंदु",
    "रस_समाधि — कुछ चखो; स्वाद में रहो, वस्तु में नहीं",
    "गंध_लय — सुगंध में विलीन हो जाओ; सूँघने वाले को भूल जाओ",
    "स्पर्श_समाधि — स्पर्श की अनुभूति में रहो, स्पर्श करने वाली वस्तु में नहीं",
    "शब्द_समाधि — ध्वनि का उठना और डूबना देखो; जो शेष रहे वह देखो",
    "दृश्य_अंत — एक दृश्य के बाद, अगले से पहले, एक रिक्तता है",
    "विचार_अंत — एक विचार के बाद, अगले से पहले, शुद्ध चेतना है",
    "कल्प_क्षय — ब्रह्मांड के विलुप्त होने की कल्पना करो; तुम तब क्या हो?",
    // 21-30: breath and energy
    "प्राण_मध्य — श्वास लेने और छोड़ने के बीच रिक्तता में विश्राम करो",
    "अपान_मध्य — श्वास छोड़ने और लेने के बीच भी रिक्तता है; वहाँ ठहरो",
    "प्राणापान_समता — अंदर और बाहर की श्वास को समान करो",
    "कुम्भक_स्थिति — श्वास रोकने में चेतना स्पष्ट हो जाती है",
    "श्वास_अनुगमन — श्वास को उसके उद्गम तक खोजो; कहाँ से आती है?",
    "मन्त्र_उद्भव — मन्त्र के उच्चारण से पहले उसके उठने के क्षण पर ध्यान दो",
    "मन्त्र_अंत — मन्त्र के अंतिम स्वर के बाद की शांति में ठहरो",
    "ॐकार_ध्यान — ओम् की ध्वनि से उसकी अनुगूंज तक विलीन हो जाओ",
    "बीज_ध्यान — बीज-मंत्र पर ध्यान करो जब तक केवल चेतना की अग्नि शेष न रहे",
    "नाद_अनुसंधान — ध्वनि को उसके स्रोत तक खोजो; शुद्ध स्पंदन पाओ",
    // 31-40: body and sensation
    "सुख_समाधि — गहरे आनंद में आनंद बनो, अनुभव करने वाले नहीं",
    "दुख_समाधि — गहरी पीड़ा में पीड़ा बनो; वह मुक्त करती है",
    "भय_समाधि — भय के क्षण में भयभीत होने वाले पर ध्यान दो",
    "क्रोध_समाधि — क्रोध उठते समय उसकी ऊर्जा पर ध्यान दो, विषय पर नहीं",
    "द्वन्द्व_शांति — दो विपरीत के मिलन बिंदु पर, बीच में विश्राम करो",
    "शरीर_आकाश — शरीर को खोखला आकाश देखो; त्वचा केवल रूप की सीमा है",
    "अस्थि_ध्यान — भीतर के कंकाल पर विचार करो; क्या शेष रहता है?",
    "विष्णु_स्मरण — किसी पूजनीय का स्मरण करते समय उस विचार में समाहित हो जाओ",
    "मृत्यु_ध्यान — अपनी मृत्यु का विस्तार से चिंतन करो; कौन शेष रहता है?",
    "जन्म_ध्यान — अपने जन्म का चिंतन करो; पहली श्वास से पहले की चेतना",
    // 41-50: mind and thought
    "निर्विकल्प — बिना किसी मानसिक वस्तु के, शुद्ध चेतना क्या है?",
    "एकाग्रता — एक बिंदु पर इतना ध्यान केंद्रित करो कि एकाग्र होने वाला विलुप्त हो",
    "विचार_साक्षी — विचारों को आते-जाते देखो; किसी का अनुसरण मत करो",
    "अहं_अनुसंधान — 'मैं' की भावना को उसकी जड़ तक खोजो",
    "भावना_शुद्धि — एक शुद्ध सत्ता की कल्पना करो; वह शुद्धता बन जाओ",
    "स्वप्न_जागरण — स्वप्न और जागरण के बीच का क्षण; शुद्ध चेतना",
    "सुषुप्ति_सेतु — गहरी नींद से जागने से पहले का क्षण याद करो",
    "तुरीय_अनुभव — जागरण, स्वप्न और गहरी नींद के नीचे चौथी अवस्था",
    "चित्त_विश्रांति — मन को पूर्णतः विश्राम दो; न दबाओ, न जाने दो",
    "चिंता_मुक्ति — जान लो कि चिंता करने वाला और चिंता एक ही हैं",
    // 51-60: knowledge and identity
    "अहं_ब्रह्म — 'मैं ब्रह्म हूँ' — इसे शब्दों में नहीं, सीधे अनुभव करो",
    "सर्व_ब्रह्म — 'यह सब ब्रह्म है' — हर वस्तु में दिव्य देखो",
    "ज्ञाता_ज्ञेय — जानने वाला और जाना जाने वाला एक हैं; कौन जान रहा है?",
    "दृश्य_द्रष्टा — देखा जाने वाला और देखने वाला एक हैं; क्या देख रहा है?",
    "प्रकाश_अनुभव — अपने शरीर को प्रकाश से भरा कल्पना करो; प्रकाश बन जाओ",
    "अंधकार_अनुभव — बंद आँखों के अंधकार में बैठो; चेतना को देखो",
    "निर्गुण_ध्यान — जिसमें कोई गुण नहीं, उस पर ध्यान करो; शुद्ध सत्ता",
    "सगुण_मुक्ति — किसी रूप की पूर्ण भक्ति से उस रूप से परे हो जाओ",
    "नाम_लय — अपना नाम तब तक दोहराओ जब तक नाम और नामी विलुप्त न हों",
    "रूप_विसर्जन — अपने आप को रूपहीन कल्पना करो; 'तुम' कहाँ जाओगे?",
    // 61-70: space-time
    "व्यापक_आकाश — सभी दिशाओं में अनंत आकाश; चेतना = आकाश",
    "बिन्दु_ध्यान — एक दीप्तिमान बिंदु; सब कुछ उसमें सिकुड़ा हुआ",
    "कालातीत — 'जन्म से पहले भी था, मृत्यु के बाद भी रहूँगा'; कालातीत चेतना",
    "क्षण_ध्यान — इस एक क्षण पर ध्यान दो; इसमें अनंत समाया है",
    "विश्व_लय — पूरे ब्रह्मांड को विलुप्त होते कल्पना करो; राहत महसूस करो",
    "सृष्टि_उद्भव — इसी क्षण ब्रह्मांड को उठते देखो; तुम साक्षी हो",
    "माया_दर्शन — सभी रूपों को शुद्ध चेतना में प्रतिरूप के रूप में देखो",
    "स्वतंत्र_अनुभव — सभी शर्तों से स्वतंत्रता; यही स्व का स्वभाव है",
    "परम_शांति — पूर्ण स्थिरता; चेतना में कोई हलचल नहीं",
    "आनंद_स्वरूप — अपना सच्चा स्वभाव आनंद है; कारणरहित आनंद खोजो",
    // 71-80: advanced
    "सर्वेंद्रिय_एकत्व — सभी इंद्रियाँ एक साथ; स्वाद-गंध-स्पर्श-दृष्टि-श्रवण",
    "इंद्रिय_शून्यता — सभी इंद्रियों को वापस लो; तुम्हारा क्या शेष रहता है?",
    "मन_शून्यता — कोई मानसिक सामग्री नहीं; बिना विषय के चेतना क्या है?",
    "स्फुरण — चेतना के हृदय में प्राथमिक स्पंदन (स्पन्द)",
    "उन्मेष — वह 'खुलने' का क्षण जब चेतना फैलती है",
    "निमेष — वह 'बंद होने' का क्षण जब चेतना सिकुड़ती है",
    "तुरीयातीत — चौथी अवस्था से परे; जो तुरीय को जानता है",
    "अनाहत_नाद — अनाहत ध्वनि; स्पंदन से पहले का स्पंदन",
    "अहं_स्फुरण — 'मैं' का उठना और शुद्ध चेतना में विलुप्त होना",
    "चित्_शक्ति — चेतना-ऊर्जा; जागरूकता और शक्ति एक हैं",
    // 81-90: integration
    "शिव_शक्ति_मिलन — स्थिर और गतिशील तत्वों का मिलन",
    "द्वादशांत — सिर के ऊपर बारह अंगुल का बिंदु; उस परे का आकाश",
    "निजानंद — अपना अंतर्निहित आनंद, परिस्थितियों से स्वतंत्र",
    "पूर्णता — पूर्णता; न जोड़ने को, न घटाने को",
    "स्वभाव — अपना सच्चा स्वभाव; सहज होना",
    "परम_द्वैतातीत — सभी द्वंद्व से परे; यही पहचान मुक्ति है",
    "क्षण_क्षण_नूतन — प्रत्येक क्षण नया; कोई निरंतरता नहीं, केवल उपस्थिति",
    "सहज_समाधि — प्राकृतिक निरंतर अवस्था; सहज मुक्ति",
    "जीवन_मुक्ति — जीते हुए मुक्ति; संसार में स्वतंत्र क्रिया",
    "परा_संवित् — परम ज्ञान; शुद्ध चेतना अपने आप को जानती है",
    // 91-100: supreme
    "अभिनव_गुप्त_प्रसाद — पहचान का अनुग्रह; जो सदा यहाँ था उसे देखना",
    "महाप्रकाश — महान प्रकाश; शुद्ध चेतना सभी का आधार है",
    "स्वात्म_प्रकाश — आत्मा का प्रकाश; भीतर से प्रकाशित",
    "परमार्थ_दर्शन — परम सत्य का दर्शन; जो देखता है वही दृश्य है",
    "विश्व_चैतन्य — विश्व-चेतना; हर कण में चेतना",
    "स्पन्द_समाधि — स्पंदन में समाधि; कंपन ही ब्रह्म है",
    "अकुल_कुल — बिना कुल के; सभी परिवारों का जनक",
    "परमशिव — परम शिव-तत्व; सभी अनुभवों का आधार",
    "नित्य_उदित — सदा उगता हुआ; चेतना कभी अस्त नहीं होती",
    "सर्वज्ञ — सर्वज्ञता; जो जानता है वह सब में व्याप्त है",
    // 101-112: liberation
    "अनुत्तर — जिससे परे कुछ नहीं; परम वास्तविकता",
    "क्रम_मुक्ति — क्रमिक मुक्ति; एक एक कर सभी बंधन टूटते हैं",
    "अकस्मात्_बोध — अचानक जागरण; बिना कारण का बोध",
    "सतत_स्मरण — निरंतर स्मरण; हर पल में पहचान",
    "परिपूर्ण_बोध — पूर्ण जागरण; कुछ भी बाहर नहीं",
    "सिद्ध_स्वभाव — सिद्ध का स्वभाव; प्रयास के बिना पूर्णता",
    "शाश्वत_वर्तमान — शाश्वत वर्तमान; भूत और भविष्य वर्तमान में हैं",
    "अद्वय_बोध — अद्वैत बोध; दो नहीं, एक ही है",
    "परम_प्रसाद — परम अनुग्रह; प्रयास के बिना आने वाली कृपा",
    "विश्राम — विश्राम; कहीं नहीं जाना, कुछ नहीं करना",
    "सहज_स्थिति — सहज स्थिति; स्वाभाविक और निरंतर",
    "परम_स्वातंत्र्य — परम स्वतंत्रता; यह पहचान ही मोक्ष है",
];

fn fn_vijnanabhairava(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "विज्ञानभैरव")? as usize;
    if n < 1 || n > 112 { return Err("विज्ञानभैरव: 1 से 112 के बीच होना चाहिए".into()); }
    Ok(Value::Str(format!("धारणा {}: {}", n, VIJNANABHAIRAVA[n - 1])))
}

// सभी_धारणाएँ: returns all 112 dharana names as a List of Strings
fn fn_sabhi_dharana(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(VIJNANABHAIRAVA.iter().enumerate()
        .map(|(i, s)| {
            let title = s.split(" — ").next().unwrap_or(s);
            Value::Str(format!("{}: {}", i + 1, title))
        })
        .collect()))
}

pub fn vigyan_registry() -> Registry {
    vec![
        ("विज्ञानभैरव",   fn_vijnanabhairava as NativeFn),
        ("सभी_धारणाएँ",  fn_sabhi_dharana   as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.json  (JSON parse / serialize)
//
// Hand-written recursive-descent parser, pure Rust, no crates.
//   json_पढ़ो(text)  — JSON string → LIPI Value
//   json_लिखो(value) — LIPI Value → JSON string (keys sorted)
// ════════════════════════════════════════════════════════════════

struct JsonParser {
    chars: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn new(text: &str) -> Self {
        JsonParser { chars: text.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() { self.pos += 1; }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.pos += 1;
        }
    }

    fn err(&self, msg: &str) -> String {
        format!("JSON त्रुटि (स्थान {}): {}", self.pos, msg)
    }

    /// Expect the exact keyword `kw` starting at current position.
    fn expect_keyword(&mut self, kw: &str) -> Result<(), String> {
        for expected in kw.chars() {
            match self.bump() {
                Some(c) if c == expected => {}
                _ => return Err(self.err(&format!("'{}' अपेक्षित था", kw))),
            }
        }
        Ok(())
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => Ok(Value::Str(self.parse_string()?)),
            Some('t') => { self.expect_keyword("true")?;  Ok(Value::Bool(true)) }
            Some('f') => { self.expect_keyword("false")?; Ok(Value::Bool(false)) }
            Some('n') => { self.expect_keyword("null")?;  Ok(Value::Nil) }
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(self.err(&format!("अनपेक्षित अक्षर '{}'", c))),
            None    => Err(self.err("अधूरा JSON — मान अपेक्षित था")),
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.bump(); // consume '{'
        let mut map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
        self.skip_ws();
        if self.peek() == Some('}') { self.bump(); return Ok(Value::Dict(map)); }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err(self.err("कोश की कुंजी \" से शुरू होनी चाहिए"));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(':') {
                return Err(self.err("कुंजी के बाद ':' अपेक्षित था"));
            }
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => return Ok(Value::Dict(map)),
                _ => return Err(self.err("',' या '}' अपेक्षित था")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.bump(); // consume '['
        let mut items: Vec<Value> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') { self.bump(); return Ok(Value::List(items)); }
        loop {
            let val = self.parse_value()?;
            items.push(val);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => return Ok(Value::List(items)),
                _ => return Err(self.err("',' या ']' अपेक्षित था")),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let mut code: u32 = 0;
        for _ in 0..4 {
            let c = self.bump().ok_or_else(|| self.err("\\u के बाद 4 hex अंक अपेक्षित थे"))?;
            let d = c.to_digit(16).ok_or_else(|| self.err(&format!("'{}' hex अंक नहीं है", c)))?;
            code = code * 16 + d;
        }
        Ok(code)
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.bump(); // consume opening '"'
        let mut s = String::new();
        loop {
            match self.bump() {
                None => return Err(self.err("वाक्य का अंत \" नहीं मिला")),
                Some('"') => return Ok(s),
                Some('\\') => match self.bump() {
                    Some('"')  => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/')  => s.push('/'),
                    Some('n')  => s.push('\n'),
                    Some('t')  => s.push('\t'),
                    Some('r')  => s.push('\r'),
                    Some('b')  => s.push('\u{0008}'),
                    Some('f')  => s.push('\u{000C}'),
                    Some('u')  => {
                        let hi = self.parse_hex4()?;
                        let cp = if (0xD800..=0xDBFF).contains(&hi) {
                            // surrogate pair — expect \uXXXX low surrogate
                            if self.bump() != Some('\\') || self.bump() != Some('u') {
                                return Err(self.err("सरोगेट जोड़ी अधूरी है — \\u अपेक्षित था"));
                            }
                            let lo = self.parse_hex4()?;
                            if !(0xDC00..=0xDFFF).contains(&lo) {
                                return Err(self.err("अमान्य सरोगेट जोड़ी"));
                            }
                            0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                        } else if (0xDC00..=0xDFFF).contains(&hi) {
                            return Err(self.err("अकेला निम्न सरोगेट अमान्य है"));
                        } else {
                            hi
                        };
                        match char::from_u32(cp) {
                            Some(c) => s.push(c),
                            None => return Err(self.err("अमान्य यूनिकोड कोड बिंदु")),
                        }
                    }
                    Some(c) => return Err(self.err(&format!("अमान्य escape '\\{}'", c))),
                    None => return Err(self.err("अधूरा escape क्रम")),
                },
                Some(c) => s.push(c),
            }
        }
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        if self.peek() == Some('-') { self.bump(); }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) { self.bump(); }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) { self.bump(); }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) { self.bump(); }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) { self.bump(); }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        text.parse::<f64>()
            .map(Value::Number)
            .map_err(|_| self.err(&format!("'{}' मान्य संख्या नहीं है", text)))
    }
}

/// Escape a Rust string into a JSON string literal (with surrounding quotes).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Serialize a LIPI Value to a JSON string. Dict keys sorted for determinism.
fn json_serialize(v: &Value) -> Result<String, String> {
    match v {
        Value::Nil       => Ok("null".to_string()),
        Value::Bool(b)   => Ok(if *b { "true".into() } else { "false".into() }),
        Value::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                return Err("json_लिखो: अनंत/NaN को JSON में नहीं लिखा जा सकता".into());
            }
            Ok(format_num(*n))
        }
        Value::Str(s)    => Ok(json_escape(s)),
        Value::List(items) => {
            let parts: Result<Vec<String>, String> = items.iter().map(json_serialize).collect();
            Ok(format!("[{}]", parts?.join(", ")))
        }
        Value::Dict(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut parts: Vec<String> = Vec::with_capacity(keys.len());
            for k in keys {
                parts.push(format!("{}: {}", json_escape(k), json_serialize(&map[k])?));
            }
            Ok(format!("{{{}}}", parts.join(", ")))
        }
        other => Err(format!("json_लिखो: इस मान को JSON में नहीं बदला जा सकता: {}", other)),
    }
}

fn fn_json_padho(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        Some(other) => return Err(format!("json_पढ़ो: वाक्य अपेक्षित, मिला {:?}", other)),
        None => return Err("json_पढ़ो: 1 वाँ तर्क आवश्यक है".into()),
    };
    let mut p = JsonParser::new(&text);
    let val = p.parse_value()?;
    p.skip_ws();
    if p.pos < p.chars.len() {
        return Err(p.err("JSON के अंत के बाद अतिरिक्त सामग्री"));
    }
    Ok(val)
}

fn fn_json_likho(args: Vec<Value>) -> Result<Value, String> {
    let v = args.first().ok_or("json_लिखो: 1 वाँ तर्क आवश्यक है")?;
    Ok(Value::Str(json_serialize(v)?))
}

pub fn json_registry() -> Registry {
    vec![
        ("json_पढ़ो", fn_json_padho as NativeFn),
        ("json_लिखो", fn_json_likho as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.समय  (Date / Time)
//
// Pure Rust, no crates. Canonical value = UTC epoch seconds (Number).
// Human-facing breakdown uses IST (UTC+5:30, no DST) — this is an
// Indian language. Civil-date math via the Howard Hinnant algorithms.
//
//   समय_अभी()                            — current UTC epoch seconds
//   समय_बनाओ(वर्ष, माह, दिन, [घं, मि, से]) — IST wall time → epoch
//   समय_विवरण(epoch)                      — epoch → Dict of IST fields
//   समय_स्वरूप(epoch)                     — "YYYY-MM-DD HH:MM:SS" (IST)
//   दिनांक_पार्स(text)                    — "YYYY-MM-DD[ HH:MM:SS]" → epoch
//   समय_जोड़ो(epoch, दिन, [घं, मि, से])   — shift an epoch
//   दिन_अंतर(e1, e2)                      — IST calendar days between epochs
//   अधिवर्ष(वर्ष)                          — leap year → Bool
//   माह_दिन(वर्ष, माह)                     — days in month
// ════════════════════════════════════════════════════════════════

const IST_OFFSET: i64 = 5 * 3600 + 30 * 60; // +5:30, no DST

const VAAR_NAAM: [&str; 7] = [
    "रविवार", "सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार",
];
const MAAH_NAAM: [&str; 12] = [
    "जनवरी", "फरवरी", "मार्च", "अप्रैल", "मई", "जून",
    "जुलाई", "अगस्त", "सितंबर", "अक्टूबर", "नवंबर", "दिसंबर",
];

fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
        _ => 0,
    }
}

/// Civil date → days since 1970-01-01 (Hinnant `days_from_civil`).
fn days_from_civil(mut y: i64, m: i64, d: i64) -> i64 {
    if m <= 2 { y -= 1; }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;                                   // [0, 399]
    let mp  = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;                      // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;           // [0, 146096]
    era * 146097 + doe - 719468
}

/// Days since 1970-01-01 → civil date (Hinnant `civil_from_days`).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;                                // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y   = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);         // [0, 365]
    let mp  = (5 * doy + 2) / 153;                             // [0, 11]
    let d   = doy - (153 * mp + 2) / 5 + 1;                    // [1, 31]
    let m   = if mp < 10 { mp + 3 } else { mp - 9 };           // [1, 12]
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

fn samay_num_arg(args: &[Value], i: usize, fname: &str) -> Result<i64, String> {
    match args.get(i) {
        Some(Value::Number(n)) => Ok(n.floor() as i64),
        Some(other) => Err(format!("{}: {} वाँ तर्क संख्या होनी चाहिए, मिला {}", fname, i + 1, other)),
        None => Err(format!("{}: {} वाँ तर्क आवश्यक है", fname, i + 1)),
    }
}

fn samay_opt_arg(args: &[Value], i: usize, fname: &str) -> Result<i64, String> {
    if args.len() > i { samay_num_arg(args, i, fname) } else { Ok(0) }
}

/// Validate IST wall-time fields and convert to UTC epoch seconds.
fn wall_to_epoch(y: i64, mo: i64, d: i64, h: i64, mi: i64, s: i64, fname: &str) -> Result<i64, String> {
    if !(1..=12).contains(&mo) {
        return Err(format!("{}: माह 1–12 में होना चाहिए, मिला {}", fname, mo));
    }
    let dim = days_in_month(y, mo);
    if !(1..=dim).contains(&d) {
        return Err(format!("{}: {}/{} के लिए दिन 1–{} में होना चाहिए, मिला {}", fname, y, mo, dim, d));
    }
    if !(0..=23).contains(&h) || !(0..=59).contains(&mi) || !(0..=59).contains(&s) {
        return Err(format!("{}: समय अमान्य — {}:{}:{}", fname, h, mi, s));
    }
    Ok(days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + s - IST_OFFSET)
}

fn fn_samay_abhi(_args: Vec<Value>) -> Result<Value, String> {
    #[cfg(target_arch = "wasm32")]
    return Err("समय_अभी(): WASM में सिस्टम घड़ी उपलब्ध नहीं".into());
    #[cfg(not(target_arch = "wasm32"))]
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("समय_अभी(): घड़ी त्रुटि — {e}"))?;
        Ok(Value::Number(now.as_secs() as f64))
    }
}

fn fn_samay_banao(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "समय_बनाओ";
    let y  = samay_num_arg(&args, 0, F)?;
    let mo = samay_num_arg(&args, 1, F)?;
    let d  = samay_num_arg(&args, 2, F)?;
    let h  = samay_opt_arg(&args, 3, F)?;
    let mi = samay_opt_arg(&args, 4, F)?;
    let s  = samay_opt_arg(&args, 5, F)?;
    Ok(Value::Number(wall_to_epoch(y, mo, d, h, mi, s, F)? as f64))
}

/// Split a UTC epoch into IST civil-date parts: (y, mo, d, h, mi, s, weekday 0=रविवार).
fn epoch_to_wall(epoch: i64) -> (i64, i64, i64, i64, i64, i64, i64) {
    let t = epoch + IST_OFFSET;
    let days = t.div_euclid(86400);
    let sod  = t.rem_euclid(86400);
    let (y, mo, d) = civil_from_days(days);
    let wd = (days + 4).rem_euclid(7); // 1970-01-01 = Thursday = index 4
    (y, mo, d, sod / 3600, (sod % 3600) / 60, sod % 60, wd)
}

fn fn_samay_vivaran(args: Vec<Value>) -> Result<Value, String> {
    let epoch = samay_num_arg(&args, 0, "समय_विवरण")?;
    let (y, mo, d, h, mi, s, wd) = epoch_to_wall(epoch);
    let mut m = std::collections::HashMap::new();
    m.insert("वर्ष".to_string(),     Value::Number(y as f64));
    m.insert("माह".to_string(),      Value::Number(mo as f64));
    m.insert("दिन".to_string(),      Value::Number(d as f64));
    m.insert("घंटा".to_string(),     Value::Number(h as f64));
    m.insert("मिनट".to_string(),     Value::Number(mi as f64));
    m.insert("सेकंड".to_string(),    Value::Number(s as f64));
    m.insert("वार".to_string(),      Value::Number(wd as f64));
    m.insert("वार_नाम".to_string(),  Value::Str(VAAR_NAAM[wd as usize].to_string()));
    m.insert("माह_नाम".to_string(),  Value::Str(MAAH_NAAM[(mo - 1) as usize].to_string()));
    Ok(Value::Dict(m))
}

fn fn_samay_swaroop(args: Vec<Value>) -> Result<Value, String> {
    let epoch = samay_num_arg(&args, 0, "समय_स्वरूप")?;
    let (y, mo, d, h, mi, s, _) = epoch_to_wall(epoch);
    Ok(Value::Str(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, mi, s)))
}

fn fn_dinank_parse(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "दिनांक_पार्स";
    let text = match args.first() {
        Some(Value::Str(s)) => s.trim().to_string(),
        Some(other) => return Err(format!("{}: वाक्य अपेक्षित, मिला {}", F, other)),
        None => return Err(format!("{}: 1 वाँ तर्क आवश्यक है", F)),
    };
    let bad = || format!("{}: स्वरूप \"YYYY-MM-DD\" या \"YYYY-MM-DD HH:MM:SS\" अपेक्षित, मिला \"{}\"", F, text);
    let mut halves = text.split_whitespace();
    let date = halves.next().ok_or_else(bad)?;
    let time = halves.next();
    if halves.next().is_some() { return Err(bad()); }

    let dp: Vec<i64> = date.split('-')
        .map(|p| p.parse::<i64>())
        .collect::<Result<_, _>>().map_err(|_| bad())?;
    if dp.len() != 3 { return Err(bad()); }

    let (h, mi, s) = match time {
        None => (0, 0, 0),
        Some(t) => {
            let tp: Vec<i64> = t.split(':')
                .map(|p| p.parse::<i64>())
                .collect::<Result<_, _>>().map_err(|_| bad())?;
            if tp.len() != 3 { return Err(bad()); }
            (tp[0], tp[1], tp[2])
        }
    };
    Ok(Value::Number(wall_to_epoch(dp[0], dp[1], dp[2], h, mi, s, F)? as f64))
}

fn fn_samay_jodo(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "समय_जोड़ो";
    let epoch = samay_num_arg(&args, 0, F)?;
    let d  = samay_num_arg(&args, 1, F)?;
    let h  = samay_opt_arg(&args, 2, F)?;
    let mi = samay_opt_arg(&args, 3, F)?;
    let s  = samay_opt_arg(&args, 4, F)?;
    Ok(Value::Number((epoch + d * 86400 + h * 3600 + mi * 60 + s) as f64))
}

fn fn_din_antar(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "दिन_अंतर";
    let e1 = samay_num_arg(&args, 0, F)?;
    let e2 = samay_num_arg(&args, 1, F)?;
    let d1 = (e1 + IST_OFFSET).div_euclid(86400);
    let d2 = (e2 + IST_OFFSET).div_euclid(86400);
    Ok(Value::Number((d2 - d1) as f64))
}

fn fn_adhivarsh(args: Vec<Value>) -> Result<Value, String> {
    let y = samay_num_arg(&args, 0, "अधिवर्ष")?;
    Ok(Value::Bool(is_leap(y)))
}

fn fn_maah_din(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "माह_दिन";
    let y = samay_num_arg(&args, 0, F)?;
    let m = samay_num_arg(&args, 1, F)?;
    if !(1..=12).contains(&m) {
        return Err(format!("{}: माह 1–12 में होना चाहिए, मिला {}", F, m));
    }
    Ok(Value::Number(days_in_month(y, m) as f64))
}

pub fn samay_registry() -> Registry {
    vec![
        ("समय_अभी",     fn_samay_abhi as NativeFn),
        ("समय_बनाओ",    fn_samay_banao as NativeFn),
        ("समय_विवरण",   fn_samay_vivaran as NativeFn),
        ("समय_स्वरूप",   fn_samay_swaroop as NativeFn),
        ("दिनांक_पार्स", fn_dinank_parse as NativeFn),
        ("समय_जोड़ो",   fn_samay_jodo as NativeFn),
        ("दिन_अंतर",    fn_din_antar as NativeFn),
        ("अधिवर्ष",      fn_adhivarsh as NativeFn),
        ("माह_दिन",     fn_maah_din as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.csv  (CSV parse / serialize, RFC 4180)
//
//   csv_पढ़ो(text)         — CSV text → List of List of Str
//   csv_शीर्षक_पढ़ो(text)   — first row = headers → List of Dict
//   csv_लिखो(rows)         — List of Lists → CSV text
//
// Fields stay strings — convert with पूर्णांक() as needed.
// ════════════════════════════════════════════════════════════════

fn csv_parse(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut row_has_content = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') { chars.next(); field.push('"'); }
                    else { in_quotes = false; }
                }
                _ => field.push(c),
            }
        } else {
            match c {
                '"' if field.is_empty() => { in_quotes = true; row_has_content = true; }
                ',' => {
                    row.push(std::mem::take(&mut field));
                    row_has_content = true;
                }
                '\r' | '\n' => {
                    if c == '\r' && chars.peek() == Some(&'\n') { chars.next(); }
                    if row_has_content || !field.is_empty() || !row.is_empty() {
                        row.push(std::mem::take(&mut field));
                        rows.push(std::mem::take(&mut row));
                    }
                    row_has_content = false; // blank lines are skipped
                }
                _ => { field.push(c); row_has_content = true; }
            }
        }
    }
    if in_quotes {
        return Err("csv_पढ़ो: उद्धरण (\") बंद नहीं हुआ".into());
    }
    if row_has_content || !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    Ok(rows)
}

fn fn_csv_padho(args: Vec<Value>) -> Result<Value, String> {
    let text = match args.first() {
        Some(Value::Str(s)) => s,
        Some(other) => return Err(format!("csv_पढ़ो: वाक्य अपेक्षित, मिला {}", other)),
        None => return Err("csv_पढ़ो: 1 वाँ तर्क आवश्यक है".into()),
    };
    let rows = csv_parse(text)?;
    Ok(Value::List(rows.into_iter()
        .map(|r| Value::List(r.into_iter().map(Value::Str).collect()))
        .collect()))
}

fn fn_csv_shirshak_padho(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "csv_शीर्षक_पढ़ो";
    let text = match args.first() {
        Some(Value::Str(s)) => s,
        Some(other) => return Err(format!("{}: वाक्य अपेक्षित, मिला {}", F, other)),
        None => return Err(format!("{}: 1 वाँ तर्क आवश्यक है", F)),
    };
    let mut rows = csv_parse(text)?.into_iter();
    let headers = rows.next().ok_or_else(|| format!("{}: शीर्षक पंक्ति नहीं मिली", F))?;
    let mut out = Vec::new();
    for (i, row) in rows.enumerate() {
        if row.len() != headers.len() {
            return Err(format!(
                "{}: पंक्ति {} में {} फ़ील्ड हैं, शीर्षक में {}", F, i + 2, row.len(), headers.len()
            ));
        }
        let mut m = std::collections::HashMap::new();
        for (h, v) in headers.iter().zip(row) { m.insert(h.clone(), Value::Str(v)); }
        out.push(Value::Dict(m));
    }
    Ok(Value::List(out))
}

fn csv_escape_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn fn_csv_likho(args: Vec<Value>) -> Result<Value, String> {
    const F: &str = "csv_लिखो";
    let rows = match args.first() {
        Some(Value::List(rows)) => rows,
        Some(other) => return Err(format!("{}: सूची अपेक्षित, मिला {}", F, other)),
        None => return Err(format!("{}: 1 वाँ तर्क आवश्यक है", F)),
    };
    let mut lines = Vec::with_capacity(rows.len());
    for (i, row) in rows.iter().enumerate() {
        let cells = match row {
            Value::List(cells) => cells,
            other => return Err(format!("{}: पंक्ति {} सूची होनी चाहिए, मिली {}", F, i + 1, other)),
        };
        let line: Vec<String> = cells.iter().map(|c| match c {
            Value::Str(s) => csv_escape_field(s),
            other => csv_escape_field(&format!("{}", other)),
        }).collect();
        lines.push(line.join(","));
    }
    let mut out = lines.join("\n");
    if !out.is_empty() { out.push('\n'); }
    Ok(Value::Str(out))
}

pub fn csv_registry() -> Registry {
    vec![
        ("csv_पढ़ो",        fn_csv_padho as NativeFn),
        ("csv_शीर्षक_पढ़ो",  fn_csv_shirshak_padho as NativeFn),
        ("csv_लिखो",        fn_csv_likho as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.कूट  (Hashing + Base64)
//
//   sha256(text)       — SHA-256 hex digest of UTF-8 bytes
//   md5(text)          — MD5 hex digest (checksums only — not secure)
//   base64_कूट(text)   — UTF-8 → Base64 (standard alphabet, padded)
//   base64_खोलो(text)  — Base64 → original text (must be valid UTF-8)
//
// Pure Rust reference implementations, no crates.
// ════════════════════════════════════════════════════════════════

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bitlen.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4*i], chunk[4*i+1], chunk[4*i+2], chunk[4*i+3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{:08x}", x)).collect()
}

fn md5_hex(data: &[u8]) -> String {
    // Per-round left-rotate amounts (RFC 1321)
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20, 5,  9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    // K[i] = floor(|sin(i+1)| * 2^32) — computed, matches the spec definition
    let k: Vec<u32> = (0..64)
        .map(|i| (((i as f64 + 1.0).sin().abs()) * 4294967296.0) as u32)
        .collect();

    let (mut a0, mut b0, mut c0, mut d0) =
        (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);

    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bitlen.to_le_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([chunk[4*i], chunk[4*i+1], chunk[4*i+2], chunk[4*i+3]]);
        }
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f).wrapping_add(k[i]).wrapping_add(m[g]).rotate_left(S[i]),
            );
            a = tmp;
        }
        a0 = a0.wrapping_add(a); b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c); d0 = d0.wrapping_add(d);
    }
    [a0, b0, c0, d0].iter()
        .flat_map(|x| x.to_le_bytes())
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = u32::from_be_bytes([0, b[0], b[1], b[2]]);
        out.push(B64_ALPHABET[(n >> 18 & 63) as usize] as char);
        out.push(B64_ALPHABET[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 { B64_ALPHABET[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64_ALPHABET[(n & 63) as usize] as char } else { '=' });
    }
    out
}

fn base64_decode(text: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = text.bytes().filter(|b| !b" \t\r\n".contains(b)).collect();
    if clean.len() % 4 != 0 {
        return Err("base64_खोलो: लम्बाई 4 की गुणज होनी चाहिए".into());
    }
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    for quad in clean.chunks_exact(4) {
        let mut n: u32 = 0;
        let mut pad = 0;
        for (i, &ch) in quad.iter().enumerate() {
            let v = if ch == b'=' {
                if i < 2 { return Err("base64_खोलो: '=' गलत स्थान पर".into()); }
                pad += 1;
                0
            } else {
                if pad > 0 { return Err("base64_खोलो: '=' के बाद अक्षर".into()); }
                B64_ALPHABET.iter().position(|&a| a == ch)
                    .ok_or_else(|| format!("base64_खोलो: अमान्य अक्षर '{}'", ch as char))? as u32
            };
            n = (n << 6) | v;
        }
        let bytes = n.to_be_bytes();
        out.push(bytes[1]);
        if pad < 2 { out.push(bytes[2]); }
        if pad < 1 { out.push(bytes[3]); }
    }
    Ok(out)
}

fn koot_str_arg<'a>(args: &'a [Value], fname: &str) -> Result<&'a str, String> {
    match args.first() {
        Some(Value::Str(s)) => Ok(s),
        Some(other) => Err(format!("{}: वाक्य अपेक्षित, मिला {}", fname, other)),
        None => Err(format!("{}: 1 वाँ तर्क आवश्यक है", fname)),
    }
}

fn fn_sha256(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(sha256_hex(koot_str_arg(&args, "sha256")?.as_bytes())))
}

fn fn_md5(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(md5_hex(koot_str_arg(&args, "md5")?.as_bytes())))
}

fn fn_base64_koot(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(base64_encode(koot_str_arg(&args, "base64_कूट")?.as_bytes())))
}

fn fn_base64_kholo(args: Vec<Value>) -> Result<Value, String> {
    let bytes = base64_decode(koot_str_arg(&args, "base64_खोलो")?)?;
    String::from_utf8(bytes)
        .map(Value::Str)
        .map_err(|_| "base64_खोलो: परिणाम मान्य UTF-8 नहीं है".into())
}

pub fn koot_registry() -> Registry {
    vec![
        ("sha256",       fn_sha256 as NativeFn),
        ("md5",          fn_md5 as NativeFn),
        ("base64_कूट",   fn_base64_koot as NativeFn),
        ("base64_खोलो",  fn_base64_kholo as NativeFn),
    ]
}

// ════════════════════════════════════════════════════════════════
// MODULE — भारत.http  (HTTP/1.1 client — pure Rust std::net, no TLS)
//
//   http_पाओ(url)                  — GET
//   http_पाओ(url, headers)         — GET with custom headers (Dict)
//   http_भेजो(url, body)           — POST (default Content-Type: application/json)
//   http_भेजो(url, body, headers)  — POST with custom headers
//
// Both return a Dict:
//   { "स्थिति": Number, "शीर्षक": Dict (keys lowercased), "सामग्री": Str }
//
// http:// only — https:// raises a catchable error (no TLS in pure Rust).
// Connection: close is always sent; body is read to EOF, with
// Transfer-Encoding: chunked decoded when present. 10 s timeouts.
// Not available in WASM builds.
// ════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
struct HttpUrl {
    host: String,
    port: u16,
    path: String, // includes query, always starts with '/'
}

#[cfg(not(target_arch = "wasm32"))]
fn http_parse_url(url: &str, fname: &str) -> Result<HttpUrl, String> {
    let url = url.trim();
    if url.starts_with("https://") {
        return Err(format!(
            "{}: https:// अभी समर्थित नहीं है (TLS उपलब्ध नहीं) — केवल http:// प्रयोग करें", fname));
    }
    let rest = match url.strip_prefix("http://") {
        Some(r) => r,
        None => {
            if let Some(pos) = url.find("://") {
                return Err(format!("{}: अज्ञात स्कीम \"{}\" — केवल http:// समर्थित है",
                                   fname, &url[..pos]));
            }
            return Err(format!("{}: अमान्य URL \"{}\" — http:// से शुरू होना चाहिए", fname, url));
        }
    };
    let (hostport, path) = match rest.find(|c| c == '/' || c == '?') {
        Some(i) => {
            let (h, p) = rest.split_at(i);
            let path = if p.starts_with('?') { format!("/{}", p) } else { p.to_string() };
            (h, path)
        }
        None => (rest, "/".to_string()),
    };
    if hostport.is_empty() {
        return Err(format!("{}: अमान्य URL \"{}\" — होस्ट नहीं मिला", fname, url));
    }
    let (host, port) = match hostport.rfind(':') {
        Some(i) => {
            let pstr = &hostport[i + 1..];
            let p = pstr.parse::<u16>()
                .map_err(|_| format!("{}: अमान्य पोर्ट \"{}\"", fname, pstr))?;
            (hostport[..i].to_string(), p)
        }
        None => (hostport.to_string(), 80),
    };
    if host.is_empty() {
        return Err(format!("{}: अमान्य URL \"{}\" — होस्ट नहीं मिला", fname, url));
    }
    Ok(HttpUrl { host, port, path })
}

#[cfg(not(target_arch = "wasm32"))]
fn http_str_arg(args: &[Value], i: usize, fname: &str) -> Result<String, String> {
    match args.get(i) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!("{}: वाक्य अपेक्षित, मिला {}", fname, other)),
        None => Err(format!("{}: {} वाँ तर्क आवश्यक है", fname, i + 1)),
    }
}

/// Optional headers Dict → sorted (name, value) pairs, validated against
/// header injection (no CR/LF, ASCII names only).
#[cfg(not(target_arch = "wasm32"))]
fn http_headers_arg(args: &[Value], i: usize, fname: &str) -> Result<Vec<(String, String)>, String> {
    let dict = match args.get(i) {
        None => return Ok(Vec::new()),
        Some(Value::Dict(d)) => d,
        Some(other) => return Err(format!("{}: शीर्षक के लिए कोश अपेक्षित, मिला {}", fname, other)),
    };
    let mut out: Vec<(String, String)> = Vec::with_capacity(dict.len());
    for (k, v) in dict {
        let val = match v {
            Value::Str(s) => s.clone(),
            Value::Number(_) | Value::Bool(_) => format!("{}", v),
            other => return Err(format!(
                "{}: शीर्षक \"{}\" का मान वाक्य/संख्या होना चाहिए, मिला {}", fname, k, other)),
        };
        if k.is_empty() || !k.is_ascii()
            || k.chars().any(|c| c == ':' || c.is_ascii_whitespace() || c.is_ascii_control()) {
            return Err(format!("{}: अमान्य शीर्षक नाम \"{}\"", fname, k));
        }
        if val.contains('\r') || val.contains('\n') {
            return Err(format!("{}: शीर्षक \"{}\" के मान में नई पंक्ति वर्जित है", fname, k));
        }
        out.push((k.clone(), val));
    }
    out.sort(); // deterministic request bytes (HashMap order is random)
    Ok(out)
}

/// Decode a Transfer-Encoding: chunked body. Trailers are ignored.
#[cfg(not(target_arch = "wasm32"))]
fn http_dechunk(data: &[u8], fname: &str) -> Result<Vec<u8>, String> {
    let bad = || format!("{}: विकृत प्रतिक्रिया — chunked सामग्री अधूरी या अमान्य है", fname);
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0usize;
    loop {
        // size line ends at \n (tolerate bare \n as well as \r\n)
        let nl = data[i..].iter().position(|&b| b == b'\n').ok_or_else(bad)? + i;
        let line = std::str::from_utf8(&data[i..nl]).map_err(|_| bad())?;
        let size_str = line.trim().split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_str, 16).map_err(|_| bad())?;
        i = nl + 1;
        if size == 0 {
            return Ok(out); // ignore optional trailers
        }
        if i + size > data.len() {
            return Err(bad());
        }
        out.extend_from_slice(&data[i..i + size]);
        i += size;
        if data.get(i) == Some(&b'\r') { i += 1; }
        if data.get(i) == Some(&b'\n') { i += 1; } else { return Err(bad()); }
    }
}

/// Parse a raw HTTP/1.x response into (status, headers-lowercased, body bytes).
/// Returns Err if the response is incomplete or malformed.
#[cfg(not(target_arch = "wasm32"))]
fn http_parse_response(raw: &[u8], fname: &str)
    -> Result<(f64, std::collections::HashMap<String, Value>, Vec<u8>), String>
{
    let bad = |what: &str| format!("{}: विकृत प्रतिक्रिया — {}", fname, what);
    // locate end of headers: \r\n\r\n (tolerate \n\n)
    let (head_end, body_start) = {
        let crlf = raw.windows(4).position(|w| w == b"\r\n\r\n");
        match crlf {
            Some(p) => (p, p + 4),
            None => match raw.windows(2).position(|w| w == b"\n\n") {
                Some(p) => (p, p + 2),
                None => return Err(bad("शीर्षक का अंत नहीं मिला")),
            },
        }
    };
    let head = std::str::from_utf8(&raw[..head_end])
        .map_err(|_| bad("शीर्षक मान्य UTF-8 नहीं"))?;
    let mut lines = head.split('\n').map(|l| l.trim_end_matches('\r'));
    let status_line = lines.next().ok_or_else(|| bad("स्थिति-पंक्ति नहीं मिली"))?;
    let mut parts = status_line.split_whitespace();
    let proto = parts.next().unwrap_or("");
    if !proto.starts_with("HTTP/") {
        return Err(bad("स्थिति-पंक्ति HTTP/ से शुरू नहीं होती"));
    }
    let status: u16 = parts.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| bad("स्थिति कोड संख्या नहीं है"))?;

    let mut headers: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for line in lines {
        if line.is_empty() { continue; }
        let (k, v) = match line.split_once(':') {
            Some(kv) => kv,
            None => continue, // tolerate junk header lines
        };
        headers.insert(k.trim().to_ascii_lowercase(), Value::Str(v.trim().to_string()));
    }

    let body_raw = &raw[body_start..];
    let chunked = matches!(headers.get("transfer-encoding"),
        Some(Value::Str(te)) if te.to_ascii_lowercase().contains("chunked"));
    let body = if chunked {
        http_dechunk(body_raw, fname)?
    } else if let Some(Value::Str(cl)) = headers.get("content-length") {
        let n: usize = cl.trim().parse()
            .map_err(|_| bad("Content-Length संख्या नहीं है"))?;
        if body_raw.len() < n {
            return Err(bad("सामग्री Content-Length से छोटी है (अधूरी)"));
        }
        body_raw[..n].to_vec()
    } else {
        body_raw.to_vec() // Connection: close → body runs to EOF
    };
    Ok((status as f64, headers, body))
}

/// Perform one HTTP request and build the LIPI result Dict.
#[cfg(not(target_arch = "wasm32"))]
fn http_request(method: &str, url: &str, body: Option<&str>,
                user_headers: Vec<(String, String)>, fname: &str) -> Result<Value, String> {
    use std::io::Read;
    use std::io::Write;
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    const TIMEOUT: Duration = Duration::from_secs(10);
    let u = http_parse_url(url, fname)?;

    // ── build request bytes ────────────────────────────────────
    let user_has = |name: &str| user_headers.iter()
        .any(|(k, _)| k.eq_ignore_ascii_case(name));
    let mut req = format!("{} {} HTTP/1.1\r\n", method, u.path);
    if !user_has("Host") {
        if u.port == 80 {
            req.push_str(&format!("Host: {}\r\n", u.host));
        } else {
            req.push_str(&format!("Host: {}:{}\r\n", u.host, u.port));
        }
    }
    // Connection: close is forced — body framing relies on EOF.
    req.push_str("Connection: close\r\n");
    if let Some(b) = body {
        if !user_has("Content-Type") {
            req.push_str("Content-Type: application/json\r\n");
        }
        // Content-Length is always computed — a wrong user value would corrupt framing.
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    for (k, v) in &user_headers {
        if k.eq_ignore_ascii_case("Connection") || k.eq_ignore_ascii_case("Content-Length") {
            continue;
        }
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    req.push_str("\r\n");
    let mut req_bytes = req.into_bytes();
    if let Some(b) = body {
        req_bytes.extend_from_slice(b.as_bytes());
    }

    // ── connect (try every resolved address) ───────────────────
    let addrs: Vec<_> = (u.host.as_str(), u.port).to_socket_addrs()
        .map_err(|e| format!("{}: होस्ट \"{}\" नहीं मिला — {}", fname, u.host, e))?
        .collect();
    if addrs.is_empty() {
        return Err(format!("{}: होस्ट \"{}\" का पता नहीं मिला", fname, u.host));
    }
    let mut stream: Option<TcpStream> = None;
    let mut last_err = String::new();
    for addr in &addrs {
        match TcpStream::connect_timeout(addr, TIMEOUT) {
            Ok(s) => { stream = Some(s); break; }
            Err(e) => last_err = e.to_string(),
        }
    }
    let mut stream = stream.ok_or_else(|| format!(
        "{}: कनेक्शन विफल ({}:{}) — {}", fname, u.host, u.port, last_err))?;
    stream.set_read_timeout(Some(TIMEOUT)).ok();
    stream.set_write_timeout(Some(TIMEOUT)).ok();

    // ── send ────────────────────────────────────────────────────
    stream.write_all(&req_bytes)
        .map_err(|e| format!("{}: अनुरोध भेजने में त्रुटि — {}", fname, e))?;

    // ── receive until EOF (Connection: close) ──────────────────
    let mut raw: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&tmp[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut
                   || e.kind() == std::io::ErrorKind::WouldBlock => {
                // server kept the socket open — accept if response already complete
                if http_parse_response(&raw, fname).is_ok() { break; }
                return Err(format!(
                    "{}: समय सीमा समाप्त (10 सेकंड) — सर्वर ने पूरा उत्तर नहीं दिया", fname));
            }
            // some servers reset instead of FIN after Connection: close
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset && !raw.is_empty() => break,
            Err(e) => return Err(format!("{}: पढ़ने में त्रुटि — {}", fname, e)),
        }
    }
    if raw.is_empty() {
        return Err(format!("{}: सर्वर से कोई उत्तर नहीं मिला", fname));
    }

    let (status, headers, body_bytes) = http_parse_response(&raw, fname)?;
    let body_str = String::from_utf8(body_bytes)
        .map_err(|_| format!("{}: सामग्री मान्य UTF-8 नहीं है", fname))?;

    let mut result = std::collections::HashMap::new();
    result.insert("स्थिति".to_string(),  Value::Number(status));
    result.insert("शीर्षक".to_string(),  Value::Dict(headers));
    result.insert("सामग्री".to_string(), Value::Str(body_str));
    Ok(Value::Dict(result))
}

fn fn_http_pao(args: Vec<Value>) -> Result<Value, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = args;
        Err("http_पाओ(): WASM में उपलब्ध नहीं".into())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        const F: &str = "http_पाओ";
        let url = http_str_arg(&args, 0, F)?;
        let headers = http_headers_arg(&args, 1, F)?;
        http_request("GET", &url, None, headers, F)
    }
}

fn fn_http_bhejo(args: Vec<Value>) -> Result<Value, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = args;
        Err("http_भेजो(): WASM में उपलब्ध नहीं".into())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        const F: &str = "http_भेजो";
        let url = http_str_arg(&args, 0, F)?;
        let body = http_str_arg(&args, 1, F)?;
        let headers = http_headers_arg(&args, 2, F)?;
        http_request("POST", &url, Some(&body), headers, F)
    }
}

pub fn http_registry() -> Registry {
    vec![
        ("http_पाओ",   fn_http_pao as NativeFn),
        ("http_भेजो",  fn_http_bhejo as NativeFn),
    ]
}
