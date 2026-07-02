//! भारत.प्रतिमान — pure-Rust regex engine (Phase 17B)
//!
//! Backtracking VM in the Pike/Cox style: the pattern compiles to a small
//! instruction list (Char/Class/Split/Jump/Save/Match) executed with an
//! explicit backtrack stack — no Rust-stack recursion, WASM-safe, and a
//! step budget guards against catastrophic backtracking.
//!
//! Supported syntax:
//!   literals  . ^ $ |  ( ) (?: )  [ ] [^ ] with ranges
//!   \d \D \w \W \s \S  \n \t \r  \<punct> escapes
//!   * + ? {n} {n,} {n,m}  and lazy variants (*? +? ?? {..}?)
//!
//! Unicode-aware: matching is over `char`s; \w covers Devanagari letters.

use crate::interpreter::Value;
use crate::bharat_stdlib::{NativeFn, Registry};

// ── Pattern AST ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Node {
    Char(char),
    Any,                       // .  (anything except \n)
    Class(ClassDef),
    Start,                     // ^
    End,                       // $
    Seq(Vec<Node>),
    Alt(Vec<Node>),            // a|b|c
    Group { idx: Option<usize>, inner: Box<Node> },
    Repeat { node: Box<Node>, min: u32, max: Option<u32>, lazy: bool },
}

#[derive(Debug, Clone)]
struct ClassDef {
    neg: bool,
    items: Vec<ClassItem>,
}

#[derive(Debug, Clone)]
enum ClassItem {
    Ch(char),
    Range(char, char),
    Digit, NotDigit,
    Word, NotWord,
    Space, NotSpace,
}

/// \w — alphanumeric, '_', plus the whole Devanagari block: matras, halant
/// and other combining signs (U+0900–U+097F) are NOT alphanumeric in Unicode,
/// but a Hindi word like नमस्ते is unusable without them.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || ('\u{0900}'..='\u{097F}').contains(&c)
}

impl ClassDef {
    fn matches(&self, c: char) -> bool {
        let hit = self.items.iter().any(|it| match it {
            ClassItem::Ch(x) => *x == c,
            ClassItem::Range(a, b) => *a <= c && c <= *b,
            ClassItem::Digit => c.is_ascii_digit(),
            ClassItem::NotDigit => !c.is_ascii_digit(),
            ClassItem::Word => is_word_char(c),
            ClassItem::NotWord => !is_word_char(c),
            ClassItem::Space => c.is_whitespace(),
            ClassItem::NotSpace => !c.is_whitespace(),
        });
        hit != self.neg
    }
}

// ── Pattern parser ────────────────────────────────────────────────────────────

struct PatParser {
    chars: Vec<char>,
    pos: usize,
    n_groups: usize,
}

