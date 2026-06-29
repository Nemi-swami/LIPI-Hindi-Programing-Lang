//! Minimal in-memory SQL database for LIPI — pure Rust, no external crates.
//!
//! Exposed as the stdlib module `भारत.संग्रह`. A database is an opaque handle
//! (Number) in a thread-local registry. Supports a practical SQL subset:
//!   CREATE TABLE t (c1, c2, ...)
//!   INSERT INTO t [ (cols) ] VALUES (v1, v2, ...)
//!   SELECT * | c1, c2 FROM t [WHERE cond] [ORDER BY col [ASC|DESC]] [LIMIT n]
//!   UPDATE t SET c = v [, ...] [WHERE cond]
//!   DELETE FROM t [WHERE cond]
//!   DROP TABLE t
//! WHERE supports comparisons (= != <> < > <= >=) joined by AND / OR (AND binds
//! tighter; no parentheses). Literals: numbers, 'single-quoted strings'.
//!
//!   db_नया()              → handle
//!   db_चलाओ(handle, sql)  → SELECT: List of Dict (rows); others: Number (affected)
//!   db_सहेजो(handle, path)→ persist to file
//!   db_खोलो(path)         → load file → handle
//!   db_बंद(handle)        → close

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::cell::{Cell as StdCell, RefCell};
use std::collections::HashMap;

#[derive(Clone)]
struct Table { columns: Vec<String>, rows: Vec<Vec<Cell>> }

#[derive(Clone, PartialEq)]
enum Cell { Num(f64), Text(String), Null }

impl Cell {
    fn to_value(&self) -> Value {
        match self {
            Cell::Num(n) => Value::Number(*n),
            Cell::Text(s) => Value::Str(s.clone()),
            Cell::Null => Value::Nil,
        }
    }
}

type Database = HashMap<String, Table>;

thread_local! {
    static DBS: RefCell<HashMap<u64, Database>> = RefCell::new(HashMap::new());
    static NEXT_DB: StdCell<u64> = const { StdCell::new(1) };
}

fn alloc_db(db: Database) -> u64 {
    let id = NEXT_DB.with(|n| { let v = n.get(); n.set(v + 1); v });
    DBS.with(|m| m.borrow_mut().insert(id, db));
    id
}

// ── SQL tokenizer ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Word(String),     // keyword or identifier (kept verbatim; keywords matched uppercase)
    Str(String),      // 'literal'
    Num(f64),
    Sym(String),      // ( ) , * = != <> < > <= >=
}

fn tokenize_sql(s: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == ';' { i += 1; continue; }
        if c == '\'' {
            // string literal; '' is an escaped quote
            i += 1;
            let mut buf = String::new();
            loop {
                if i >= chars.len() { return Err("SQL: अधूरा वाक्य-शाब्दिक".to_string()); }
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' { buf.push('\''); i += 2; continue; }
                    i += 1; break;
                }
                buf.push(chars[i]); i += 1;
            }
            out.push(Tok::Str(buf));
            continue;
        }
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()
            && matches!(out.last(), Some(Tok::Sym(_)) | None)) {
            let start = i;
            if chars[i] == '-' { i += 1; }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
            let numstr: String = chars[start..i].iter().collect();
            let n = numstr.parse::<f64>().map_err(|_| format!("SQL: अमान्य संख्या '{numstr}'"))?;
            out.push(Tok::Num(n));
            continue;
        }
        if c == '(' || c == ')' || c == ',' || c == '*' {
            out.push(Tok::Sym(c.to_string())); i += 1; continue;
        }
        if c == '=' { out.push(Tok::Sym("=".into())); i += 1; continue; }
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '=' { out.push(Tok::Sym("!=".into())); i += 2; continue; }
        if c == '<' {
            if i + 1 < chars.len() && chars[i + 1] == '>' { out.push(Tok::Sym("!=".into())); i += 2; continue; }
            if i + 1 < chars.len() && chars[i + 1] == '=' { out.push(Tok::Sym("<=".into())); i += 2; continue; }
            out.push(Tok::Sym("<".into())); i += 1; continue;
        }
        if c == '>' {
            if i + 1 < chars.len() && chars[i + 1] == '=' { out.push(Tok::Sym(">=".into())); i += 2; continue; }
            out.push(Tok::Sym(">".into())); i += 1; continue;
        }
        // identifier / keyword: letters, digits, _, and any non-ASCII (Devanagari)
        let start = i;
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_whitespace() || "();,*='!<>".contains(ch) { break; }
            i += 1;
        }
        let w: String = chars[start..i].iter().collect();
        out.push(Tok::Word(w));
    }
    Ok(out)
}

