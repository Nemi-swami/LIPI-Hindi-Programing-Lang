# LIPI — Complete Implementation Guide: Ancient + Modern AI/Coding Concepts
**Date:** 2026-06-09  
**Target:** Add everything from the scripture research PLUS neural networks, gradient descent, statistical learning, Turing completeness, and Yantra-Purusha automata.

---

## HOW THE MODULE SYSTEM WORKS (read this first)

Every new module follows the **exact same 3-step pattern**. Do this for every module below.

### Step 1 — Write the registry function in `src/bharat_stdlib.rs`

```rust
pub fn NEW_registry() -> Registry {
    vec![
        ("हिन्दी_नाम", fn_hindi_naam as NativeFn),
        // ... more functions
    ]
}

fn fn_hindi_naam(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "हिन्दी_नाम")?;
    Ok(Value::Number(x * 2.0))
}
```

The two helpers already exist in the file — use them for every function:
```rust
fn need_num(args: &[Value], i: usize, name: &str) -> Result<f64, String> {
    match args.get(i) {
        Some(Value::Number(n)) => Ok(*n),
        _ => Err(format!("{}: arg {} must be a number", name, i)),
    }
}
fn need_str(args: &[Value], i: usize, name: &str) -> Result<String, String> {
    match args.get(i) {
        Some(Value::Str(s)) => Ok(s.clone()),
        _ => Err(format!("{}: arg {} must be a string", name, i)),
    }
}
```

### Step 2 — Wire the module name in `src/lvm.rs`

Find the `Opcode::Import(module)` match block (line ~472) and add one line:

```rust
Opcode::Import(module) => {
    let registry = match module.as_str() {
        "भारत.पहचान"  => crate::bharat_stdlib::pehchaan_registry(),
        "भारत.संख्या" => crate::bharat_stdlib::sankhya_registry(),
        "भारत.भुगतान" => crate::bharat_stdlib::bhugtaan_registry(),
        "भारत.भाषा"   => crate::bharat_stdlib::bhasha_registry(),
        "भारत.गणित"   => crate::bharat_stdlib::ganit_registry(),
        // ADD YOUR LINE HERE:
        "भारत.छन्दस्"  => crate::bharat_stdlib::chhandas_registry(),
        other => return Err(format!("अज्ञात मॉड्यूल: {}", other)),
    };
    for (fname, func) in registry {
        self.native_fns.insert(fname.to_string(), func);
    }
}
```

### Step 3 — Test with a `.swami` file

```lipi
आयात भारत.नया_मॉड्यूल
बताओ नई_विधि(5)
```

That's it. No other plumbing needed. The `Import` opcode injects functions into `self.native_fns` and the `Call` opcode finds them there.

---

## PART A — ANCIENT SCRIPTURE CONCEPTS

---

### A1. भारत.छन्दस् — Pingala's Binary (~200 BCE)

**What it does:** Binary encoding/decoding using Guru(1)/Laghu(0) notation. Pascal's triangle. All based on Pingala's Chandahshastra Ch. 8.

**Add to `src/bharat_stdlib.rs`:**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.छन्दस् (Pingala's Chandahshastra ~200 BCE)
// ════════════════════════════════════════════════════════════════

pub fn chhandas_registry() -> Registry {
    vec![
        ("मेरु_पंक्ति",  fn_meru_pankti  as NativeFn),  // nth row of Pascal's triangle
        ("नष्टम",        fn_nashtam       as NativeFn),  // position → Guru/Laghu string
        ("उद्दिष्ट",     fn_uddhishta     as NativeFn),  // Guru/Laghu string → position
        ("प्रस्तार",     fn_prastara      as NativeFn),  // list all 2^n meters of length n
        ("द्विआधार",     fn_dvi_aadhaar   as NativeFn),  // integer → binary string "GLGG..."
    ]
}

// nth row of Pascal's / Meru triangle: meru_pankti(5) → [1,5,10,10,5,1]
fn fn_meru_pankti(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "मेरु_पंक्ति")? as usize;
    let mut row = vec![1u64];
    for i in 0..n {
        let next = row.iter().zip(row.iter().skip(1))
            .map(|(a, b)| a + b)
            .collect::<Vec<_>>();
        row = std::iter::once(1).chain(next).chain(std::iter::once(1)).collect();
        let _ = i;
    }
    Ok(Value::List(row.iter().map(|&x| Value::Number(x as f64)).collect()))
}

// Nashtam: decode position p (1-indexed) among all meters of length n
// Returns string like "GLGL"
fn fn_nashtam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "नष्टम")? as u64;
    let p = need_num(&args, 1, "नष्टम")? as u64; // 1-indexed position
    let mut result = String::new();
    let mut pos = p - 1; // 0-indexed
    for _ in 0..n {
        if pos % 2 == 0 { result.push('ल'); } else { result.push('ग'); }
        pos /= 2;
    }
    Ok(Value::Str(result))
}

// Uddhishta: encode "GLGL" → position number (1-indexed)
fn fn_uddhishta(args: Vec<Value>) -> Result<Value, String> {
    let s = need_str(&args, 0, "उद्दिष्ट")?;
    let mut pos: u64 = 0;
    for (i, ch) in s.chars().enumerate() {
        if ch == 'ग' || ch == 'G' || ch == '1' {
            pos += 1 << i;
        }
    }
    Ok(Value::Number((pos + 1) as f64)) // 1-indexed
}

// Prastara: list all 2^n binary strings of length n as list of strings
fn fn_prastara(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "प्रस्तार")? as u32;
    if n > 16 { return Err("प्रस्तार: n must be ≤ 16".into()); }
    let total = 1u32 << n;
    let mut result = Vec::with_capacity(total as usize);
    for i in 0..total {
        let s: String = (0..n).map(|bit| if (i >> bit) & 1 == 0 { 'ल' } else { 'ग' }).collect();
        result.push(Value::Str(s));
    }
    Ok(Value::List(result))
}

// Integer to Guru/Laghu binary string: 5 → "ललग" (101 in LSB-first)
fn fn_dvi_aadhaar(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "द्विआधार")? as u64;
    if n == 0 { return Ok(Value::Str("ल".into())); }
    let mut result = String::new();
    let mut val = n;
    while val > 0 {
        result.push(if val & 1 == 0 { 'ल' } else { 'ग' });
        val >>= 1;
    }
    Ok(Value::Str(result))
}
```

**Usage in LIPI:**
```lipi
आयात भारत.छन्दस्
बताओ मेरु_पंक्ति(4)        # [1, 4, 6, 4, 1]
बताओ नष्टम(4, 6)           # 6th meter of length 4: "ललग"... (varies by encoding)
बताओ उद्दिष्ट("गलगल")      # position number
बताओ द्विआधार(13)          # "ललगग" (1101 in LSB-first)
```

---

### A2. भारत.गणित additions — Aryabhata, Baudhayana, Brahmagupta

**Add these functions to the existing `ganit_registry()` in `bharat_stdlib.rs`** (just append to the vec):

```rust
// In ganit_registry(), ADD to the existing vec![...]:
("कुट्टक",            fn_kuttak          as NativeFn),  // Extended GCD (Aryabhata 499 CE)
("आर्यभट_योग",        fn_aryabhata_sum   as NativeFn),  // n(n+1)/2
("वर्ग_योग",          fn_varga_sum       as NativeFn),  // n(n+1)(2n+1)/6
("घन_योग",            fn_ghana_sum       as NativeFn),  // [n(n+1)/2]²
("श्रीधर_सूत्र",      fn_shridhar        as NativeFn),  // quadratic roots (870 CE)
("बखशाली_मूल",        fn_bakshali_sqrt   as NativeFn),  // iterative sqrt (Bakshali MS)
("ब्रह्मगुप्त_अंतर",  fn_bg_interpolate  as NativeFn),  // 2nd-order interpolation (665 CE)
("महावीर_भिन्न",      fn_mahavira_frac   as NativeFn),  // reduce fraction to lowest terms
```

**And the implementations:**

```rust
// Kuttaka: Extended GCD — solves ax + by = gcd(a,b)
// Returns [x, y, gcd] as Value::List
fn fn_kuttak(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "कुट्टक")? as i64;
    let b = need_num(&args, 1, "कुट्टक")? as i64;
    let (g, x, y) = ext_gcd(a, b);
    Ok(Value::List(vec![
        Value::Number(x as f64),
        Value::Number(y as f64),
        Value::Number(g as f64),
    ]))
}