impl PatParser {
    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn next(&mut self) -> Option<char> { let c = self.peek(); if c.is_some() { self.pos += 1; } c }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) { self.pos += 1; true } else { false }
    }

    /// alt := seq ('|' seq)*
    fn parse_alt(&mut self) -> Result<Node, String> {
        let mut branches = vec![self.parse_seq()?];
        while self.eat('|') {
            branches.push(self.parse_seq()?);
        }
        Ok(if branches.len() == 1 { branches.pop().unwrap() } else { Node::Alt(branches) })
    }

    /// seq := (atom quantifier?)*
    fn parse_seq(&mut self) -> Result<Node, String> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' { break; }
            let atom = self.parse_atom()?;
            items.push(self.parse_quantifier(atom)?);
        }
        Ok(if items.len() == 1 { items.pop().unwrap() } else { Node::Seq(items) })
    }

    fn parse_quantifier(&mut self, atom: Node) -> Result<Node, String> {
        let (min, max) = match self.peek() {
            Some('*') => { self.pos += 1; (0, None) }
            Some('+') => { self.pos += 1; (1, None) }
            Some('?') => { self.pos += 1; (0, Some(1)) }
            Some('{') => {
                // {n} {n,} {n,m} — if it doesn't parse as a counter, treat '{' literally
                let save = self.pos;
                self.pos += 1;
                match self.parse_counts() {
                    Some(c) => c,
                    None => { self.pos = save; return Ok(atom); }
                }
            }
            _ => return Ok(atom),
        };
        if matches!(atom, Node::Start | Node::End) {
            return Err("प्रतिमान: ^ या $ पर परिमाणक नहीं लग सकता".into());
        }
        if let Some(m) = max {
            if min > m { return Err(format!("प्रतिमान: {{{},{}}} में min > max", min, m)); }
        }
        if min > 1000 || max.is_some_and(|m| m > 1000) {
            return Err("प्रतिमान: परिमाणक सीमा 1000 से अधिक".into());
        }
        let lazy = self.eat('?');
        Ok(Node::Repeat { node: Box::new(atom), min, max, lazy })
    }

    /// After '{': n[,[m]]} → Some((min,max)); anything else → None (literal brace)
    fn parse_counts(&mut self) -> Option<(u32, Option<u32>)> {
        let n = self.parse_int()?;
        if self.eat('}') { return Some((n, Some(n))); }
        if !self.eat(',') { return None; }
        if self.eat('}') { return Some((n, None)); }
        let m = self.parse_int()?;
        if self.eat('}') { Some((n, Some(m))) } else { None }
    }

    fn parse_int(&mut self) -> Option<u32> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) { self.pos += 1; }
        if self.pos == start { return None; }
        self.chars[start..self.pos].iter().collect::<String>().parse().ok()
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        match self.next() {
            None => Err("प्रतिमान: अधूरा".into()),
            Some('.') => Ok(Node::Any),
            Some('^') => Ok(Node::Start),
            Some('$') => Ok(Node::End),
            Some('(') => {
                let idx = if self.peek() == Some('?') {
                    // only (?: ) supported
                    self.pos += 1;
                    if !self.eat(':') {
                        return Err("प्रतिमान: '(?' के बाद केवल ':' समर्थित".into());
                    }
                    None
                } else {
                    self.n_groups += 1;
                    Some(self.n_groups)
                };
                let inner = self.parse_alt()?;
                if !self.eat(')') {
                    return Err("प्रतिमान: ')' अपेक्षित".into());
                }
                Ok(Node::Group { idx, inner: Box::new(inner) })
            }
            Some(')') => Err("प्रतिमान: अनपेक्षित ')'".into()),
            Some('[') => self.parse_class(),
            Some('*') | Some('+') | Some('?') => Err("प्रतिमान: परिमाणक से पहले कुछ चाहिए".into()),
            Some('\\') => self.parse_escape(),
            Some(c) => Ok(Node::Char(c)),
        }
    }

    fn parse_escape(&mut self) -> Result<Node, String> {
        let c = self.next().ok_or("प्रतिमान: '\\' के बाद कुछ चाहिए")?;
        let single = |it: ClassItem| Node::Class(ClassDef { neg: false, items: vec![it] });
        Ok(match c {
            'd' => single(ClassItem::Digit),
            'D' => single(ClassItem::NotDigit),
            'w' => single(ClassItem::Word),
            'W' => single(ClassItem::NotWord),
            's' => single(ClassItem::Space),
            'S' => single(ClassItem::NotSpace),
            'n' => Node::Char('\n'),
            't' => Node::Char('\t'),
            'r' => Node::Char('\r'),
            other => Node::Char(other), // \. \( \\ \+ etc.
        })
    }

    /// '[' already consumed
    fn parse_class(&mut self) -> Result<Node, String> {
        let neg = self.eat('^');
        let mut items = Vec::new();
        // first ']' is a literal: []abc] matches ] a b c
        if self.eat(']') { items.push(ClassItem::Ch(']')); }
        loop {
            let c = self.next().ok_or("प्रतिमान: ']' अपेक्षित")?;
            if c == ']' { break; }
            if c == '\\' {
                let e = self.next().ok_or("प्रतिमान: '\\' के बाद कुछ चाहिए")?;
                items.push(match e {
                    'd' => ClassItem::Digit, 'D' => ClassItem::NotDigit,
                    'w' => ClassItem::Word,  'W' => ClassItem::NotWord,
                    's' => ClassItem::Space, 'S' => ClassItem::NotSpace,
                    'n' => ClassItem::Ch('\n'), 't' => ClassItem::Ch('\t'),
                    'r' => ClassItem::Ch('\r'),
                    o => ClassItem::Ch(o),
                });
                continue;
            }
            // range a-z (a '-' at the end or before ']' is a literal)
            if self.peek() == Some('-') && self.chars.get(self.pos + 1).is_some_and(|&n| n != ']') {
                self.pos += 1; // consume '-'
                let hi = self.next().ok_or("प्रतिमान: ']' अपेक्षित")?;
                let hi = if hi == '\\' { self.next().ok_or("प्रतिमान: '\\' के बाद कुछ चाहिए")? } else { hi };
                if c > hi { return Err(format!("प्रतिमान: उल्टी सीमा {}-{}", c, hi)); }
                items.push(ClassItem::Range(c, hi));
            } else {
                items.push(ClassItem::Ch(c));
            }
        }
        Ok(Node::Class(ClassDef { neg, items }))
    }
}

