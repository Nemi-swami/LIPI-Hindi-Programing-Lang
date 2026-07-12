/// LIPI 2.0 Lexer — Devanagari-aware, Python-style INDENT/DEDENT

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Str(String),
    Bool(bool),
    Ident(String),

    // Keywords
    Hai,       // है
    Vidhi,     // विधि
    Fal,       // फल
    Utpann,    // उत्पन्न (yield)
    Pratiksha, // प्रतीक्षा (await)
    Yadi,      // यदि
    Anyatha,   // अन्यथा
    SeAdhik,   // से अधिक
    SeKam,     // से कम
    Barabar,   // बराबर
    BarKaro,   // N बार करो
    KeeLiye,   // के लिए
    Mein,      // में
    Batao,     // बताओ
    // All 6 Karakas (Sanskrit case roles)
    Karta,     // कर्ता     — doer/agent
    Karma,     // कर्म      — object/patient
    Karan,     // करण       — instrument
    Sampradan, // सम्प्रदान — recipient
    Apadan,    // अपादान    — source
    Adhikaran, // अधिकरण    — location/context
    Aayat,     // आयात      — import statement
    Varg,      // वर्ग       — class definition (Phase 6)

    // Phase 7 — interactive architecture
    JabTak,    // जब तक    — while loop
    BandKaro,  // बंद करो  — break
    Agla,      // अगला     — continue
    Likho,     // लिखो     — print inline (no newline)

    // Phase 9 — error handling
    Koshish,   // कोशिश    — try
    Pakdo,     // पकड़ो    — catch
    Phenko,    // फेंको    — throw (Phase 17A typed exceptions)

    // Phase 10 — first-class functions
    Lambda,    // लाम्डा   — lambda expression

    // Phase 12 — bitwise, ternary, varargs
    BitAnd,    // &
    BitOr,     // |
    BitXor,    // ^
    BitNot,    // ~
    LShift,    // <<
    RShift,    // >>
    Toh,       // तो  (ternary "then")
    Star2,     // * used as vararg marker in param lists (reuses Star in parser)

    // Phase 13 — boolean keywords, global
    Aur,       // और      — logical AND
    Ya,        // या      — logical OR
    Nahin,     // नहीं    — logical NOT
    MeinHai,   // में_है   — membership test (Phase 17)
    NahinHai,  // नहीं_है — negated membership (Phase 17)
    Vaishvik,  // वैश्विक — global variable declaration

    // Phase 15 — enums and pattern matching
    Vikalp,    // विकल्प — enum definition
    Milao,     // मिलाओ  — match statement

    // Phase 16 — Nyaya assert + Samkhya const + Gita pure
    Jancho,    // जाँचो  — assert (Nyaya Pratijna verification)
    Sthir,     // स्थिर  — immutable constant (Samkhya Purusha = unchanging)
    Shuddha,   // शुद्ध  — pure function (Gita karma yoga without side effects)
    Sajha,     // साझा  — static (shared) class method, no यह (Phase 17)
    Sar,       // सार   — abstract class marker, सार वर्ग (Phase 17)
    Saath,     // साथ   — context manager (with), Phase 17
    KeRupMein, // के_रूप_में — "as" binder in a साथ block (Phase 17)
    Abhilekh,  // अभिलेख — record / dataclass shorthand (Phase 17)

    // Phase 17 — test framework
    Parikshan, // परीक्षण — test block (run via `lipi test file.swami`)

    Plus, Minus, Star, Slash, Percent,
    SlashSlash,  // // — floor (integer) division (Phase 17)
    EqEq, NotEq, Lt, Gt, LtEq, GtEq,
    Assign,    // = — default parameter values (Phase 17)
    ColonEq,   // := — walrus / inline assignment (Phase 17)
    Arrow,     // -> — return type annotation (Phase 18 #7)
    LBracket, RBracket,  // [ ]
    LBrace, RBrace,      // { }
    Colon, Dot, Comma, LParen, RParen,

    At,        // @ — decorator prefix (Phase 17)

    Newline, Indent, Dedent, Eof,
    Unknown(char),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