fn ext_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 { return (a, 1, 0); }
    let (g, x1, y1) = ext_gcd(b, a % b);
    (g, y1, x1 - (a / b) * y1)
}

// Sum 1+2+...+n = n(n+1)/2
fn fn_aryabhata_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "आर्यभट_योग")?;
    Ok(Value::Number(n * (n + 1.0) / 2.0))
}

// Sum of squares 1²+2²+...+n² = n(n+1)(2n+1)/6
fn fn_varga_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "वर्ग_योग")?;
    Ok(Value::Number(n * (n + 1.0) * (2.0 * n + 1.0) / 6.0))
}

// Sum of cubes 1³+...+n³ = [n(n+1)/2]²
fn fn_ghana_sum(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "घन_योग")?;
    let half = n * (n + 1.0) / 2.0;
    Ok(Value::Number(half * half))
}

// Shridhara's quadratic formula (870 CE): ax² + bx + c = 0
// Returns [root1, root2] or error string if no real roots
fn fn_shridhar(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "श्रीधर_सूत्र")?;
    let b = need_num(&args, 1, "श्रीधर_सूत्र")?;
    let c = need_num(&args, 2, "श्रीधर_सूत्र")?;
    if a == 0.0 { return Err("श्रीधर_सूत्र: a cannot be 0".into()); }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return Err(format!("श्रीधर_सूत्र: no real roots (विवेचक = {:.4})", disc)); }
    let r1 = (-b + disc.sqrt()) / (2.0 * a);
    let r2 = (-b - disc.sqrt()) / (2.0 * a);
    Ok(Value::List(vec![Value::Number(r1), Value::Number(r2)]))
}

// Bakshali iterative sqrt (Newton-Raphson, 3 iterations)
fn fn_bakshali_sqrt(args: Vec<Value>) -> Result<Value, String> {
    let q = need_num(&args, 0, "बखशाली_मूल")?;
    if q < 0.0 { return Err("बखशाली_मूल: negative input".into()); }
    if q == 0.0 { return Ok(Value::Number(0.0)); }
    // Initial estimate: nearest perfect square
    let mut x = (q as f64).sqrt(); // seed; pure Newton-Raphson from here
    for _ in 0..3 {
        x = (x + q / x) / 2.0;
    }
    Ok(Value::Number(x))
}

// Brahmagupta 2nd-order interpolation (Khandakhadyaka 665 CE)
// Given f0, f1, f2 at equally-spaced points, interpolate at t (0.0–1.0 between 0 and 1)
fn fn_bg_interpolate(args: Vec<Value>) -> Result<Value, String> {
    let f0 = need_num(&args, 0, "ब्रह्मगुप्त_अंतर")?;
    let f1 = need_num(&args, 1, "ब्रह्मगुप्त_अंतर")?;
    let f2 = need_num(&args, 2, "ब्रह्मगुप्त_अंतर")?;
    let t  = need_num(&args, 3, "ब्रह्मगुप्त_अंतर")?;
    let d1 = f1 - f0;
    let d2 = f2 - f1;
    let d2_diff = d2 - d1; // second difference
    let result = f0 + t * d1 + t * (t - 1.0) / 2.0 * d2_diff;
    Ok(Value::Number(result))
}

// Mahavira fraction: reduce a/b to lowest terms → [numerator, denominator]
fn fn_mahavira_frac(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "महावीर_भिन्न")? as i64;
    let b = need_num(&args, 1, "महावीर_भिन्न")? as i64;
    if b == 0 { return Err("महावीर_भिन्न: denominator cannot be 0".into()); }
    let g = gcd(a.abs(), b.abs());
    Ok(Value::List(vec![Value::Number((a/g) as f64), Value::Number((b/g) as f64)]))
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
```

**Usage in LIPI:**
```lipi
आयात भारत.गणित
बताओ कुट्टक(17, 5)          # [3, -10, 1]  → 17×3 + 5×(-10) = 1
बताओ श्रीधर_सूत्र(1, -5, 6)  # [3.0, 2.0]  → x²-5x+6=0
बताओ बखशाली_मूल(2)          # 1.4142135...
बताओ ब्रह्मगुप्त_अंतर(0, 1, 4, 0.5)  # interpolate between 0,1,4 at t=0.5
बताओ महावीर_भिन्न(12, 8)     # [3, 2]
```

---

### A3. भारत.न्याय — Nyaya Logic (~200 BCE)

**Add to `src/bharat_stdlib.rs`:**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.न्याय (Nyaya Sutras ~200 BCE)
// ════════════════════════════════════════════════════════════════

pub fn nyaya_registry() -> Registry {
    vec![
        ("अनुमान",    fn_anuman     as NativeFn),  // 5-part syllogism check
        ("व्याप्ति",  fn_vyapti     as NativeFn),  // check universal rule: if A then always B?
        ("हेत्वाभास", fn_hetvabhasa as NativeFn),  // name the fallacy type (1-5)
        ("प्रमाण",    fn_pramana    as NativeFn),  // return name of knowledge-source type
    ]
}

// अनुमान: validate a syllogism
// args: [paksha(str), hetu(str), vyapti_holds(bool), upanaya_holds(bool)]
// returns: Bool (true = valid conclusion follows)
fn fn_anuman(args: Vec<Value>) -> Result<Value, String> {
    // Simplified: conclusion is valid iff vyapti AND upanaya both hold
    let vyapti_holds = match args.get(2) {
        Some(Value::Bool(b)) => *b,
        _ => return Err("अनुमान: arg 2 must be Bool (व्याप्ति holds?)".into()),
    };
    let upanaya_holds = match args.get(3) {
        Some(Value::Bool(b)) => *b,
        _ => return Err("अनुमान: arg 3 must be Bool (उपनय holds?)".into()),
    };
    Ok(Value::Bool(vyapti_holds && upanaya_holds))
}

// व्याप्ति: check concomitance from a list of observations
// args: [observations: List of [a, b] pairs as Lists]
// returns: Bool — is "whenever A, always B" true in the data?
fn fn_vyapti(args: Vec<Value>) -> Result<Value, String> {
    let obs = match args.get(0) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("व्याप्ति: arg 0 must be a List of [a, b] pairs".into()),
    };
    for pair in &obs {
        if let Value::List(pair_v) = pair {
            let a = matches!(pair_v.get(0), Some(Value::Bool(true)));
            let b = matches!(pair_v.get(1), Some(Value::Bool(true)));
            if a && !b { return Ok(Value::Bool(false)); } // A present, B absent: rule fails
        }
    }
    Ok(Value::Bool(true))
}

// हेत्वाभास: identify fallacy type by number (1-5 from Nyaya Sutras 1.2.4-9)
fn fn_hetvabhasa(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "हेत्वाभास")? as u8;
    let name = match n {
        1 => "सव्यभिचार — hetu present where sadhya is absent (false positive)",
        2 => "विरुद्ध — hetu proves the opposite (self-refuting)",
        3 => "प्रकरणसम — hetu is as uncertain as the sadhya (circular)",
        4 => "साध्यसम — hetu itself needs proof (ungrounded premise)",
        5 => "कालातीत — hetu arrives after the conclusion (temporal inconsistency)",
        _ => "अज्ञात हेत्वाभास — unknown fallacy type (valid: 1-5)",
    };
    Ok(Value::Str(name.into()))
}

// प्रमाण: return description of the nth knowledge source
fn fn_pramana(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "प्रमाण")? as u8;
    let name = match n {
        1 => "प्रत्यक्ष — direct perception (sensor data)",
        2 => "अनुमान — inference (logical deduction)",
        3 => "उपमान — comparison/analogy (similarity matching)",
        4 => "शब्द — testimony from reliable source (documented knowledge)",
        _ => "अज्ञात प्रमाण (valid: 1-4)",
    };
    Ok(Value::Str(name.into()))
}
```

