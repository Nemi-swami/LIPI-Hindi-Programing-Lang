// Phonetic (character-level) Roman → Devanagari transliterator.
//
// `.vani` files let users write LIPI source using phonetic Roman spellings —
// no Devanagari keyboard or IME needed.
//
// CLI:
//   lipi phonetic foo.vani         run
//   lipi phonetic-show foo.vani    print translated source
//   lipi foo.vani                  auto-detected by extension
//
// Two-pass translation:
//   1. roman.rs keyword map  (hai→है, batao→बताओ, …)
//   2. phonetic conversion of remaining ASCII identifier words
//
// Vowel scheme (standalone → matra after consonant):
//   a→अ/∅   aa→आ/ा   i→इ/ि   ee,ii→ई/ी   u→उ/ु   oo,uu→ऊ/ू
//   e→ए/े   ai→ऐ/ै   o→ओ/ो   au,ou,ow→औ/ौ   ri→ऋ/ृ
//
// Consonant scheme (double-letter = retroflex):
//   k kh g gh ng ch chh j jh ny   (dental) t th d dh n   p ph/f b bh m
//   (retroflex) tt tth dd ddh nn → ट ठ ड ढ ण
//   y r l v/w sh ssh s h  |  ksh/x→क्ष  gy/gny→ज्ञ

use crate::roman;

// ── Token kind ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Ph {
    /// Consonant — Devanagari string (may contain embedded halant for conjuncts)
    Con(&'static str),
    /// Vowel — (standalone_form, matra_form)
    /// matra_form = "" means inherent 'a' — nothing extra added after a consonant
    Vow(&'static str, &'static str),
}

// ── Pattern table (ALL multi-char before single-char) ────────────────────────

static PH: &[(&str, Ph)] = &[
    // ── 4-char ────────────────────────────────────────────────────────────────
    ("ksha", Ph::Con("क्ष")),
    // ── 3-char consonant clusters ─────────────────────────────────────────────
    ("ksh",  Ph::Con("क्ष")),
    ("gny",  Ph::Con("ज्ञ")),
    ("ssh",  Ph::Con("ष")),    // retroflex sibilant
    ("chh",  Ph::Con("छ")),
    ("tth",  Ph::Con("ठ")),    // retroflex aspirated T
    ("ddh",  Ph::Con("ढ")),    // retroflex aspirated D
    // ── 2-char vowels ─────────────────────────────────────────────────────────
    ("aa",   Ph::Vow("आ", "ा")),
    ("ai",   Ph::Vow("ऐ", "ै")),
    ("au",   Ph::Vow("औ", "ौ")),
    ("ou",   Ph::Vow("औ", "ौ")),
    ("ow",   Ph::Vow("औ", "ौ")),
    ("ee",   Ph::Vow("ई", "ी")),
    ("ii",   Ph::Vow("ई", "ी")),
    ("oo",   Ph::Vow("ऊ", "ू")),
    ("uu",   Ph::Vow("ऊ", "ू")),
    ("ri",   Ph::Vow("ऋ", "ृ")),
    // ── 2-char consonants ─────────────────────────────────────────────────────
    ("kh",   Ph::Con("ख")),
    ("gh",   Ph::Con("घ")),
    ("ng",   Ph::Con("ङ")),
    ("ch",   Ph::Con("च")),
    ("jh",   Ph::Con("झ")),
    ("ny",   Ph::Con("ञ")),
    ("gy",   Ph::Con("ज्ञ")),
    ("tt",   Ph::Con("ट")),    // retroflex T (double)
    ("dd",   Ph::Con("ड")),    // retroflex D (double)
    ("nn",   Ph::Con("ण")),    // retroflex N (double)
    ("th",   Ph::Con("थ")),    // dental aspirated
    ("dh",   Ph::Con("ध")),    // dental aspirated
    ("sh",   Ph::Con("श")),
    ("ph",   Ph::Con("फ")),
    ("bh",   Ph::Con("भ")),
    // ── Single vowels ─────────────────────────────────────────────────────────
    ("a",    Ph::Vow("अ", "")),  // "" = inherent, no matra
    ("i",    Ph::Vow("इ", "ि")),
    ("u",    Ph::Vow("उ", "ु")),
    ("e",    Ph::Vow("ए", "े")),
    ("o",    Ph::Vow("ओ", "ो")),
    // ── Single consonants ─────────────────────────────────────────────────────
    ("k",    Ph::Con("क")),
    ("g",    Ph::Con("ग")),
    ("c",    Ph::Con("च")),
    ("j",    Ph::Con("ज")),
    ("t",    Ph::Con("त")),
    ("d",    Ph::Con("द")),
    ("n",    Ph::Con("न")),
    ("p",    Ph::Con("प")),
    ("b",    Ph::Con("ब")),
    ("m",    Ph::Con("म")),
    ("y",    Ph::Con("य")),
    ("r",    Ph::Con("र")),
    ("l",    Ph::Con("ल")),
    ("v",    Ph::Con("व")),
    ("w",    Ph::Con("व")),
    ("s",    Ph::Con("स")),
    ("h",    Ph::Con("ह")),
    ("f",    Ph::Con("फ")),
    ("x",    Ph::Con("क्ष")),  // x = ksha
    ("q",    Ph::Con("क")),   // Urdu-origin words
    ("z",    Ph::Con("ज")),   // loanwords
];