/// Canonical decomposition for a Devanagari precomposed character, if any.
/// These code points (nukta consonants U+0958–095F and U+0929/0931/0934) carry a
/// canonical decomposition to `base + U+093C (nukta)` and are on the Unicode
/// composition-exclusion list, so NFC keeps them *decomposed* — i.e. this mapping
/// IS the NFC normal form. Returns None for all other characters.
pub fn devanagari_decompose(c: char) -> Option<(char, char)> {
    let base = match c {
        '\u{0929}' => '\u{0928}', // ऩ → न
        '\u{0931}' => '\u{0930}', // ऱ → र
        '\u{0934}' => '\u{0933}', // ऴ → ळ
        '\u{0958}' => '\u{0915}', // क़ → क
        '\u{0959}' => '\u{0916}', // ख़ → ख
        '\u{095A}' => '\u{0917}', // ग़ → ग
        '\u{095B}' => '\u{091C}', // ज़ → ज
        '\u{095C}' => '\u{0921}', // ड़ → ड
        '\u{095D}' => '\u{0922}', // ढ़ → ढ
        '\u{095E}' => '\u{092B}', // फ़ → फ
        '\u{095F}' => '\u{092F}', // य़ → य
        _ => return None,
    };
    Some((base, '\u{093C}'))
}

/// Normalize a string to NFC for the Devanagari block: decompose the precomposed
/// nukta letters to `base + ़` (their canonical/NFC form). This makes equivalent
/// spellings (e.g. ड़ as U+095C vs ड+़) compare and lex identically. Non-Devanagari
/// text is returned unchanged — this is a bounded, block-specific normalizer.
pub fn normalize_devanagari(src: &str) -> String {
    // Fast path: nothing to do if no precomposed nukta chars are present
    if !src.chars().any(|c| devanagari_decompose(c).is_some()) {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len() + 8);
    for c in src.chars() {
        match devanagari_decompose(c) {
            Some((base, nukta)) => { out.push(base); out.push(nukta); }
            None => out.push(c),
        }
    }
    out
}

/// Replace `"""..."""` triple-quoted strings with escaped single-line strings.
/// This runs as a pre-pass before line-by-line tokenization.
/// Block comments `।।…।।` may span multiple lines. This pre-pass strips them
/// while preserving `\n` characters so downstream line numbers stay accurate.
/// Runs AFTER `preprocess_triple_quotes` so that `"..."` never contains a raw
/// newline — string-literal tracking here only needs to handle single-line
/// double-quoted spans with backslash escapes.
fn preprocess_block_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut in_str = false;
    let mut escape = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if escape { escape = false; }
            else if c == '\\' { escape = true; }
            else if c == '"' { in_str = false; }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        // Block-comment opener `।।` outside a string
        if c == '।' && i + 1 < chars.len() && chars[i+1] == '।' {
            i += 2;
            while i < chars.len() {
                if chars[i] == '।' && i + 1 < chars.len() && chars[i+1] == '।' {
                    i += 2;
                    break;
                }
                if chars[i] == '\n' { out.push('\n'); } // preserve line count
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn preprocess_triple_quotes(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            // Check for triple-quote start
            if chars.peek() == Some(&'"') {
                let mut peek_iter = chars.clone();
                peek_iter.next(); // consume second "
                if peek_iter.peek() == Some(&'"') {
                    // It's a triple-quote — consume the two extra quotes
                    chars.next();
                    chars.next();
                    // Collect all content until the closing """
                    let mut content = String::new();
                    loop {
                        match chars.next() {
                            None => break,
                            Some('"') if chars.clone().next() == Some('"') => {
                                // Check for triple close
                                let mut tmp = chars.clone();
                                tmp.next();
                                if tmp.next() == Some('"') {
                                    chars.next(); // consume second "
                                    chars.next(); // consume third "
                                    break;
                                } else {
                                    content.push_str("\\\"");
                                }
                            }
                            Some('\n') => content.push_str("\\n"),
                            Some('\r') => {}
                            Some('"') => content.push_str("\\\""),
                            Some('\\') => content.push_str("\\\\"),
                            Some(other) => content.push(other),
                        }
                    }
                    out.push('"');
                    out.push_str(&content);
                    out.push('"');
                    continue;
                }
            }
            out.push(c);
        } else {
            out.push(c);
        }
    }
    out
}