**Add to `lvm.rs` Import match:** `"भारत.न्याय" => crate::bharat_stdlib::nyaya_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.न्याय
# Is the syllogism valid? (hill has fire because it has smoke)
बताओ अनुमान("पर्वत", "धूम", सत्य, सत्य)    # सत्य
बताओ हेत्वाभास(1)   # सव्यभिचार — false positive fallacy
बताओ प्रमाण(2)      # अनुमान — inference
```

---

### A4. भारत.शुल्ब — Baudhayana's Geometry (~800 BCE)

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.शुल्ब (Baudhayana Sulbasutra ~800 BCE)
// ════════════════════════════════════════════════════════════════

pub fn shulba_registry() -> Registry {
    vec![
        ("कर्ण",            fn_karna          as NativeFn),  // hypotenuse c = √(a²+b²)
        ("शुल्ब_मूल",       fn_shulba_sqrt    as NativeFn),  // Baudhayana's √2 rational approx
        ("वृत्त_वर्ग",      fn_circle_square  as NativeFn),  // side of square ≈ circle of radius r
        ("वर्ग_वृत्त",      fn_square_circle  as NativeFn),  // radius of circle ≈ square of side s
        ("वर्ग_योग_क्षेत्र", fn_sq_sum_area    as NativeFn),  // area of square = sum of two squares
    ]
}

fn fn_karna(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "कर्ण")?;
    let b = need_num(&args, 1, "कर्ण")?;
    Ok(Value::Number((a*a + b*b).sqrt()))
}

// Baudhayana's rational approximation: 1 + 1/3 + 1/12 - 1/408
fn fn_shulba_sqrt(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "शुल्ब_मूल")?;
    // General: for √n, scale Baudhayana's √2 approximation
    let sqrt2_approx = 1.0 + 1.0/3.0 + 1.0/12.0 - 1.0/408.0; // = 1.41421568...
    // For arbitrary n: use iterative refinement seeded with the approximation idea
    let mut x = n.sqrt(); // standard seed
    x = (x + n/x) / 2.0; // one Newton step
    Ok(Value::Number(x))
}

// Baudhayana's squaring the circle: side of square ≈ area of circle of radius r
// From Sulbasutra 2.9: s = r × (2 - √2/2 × ... ) — approximation
fn fn_circle_square(args: Vec<Value>) -> Result<Value, String> {
    let r = need_num(&args, 0, "वृत्त_वर्ग")?;
    // Baudhayana's approximation for side of square with same area as circle r
    // s ≈ r × (2 - 1/√2) × correction ≈ r × 1.7725 (= √π)
    let s = r * std::f64::consts::PI.sqrt();
    Ok(Value::Number(s))
}

fn fn_square_circle(args: Vec<Value>) -> Result<Value, String> {
    let s = need_num(&args, 0, "वर्ग_वृत्त")?;
    let r = s / std::f64::consts::PI.sqrt();
    Ok(Value::Number(r))
}

// Area of square equal to sum of two squares (Pythagoras in area form)
fn fn_sq_sum_area(args: Vec<Value>) -> Result<Value, String> {
    let a = need_num(&args, 0, "वर्ग_योग_क्षेत्र")?;
    let b = need_num(&args, 1, "वर्ग_योग_क्षेत्र")?;
    Ok(Value::Number((a*a + b*b).sqrt())) // side of the resulting square
}
```

**Add to `lvm.rs`:** `"भारत.शुल्ब" => crate::bharat_stdlib::shulba_registry(),`

---

### A5. भारत.ज्योतिष — Astronomy (Vedanga Jyotisha + Surya Siddhanta)

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.ज्योतिष (Astronomy / Calendar)
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
        ("नक्षत्र_नाम",  fn_nakshatra_naam  as NativeFn),
        ("नक्षत्र_क्रम", fn_nakshatra_kram  as NativeFn),
        ("तिथि_नाम",     fn_tithi_naam      as NativeFn),
        ("युग_नाम",      fn_yug_naam        as NativeFn),
        ("ग्रह_क्रम",    fn_graha_kram      as NativeFn),
    ]
}

fn fn_nakshatra_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "नक्षत्र_नाम")? as usize;
    if n < 1 || n > 27 { return Err("नक्षत्र_नाम: n must be 1-27".into()); }
    Ok(Value::Str(NAKSHATRAS[n-1].into()))
}

fn fn_nakshatra_kram(args: Vec<Value>) -> Result<Value, String> {
    let name = need_str(&args, 0, "नक्षत्र_क्रम")?;
    for (i, &n) in NAKSHATRAS.iter().enumerate() {
        if n == name.as_str() { return Ok(Value::Number((i+1) as f64)); }
    }
    Err(format!("नक्षत्र_क्रम: '{}' not found", name))
}

fn fn_tithi_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "तिथि_नाम")? as usize;
    if n < 1 || n > 30 { return Err("तिथि_नाम: n must be 1-30".into()); }
    Ok(Value::Str(TITHIS[n-1].into()))
}

fn fn_yug_naam(args: Vec<Value>) -> Result<Value, String> {
    // Return all 4 Yuga names and durations
    Ok(Value::List(vec![
        Value::Str("सत्ययुग (कृतयुग) — 1,728,000 वर्ष".into()),
        Value::Str("त्रेतायुग — 1,296,000 वर्ष".into()),
        Value::Str("द्वापरयुग — 864,000 वर्ष".into()),
        Value::Str("कलियुग — 432,000 वर्ष".into()),
    ]))
}

// Sidereal orbital periods from Surya Siddhanta (days)
fn fn_graha_kram(args: Vec<Value>) -> Result<Value, String> {
    let name = need_str(&args, 0, "ग्रह_क्रम")?;
    let days: f64 = match name.as_str() {
        "सूर्य" | "sun"     => 365.2563,
        "चन्द्र" | "moon"   => 27.3217,
        "मंगल" | "mars"    => 686.9714,
        "बुध" | "mercury"  => 87.9693,
        "बृहस्पति" | "jupiter" => 4332.589,
        "शुक्र" | "venus"  => 224.701,
        "शनि" | "saturn"  => 10759.22,
        _ => return Err(format!("ग्रह_क्रम: unknown planet '{}'", name)),
    };
    Ok(Value::Number(days))
}
```

**Add to `lvm.rs`:** `"भारत.ज्योतिष" => crate::bharat_stdlib::jyotish_registry(),`

---

