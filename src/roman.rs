// Roman QWERTY → Devanagari LIPI preprocessor
//
// Lets users write LIPI source files using phonetic Roman spellings instead of
// needing a Devanagari keyboard layout or IME.
//
// Usage (file extension .roman or .r):
//   lipi foo.roman      ← auto-detected
//   lipi roman foo.txt  ← explicit subcommand
//
// Rules:
//   • Only replaces whole words (non-alphanumeric / underscore boundaries)
//   • Longer multi-word phrases take priority over shorter prefixes
//   • Contents of "..." string literals are preserved verbatim
//   • #-comments and ।...। comments are preserved verbatim
//   • Case-insensitive for keyword matching

/// Convert a Roman-script LIPI source to Devanagari LIPI source.
pub fn roman_to_devanagari(source: &str) -> String {
    let mut out = String::with_capacity(source.len() * 2);
    let mut lines = source.split('\n').peekable();
    let last_has_newline = source.ends_with('\n');

    while let Some(line) = lines.next() {
        translate_line(line, &mut out);
        if lines.peek().is_some() || last_has_newline {
            out.push('\n');
        }
    }
    out
}

// ── Keyword table ─────────────────────────────────────────────────────────────
//
// Sorted longest-first so that multi-word phrases are tried before their
// single-word prefixes (e.g. "anyatha yadi" before "anyatha").