/// Entry point: tokenize a full source file with INDENT/DEDENT handling.
pub fn tokenize(src: &str) -> Vec<Token> {
    // Editors on Windows often save UTF-8 with a BOM — ignore it.
    // Normalize Devanagari to NFC so precomposed nukta letters and their
    // base+nukta equivalents lex identically (Phase 17).
    let normalized = normalize_devanagari(src.trim_start_matches('\u{feff}'));
    let preprocessed = preprocess_triple_quotes(&normalized);
    let preprocessed = preprocess_block_comments(&preprocessed);
    let src = preprocessed.as_str();
    let mut out: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    // Running depth of open ( [ { across lines — when > 0 we are inside a
    // multiline collection / call, so indentation and Newline are suppressed
    // until the brackets balance (Phase 17 multiline collections).
    let mut bracket_depth: i32 = 0;

    for (line_idx, line) in src.lines().enumerate() {
        let line_num = line_idx + 1;
        let content = line[leading_char_count(line)..].trim_end();
        let trimmed = strip_comment(content).trim();

        if bracket_depth > 0 {
            // Continuation line inside open brackets — no INDENT/DEDENT, and
            // no Newline until the brackets close.
            if trimmed.is_empty() { continue; }
            let line_toks = lex_line(trimmed, line_num);
            bracket_depth += bracket_delta(&line_toks);
            out.extend(line_toks);
            if bracket_depth <= 0 {
                bracket_depth = 0;
                out.push(Token { kind: TokenKind::Newline, line: line_num });
            }
            continue;
        }

        // Skip blank / comment-only lines
        if trimmed.is_empty() { continue; }

        let indent = leading_indent(line);
        let cur = *indent_stack.last().unwrap();
        if indent > cur {
            indent_stack.push(indent);
            out.push(Token { kind: TokenKind::Indent, line: line_num });
        } else if indent < cur {
            while *indent_stack.last().unwrap() > indent {
                indent_stack.pop();
                out.push(Token { kind: TokenKind::Dedent, line: line_num });
            }
        }

        let line_toks = lex_line(trimmed, line_num);
        bracket_depth += bracket_delta(&line_toks);
        out.extend(line_toks);
        if bracket_depth <= 0 {
            bracket_depth = 0;
            out.push(Token { kind: TokenKind::Newline, line: line_num });
        }
        // else: open bracket — suppress Newline, next lines are continuations
    }

    // Close any remaining open blocks
    while indent_stack.len() > 1 {
        indent_stack.pop();
        let ln = out.last().map(|t| t.line).unwrap_or(1);
        out.push(Token { kind: TokenKind::Dedent, line: ln });
    }

    let ln = out.last().map(|t| t.line).unwrap_or(1);
    out.push(Token { kind: TokenKind::Eof, line: ln });
    out
}

// ===== LINE LEXER =====

/// Net change in open-bracket depth contributed by a line's tokens —
/// counts ( [ { as +1 and ) ] } as -1. Brackets inside string literals are
/// already folded into `Str` tokens, so they don't count (Phase 17).
fn bracket_delta(toks: &[Token]) -> i32 {
    let mut d = 0;
    for t in toks {
        match t.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => d += 1,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => d -= 1,
            _ => {}
        }
    }
    d
}