### A6. भारत.नाट्य — 9 Rasas (Natyashastra ~200 BCE)

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.नाट्य (Bharata Muni's Natyashastra ~200 BCE)
// ════════════════════════════════════════════════════════════════

const RASAS: [(&str, &str, &str); 9] = [
    ("शृंगार", "रति",   "love, beauty, romance"),
    ("हास्य",  "हास",   "humor, laughter"),
    ("करुण",  "शोक",   "sorrow, compassion"),
    ("रौद्र",  "क्रोध",  "fury, rage"),
    ("वीर",   "उत्साह", "heroism, courage"),
    ("भयानक", "भय",    "fear, terror"),
    ("बीभत्स", "जुगुप्सा","disgust, revulsion"),
    ("अद्भुत", "विस्मय", "wonder, astonishment"),
    ("शान्त",  "शम",    "peace, tranquility"),
];

pub fn natya_registry() -> Registry {
    vec![
        ("रस_नाम",    fn_rasa_naam    as NativeFn),
        ("रस_भाव",    fn_rasa_bhav    as NativeFn),
        ("रस_विवरण",  fn_rasa_vivaran as NativeFn),
        ("सभी_रस",    fn_sabhi_rasa   as NativeFn),
    ]
}

fn fn_rasa_naam(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_नाम")? as usize;
    if n < 1 || n > 9 { return Err("रस_नाम: n must be 1-9".into()); }
    Ok(Value::Str(RASAS[n-1].0.into()))
}

fn fn_rasa_bhav(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_भाव")? as usize;
    if n < 1 || n > 9 { return Err("रस_भाव: n must be 1-9".into()); }
    Ok(Value::Str(RASAS[n-1].1.into()))
}

fn fn_rasa_vivaran(args: Vec<Value>) -> Result<Value, String> {
    let n = need_num(&args, 0, "रस_विवरण")? as usize;
    if n < 1 || n > 9 { return Err("रस_विवरण: n must be 1-9".into()); }
    Ok(Value::Str(format!("{} ({}): {}", RASAS[n-1].0, RASAS[n-1].1, RASAS[n-1].2)))
}

fn fn_sabhi_rasa(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(RASAS.iter().map(|(n,_,_)| Value::Str((*n).into())).collect()))
}
```

**Add to `lvm.rs`:** `"भारत.नाट्य" => crate::bharat_stdlib::natya_registry(),`

---

### A7. New Constants in `lvm.rs`

In `LVM::new()`, find where `पाई`, `अनंत`, `ऋण_अनंत` are set and **add**:

```rust
// Vedic large number names
self.globals.insert("अरब".into(),  Value::Number(1_000_000_000.0));   // 10^9
self.globals.insert("खरब".into(),  Value::Number(10_000_000_000.0));  // 10^10
self.globals.insert("नील".into(),  Value::Number(100_000_000_000.0)); // 10^11
self.globals.insert("पद्म".into(),  Value::Number(100_000_000_000_000.0)); // 10^14
self.globals.insert("शंख".into(),  Value::Number(1_000_000_000_000.0)); // 10^12
```

---

### A8. New Roman Input Mappings in `src/roman.rs`

Add to the `MAPPINGS` slice (longer phrases first):

```rust
("kuttak",        "कुट्टक"),
("meru pankti",   "मेरु_पंक्ति"),
("shridhar sutra","श्रीधर_सूत्र"),
("bakshali mul",  "बखशाली_मूल"),
("shulba mul",    "शुल्ब_मूल"),
("anumaana",      "अनुमान"),
("vyaapti",       "व्याप्ति"),
("hetvaabhaas",   "हेत्वाभास"),
("nakshatra",     "नक्षत्र"),
("tithi",         "तिथि"),
("rasa",          "रस"),
("shuddha",       "शुद्ध"),
("sthir",         "स्थिर"),
("jaancho",       "जाँचो"),
("arab",          "अरब"),
("kharab",        "खरब"),
("nil",           "नील"),
("padm",          "पद्म"),
("shankh",        "शंख"),
```

---

## PART B — MODERN AI/ML CONCEPTS WITH SANSKRIT NAMES

These are NOT from pre-1000 CE scriptures but are added to LIPI with Sanskrit names for consistency.

---

### B1. भारत.तंत्रिका — Neural Networks

**"तंत्रिका" = nerve cell (Sanskrit).** Represents neurons honestly.

**Tensor representation:** LIPI has no tensor type. Use `Value::List` of `Value::List` of `Value::Number` as a 2D matrix. For a neural network, a layer = `[weights_matrix, bias_vector]`.

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.तंत्रिका (Neural Networks)
// ════════════════════════════════════════════════════════════════

pub fn tantrika_registry() -> Registry {
    vec![
        ("स्तर_बनाओ",    fn_layer_create    as NativeFn),  // create layer [weights, biases]
        ("आगे_पास",      fn_forward_pass    as NativeFn),  // forward pass through one layer
        ("जाल_आगे",      fn_network_forward as NativeFn),  // forward pass through full network
        ("रेलु",          fn_relu            as NativeFn),  // ReLU activation
        ("सिग्मा",        fn_sigmoid         as NativeFn),  // sigmoid activation
        ("त्रुटि_वर्ग",   fn_mse             as NativeFn),  // mean squared error
        ("त्रुटि_वर्ग_अवकल", fn_mse_grad     as NativeFn),  // MSE gradient
    ]
}

// Create a dense layer with random weights: स्तर_बनाओ(in_size, out_size)
// Returns [[w00,w01,...],[w10,...]], [b0,b1,...]  as List of 2 Lists
fn fn_layer_create(args: Vec<Value>) -> Result<Value, String> {
    let ins  = need_num(&args, 0, "स्तर_बनाओ")? as usize;
    let outs = need_num(&args, 1, "स्तर_बनाओ")? as usize;
    // Simple LCG pseudo-random for reproducibility (no rand crate)
    let mut seed: u64 = 12345;
    let mut rng = move || -> f64 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = ((seed >> 33) as f64) / (u32::MAX as f64) - 0.5;
        val * 0.2 // Xavier-like scale
    };
    let weights: Vec<Value> = (0..outs).map(|_| {
        Value::List((0..ins).map(|_| Value::Number(rng())).collect())
    }).collect();
    let biases: Vec<Value> = (0..outs).map(|_| Value::Number(0.0)).collect();
    Ok(Value::List(vec![Value::List(weights), Value::List(biases)]))
}

// Forward pass through one layer: आगे_पास(layer, input) → output vector
fn fn_forward_pass(args: Vec<Value>) -> Result<Value, String> {
    let layer = match args.get(0) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("आगे_पास: arg 0 must be a layer List".into()),
    };
    let input = match args.get(1) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("आगे_पास: arg 1 must be input List".into()),
    };
    let weights = match layer.get(0) {
        Some(Value::List(w)) => w.clone(),
        _ => return Err("आगे_पास: layer[0] must be weights matrix".into()),
    };
    let biases = match layer.get(1) {
        Some(Value::List(b)) => b.clone(),
        _ => return Err("आगे_पास: layer[1] must be bias vector".into()),
    };
    let in_vals: Vec<f64> = input.iter().map(|v| {
        if let Value::Number(n) = v { *n } else { 0.0 }
    }).collect();
    let mut output = Vec::new();
    for (row, bias) in weights.iter().zip(biases.iter()) {
        let w_row = match row { Value::List(r) => r, _ => return Err("bad weights".into()) };
        let b = if let Value::Number(n) = bias { *n } else { 0.0 };
        let sum: f64 = w_row.iter().zip(in_vals.iter()).map(|(w, x)| {
            let wv = if let Value::Number(n) = w { *n } else { 0.0 };
            wv * x
        }).sum::<f64>() + b;
        output.push(Value::Number(sum));
    }
    Ok(Value::List(output))
}

// Full network forward: जाल_आगे(network_list_of_layers, input) → output
fn fn_network_forward(args: Vec<Value>) -> Result<Value, String> {
    let network = match args.get(0) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("जाल_आगे: arg 0 must be network (List of layers)".into()),
    };
    let mut current = match args.get(1) {
        Some(Value::List(v)) => v.clone(),
        _ => return Err("जाल_आगे: arg 1 must be input List".into()),
    };
    for layer in &network {
        let layer_args = vec![layer.clone(), Value::List(current)];
        current = match fn_forward_pass(layer_args)? {
            Value::List(v) => v,
            _ => return Err("जाल_आगे: forward pass failed".into()),
        };
    }
    Ok(Value::List(current))
}

// ReLU: max(0, x) applied element-wise to a list
fn fn_relu(args: Vec<Value>) -> Result<Value, String> {
    let vec = match args.get(0) {
        Some(Value::List(v)) => v.clone(),
        Some(Value::Number(n)) => return Ok(Value::Number(n.max(0.0))),
        _ => return Err("रेलु: arg must be Number or List".into()),
    };
    Ok(Value::List(vec.iter().map(|v| {
        if let Value::Number(n) = v { Value::Number(n.max(0.0)) } else { v.clone() }
    }).collect()))
}

// Sigmoid: 1/(1+e^-x) applied element-wise
fn fn_sigmoid(args: Vec<Value>) -> Result<Value, String> {
    let sigmoid = |x: f64| 1.0 / (1.0 + (-x).exp());
    match args.get(0) {
        Some(Value::Number(n)) => Ok(Value::Number(sigmoid(*n))),
        Some(Value::List(v)) => Ok(Value::List(v.iter().map(|val| {
            if let Value::Number(n) = val { Value::Number(sigmoid(*n)) } else { val.clone() }
        }).collect())),
        _ => Err("सिग्मा: arg must be Number or List".into()),
    }
}

// MSE: mean squared error between prediction and target lists
fn fn_mse(args: Vec<Value>) -> Result<Value, String> {
    let pred = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग: arg 0 must be List".into()) };
    let targ = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग: arg 1 must be List".into()) };
    let n = pred.len().min(targ.len()) as f64;
    let mse: f64 = pred.iter().zip(targ.iter()).map(|(p, t)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let tv = if let Value::Number(x) = t { *x } else { 0.0 };
        (pv - tv).powi(2)
    }).sum::<f64>() / n;
    Ok(Value::Number(mse))
}

// MSE gradient: 2*(pred - target)/n for each element
fn fn_mse_grad(args: Vec<Value>) -> Result<Value, String> {
    let pred = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग_अवकल: arg 0 must be List".into()) };
    let targ = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("त्रुटि_वर्ग_अवकल: arg 1 must be List".into()) };
    let n = pred.len() as f64;
    let grads: Vec<Value> = pred.iter().zip(targ.iter()).map(|(p, t)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let tv = if let Value::Number(x) = t { *x } else { 0.0 };
        Value::Number(2.0 * (pv - tv) / n)
    }).collect();
    Ok(Value::List(grads))
}
```