// ── Parser + executor ────────────────────────────────────────────────────────

struct Parser { toks: Vec<Tok>, pos: usize }

impl Parser {
    fn peek(&self) -> Option<&Tok> { self.toks.get(self.pos) }
    fn next(&mut self) -> Option<Tok> { let t = self.toks.get(self.pos).cloned(); self.pos += 1; t }
    fn kw(&mut self, k: &str) -> bool {
        if let Some(Tok::Word(w)) = self.peek() {
            if w.eq_ignore_ascii_case(k) { self.pos += 1; return true; }
        }
        false
    }
    fn expect_sym(&mut self, s: &str) -> Result<(), String> {
        match self.next() { Some(Tok::Sym(x)) if x == s => Ok(()), other => Err(format!("SQL: '{s}' अपेक्षित, मिला {other:?}")) }
    }
    fn ident(&mut self) -> Result<String, String> {
        match self.next() { Some(Tok::Word(w)) => Ok(w), other => Err(format!("SQL: नाम अपेक्षित, मिला {other:?}")) }
    }
}

#[derive(Clone)]
enum Pred { Cmp { col: String, op: String, val: Cell }, And(Box<Pred>, Box<Pred>), Or(Box<Pred>, Box<Pred>), True }

fn parse_value(p: &mut Parser) -> Result<Cell, String> {
    match p.next() {
        Some(Tok::Num(n)) => Ok(Cell::Num(n)),
        Some(Tok::Str(s)) => Ok(Cell::Text(s)),
        Some(Tok::Word(w)) if w.eq_ignore_ascii_case("NULL") => Ok(Cell::Null),
        other => Err(format!("SQL: मान अपेक्षित, मिला {other:?}")),
    }
}

fn parse_where(p: &mut Parser) -> Result<Pred, String> { parse_or(p) }

fn parse_or(p: &mut Parser) -> Result<Pred, String> {
    let mut left = parse_and(p)?;
    while p.kw("OR") { let right = parse_and(p)?; left = Pred::Or(Box::new(left), Box::new(right)); }
    Ok(left)
}
fn parse_and(p: &mut Parser) -> Result<Pred, String> {
    let mut left = parse_cmp(p)?;
    while p.kw("AND") { let right = parse_cmp(p)?; left = Pred::And(Box::new(left), Box::new(right)); }
    Ok(left)
}
fn parse_cmp(p: &mut Parser) -> Result<Pred, String> {
    let col = p.ident()?;
    let op = match p.next() {
        Some(Tok::Sym(s)) if ["=","!=","<",">","<=",">="].contains(&s.as_str()) => s,
        other => return Err(format!("SQL: तुलना चिह्न अपेक्षित, मिला {other:?}")),
    };
    let val = parse_value(p)?;
    Ok(Pred::Cmp { col, op, val })
}

fn cmp_cells(a: &Cell, b: &Cell) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Cell::Num(x), Cell::Num(y)) => x.partial_cmp(y),
        (Cell::Text(x), Cell::Text(y)) => Some(x.cmp(y)),
        (Cell::Null, Cell::Null) => Some(std::cmp::Ordering::Equal),
        _ => None,
    }
}

fn eval_pred(pred: &Pred, cols: &[String], row: &[Cell]) -> Result<bool, String> {
    match pred {
        Pred::True => Ok(true),
        Pred::And(a, b) => Ok(eval_pred(a, cols, row)? && eval_pred(b, cols, row)?),
        Pred::Or(a, b) => Ok(eval_pred(a, cols, row)? || eval_pred(b, cols, row)?),
        Pred::Cmp { col, op, val } => {
            let idx = cols.iter().position(|c| c == col)
                .ok_or_else(|| format!("SQL: अज्ञात स्तंभ '{col}'"))?;
            let cell = &row[idx];
            let ord = cmp_cells(cell, val);
            Ok(match op.as_str() {
                "=" => ord == Some(std::cmp::Ordering::Equal),
                "!=" => ord != Some(std::cmp::Ordering::Equal),
                "<" => ord == Some(std::cmp::Ordering::Less),
                ">" => ord == Some(std::cmp::Ordering::Greater),
                "<=" => matches!(ord, Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal)),
                ">=" => matches!(ord, Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)),
                _ => false,
            })
        }
    }
}