// ── Compiler: AST → instructions ──────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Inst {
    Char(char),
    Any,
    Class(usize),        // index into Regex::classes
    Start,
    End,
    Save(usize),         // saves[n] = current position
    Split(usize, usize), // try first branch, push second for backtracking
    Jump(usize),
    Match,
}

pub struct Regex {
    prog: Vec<Inst>,
    classes: Vec<ClassDef>,
    pub n_groups: usize,
}

struct RxCompiler {
    prog: Vec<Inst>,
    classes: Vec<ClassDef>,
}

impl RxCompiler {
    fn emit(&mut self, i: Inst) -> usize {
        self.prog.push(i);
        self.prog.len() - 1
    }
    fn here(&self) -> usize { self.prog.len() }

    fn compile(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::Char(c) => { self.emit(Inst::Char(*c)); }
            Node::Any => { self.emit(Inst::Any); }
            Node::Start => { self.emit(Inst::Start); }
            Node::End => { self.emit(Inst::End); }
            Node::Class(def) => {
                self.classes.push(def.clone());
                let idx = self.classes.len() - 1;
                self.emit(Inst::Class(idx));
            }
            Node::Seq(items) => for it in items { self.compile(it)?; },
            Node::Alt(branches) => {
                // split → b1 → jump end; split → b2 → jump end; … last branch plain
                let mut end_jumps = Vec::new();
                for (i, br) in branches.iter().enumerate() {
                    if i + 1 < branches.len() {
                        let sp = self.emit(Inst::Split(0, 0));
                        self.prog[sp] = Inst::Split(self.here(), 0); // first = this branch
                        self.compile(br)?;
                        end_jumps.push(self.emit(Inst::Jump(0)));
                        let next = self.here();
                        if let Inst::Split(a, _) = self.prog[sp] {
                            self.prog[sp] = Inst::Split(a, next);
                        }
                    } else {
                        self.compile(br)?;
                    }
                }
                let end = self.here();
                for j in end_jumps {
                    self.prog[j] = Inst::Jump(end);
                }
            }
            Node::Group { idx, inner } => {
                if let Some(g) = idx { self.emit(Inst::Save(2 * g)); }
                self.compile(inner)?;
                if let Some(g) = idx { self.emit(Inst::Save(2 * g + 1)); }
            }
            Node::Repeat { node, min, max, lazy } => {
                // mandatory copies
                for _ in 0..*min { self.compile(node)?; }
                match max {
                    Some(m) => {
                        // (m - min) optional copies: split over each
                        let mut exits = Vec::new();
                        for _ in *min..*m {
                            let sp = self.emit(Inst::Split(0, 0));
                            exits.push(sp);
                            let body = self.here();
                            self.compile(node)?;
                            // greedy prefers entering the body; lazy prefers the
                            // exit (patched to `end` below)
                            self.prog[sp] = if *lazy {
                                Inst::Split(0, body)
                            } else {
                                Inst::Split(body, 0)
                            };
                        }
                        let end = self.here();
                        for sp in exits {
                            self.prog[sp] = match (&self.prog[sp], lazy) {
                                (Inst::Split(a, _), false) => Inst::Split(*a, end),
                                (Inst::Split(_, b), true)  => Inst::Split(end, *b),
                                _ => unreachable!(),
                            };
                        }
                    }
                    None => {
                        // unbounded tail: L: split(body, end); body; jump L
                        let l = self.emit(Inst::Split(0, 0));
                        let body = self.here();
                        self.compile(node)?;
                        self.emit(Inst::Jump(l));
                        let end = self.here();
                        self.prog[l] = if *lazy {
                            Inst::Split(end, body)
                        } else {
                            Inst::Split(body, end)
                        };
                    }
                }
            }
        }
        Ok(())
    }
}

// ── Execution ─────────────────────────────────────────────────────────────────

/// Total step budget per top-level operation — guards catastrophic backtracking.
const STEP_BUDGET: usize = 2_000_000;

impl Regex {
    pub fn new(pattern: &str) -> Result<Regex, String> {
        let mut p = PatParser { chars: pattern.chars().collect(), pos: 0, n_groups: 0 };
        let ast = p.parse_alt()?;
        if p.pos < p.chars.len() {
            return Err(format!("प्रतिमान: स्थान {} पर अनपेक्षित '{}'", p.pos, p.chars[p.pos]));
        }
        let mut c = RxCompiler { prog: Vec::new(), classes: Vec::new() };
        c.emit(Inst::Save(0));
        c.compile(&ast)?;
        c.emit(Inst::Save(1));
        c.emit(Inst::Match);
        if c.prog.len() > 40_000 {
            return Err("प्रतिमान बहुत बड़ा".into());
        }
        Ok(Regex { prog: c.prog, classes: c.classes, n_groups: p.n_groups })
    }