static KW: &[(&str, &str)] = &[
    // ── Multi-word keywords ───────────────────────────────────────────────────
    ("anyatha yadi",          "अन्यथा यदि"),
    ("jab tak",               "जब तक"),
    ("ke liye",               "के लिए"),
    ("bar karo",              "बार करो"),
    ("band karo",             "बंद करो"),
    ("se adhik",              "से अधिक"),
    ("se kam",                "से कम"),
    // भारत module paths  (must come before single-segment names)
    ("bharat.ganit",          "भारत.गणित"),
    ("bharat.sankhya",        "भारत.संख्या"),
    ("bharat.pehchaan",       "भारत.पहचान"),
    ("bharat.bhugtaan",       "भारत.भुगतान"),
    ("bharat.bhasha",         "भारत.भाषा"),
    // Long function names first
    ("brahmaguptha_kshetra",  "ब्रह्मगुप्त_क्षेत्र"),
    ("heron_kshetra",         "हेरॉन_क्षेत्र"),
    ("laghuganak_das",        "लघुगणक_दस"),
    ("vyutkram_sparshjya",    "व्युत्क्रम_स्पर्श"),
    ("vyutkram_kojya",        "व्युत्क्रम_कोज्या"),
    ("vyutkram_jya",          "व्युत्क्रम_ज्या"),
    // New Phase 16 multi-word
    ("bharat vyakaran",       "भारत.व्याकरण"),
    ("bharat vigyan",         "भारत.विज्ञान"),
    ("guru laghu",            "गुरु_लघु_संकेत"),
    ("sandhi prakar",         "संधि_प्रकार"),
    ("samas prakar",          "समास_प्रकार"),
    ("shiva sutra",           "शिव_सूत्र"),
    ("chhand sthan",          "छन्द_स्थान"),
    ("matra bhar",            "मात्रा_भार"),
    ("ras sutra",             "रस_सूत्र"),
    ("aryabhat pai",          "आर्यभट_पाई"),
    ("nakshatra sankhya",     "नक्षत्र_संख्या"),
    ("tithi sankhya",         "तिथि_संख्या"),
    ("yug varsh",             "युग_वर्ष"),

    // ── Single-word keywords ──────────────────────────────────────────────────
    ("batao",        "बताओ"),
    ("likho",        "लिखो"),
    ("anyatha",      "अन्यथा"),
    ("yadi",         "यदि"),
    ("toh",          "तो"),          // 'to' too risky; use 'toh' → तो
    ("vidhi",        "विधि"),
    ("phal",         "फल"),
    ("satya",        "सत्य"),
    ("asatya",       "असत्य"),
    ("nahin",        "नहीं"),
    ("vaishvik",     "वैश्विक"),
    ("lamda",        "लाम्डा"),
    ("agla",         "अगला"),
    ("barabar",      "बराबर"),
    ("aayat",        "आयात"),
    ("koshish",      "कोशिश"),
    ("pakdo",        "पकड़ो"),
    ("truti",        "त्रुटि"),
    ("banao",        "बनाओ"),
    ("mein",         "में"),
    ("hai",          "है"),
    ("yah",          "यह"),
    ("aur",          "और"),
    ("swaroop",      "स्वरूप"),
    ("purnanks",     "पूर्णांक"),
    ("vargmool",     "वर्गमूल"),
    ("nirapeksh",    "निरपेक्ष"),
    ("manchitra",    "मानचित्र"),
    ("chhano",       "छानो"),
    ("modo",         "मोड़ो"),
    ("lambai",       "लम्बाई"),
    ("joodo",        "जोड़ो"),
    ("hatao",        "हटाओ"),
    ("ulta",         "उलटा"),
    ("krambaddh",    "क्रमबद्ध"),
    ("milao",        "मिलाओ"),
    ("shuru_mein",   "शुरू_में"),
    ("ant_mein",     "अंत_में"),
    ("khojo",        "खोजो"),
    ("vibhajit",     "विभाजित"),
    ("badlo",        "बदलो"),
    ("yadrichhik",   "यादृच्छिक"),
    ("nirgam",       "निर्गम"),
    ("ghataank",     "घातांक"),
    ("ghaat",        "घात"),
    ("padho",        "पढ़ो"),
    ("vakya",        "वाक्य"),
    ("prakar",       "प्रकार"),
    ("laghuganak",   "लघुगणक"),
    ("gol",          "गोल"),
    ("pai",          "पाई"),
    ("anant",        "अनंत"),
    ("sparshjya",    "स्पर्शज्या"),
    ("kojya",        "कोज्या"),
    ("jya",          "ज्या"),
    ("virahank",     "विरहांक"),
    ("sanyojan",     "संयोजन"),
    ("kramchay",     "क्रमचय"),
    ("suchi",        "सूची"),
    ("kosh",         "कोश"),
    ("varg",         "वर्ग"),
    // New module functions (multi-word first)
    ("meru_pankti",         "मेरु_पंक्ति"),
    ("brahmagupat_antar",   "ब्रह्मगुप्त_अंतर"),
    ("shridhar_sutra",      "श्रीधर_सूत्र"),
    ("bakshali_mul",        "बखशाली_मूल"),
    ("mahavir_bhinn",       "महावीर_भिन्न"),
    ("aryabhat_yog",        "आर्यभट_योग"),
    ("varga_yog",           "वर्ग_योग"),
    ("ghana_yog",           "घन_योग"),
    ("nakshatra_naam",      "नक्षत्र_नाम"),
    ("nakshatra_kram",      "नक्षत्र_क्रम"),
    ("tithi_naam",          "तिथि_नाम"),
    ("yug_naam",            "युग_नाम"),
    ("graha_kram",          "ग्रह_क्रम"),
    ("sabhi_nakshatra",     "सभी_नक्षत्र"),
    ("ras_naam",            "रस_नाम"),
    ("ras_bhav",            "रस_भाव"),
    ("ras_vivaran",         "रस_विवरण"),
    ("sabhi_ras",           "सभी_रस"),
    ("sttar_banao",         "स्तर_बनाओ"),
    ("aage_paas",           "आगे_पास"),
    ("jal_aage",            "जाल_आगे"),
    ("truti_varg_avkal",    "त्रुटि_वर्ग_अवकल"),
    ("truti_varg",          "त्रुटि_वर्ग"),
    ("dhal_avaroh",         "ढाल_अवरोह"),
    ("adam_charan",         "आदम_चरण"),
    ("dhal_sankhyik",       "ढाल_संख्यिक"),
    ("sikhne_ki_dar",       "सीखने_की_दर"),
    ("rekhiy_pratigraman",  "रेखीय_प्रतिगमन"),
    ("bhavisyavani",        "भविष्यवाणी"),
    ("sahsambandh",         "सहसम्बन्ध"),
    ("ka_sadhan",           "क_साधन"),
    ("nikattam_padosi",     "निकटतम_पड़ोसी"),
    ("turing_chalao",       "तुरिंग_चलाओ"),
    ("turing_siddh",        "तुरिंग_सिद्ध"),
    ("yantra_banao",        "यंत्र_बनाओ"),
    ("yantra_chalao",       "यंत्र_चलाओ"),
    ("yantra_sthiti",       "यंत्र_स्थिति"),
    ("niyam_jodo",          "नियम_जोड़ो"),
    ("purush_banao",        "पुरुष_बनाओ"),
    ("purush_socho",        "पुरुष_सोचो"),
    // Single-word new functions
    ("kuttak",              "कुट्टक"),
    ("nashtam",             "नष्टम"),
    ("uddiisht",            "उद्दिष्ट"),
    ("prastaar",            "प्रस्तार"),
    ("dvi_aadhaar",         "द्विआधार"),
    ("anumaan",             "अनुमान"),
    ("vyaapti",             "व्याप्ति"),
    ("hetvabhaas",          "हेत्वाभास"),
    ("pramaan",             "प्रमाण"),
    ("karna",               "कर्ण"),
    ("shulba_mul",          "शुल्ब_मूल"),
    ("vrtta_varg",          "वृत्त_वर्ग"),
    ("varg_vrtta",          "वर्ग_वृत्त"),
    ("madhyika",            "माध्यिका"),
    ("vichaln",             "विचलन"),
    ("avsaar",              "औसत"),
    ("sigmachar",           "सिग्मा"),
    ("relu",                "रेलु"),
    ("arab",                "अरब"),
    ("kharab",              "खरब"),
    ("nil",                 "नील"),
    ("shankh",              "शंख"),
    ("padm",                "पद्म"),
    // Phase 16 — new modules and functions
    ("bharat.vyakaran",     "भारत.व्याकरण"),
    ("bharat.vigyan",       "भारत.विज्ञान"),
    ("brahmagupat_gunan",   "ब्रह्मगुप्त_गुणन"),
    ("brahmagupat_antarveshan", "ब्रह्मगुप्त_अंतर्वेशन"),
    ("aryabhat_varg_yog",   "आर्यभट_वर्ग_योग"),
    ("aryabhat_ghan_yog",   "आर्यभट_घन_योग"),
    ("katapayadi",          "कटपयादि"),
    ("vigyanabhairav",      "विज्ञानभैरव"),
    ("sabhi_dharana",       "सभी_धारणाएँ"),
    ("sandhi_prakar",       "संधि_प्रकार"),
    ("samas_prakar",        "समास_प्रकार"),
    ("sphota",              "स्फोट_परीक्षण"),
    ("shiva_sutra",         "शिव_सूत्र"),
    ("guru_laghu",          "गुरु_लघु_संकेत"),
    ("chhand_sthan",        "छन्द_स्थान"),
    ("matra_bhar",          "मात्रा_भार"),
    ("ras_sutra",           "रस_सूत्र"),
    // Phase 16 — keywords
    ("jancho",              "जाँचो"),
    ("sthir",               "स्थिर"),
    ("shuddha",             "शुद्ध"),
    // Phase 16 — constants
    ("aryabhat_pai",        "आर्यभट_पाई"),
    ("aryabhat_kon",        "आर्यभट_कोण"),
    ("nakshatra_sankhya",   "नक्षत्र_संख्या"),
    ("tithi_sankhya",       "तिथि_संख्या"),
    ("yug_varsh",           "युग_वर्ष"),
    ("brahmagupat_shunya",  "ब्रह्मगुप्त_शून्य"),
    ("ya",           "या"),          // short — comes after all ya-prefixed names
];