fn exec_sql(db: &mut Database, sql: &str) -> Result<Value, String> {
    let toks = tokenize_sql(sql)?;
    if toks.is_empty() { return Ok(Value::Number(0.0)); }
    let mut p = Parser { toks, pos: 0 };
    if p.kw("CREATE") {
        if !p.kw("TABLE") { return Err("SQL: CREATE के बाद TABLE अपेक्षित".to_string()); }
        let name = p.ident()?;
        p.expect_sym("(")?;
        let mut cols = Vec::new();
        loop {
            cols.push(p.ident()?);
            match p.next() {
                Some(Tok::Sym(s)) if s == "," => continue,
                Some(Tok::Sym(s)) if s == ")" => break,
                other => return Err(format!("SQL: ',' या ')' अपेक्षित, मिला {other:?}")),
            }
        }
        if db.contains_key(&name) { return Err(format!("SQL: तालिका '{name}' पहले से मौजूद है")); }
        db.insert(name, Table { columns: cols, rows: Vec::new() });
        return Ok(Value::Number(0.0));
    }
    if p.kw("DROP") {
        p.kw("TABLE");
        let name = p.ident()?;
        return Ok(Value::Number(if db.remove(&name).is_some() { 1.0 } else { 0.0 }));
    }
    if p.kw("INSERT") {
        p.kw("INTO");
        let name = p.ident()?;
        let table = db.get(&name).ok_or_else(|| format!("SQL: अज्ञात तालिका '{name}'"))?.clone();
        // optional column list
        let mut col_order: Option<Vec<String>> = None;
        if let Some(Tok::Sym(s)) = p.peek() { if s == "(" {
            p.expect_sym("(")?;
            let mut cs = Vec::new();
            loop {
                cs.push(p.ident()?);
                match p.next() {
                    Some(Tok::Sym(s)) if s == "," => continue,
                    Some(Tok::Sym(s)) if s == ")" => break,
                    other => return Err(format!("SQL: ',' या ')' अपेक्षित, मिला {other:?}")),
                }
            }
            col_order = Some(cs);
        }}
        if !p.kw("VALUES") { return Err("SQL: VALUES अपेक्षित".to_string()); }
        p.expect_sym("(")?;
        let mut vals = Vec::new();
        loop {
            vals.push(parse_value(&mut p)?);
            match p.next() {
                Some(Tok::Sym(s)) if s == "," => continue,
                Some(Tok::Sym(s)) if s == ")" => break,
                other => return Err(format!("SQL: ',' या ')' अपेक्षित, मिला {other:?}")),
            }
        }
        // build full row in table column order
        let mut row = vec![Cell::Null; table.columns.len()];
        match col_order {
            Some(cs) => {
                if cs.len() != vals.len() { return Err("SQL: स्तंभ और मान संख्या असमान".to_string()); }
                for (c, v) in cs.iter().zip(vals) {
                    let idx = table.columns.iter().position(|x| x == c)
                        .ok_or_else(|| format!("SQL: अज्ञात स्तंभ '{c}'"))?;
                    row[idx] = v;
                }
            }
            None => {
                if vals.len() != table.columns.len() { return Err("SQL: मान संख्या स्तंभ संख्या से मेल नहीं खाती".to_string()); }
                row = vals;
            }
        }
        db.get_mut(&name).unwrap().rows.push(row);
        return Ok(Value::Number(1.0));
    }
    if p.kw("SELECT") {
        // columns
        let mut sel_cols: Option<Vec<String>> = None;
        if let Some(Tok::Sym(s)) = p.peek() { if s == "*" { p.pos += 1; } }
        else {
            let mut cs = Vec::new();
            loop {
                cs.push(p.ident()?);
                if let Some(Tok::Sym(s)) = p.peek() { if s == "," { p.pos += 1; continue; } }
                break;
            }
            sel_cols = Some(cs);
        }
        if !p.kw("FROM") { return Err("SQL: FROM अपेक्षित".to_string()); }
        let name = p.ident()?;
        let table = db.get(&name).ok_or_else(|| format!("SQL: अज्ञात तालिका '{name}'"))?;
        let pred = if p.kw("WHERE") { parse_where(&mut p)? } else { Pred::True };
        // filter
        let mut result: Vec<&Vec<Cell>> = Vec::new();
        for row in &table.rows {
            if eval_pred(&pred, &table.columns, row)? { result.push(row); }
        }
        // ORDER BY
        let mut sorted: Vec<Vec<Cell>> = result.into_iter().cloned().collect();
        if p.kw("ORDER") {
            p.kw("BY");
            let col = p.ident()?;
            let desc = p.kw("DESC");
            if !desc { p.kw("ASC"); }
            let idx = table.columns.iter().position(|c| c == &col)
                .ok_or_else(|| format!("SQL: अज्ञात स्तंभ '{col}'"))?;
            sorted.sort_by(|a, b| {
                let o = cmp_cells(&a[idx], &b[idx]).unwrap_or(std::cmp::Ordering::Equal);
                if desc { o.reverse() } else { o }
            });
        }
        // LIMIT
        if p.kw("LIMIT") {
            if let Some(Tok::Num(n)) = p.next() { sorted.truncate(n.max(0.0) as usize); }
        }
        // project to List of Dict
        let out_cols: Vec<String> = sel_cols.unwrap_or_else(|| table.columns.clone());
        let col_idx: Vec<usize> = out_cols.iter()
            .map(|c| table.columns.iter().position(|x| x == c).ok_or_else(|| format!("SQL: अज्ञात स्तंभ '{c}'")))
            .collect::<Result<_, _>>()?;
        let rows: Vec<Value> = sorted.iter().map(|row| {
            let mut m = HashMap::new();
            for (oc, &ci) in out_cols.iter().zip(&col_idx) {
                m.insert(oc.clone(), row[ci].to_value());
            }
            Value::Dict(m)
        }).collect();
        return Ok(Value::List(rows));
    }
    if p.kw("DELETE") {
        p.kw("FROM");
        let name = p.ident()?;
        let cols = db.get(&name).ok_or_else(|| format!("SQL: अज्ञात तालिका '{name}'"))?.columns.clone();
        let pred = if p.kw("WHERE") { parse_where(&mut p)? } else { Pred::True };
        let table = db.get_mut(&name).unwrap();
        let before = table.rows.len();
        let mut kept = Vec::new();
        for row in std::mem::take(&mut table.rows) {
            if !eval_pred(&pred, &cols, &row)? { kept.push(row); }
        }
        table.rows = kept;
        return Ok(Value::Number((before - table.rows.len()) as f64));
    }
    if p.kw("UPDATE") {
        let name = p.ident()?;
        let cols = db.get(&name).ok_or_else(|| format!("SQL: अज्ञात तालिका '{name}'"))?.columns.clone();
        if !p.kw("SET") { return Err("SQL: SET अपेक्षित".to_string()); }
        let mut assigns: Vec<(usize, Cell)> = Vec::new();
        loop {
            let col = p.ident()?;
            p.expect_sym("=")?;
            let val = parse_value(&mut p)?;
            let idx = cols.iter().position(|c| c == &col)
                .ok_or_else(|| format!("SQL: अज्ञात स्तंभ '{col}'"))?;
            assigns.push((idx, val));
            if let Some(Tok::Sym(s)) = p.peek() { if s == "," { p.pos += 1; continue; } }
            break;
        }
        let pred = if p.kw("WHERE") { parse_where(&mut p)? } else { Pred::True };
        let table = db.get_mut(&name).unwrap();
        let mut count = 0;
        for row in table.rows.iter_mut() {
            if eval_pred(&pred, &cols, row)? {
                for (idx, val) in &assigns { row[*idx] = val.clone(); }
                count += 1;
            }
        }
        return Ok(Value::Number(count as f64));
    }
    Err("SQL: असमर्थित या अमान्य कथन".to_string())
}