fn lex_line(src: &str, line: usize) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < chars.len() {
        let c = chars[pos];
        if c == ' ' || c == '\t' { pos += 1; continue; }

        // Inline comments ।...।
        if c == '।' {
            pos += 1;
            while pos < chars.len() && chars[pos] != '।' { pos += 1; }
            if pos < chars.len() { pos += 1; }
            continue;
        }

        // String literal
        if c == '"' {
            pos += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 1;
                    // Unknown escapes keep the backslash (Python behavior) —
                    // lets regex patterns like "\d+" work without doubling
                    match chars[pos] {
                        'n' => s.push('\n'), 't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'), '\\' => s.push('\\'),
                        o => { s.push('\\'); s.push(o); }
                    }
                } else {
                    s.push(chars[pos]);
                }
                pos += 1;
            }
            if pos < chars.len() { pos += 1; }
            tokens.push(Token { kind: TokenKind::Str(s), line });
            continue;
        }

        // Radix literals: 0x.. hex, 0o.. octal, 0b.. binary (Phase 19+ — lights up bitmask work).
        if c == '0' && pos + 1 < chars.len() {
            let nxt = chars[pos+1];
            let (radix, valid): (u32, fn(char) -> bool) = match nxt {
                'x' | 'X' => (16, |c: char| c.is_ascii_hexdigit()),
                'o' | 'O' => (8,  |c: char| ('0'..='7').contains(&c)),
                'b' | 'B' => (2,  |c: char| c == '0' || c == '1'),
                _ => (0, |_| false),
            };
            if radix != 0 && pos + 2 < chars.len() && valid(chars[pos+2]) {
                let digits_start = pos + 2;
                let mut p = digits_start;
                while p < chars.len() && (valid(chars[p]) || chars[p] == '_') { p += 1; }
                let raw: String = chars[digits_start..p].iter().filter(|&&c| c != '_').collect();
                let n = i64::from_str_radix(&raw, radix)
                    .map(|n| n as f64)
                    .unwrap_or(f64::NAN);
                pos = p;
                tokens.push(Token { kind: TokenKind::Number(n), line });
                continue;
            }
        }

        // Number (ASCII or Devanagari digits)
        if c.is_ascii_digit() || is_dev_digit(c) {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || is_dev_digit(chars[pos]) || chars[pos] == '.') {
                pos += 1;
            }
            // Scientific notation: [eE][+-]?digit+   (only consumed when followed by a digit,
            // so identifiers like `1eleven` or `3embed` stay a Number+Identifier pair.)
            if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
                let mut look = pos + 1;
                if look < chars.len() && (chars[look] == '+' || chars[look] == '-') { look += 1; }
                if look < chars.len() && chars[look].is_ascii_digit() {
                    pos = look;
                    while pos < chars.len() && chars[pos].is_ascii_digit() { pos += 1; }
                }
            }
            // Look for lakh / crore suffix
            let mut tmp = pos;
            while tmp < chars.len() && chars[tmp] == ' ' { tmp += 1; }
            let raw: String = chars[start..pos].iter().collect();
            let base: f64 = dev_to_ascii(&raw).parse().unwrap_or(0.0);
            let suffix: String = chars[tmp..].iter().take_while(|&&c| is_dev(c)).collect();
            if suffix.starts_with("लाख") {
                pos = tmp + byte_len("लाख");
                tokens.push(Token { kind: TokenKind::Number(base * 1e5), line });
            } else if suffix.starts_with("करोड़") {
                pos = tmp + byte_len("करोड़");
                tokens.push(Token { kind: TokenKind::Number(base * 1e7), line });
            } else if suffix.starts_with("करोड") {
                pos = tmp + byte_len("करोड");
                tokens.push(Token { kind: TokenKind::Number(base * 1e7), line });
            } else {
                tokens.push(Token { kind: TokenKind::Number(base), line });
            }
            continue;
        }

        // ₹ prefix → strip, treat number
        if c == '₹' {
            pos += 1;
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || is_dev_digit(chars[pos]) || chars[pos] == ',' || chars[pos] == '.') {
                pos += 1;
            }
            let raw: String = chars[start..pos].iter().filter(|&&c| c != ',').collect();
            let n: f64 = dev_to_ascii(&raw).parse().unwrap_or(0.0);
            tokens.push(Token { kind: TokenKind::Number(n), line });
            continue;
        }

        // Identifier / keyword
        if is_dev(c) || c == '_' || c.is_alphabetic() {
            let start = pos;
            while pos < chars.len() && (is_dev(chars[pos]) || chars[pos] == '_' || chars[pos].is_alphabetic() || chars[pos].is_ascii_digit() || is_dev_digit(chars[pos])) {
                pos += 1;
            }
            let word: String = chars[start..pos].iter().collect();
            tokens.push(match_kw(&word, &chars, &mut pos, line));
            continue;
        }

        let kind = match c {
            '+' => { pos += 1; TokenKind::Plus }
            '-' => {
                if pos + 1 < chars.len() && chars[pos+1] == '>' { pos += 2; TokenKind::Arrow }
                else { pos += 1; TokenKind::Minus }
            }
            '*' => { pos += 1; TokenKind::Star }
            '/' => {
                if pos + 1 < chars.len() && chars[pos+1] == '/' { pos += 2; TokenKind::SlashSlash }
                else { pos += 1; TokenKind::Slash }
            }
            '%' => { pos += 1; TokenKind::Percent }
            ':' => {
                if pos + 1 < chars.len() && chars[pos+1] == '=' { pos += 2; TokenKind::ColonEq }
                else { pos += 1; TokenKind::Colon }
            }
            '.' => { pos += 1; TokenKind::Dot }
            ',' => { pos += 1; TokenKind::Comma }
            '(' => { pos += 1; TokenKind::LParen }
            ')' => { pos += 1; TokenKind::RParen }
            '[' => { pos += 1; TokenKind::LBracket }
            ']' => { pos += 1; TokenKind::RBracket }
            '{' => { pos += 1; TokenKind::LBrace }
            '}' => { pos += 1; TokenKind::RBrace }
            '=' => {
                if pos + 1 < chars.len() && chars[pos+1] == '=' { pos += 2; TokenKind::EqEq }
                else { pos += 1; TokenKind::Assign }
            }
            '!' => {
                if pos + 1 < chars.len() && chars[pos+1] == '=' { pos += 2; TokenKind::NotEq }
                else { pos += 1; TokenKind::Unknown('!') }
            }
            '<' => {
                if pos + 1 < chars.len() && chars[pos+1] == '<' { pos += 2; TokenKind::LShift }
                else if pos + 1 < chars.len() && chars[pos+1] == '=' { pos += 2; TokenKind::LtEq }
                else { pos += 1; TokenKind::Lt }
            }
            '>' => {
                if pos + 1 < chars.len() && chars[pos+1] == '>' { pos += 2; TokenKind::RShift }
                else if pos + 1 < chars.len() && chars[pos+1] == '=' { pos += 2; TokenKind::GtEq }
                else { pos += 1; TokenKind::Gt }
            }
            '&' => { pos += 1; TokenKind::BitAnd }
            '|' => { pos += 1; TokenKind::BitOr }
            '^' => { pos += 1; TokenKind::BitXor }
            '~' => { pos += 1; TokenKind::BitNot }
            '@' => { pos += 1; TokenKind::At }
            ';' => { pos += 1; TokenKind::Newline }
            _ => { pos += 1; TokenKind::Unknown(c) }
        };
        tokens.push(Token { kind, line });
    }
    tokens
}