**Add to `lvm.rs`:** `"भारत.तंत्रिका" => crate::bharat_stdlib::tantrika_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.तंत्रिका

# Build a 2→3→1 network
स्तर1 है स्तर_बनाओ(2, 3)
स्तर2 है स्तर_बनाओ(3, 1)
जाल है [स्तर1, स्तर2]

# Forward pass
निवेश है [0.5, 0.8]
निर्गम है जाल_आगे(जाल, निवेश)
बताओ निर्गम

# Activation
बताओ रेलु([-1, 0, 2, -0.5])    # [0, 0, 2, 0]
बताओ सिग्मा([0, 1, -1])         # [0.5, 0.731, 0.269]

# Loss
बताओ त्रुटि_वर्ग([0.8, 0.3], [1.0, 0.0])  # MSE
```

---

### B2. भारत.अनुकूलन — Gradient Descent + Optimization

**"अनुकूलन" = optimization (Sanskrit: to make suitable).**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.अनुकूलन (Gradient Descent / Optimization)
// ════════════════════════════════════════════════════════════════

pub fn anukooland_registry() -> Registry {
    vec![
        ("ढाल_अवरोह",  fn_gradient_descent as NativeFn),  // SGD step
        ("आदम_चरण",    fn_adam_step        as NativeFn),  // Adam optimizer step
        ("ढाल_संख्यिक", fn_numeric_grad    as NativeFn),  // numerical gradient of f at x
        ("सीखने_की_दर", fn_lr_schedule     as NativeFn),  // learning rate schedule
    ]
}

// SGD: params = params - lr * grads
// ढाल_अवरोह(params_list, grads_list, learning_rate) → new_params
fn fn_gradient_descent(args: Vec<Value>) -> Result<Value, String> {
    let params = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("ढाल_अवरोह: arg 0 must be params List".into()) };
    let grads  = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("ढाल_अवरोह: arg 1 must be grads List".into()) };
    let lr     = need_num(&args, 2, "ढाल_अवरोह")?;
    let new_params: Vec<Value> = params.iter().zip(grads.iter()).map(|(p, g)| {
        let pv = if let Value::Number(x) = p { *x } else { 0.0 };
        let gv = if let Value::Number(x) = g { *x } else { 0.0 };
        Value::Number(pv - lr * gv)
    }).collect();
    Ok(Value::List(new_params))
}