// ── Persistence (length-prefixed text format) ────────────────────────────────

fn save_db(db: &Database) -> String {
    let mut out = String::from("LIPIDB1\n");
    let mut names: Vec<&String> = db.keys().collect();
    names.sort();
    out.push_str(&format!("{}\n", names.len()));
    for name in names {
        let t = &db[name];
        out.push_str(&format!("T {}\n", esc(name)));
        out.push_str(&format!("{}\n", t.columns.len()));
        for c in &t.columns { out.push_str(&format!("{}\n", esc(c))); }
        out.push_str(&format!("{}\n", t.rows.len()));
        for row in &t.rows {
            for cell in row {
                match cell {
                    Cell::Num(n) => out.push_str(&format!("N {n}\n")),
                    Cell::Text(s) => out.push_str(&format!("S {}\n", esc(s))),
                    Cell::Null => out.push_str("X\n"),
                }
            }
        }
    }
    out
}

fn esc(s: &str) -> String { s.replace('\\', "\\\\").replace('\n', "\\n") }
fn unesc(s: &str) -> String {
    let mut out = String::new();
    let mut ch = s.chars().peekable();
    while let Some(c) = ch.next() {
        if c == '\\' {
            match ch.next() { Some('n') => out.push('\n'), Some('\\') => out.push('\\'), Some(o) => { out.push('\\'); out.push(o); }, None => out.push('\\') }
        } else { out.push(c); }
    }
    out
}