// ===== KEYWORD MATCHING =====

fn match_kw(word: &str, chars: &[char], pos: &mut usize, line: usize) -> Token {
    let kind = match word {
        "है"       => TokenKind::Hai,
        "विधि"     => TokenKind::Vidhi,
        "फल"       => TokenKind::Fal,
        "उत्पन्न"   => TokenKind::Utpann,
        "प्रतीक्षा"  => TokenKind::Pratiksha,
        "यदि"      => TokenKind::Yadi,
        "अन्यथा" | "अन्य" => TokenKind::Anyatha,
        "बताओ"     => TokenKind::Batao,
        "सत्य"     => TokenKind::Bool(true),
        "असत्य"    => TokenKind::Bool(false),
        "कर्ता"      => TokenKind::Karta,
        "कर्म"       => TokenKind::Karma,
        "करण"        => TokenKind::Karan,
        "सम्प्रदान"  => TokenKind::Sampradan,
        "अपादान"     => TokenKind::Apadan,
        "अधिकरण"     => TokenKind::Adhikaran,
        "आयात"       => TokenKind::Aayat,
        "वर्ग"       => TokenKind::Varg,
        "अगला"       => TokenKind::Agla,
        "लिखो"       => TokenKind::Likho,
        "कोशिश"      => TokenKind::Koshish,
        "फेंको"      => TokenKind::Phenko,
        "लाम्डा"     => TokenKind::Lambda,
        "तो"         => TokenKind::Toh,
        // पकड़ो has a nukta — match exact codepoint variants (decomposed ड+़ or precomposed ड़)
        _ if matches!(word.chars().collect::<Vec<_>>().as_slice(),
             ['\u{092A}', '\u{0915}', '\u{0921}', '\u{093C}', '\u{094B}']
           | ['\u{092A}', '\u{0915}', '\u{095C}', '\u{094B}'])
             => TokenKind::Pakdo,
        "जब"         => if peek_word(chars, *pos).as_deref() == Some("तक") {
            skip_word(chars, pos); TokenKind::JabTak
        } else { TokenKind::Ident("जब".into()) },
        "बंद"        => if peek_word(chars, *pos).as_deref() == Some("करो") {
            skip_word(chars, pos); TokenKind::BandKaro
        } else { TokenKind::Ident("बंद".into()) },
        "बराबर"      => TokenKind::Barabar,
        // Normalize पढ़ो to an ASCII key so HashMap lookups work regardless of
        // how different editors encode the nukta sequence. Exact match only —
        // a loose contains-check used to swallow identifiers like पथ_जोड़ो.
        _ if matches!(word.chars().collect::<Vec<_>>().as_slice(),
             ['\u{092A}', '\u{0922}', '\u{093C}', '\u{094B}']
           | ['\u{092A}', '\u{095D}', '\u{094B}'])
             => TokenKind::Ident("__padho__".into()),
        "में"      => TokenKind::Mein,
        "से"       => match peek_word(chars, *pos).as_deref() {
            Some("अधिक") => { skip_word(chars, pos); TokenKind::SeAdhik }
            Some("कम")   => { skip_word(chars, pos); TokenKind::SeKam }
            _            => TokenKind::Ident("से".into())
        },
        "के"       => if peek_word(chars, *pos).as_deref() == Some("लिए") {
            skip_word(chars, pos); TokenKind::KeeLiye
        } else { TokenKind::Ident("के".into()) },
        "बार"      => if peek_word(chars, *pos).as_deref() == Some("करो") {
            skip_word(chars, pos); TokenKind::BarKaro
        } else { TokenKind::Ident("बार".into()) },
        "और"       => TokenKind::Aur,
        "या"       => TokenKind::Ya,
        "नहीं"    => TokenKind::Nahin,
        "में_है"   => TokenKind::MeinHai,
        "नहीं_है" => TokenKind::NahinHai,
        "वैश्विक" => TokenKind::Vaishvik,
        "विकल्प"  => TokenKind::Vikalp,
        "मिलाओ"   => TokenKind::Milao,
        "जाँचो"   => TokenKind::Jancho,
        "स्थिर"   => TokenKind::Sthir,
        "शुद्ध"   => TokenKind::Shuddha,
        // साझा/सार are keywords only before विधि/वर्ग — otherwise plain identifiers
        "साझा"    => if peek_word(chars, *pos).as_deref() == Some("विधि") {
            TokenKind::Sajha
        } else { TokenKind::Ident("साझा".into()) },
        "सार"     => if peek_word(chars, *pos).as_deref() == Some("वर्ग") {
            TokenKind::Sar
        } else { TokenKind::Ident("सार".into()) },
        "परीक्षण" => TokenKind::Parikshan,
        "साथ"     => TokenKind::Saath,
        "के_रूप_में" => TokenKind::KeRupMein,
        "अभिलेख"  => TokenKind::Abhilekh,
        other => TokenKind::Ident(other.into()),
    };
    Token { kind, line }
}