// ── Core conversion: one phonetic segment (no underscores/digits) ─────────────

fn convert_segment(word: &str) -> String {
    if word.is_empty() { return String::new(); }

    let lower = word.to_ascii_lowercase();
    let mut out = String::new();
    let mut pos = 0;
    let mut after_con = false;

    while pos < lower.len() {
        let slice = &lower[pos..];

        if let Some(&(pat, ph)) = PH.iter().find(|&&(p, _)| slice.starts_with(p)) {
            match ph {
                Ph::Con(dev) => {
                    if after_con {
                        out.push('्'); // halant U+094D
                    }
                    out.push_str(dev);
                    after_con = true;
                }
                Ph::Vow(standalone, matra) => {
                    if after_con {
                        if !matra.is_empty() {
                            out.push_str(matra);
                        }
                        // empty matra = inherent 'a', nothing added
                        after_con = false;
                    } else {
                        out.push_str(standalone);
                        after_con = false;
                    }
                }
            }
            pos += pat.len();
        } else {
            let c = slice.chars().next().unwrap();
            out.push(c);
            pos += c.len_utf8();
            after_con = false;
        }
    }

    out
}

// ── Word-level: split on underscores/digits, convert each alpha run ───────────

pub fn convert_word(word: &str) -> String {
    let mut out = String::new();
    let mut buf = String::new();

    for c in word.chars() {
        if c.is_ascii_alphabetic() {
            buf.push(c);
        } else {
            if !buf.is_empty() {
                out.push_str(&convert_segment(&buf));
                buf.clear();
            }
            out.push(c);
        }
    }
    if !buf.is_empty() {
        out.push_str(&convert_segment(&buf));
    }
    out
}

// ── Source-level second pass ──────────────────────────────────────────────────

fn second_pass(source: &str) -> String {
    let mut out = String::with_capacity(source.len() * 2);
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str = false;

    while i < n {
        let c = chars[i];

        // String literal boundary
        if c == '"' {
            if !in_str {
                in_str = true;
            } else if i == 0 || chars[i - 1] != '\\' {
                in_str = false;
            }
            out.push(c);
            i += 1;
            continue;
        }
        if in_str {
            out.push(c);
            i += 1;
            continue;
        }

        // # comment: copy to end of line verbatim
        if c == '#' {
            while i < n && chars[i] != '\n' {
                out.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // ASCII alphabetic word: collect run, phonetically convert
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            out.push_str(&convert_word(&word));
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Convert a `.vani` phonetic source to Devanagari LIPI source.
/// Pass 1: keyword map via roman.rs  (बताओ, है, …)
/// Pass 2: phonetic conversion of remaining ASCII identifier words
pub fn vani_to_devanagari(source: &str) -> String {
    let after_keywords = roman::roman_to_devanagari(source);
    second_pass(&after_keywords)
}