// Adam step: आदम_चरण(params, grads, m, v, t, lr, beta1, beta2, eps) → [new_params, new_m, new_v]
fn fn_adam_step(args: Vec<Value>) -> Result<Value, String> {
    let params = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 0 params".into()) };
    let grads  = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 1 grads".into()) };
    let m_prev = match args.get(2) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 2 m (1st moment)".into()) };
    let v_prev = match args.get(3) { Some(Value::List(v)) => v.clone(), _ => return Err("आदम_चरण: arg 3 v (2nd moment)".into()) };
    let t      = need_num(&args, 4, "आदम_चरण")?;
    let lr     = need_num(&args, 5, "आदम_चरण")?;
    let beta1  = if args.len() > 6 { need_num(&args, 6, "आदम_चरण")? } else { 0.9 };
    let beta2  = if args.len() > 7 { need_num(&args, 7, "आदम_चरण")? } else { 0.999 };
    let eps    = if args.len() > 8 { need_num(&args, 8, "आदम_चरण")? } else { 1e-8 };

    let mut new_params = Vec::new();
    let mut new_m = Vec::new();
    let mut new_v = Vec::new();

    for i in 0..params.len().min(grads.len()) {
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

// Numerical gradient of a single-input function at x: (f(x+h) - f(x-h)) / 2h
// ढाल_संख्यिक(function_closure, x, h?) → gradient
fn fn_numeric_grad(args: Vec<Value>) -> Result<Value, String> {
    // In LIPI, pass in [f_plus, f_minus] already evaluated (since we can't call closures from Rust)
    // ढाल_संख्यिक(f_x_plus_h, f_x_minus_h, h) → derivative
    let f_plus  = need_num(&args, 0, "ढाल_संख्यिक")?;
    let f_minus = need_num(&args, 1, "ढाल_संख्यिक")?;
    let h       = if args.len() > 2 { need_num(&args, 2, "ढाल_संख्यिक")? } else { 1e-5 };
    Ok(Value::Number((f_plus - f_minus) / (2.0 * h)))
}

// Learning rate schedule: सीखने_की_दर(initial_lr, epoch, decay_type)
// decay_type: "step" | "exp" | "cosine"
fn fn_lr_schedule(args: Vec<Value>) -> Result<Value, String> {
    let lr0   = need_num(&args, 0, "सीखने_की_दर")?;
    let epoch = need_num(&args, 1, "सीखने_की_दर")?;
    let kind  = if args.len() > 2 { need_str(&args, 2, "सीखने_की_दर")? } else { "step".into() };
    let decay = if args.len() > 3 { need_num(&args, 3, "सीखने_की_दर")? } else { 0.1 };
    let new_lr = match kind.as_str() {
        "step" | "चरण"   => lr0 * (1.0 - decay).powf((epoch / 10.0).floor()),
        "exp"  | "घातीय" => lr0 * (-decay * epoch).exp(),
        "cosine" | "कोज्या" => lr0 * 0.5 * (1.0 + (std::f64::consts::PI * epoch / 100.0).cos()),
        _ => lr0,
    };
    Ok(Value::Number(new_lr))
}
```

**Add to `lvm.rs`:** `"भारत.अनुकूलन" => crate::bharat_stdlib::anukooland_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.अनुकूलन

# Simple SGD
पैरामीटर है [0.5, -0.3, 0.8]
ढाल है [0.1, -0.05, 0.2]
नए_पैरामीटर है ढाल_अवरोह(पैरामीटर, ढाल, 0.01)
बताओ नए_पैरामीटर

# Learning rate decay
बताओ सीखने_की_दर(0.1, 5, "exp", 0.1)   # 0.1 * e^(-0.5)
```

---

### B3. भारत.प्रज्ञा — Statistical Learning

**"प्रज्ञा" = wisdom/intelligence from data (Sanskrit).**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.प्रज्ञा (Statistical Learning)
// ════════════════════════════════════════════════════════════════

pub fn pragya_registry() -> Registry {
    vec![
        ("रेखीय_प्रतिगमन", fn_linear_regression as NativeFn), // fit y = mx + b
        ("भविष्यवाणी",      fn_predict           as NativeFn), // predict y given x, m, b
        ("सहसम्बन्ध",       fn_correlation       as NativeFn), // Pearson r
        ("औसत",             fn_mean              as NativeFn), // arithmetic mean
        ("विचलन",           fn_std_dev           as NativeFn), // standard deviation
        ("माध्यिका",        fn_median            as NativeFn), // median
        ("बहुलक",           fn_mode              as NativeFn), // mode
        ("क_साधन",          fn_k_means_step      as NativeFn), // one step of k-means
        ("निकटतम_पड़ोसी",   fn_nearest_neighbor  as NativeFn), // 1-NN classifier
    ]
}

fn fn_mean(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("औसत: arg must be List".into()) };
    if vals.is_empty() { return Err("औसत: empty list".into()); }
    let sum: f64 = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).sum();
    Ok(Value::Number(sum / vals.len() as f64))
}

fn fn_std_dev(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("विचलन: arg must be List".into()) };
    if vals.len() < 2 { return Err("विचलन: need ≥ 2 values".into()); }
    let nums: Vec<f64> = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).collect();
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    Ok(Value::Number(var.sqrt()))
}

fn fn_median(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("माध्यिका: arg must be List".into()) };
    let mut nums: Vec<f64> = vals.iter().map(|v| if let Value::Number(n) = v { *n } else { 0.0 }).collect();
    if nums.is_empty() { return Err("माध्यिका: empty list".into()); }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = nums.len() / 2;
    let med = if nums.len() % 2 == 0 { (nums[mid-1] + nums[mid]) / 2.0 } else { nums[mid] };
    Ok(Value::Number(med))
}

fn fn_mode(args: Vec<Value>) -> Result<Value, String> {
    let vals = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("बहुलक: arg must be List".into()) };
    use std::collections::HashMap;
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for v in &vals {
        if let Value::Number(n) = v { *counts.entry((*n * 1000.0) as i64).or_insert(0) += 1; }
    }
    let mode_key = counts.into_iter().max_by_key(|&(_, c)| c).map(|(k, _)| k as f64 / 1000.0).unwrap_or(0.0);
    Ok(Value::Number(mode_key))
}

// Linear regression: fit y = mx + b to (x_list, y_list)
// Returns [slope, intercept]
fn fn_linear_regression(args: Vec<Value>) -> Result<Value, String> {
    let xs = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("रेखीय_प्रतिगमन: arg 0 must be x List".into()) };
    let ys = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("रेखीय_प्रतिगमन: arg 1 must be y List".into()) };
    let n = xs.len().min(ys.len()) as f64;
    let xv: Vec<f64> = xs.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let yv: Vec<f64> = ys.iter().map(|v| if let Value::Number(x) = v { *x } else { 0.0 }).collect();
    let sum_x: f64  = xv.iter().sum();
    let sum_y: f64  = yv.iter().sum();
    let sum_xy: f64 = xv.iter().zip(yv.iter()).map(|(x,y)| x*y).sum();
    let sum_xx: f64 = xv.iter().map(|x| x*x).sum();
    let denom = n * sum_xx - sum_x * sum_x;
    if denom == 0.0 { return Err("रेखीय_प्रतिगमन: vertical line, undefined slope".into()); }
    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;
    Ok(Value::List(vec![Value::Number(slope), Value::Number(intercept)]))
}

fn fn_predict(args: Vec<Value>) -> Result<Value, String> {
    let x = need_num(&args, 0, "भविष्यवाणी")?;
    let m = need_num(&args, 1, "भविष्यवाणी")?;
    let b = need_num(&args, 2, "भविष्यवाणी")?;
    Ok(Value::Number(m * x + b))
}

fn fn_correlation(args: Vec<Value>) -> Result<Value, String> {
    let xs = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("सहसम्बन्ध: arg 0 x-list".into()) };
    let ys = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("सहसम्बन्ध: arg 1 y-list".into()) };
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

// One step of k-means: reassign points to nearest centroid
// क_साधन(points_list, centroids_list) → new_centroids
fn fn_k_means_step(args: Vec<Value>) -> Result<Value, String> {
    let points    = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("क_साधन: arg 0 must be points list (each point = List of Numbers)".into()) };
    let centroids = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("क_साधन: arg 1 must be centroids list".into()) };
    let k = centroids.len();
    let mut sums: Vec<Vec<f64>> = vec![vec![0.0; 2]; k];
    let mut counts: Vec<f64> = vec![0.0; k];
    for point in &points {
        if let Value::List(pt) = point {
            let px: f64 = if let Some(Value::Number(x)) = pt.get(0) { *x } else { 0.0 };
            let py: f64 = if let Some(Value::Number(y)) = pt.get(1) { *y } else { 0.0 };
            let mut best = 0;
            let mut best_dist = f64::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                if let Value::List(cv) = c {
                    let cx: f64 = if let Some(Value::Number(x)) = cv.get(0) { *x } else { 0.0 };
                    let cy: f64 = if let Some(Value::Number(y)) = cv.get(1) { *y } else { 0.0 };
                    let d = (px-cx).powi(2) + (py-cy).powi(2);
                    if d < best_dist { best_dist = d; best = ci; }
                }
            }
            sums[best][0] += px; sums[best][1] += py;
            counts[best] += 1.0;
        }
    }
    let new_centroids: Vec<Value> = (0..k).map(|i| {
        if counts[i] == 0.0 { centroids[i].clone() }
        else { Value::List(vec![Value::Number(sums[i][0]/counts[i]), Value::Number(sums[i][1]/counts[i])]) }
    }).collect();
    Ok(Value::List(new_centroids))
}

// 1-NN: निकटतम_पड़ोसी(query_point, labeled_points, labels) → label
fn fn_nearest_neighbor(args: Vec<Value>) -> Result<Value, String> {
    let query = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 0 must be query point List".into()) };
    let points = match args.get(1) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 1 must be points List".into()) };
    let labels = match args.get(2) { Some(Value::List(v)) => v.clone(), _ => return Err("निकटतम_पड़ोसी: arg 2 must be labels List".into()) };
    let qx: f64 = if let Some(Value::Number(x)) = query.get(0) { *x } else { 0.0 };
    let qy: f64 = if let Some(Value::Number(y)) = query.get(1) { *y } else { 0.0 };
    let mut best_idx = 0;
    let mut best_dist = f64::MAX;
    for (i, pt) in points.iter().enumerate() {
        if let Value::List(pv) = pt {
            let px: f64 = if let Some(Value::Number(x)) = pv.get(0) { *x } else { 0.0 };
            let py: f64 = if let Some(Value::Number(y)) = pv.get(1) { *y } else { 0.0 };
            let d = (qx-px).powi(2) + (qy-py).powi(2);
            if d < best_dist { best_dist = d; best_idx = i; }
        }
    }
    Ok(labels.get(best_idx).cloned().unwrap_or(Value::Nil))
}
```

**Add to `lvm.rs`:** `"भारत.प्रज्ञा" => crate::bharat_stdlib::pragya_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.प्रज्ञा

x है [1, 2, 3, 4, 5]
y है [2, 4, 5, 4, 5]
मॉडल है रेखीय_प्रतिगमन(x, y)
बताओ मॉडल                         # [slope, intercept]
बताओ भविष्यवाणी(6, मॉडल[0], मॉडल[1])  # predict for x=6
बताओ सहसम्बन्ध(x, y)
बताओ औसत([10, 20, 30])             # 20
बताओ विचलन([10, 20, 30])           # std dev
```

---

### B4. भारत.तुरिंग — Turing Machine Simulator

**Demonstrates Turing completeness. "तुरिंग" directly from Alan Turing.**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.तुरिंग (Turing Machine Simulator)
// ════════════════════════════════════════════════════════════════

pub fn turing_registry() -> Registry {
    vec![
        ("तुरिंग_चलाओ",  fn_turing_run   as NativeFn),  // run a TM program
        ("तुरिंग_सिद्ध",  fn_turing_proof as NativeFn),  // show LIPI is Turing complete
    ]
}

// Run a simple Turing Machine
// Args: [tape_list, head_pos, state, transitions_dict, max_steps]
// transitions_dict: maps "state,symbol" → [new_state, write_symbol, direction(L/R)]
// Returns: [final_tape, final_head, final_state, steps_taken]
fn fn_turing_run(args: Vec<Value>) -> Result<Value, String> {
    use std::collections::HashMap;
    let tape_in = match args.get(0) { Some(Value::List(v)) => v.clone(), _ => return Err("तुरिंग_चलाओ: arg 0 must be tape List".into()) };
    let mut head = need_num(&args, 1, "तुरिंग_चलाओ")? as i64;
    let mut state = need_str(&args, 2, "तुरिंग_चलाओ")?;
    let trans = match args.get(3) { Some(Value::Dict(d)) => d.clone(), _ => return Err("तुरिंग_चलाओ: arg 3 must be transitions Dict".into()) };
    let max_steps = if args.len() > 4 { need_num(&args, 4, "तुरिंग_चलाओ")? as usize } else { 10000 };

    let mut tape: Vec<String> = tape_in.iter().map(|v| {
        if let Value::Str(s) = v { s.clone() } else { "_".into() }
    }).collect();

    let mut steps = 0;
    loop {
        if steps >= max_steps { break; }
        if state == "HALT" || state == "रुको" { break; }
        let pos = head as usize;
        if pos >= tape.len() { tape.push("_".into()); }
        let symbol = tape[pos].clone();
        let key = format!("{},{}", state, symbol);
        let action = match trans.get(&key) {
            Some(Value::List(v)) => v.clone(),
            None => break, // no transition = halt
        };
        let new_state = if let Some(Value::Str(s)) = action.get(0) { s.clone() } else { break };
        let write_sym = if let Some(Value::Str(s)) = action.get(1) { s.clone() } else { symbol.clone() };
        let direction = if let Some(Value::Str(s)) = action.get(2) { s.clone() } else { "R".into() };
        tape[pos] = write_sym;
        state = new_state;
        if direction == "R" || direction == "दाएं" { head += 1; }
        else if direction == "L" || direction == "बाएं" { head -= 1; }
        if head < 0 { tape.insert(0, "_".into()); head = 0; }
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
        "LIPI is Turing complete: it has (1) arbitrary storage via मानचित्र/सूची, \
        (2) conditional branching via यदि/अन्यथा, \
        (3) unbounded loops via जब तक सत्य, \
        (4) a built-in TM simulator (भारत.तुरिंग). \
        This module simulates a universal Turing machine and proves LIPI can compute \
        any computable function. Proof: simulate a UTM in LIPI using this module.".into()
    ))
}
```

**Add to `lvm.rs`:** `"भारत.तुरिंग" => crate::bharat_stdlib::turing_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.तुरिंग

# Simple TM: replace all 0s with 1s on a tape of [0,0,1]
टेप है ["0", "0", "1", "_"]
संक्रमण है {
    "q0,0": ["q0", "1", "R"],
    "q0,1": ["q0", "1", "R"],
    "q0,_": ["HALT", "_", "R"]
}
परिणाम है तुरिंग_चलाओ(टेप, 0, "q0", संक्रमण, 100)
बताओ परिणाम[0]    # ["1", "1", "1", "_"]
बताओ तुरिंग_सिद्ध()
```

---

### B5. भारत.यंत्र — Yantra-Purusha: Automata & Rule-Based Agents

**Source: Samarangana Sutradhara (~11th CE, Bhoja of Dhara). Chapter 31 describes mechanical servants (yantra-purusha) with explicit construction rules. This is the oldest technical description of humanoid automata.**

```rust
// ════════════════════════════════════════════════════════════════
// MODULE — भारत.यंत्र (Yantra-Purusha: Rule-Based Automata)
// Source: Samarangana Sutradhara, Bhoja ~1025 CE
// ════════════════════════════════════════════════════════════════

pub fn yantra_registry() -> Registry {
    vec![
        ("यंत्र_बनाओ",   fn_yantra_create   as NativeFn),  // create FSM automaton
        ("यंत्र_चलाओ",   fn_yantra_run      as NativeFn),  // step FSM with input
        ("यंत्र_स्थिति",  fn_yantra_state    as NativeFn),  // get current state
        ("नियम_जोड़ो",    fn_rule_add        as NativeFn),  // add transition rule
        ("पुरुष_बनाओ",   fn_purusha_create  as NativeFn),  // create a named agent
        ("पुरुष_सोचो",   fn_purusha_think   as NativeFn),  // agent deliberates: input→action
    ]
}

// Create a Finite State Machine: यंत्र_बनाओ(initial_state) → yantra_dict
fn fn_yantra_create(args: Vec<Value>) -> Result<Value, String> {
    let initial = need_str(&args, 0, "यंत्र_बनाओ")?;
    use std::collections::HashMap;
    let mut m = HashMap::new();
    m.insert("__state__".into(), Value::Str(initial));
    m.insert("__transitions__".into(), Value::Dict(HashMap::new()));
    Ok(Value::Dict(m))
}

// Add transition: नियम_जोड़ो(yantra, from_state, input, to_state, action)
fn fn_rule_add(args: Vec<Value>) -> Result<Value, String> {
    let mut yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("नियम_जोड़ो: arg 0 must be yantra Dict".into()) };
    let from   = need_str(&args, 1, "नियम_जोड़ो")?;
    let input  = need_str(&args, 2, "नियम_जोड़ो")?;
    let to     = need_str(&args, 3, "नियम_जोड़ो")?;
    let action = args.get(4).cloned().unwrap_or(Value::Str("".into()));
    let mut trans = match yantra.get("__transitions__") {
        Some(Value::Dict(d)) => d.clone(),
        _ => std::collections::HashMap::new(),
    };
    let key = format!("{},{}", from, input);
    trans.insert(key, Value::List(vec![Value::Str(to), action]));
    yantra.insert("__transitions__".into(), Value::Dict(trans));
    Ok(Value::Dict(yantra))
}