    /// Try to match starting exactly at `start`. Returns saves on success.
    fn exec_at(
        &self,
        text: &[char],
        start: usize,
        steps: &mut usize,
    ) -> Result<Option<Vec<Option<usize>>>, String> {
        let n_saves = 2 * (self.n_groups + 1);
        let mut stack: Vec<(usize, usize, Vec<Option<usize>>)> =
            vec![(0, start, vec![None; n_saves])];

        while let Some((mut pc, mut pos, mut saves)) = stack.pop() {
            loop {
                *steps += 1;
                if *steps > STEP_BUDGET {
                    return Err("प्रतिमान बहुत जटिल (backtracking सीमा पार)".into());
                }
                match &self.prog[pc] {
                    Inst::Char(c) => {
                        if pos < text.len() && text[pos] == *c { pc += 1; pos += 1; } else { break; }
                    }
                    Inst::Any => {
                        if pos < text.len() && text[pos] != '\n' { pc += 1; pos += 1; } else { break; }
                    }
                    Inst::Class(i) => {
                        if pos < text.len() && self.classes[*i].matches(text[pos]) {
                            pc += 1; pos += 1;
                        } else { break; }
                    }
                    Inst::Start => { if pos == 0 { pc += 1; } else { break; } }
                    Inst::End => { if pos == text.len() { pc += 1; } else { break; } }
                    Inst::Save(n) => { saves[*n] = Some(pos); pc += 1; }
                    Inst::Split(a, b) => {
                        stack.push((*b, pos, saves.clone()));
                        pc = *a;
                    }
                    Inst::Jump(a) => { pc = *a; }
                    Inst::Match => return Ok(Some(saves)),
                }
            }
        }
        Ok(None)
    }

    /// Find the leftmost match at or after `from`.
    /// Returns (start, end, saves) in char indices.
    pub fn find_from(
        &self,
        text: &[char],
        from: usize,
        steps: &mut usize,
    ) -> Result<Option<(usize, usize, Vec<Option<usize>>)>, String> {
        for s in from..=text.len() {
            if let Some(saves) = self.exec_at(text, s, steps)? {
                let end = saves[1].unwrap_or(s);
                return Ok(Some((s, end, saves)));
            }
        }
        Ok(None)
    }
}

// ── Value-level helpers ───────────────────────────────────────────────────────

fn want_str(v: &Value, what: &str) -> Result<String, String> {
    match v {
        Value::Str(s) => Ok(s.clone()),
        other => Err(format!("{}: वाक्य अपेक्षित, मिला {}", what, other)),
    }
}

fn args2(args: &[Value], fname: &str) -> Result<(Regex, Vec<char>), String> {
    if args.len() != 2 {
        return Err(format!("{}(प्रतिमान, वाक्य) — 2 तर्क चाहिए, मिले {}", fname, args.len()));
    }
    let pat = want_str(&args[0], fname)?;
    let txt = want_str(&args[1], fname)?;
    Ok((Regex::new(&pat)?, txt.chars().collect()))
}

fn slice_str(text: &[char], a: usize, b: usize) -> String {
    text[a..b].iter().collect()
}

// ── Native functions ──────────────────────────────────────────────────────────

/// ढूंढो(प्रतिमान, वाक्य) → first matched substring, or शून्य
fn fn_dhundho(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "ढूंढो")?;
    let mut steps = 0;
    Ok(match re.find_from(&text, 0, &mut steps)? {
        Some((a, b, _)) => Value::Str(slice_str(&text, a, b)),
        None => Value::Nil,
    })
}

/// ढूंढो_स्थान(प्रतिमान, वाक्य) → char index of first match, or -1
fn fn_dhundho_sthan(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "ढूंढो_स्थान")?;
    let mut steps = 0;
    Ok(match re.find_from(&text, 0, &mut steps)? {
        Some((a, _, _)) => Value::Number(a as f64),
        None => Value::Number(-1.0),
    })
}

/// ढूंढो_सब(प्रतिमान, वाक्य) → List of all (non-overlapping) matches
fn fn_dhundho_sab(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "ढूंढो_सब")?;
    let mut steps = 0;
    let mut out = Vec::new();
    let mut from = 0;
    while let Some((a, b, _)) = re.find_from(&text, from, &mut steps)? {
        out.push(Value::Str(slice_str(&text, a, b)));
        from = if b > a { b } else { b + 1 }; // empty match → step forward
        if from > text.len() { break; }
    }
    Ok(Value::List(out))
}