// ── Core translation ──────────────────────────────────────────────────────────

fn translate_line(line: &str, out: &mut String) {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str = false;

    while i < n {
        let c = chars[i];

        // ── String literal boundary ──────────────────────────────────────────
        if c == '"' {
            if !in_str {
                in_str = true;
            } else if i > 0 && chars[i - 1] == '\\' {
                // escaped quote inside string — just copy
            } else {
                in_str = false;
            }
            out.push(c);
            i += 1;
            continue;
        }

        // Inside string: copy verbatim
        if in_str {
            out.push(c);
            i += 1;
            continue;
        }

        // ── Comments: copy rest of line verbatim ─────────────────────────────
        if c == '#' || c == '।' {
            for &ch in &chars[i..] { out.push(ch); }
            return;
        }

        // ── Keyword match at word boundary ───────────────────────────────────
        let at_boundary = i == 0 || !is_word(chars[i - 1]);
        if at_boundary && c.is_ascii_alphabetic() {
            if let Some((roman, dev)) = try_match(&chars, i) {
                out.push_str(dev);
                i += roman.chars().count();
                continue;
            }
        }

        out.push(c);
        i += 1;
    }
}

/// Try to match a keyword at position `i` in `chars`.
/// Returns `(roman_str, devanagari_str)` if found, otherwise None.
fn try_match<'a>(chars: &[char], i: usize) -> Option<(&'a str, &'a str)> {
    // Build a lowercase view from position i  (borrow as a temporary &str)
    let tail: String = chars[i..].iter().collect();
    let tail_lower = tail.to_ascii_lowercase();

    for &(roman, dev) in KW {
        if tail_lower.starts_with(roman) {
            let end = i + roman.chars().count();
            // Word boundary after the match
            if chars.get(end).map_or(true, |&c| !is_word(c)) {
                return Some((roman, dev));
            }
        }
    }
    None
}

#[inline]
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// ── Show what was translated (optional debug mode) ───────────────────────────

/// Translate and print a side-by-side diff (roman → devanagari) for each changed line.
#[allow(dead_code)]
pub fn show_translation(source: &str) {
    for (n, line) in source.lines().enumerate() {
        let mut translated = String::new();
        translate_line(line, &mut translated);
        if translated != line {
            println!("{:3} │ {}", n + 1, line);
            println!("    │ {}", translated);
        }
    }
}