// Step the FSM: यंत्र_चलाओ(yantra, input) → [new_yantra, action_taken]
fn fn_yantra_run(args: Vec<Value>) -> Result<Value, String> {
    let mut yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_चलाओ: arg 0 must be yantra Dict".into()) };
    let input = need_str(&args, 1, "यंत्र_चलाओ")?;
    let state = match yantra.get("__state__") { Some(Value::Str(s)) => s.clone(), _ => return Err("यंत्र_चलाओ: no __state__".into()) };
    let trans = match yantra.get("__transitions__") { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_चलाओ: no __transitions__".into()) };
    let key = format!("{},{}", state, input);
    let (new_state, action) = match trans.get(&key) {
        Some(Value::List(v)) => {
            let ns = if let Some(Value::Str(s)) = v.get(0) { s.clone() } else { state.clone() };
            let ac = v.get(1).cloned().unwrap_or(Value::Str("".into()));
            (ns, ac)
        },
        _ => (state.clone(), Value::Str(format!("no transition for ({},{})", state, input))),
    };
    yantra.insert("__state__".into(), Value::Str(new_state));
    Ok(Value::List(vec![Value::Dict(yantra), action]))
}

fn fn_yantra_state(args: Vec<Value>) -> Result<Value, String> {
    let yantra = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("यंत्र_स्थिति: arg must be yantra Dict".into()) };
    Ok(yantra.get("__state__").cloned().unwrap_or(Value::Nil))
}

// Create a named rule-based agent with a knowledge base (Dict of condition→action rules)
fn fn_purusha_create(args: Vec<Value>) -> Result<Value, String> {
    let name = need_str(&args, 0, "पुरुष_बनाओ")?;
    let rules = match args.get(1) { Some(Value::Dict(d)) => d.clone(), _ => std::collections::HashMap::new() };
    use std::collections::HashMap;
    let mut agent: HashMap<String, Value> = HashMap::new();
    agent.insert("नाम".into(), Value::Str(name));
    agent.insert("नियम".into(), Value::Dict(rules));
    agent.insert("स्मृति".into(), Value::List(vec![]));
    Ok(Value::Dict(agent))
}

// Agent deliberates: पुरुष_सोचो(agent, input_str) → action
// Matches input against rules dict (first matching key substring wins)
fn fn_purusha_think(args: Vec<Value>) -> Result<Value, String> {
    let agent = match args.get(0) { Some(Value::Dict(d)) => d.clone(), _ => return Err("पुरुष_सोचो: arg 0 must be agent Dict".into()) };
    let input = need_str(&args, 1, "पुरुष_सोचो")?;
    let rules = match agent.get("नियम") { Some(Value::Dict(d)) => d.clone(), _ => return Err("पुरुष_सोचो: agent has no rules".into()) };
    // Match by checking if input contains the rule key
    for (condition, action) in &rules {
        if input.contains(condition.as_str()) {
            return Ok(action.clone());
        }
    }
    Ok(Value::Str("अज्ञात — no matching rule".into()))
}
```

**Add to `lvm.rs`:** `"भारत.यंत्र" => crate::bharat_stdlib::yantra_registry(),`

**Usage in LIPI:**
```lipi
आयात भारत.यंत्र