fn peek_word(chars: &[char], mut p: usize) -> Option<String> {
    while p < chars.len() && chars[p] == ' ' { p += 1; }
    if p >= chars.len() || (!is_dev(chars[p]) && !chars[p].is_alphabetic()) { return None; }
    let s = p;
    while p < chars.len() && (is_dev(chars[p]) || chars[p].is_alphabetic()) { p += 1; }
    Some(chars[s..p].iter().collect())
}

fn skip_word(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos] == ' ' { *pos += 1; }
    while *pos < chars.len() && (is_dev(chars[*pos]) || chars[*pos].is_alphabetic()) { *pos += 1; }
}

// ===== UTILITIES =====

fn is_dev(c: char) -> bool { ('\u{0900}'..='\u{097F}').contains(&c) }
fn is_dev_digit(c: char) -> bool { ('\u{0966}'..='\u{096F}').contains(&c) }

fn dev_to_ascii(s: &str) -> String {
    s.chars().map(|c| if is_dev_digit(c) {
        char::from_u32('0' as u32 + c as u32 - 0x0966).unwrap_or(c)
    } else { c }).collect()
}

fn leading_indent(line: &str) -> usize {
    let mut n = 0usize;
    for c in line.chars() {
        match c { ' ' => n += 1, '\t' => n += 4, _ => break }
    }
    n
}

fn leading_char_count(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

// String-aware: '#' / '।' inside "..." are content, not comment starts
// (was a plain find() — any "#" in a string literal truncated the line)
fn strip_comment(s: &str) -> &str {
    let mut in_str = false;
    let mut escape = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if escape { escape = false; }
            else if c == '\\' { escape = true; }
            else if c == '"' { in_str = false; }
        } else if c == '"' {
            in_str = true;
        } else if c == '#' || c == '।' {
            return &s[..i];
        }
    }
    s
}

/// Returns byte length of a string (for advancing past multi-byte chars)
fn byte_len(s: &str) -> usize { s.len() }