/// मेल_है(प्रतिमान, वाक्य) → Bool — does the WHOLE string match?
fn fn_mel_hai(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "मेल_है")?;
    let mut steps = 0;
    // anchored: must match at 0 and consume everything
    let ok = match re.exec_at(&text, 0, &mut steps)? {
        Some(saves) => saves[1] == Some(text.len()),
        None => false,
    };
    Ok(Value::Bool(ok))
}

/// समूह(प्रतिमान, वाक्य) → List [पूर्ण, समूह1, …] for the first match, or []
fn fn_samuh(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "समूह")?;
    let mut steps = 0;
    Ok(match re.find_from(&text, 0, &mut steps)? {
        Some((_, _, saves)) => {
            let mut out = Vec::new();
            for g in 0..=re.n_groups {
                let v = match (saves[2 * g], saves[2 * g + 1]) {
                    (Some(a), Some(b)) => Value::Str(slice_str(&text, a, b)),
                    _ => Value::Str(String::new()), // group didn't participate
                };
                out.push(v);
            }
            Value::List(out)
        }
        None => Value::List(vec![]),
    })
}

/// बदलो_सब(प्रतिमान, बदल, वाक्य) → all matches replaced.
/// Replacement supports $0–$9 group refs and $$ for a literal $.
fn fn_badlo_sab(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(format!("बदलो_सब(प्रतिमान, बदल, वाक्य) — 3 तर्क चाहिए, मिले {}", args.len()));
    }
    let pat = want_str(&args[0], "बदलो_सब")?;
    let repl: Vec<char> = want_str(&args[1], "बदलो_सब")?.chars().collect();
    let text: Vec<char> = want_str(&args[2], "बदलो_सब")?.chars().collect();
    let re = Regex::new(&pat)?;

    let expand = |saves: &[Option<usize>], out: &mut String| {
        let mut i = 0;
        while i < repl.len() {
            if repl[i] == '$' && i + 1 < repl.len() {
                let n = repl[i + 1];
                if n == '$' { out.push('$'); i += 2; continue; }
                if let Some(g) = n.to_digit(10) {
                    let g = g as usize;
                    if g <= re.n_groups {
                        if let (Some(a), Some(b)) = (saves[2 * g], saves[2 * g + 1]) {
                            out.push_str(&slice_str(&text, a, b));
                        }
                        i += 2;
                        continue;
                    }
                }
            }
            out.push(repl[i]);
            i += 1;
        }
    };

    let mut steps = 0;
    let mut out = String::new();
    let mut from = 0;
    while let Some((a, b, saves)) = re.find_from(&text, from, &mut steps)? {
        out.push_str(&slice_str(&text, from, a));
        expand(&saves, &mut out);
        if b > a {
            from = b;
        } else {
            // empty match: emit current char and step forward
            if b < text.len() { out.push(text[b]); }
            from = b + 1;
        }
        if from > text.len() { break; }
    }
    if from <= text.len() {
        out.push_str(&slice_str(&text, from.min(text.len()), text.len()));
    }
    Ok(Value::Str(out))
}

/// विभाजित_सब(प्रतिमान, वाक्य) → split on every match → List of Str
fn fn_vibhajit_sab(args: Vec<Value>) -> Result<Value, String> {
    let (re, text) = args2(&args, "विभाजित_सब")?;
    let mut steps = 0;
    let mut out = Vec::new();
    let mut from = 0;
    let mut search = 0;
    while let Some((a, b, _)) = re.find_from(&text, search, &mut steps)? {
        if b == a {
            // empty match: don't split here, just move the search head
            search = a + 1;
            if search > text.len() { break; }
            continue;
        }
        out.push(Value::Str(slice_str(&text, from, a)));
        from = b;
        search = b;
    }
    out.push(Value::Str(slice_str(&text, from.min(text.len()), text.len())));
    Ok(Value::List(out))
}

pub fn pratimaan_registry() -> Registry {
    vec![
        ("ढूंढो",        fn_dhundho as NativeFn),
        ("ढूंढो_स्थान",   fn_dhundho_sthan as NativeFn),
        ("ढूंढो_सब",     fn_dhundho_sab as NativeFn),
        ("मेल_है",       fn_mel_hai as NativeFn),
        ("समूह",         fn_samuh as NativeFn),
        ("बदलो_सब",     fn_badlo_sab as NativeFn),
        ("विभाजित_सब",  fn_vibhajit_sab as NativeFn),
    ]
}