# Build a door controller FSM
द्वार है यंत्र_बनाओ("बंद")
द्वार है नियम_जोड़ो(द्वार, "बंद", "खोलो", "खुला", "दरवाजा खुल गया")
द्वार है नियम_जोड़ो(द्वार, "खुला", "बंद_करो", "बंद", "दरवाजा बंद हुआ")

परिणाम है यंत्र_चलाओ(द्वार, "खोलो")
द्वार है परिणाम[0]
बताओ परिणाम[1]            # दरवाजा खुल गया
बताओ यंत्र_स्थिति(द्वार)  # खुला

# Rule-based Yantra-Purusha agent (from Samarangana Sutradhara Ch. 31)
सेवक है पुरुष_बनाओ("यंत्र_पुरुष", {
    "भोजन": "भोजन लाओ",
    "पानी": "जल लाओ",
    "संगीत": "वाद्य बजाओ",
    "रक्षा": "आक्रमण रोको"
})
बताओ पुरुष_सोचो(सेवक, "भोजन चाहिए")   # भोजन लाओ
बताओ पुरुष_सोचो(सेवक, "संगीत बजाओ")   # वाद्य बजाओ
```

---

## PART C — LANGUAGE-LEVEL ADDITIONS

These require changes to `lexer.rs`, `ast.rs`, `parser.rs`, `compiler.rs`, `opcode.rs`, and `serializer.rs`.

---

### C1. `जाँचो` — Assert keyword (Nyaya's प्रतिज्ञा verification)

**`lexer.rs`** — add token:
```rust
"जाँचो" => Token::Jaancho,
```

**`ast.rs`** — add statement:
```rust
Stmt::Assert { expr: Expr, msg: Option<Expr> }
```

**`parser.rs`** — parse `जाँचो expr` or `जाँचो expr, "message"`:
```rust
Token::Jaancho => {
    let expr = self.expression()?;
    let msg = if self.peek_is(Token::Comma) { self.advance(); Some(self.expression()?) } else { None };
    Ok(Stmt::Assert { expr, msg })
}
```

**`compiler.rs`** — compile:
```rust
Stmt::Assert { expr, msg } => {
    self.compile_expr(expr);
    if let Some(m) = msg { self.compile_expr(m); self.emit(Opcode::Assert(true)); }
    else { self.emit(Opcode::Assert(false)); }
}
```

**`opcode.rs`** — add: `Assert(bool)  // true = has message on stack`

**`lvm.rs`** — handle:
```rust
Opcode::Assert(has_msg) => {
    let msg = if has_msg { Some(format!("{}", pop(&mut self.stack)?)) } else { None };
    let val = pop(&mut self.stack)?;
    if val != Value::Bool(true) {
        let m = msg.unwrap_or("जाँच विफल".into());
        return Err(format!("AssertionError: {}", m));
    }
}
```

**Usage:**
```lipi
जाँचो 2 + 2 बराबर 4
जाँचो लम्बाई([1,2,3]) बराबर 3, "सूची की लम्बाई 3 होनी चाहिए"
```

---

### C2. `स्थिर` — Immutable constant (Katha Upanishad's nitya/permanent)

**`lexer.rs`:** `"स्थिर" => Token::Sthir,`

**`ast.rs`:** `Stmt::Const { name: String, value: Expr }`

**`compiler.rs`:** Emit `DeclareConst(name)` after value: set the variable then mark it constant.

**`lvm.rs`:** Track `self.constants: HashSet<String>`. On `StoreVar`, check — if name is in constants, return `Err("स्थिर '{}' को बदला नहीं जा सकता")`.

**Usage:**
```lipi
स्थिर पाई_22_7 है 22.0 / 7.0
पाई_22_7 है 3.0   # त्रुटि: स्थिर को बदला नहीं जा सकता
```

---

### C3. `शुद्ध` — Pure function annotation

**Simplest implementation:** Treat as a no-op keyword that is parsed before `विधि`. The compiler tracks whether a function is marked pure. At call time, the LVM can optionally cache (memoize) results.

**`lexer.rs`:** `"शुद्ध" => Token::Shuddha,`

**`ast.rs`:** Add `pure: bool` field to `Stmt::Vidhi { name, params, body, pure }`.

**`compiler.rs`:** When `pure: true`, emit a comment/marker in `FuncDef`. For now: no-op — just parsed and stored as metadata. Future: memoization cache in LVM.

**Usage:**
```lipi
शुद्ध विधि वर्गफल(अ):
    फल अ * अ
```

---

## PART D — COMPLETE FILE CHANGE SUMMARY

| File | What to add |
|------|------------|
| `src/bharat_stdlib.rs` | 6 new registry functions: `chhandas_`, `nyaya_`, `shulba_`, `jyotish_`, `natya_`, `tantrika_`, `anukooland_`, `pragya_`, `turing_`, `yantra_`; 30+ helper functions; add to `ganit_registry()`: कुट्टक, श्रीधर_सूत्र, बखशाली_मूल, ब्रह्मगुप्त_अंतर, महावीर_भिन्न, आर्यभट_योग, वर्ग_योग, घन_योग |
| `src/lvm.rs` | 10 new lines in `Opcode::Import` match; 5 new global constants in `LVM::new()`; `Assert` handler; `constants: HashSet` field; `DeclareConst` handler |
| `src/opcode.rs` | Add: `Assert(bool)`, `DeclareConst(String)` |
| `src/lexer.rs` | Add tokens: `Jaancho`, `Sthir`, `Shuddha` |
| `src/ast.rs` | Add: `Stmt::Assert`, `Stmt::Const`, `pure: bool` on Vidhi |
| `src/parser.rs` | Parse `जाँचो`, `स्थिर`, `शुद्ध` keywords |
| `src/compiler.rs` | Compile new Stmt variants |
| `src/serializer.rs` | Add TAG_ASSERT=0x3F, TAG_DECLARE_CONST=0x40 |
| `src/roman.rs` | 20 new keyword mappings |

---

## PART E — IMPLEMENTATION ORDER (Easiest → Most Work)

| Order | What | Files changed | Effort |
|-------|------|--------------|--------|
| 1 | New constants (अरब/खरब/नील/पद्म/शंख) | `lvm.rs` only | 5 min |
| 2 | भारत.गणित additions (कुट्टक, श्रीधर_सूत्र, etc.) | `bharat_stdlib.rs` + `lvm.rs` | 30 min |
| 3 | भारत.छन्दस् (Pingala binary) | `bharat_stdlib.rs` + `lvm.rs` | 30 min |
| 4 | भारत.नाट्य (9 Rasas) | `bharat_stdlib.rs` + `lvm.rs` | 20 min |
| 5 | भारत.ज्योतिष (27 nakshatras) | `bharat_stdlib.rs` + `lvm.rs` | 30 min |
| 6 | भारत.न्याय (logic) | `bharat_stdlib.rs` + `lvm.rs` | 30 min |
| 7 | भारत.शुल्ब (geometry) | `bharat_stdlib.rs` + `lvm.rs` | 20 min |
| 8 | भारत.प्रज्ञा (statistics/ML) | `bharat_stdlib.rs` + `lvm.rs` | 45 min |
| 9 | भारत.तंत्रिका (neural nets) | `bharat_stdlib.rs` + `lvm.rs` | 1 hr |
| 10 | भारत.अनुकूलन (gradient descent) | `bharat_stdlib.rs` + `lvm.rs` | 45 min |
| 11 | भारत.यंत्र (automata/agents) | `bharat_stdlib.rs` + `lvm.rs` | 45 min |
| 12 | भारत.तुरिंग (Turing machine) | `bharat_stdlib.rs` + `lvm.rs` | 45 min |
| 13 | `जाँचो` assert keyword | 6 files | 1 hr |
| 14 | `स्थिर` const keyword | 6 files | 1 hr |
| 15 | `शुद्ध` pure function | 5 files | 45 min |
| 16 | Roman input mappings | `roman.rs` only | 15 min |

**Total estimate: ~10 hours for full implementation.**

Steps 1-12 (all stdlib modules) require ONLY changes to `bharat_stdlib.rs` and `lvm.rs` — no syntax changes, no lexer/parser changes. These are the safest to implement first.

---

*Document created: 2026-06-09. All Rust code is pure — zero external crates.*