fn load_db(text: &str) -> Result<Database, String> {
    let mut lines = text.lines();
    if lines.next() != Some("LIPIDB1") { return Err("db_खोलो(): अमान्य DB फ़ाइल".to_string()); }
    let tcount: usize = lines.next().and_then(|l| l.parse().ok()).ok_or("db_खोलो(): तालिका गिनती गलत")?;
    let mut db = Database::new();
    for _ in 0..tcount {
        let header = lines.next().ok_or("db_खोलो(): तालिका शीर्ष नहीं")?;
        let name = unesc(header.strip_prefix("T ").ok_or("db_खोलो(): 'T' अपेक्षित")?);
        let ccount: usize = lines.next().and_then(|l| l.parse().ok()).ok_or("db_खोलो(): स्तंभ गिनती गलत")?;
        let mut cols = Vec::new();
        for _ in 0..ccount { cols.push(unesc(lines.next().ok_or("db_खोलो(): स्तंभ नहीं")?)); }
        let rcount: usize = lines.next().and_then(|l| l.parse().ok()).ok_or("db_खोलो(): पंक्ति गिनती गलत")?;
        let mut rows = Vec::new();
        for _ in 0..rcount {
            let mut row = Vec::with_capacity(ccount);
            for _ in 0..ccount {
                let l = lines.next().ok_or("db_खोलो(): कक्ष नहीं")?;
                let cell = if let Some(rest) = l.strip_prefix("N ") {
                    Cell::Num(rest.parse().map_err(|_| "db_खोलो(): संख्या गलत".to_string())?)
                } else if let Some(rest) = l.strip_prefix("S ") {
                    Cell::Text(unesc(rest))
                } else { Cell::Null };
                row.push(cell);
            }
            rows.push(row);
        }
        db.insert(name, Table { columns: cols, rows });
    }
    Ok(db)
}

// ── Native functions ─────────────────────────────────────────────────────────

fn db_naya(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Number(alloc_db(Database::new()) as f64))
}

fn db_chalao(args: Vec<Value>) -> Result<Value, String> {
    let id = match args.first() { Some(Value::Number(n)) => *n as u64, _ => return Err("db_चलाओ(): हैंडल अपेक्षित".to_string()) };
    let sql = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db_चलाओ(): SQL वाक्य अपेक्षित".to_string()) };
    DBS.with(|m| {
        let mut map = m.borrow_mut();
        let db = map.get_mut(&id).ok_or("db_चलाओ(): अमान्य हैंडल")?;
        exec_sql(db, &sql)
    })
}

fn db_sahejo(args: Vec<Value>) -> Result<Value, String> {
    let id = match args.first() { Some(Value::Number(n)) => *n as u64, _ => return Err("db_सहेजो(): हैंडल अपेक्षित".to_string()) };
    let path = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db_सहेजो(): पथ अपेक्षित".to_string()) };
    DBS.with(|m| {
        let map = m.borrow();
        let db = map.get(&id).ok_or("db_सहेजो(): अमान्य हैंडल")?;
        std::fs::write(&path, save_db(db)).map_err(|e| format!("db_सहेजो(): {e}"))?;
        Ok(Value::Bool(true))
    })
}

fn db_kholo(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db_खोलो(): पथ अपेक्षित".to_string()) };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("db_खोलो(): {e}"))?;
    let db = load_db(&text)?;
    Ok(Value::Number(alloc_db(db) as f64))
}

fn db_band(args: Vec<Value>) -> Result<Value, String> {
    let id = match args.first() { Some(Value::Number(n)) => *n as u64, _ => return Err("db_बंद(): हैंडल अपेक्षित".to_string()) };
    let removed = DBS.with(|m| m.borrow_mut().remove(&id).is_some());
    Ok(Value::Bool(removed))
}

pub fn sangraha_registry() -> Registry {
    let list: Vec<(&'static str, NativeFn)> = vec![
        ("db_नया", db_naya),
        ("db_चलाओ", db_chalao),
        ("db_सहेजो", db_sahejo),
        ("db_खोलो", db_kholo),
        ("db_बंद", db_band),
    ];
    list
}
