// LIPI Editor v2 — vim-inspired terminal line editor

use std::io::{self, BufRead, Write};

const CLR:     &str = "\x1b[2J\x1b[H";
const RST:     &str = "\x1b[0m";
const BOLD:    &str = "\x1b[1m";
const DIM:     &str = "\x1b[2m";
const CYAN:    &str = "\x1b[96m";
const GRN:     &str = "\x1b[92m";
const YLW:     &str = "\x1b[93m";
const RED:     &str = "\x1b[91m";
const BLU:     &str = "\x1b[94m";
const MAG:     &str = "\x1b[35m";
const WHT:     &str = "\x1b[97m";
const SAFFRON: &str = "\x1b[38;5;208m";
const CUR_BG:  &str = "\x1b[48;5;235m";

const PAGE:  usize = 22;
const WIDTH: usize = 80;

pub struct Editor {
    path:       String,
    lines:      Vec<String>,
    page:       usize,
    cursor:     usize,           // 1-based; 0 = unset
    modified:   bool,
    message:    String,
    msg_ok:     bool,
    undo_stack: Vec<Vec<String>>,
    clipboard:  Vec<String>,
    find_query: String,
    find_from:  usize,
}

impl Editor {
    pub fn open(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<String> = if content.is_empty() {
            Vec::new()
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        let cursor = if lines.is_empty() { 0 } else { 1 };
        Editor {
            path: path.to_string(),
            lines,
            page: 0,
            cursor,
            modified: false,
            message: String::new(),
            msg_ok: true,
            undo_stack: Vec::new(),
            clipboard: Vec::new(),
            find_query: String::new(),
            find_from: 0,
        }
    }

    fn snapshot(&mut self) {
        if self.undo_stack.len() >= 50 { self.undo_stack.remove(0); }
        self.undo_stack.push(self.lines.clone());
    }

    fn fname(&self) -> &str {
        std::path::Path::new(&self.path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&self.path)
    }

    fn display(&self) {
        print!("{CLR}");
        let total  = self.lines.len();
        let ndig   = total.to_string().len().max(3);
        let cur_ln = if self.cursor > 0 { self.cursor } else { 1 };
        let fname  = self.fname();
        let mod_s  = if self.modified { format!(" {YLW}[*]{RST}") } else { String::new() };

        // ── Title bar ─────────────────────────────────────────────────────
        println!("  {BOLD}{SAFFRON}LIPI{RST}  {BLU}{BOLD}{}{RST}{}  \
                   {DIM}Ln {}  Col 1  ·  Pg {}/{}{RST}",
            fname, mod_s, cur_ln, self.page + 1, self.total_pages());
        println!("{CYAN}{}{RST}", "━".repeat(WIDTH));

        // ── Content ───────────────────────────────────────────────────────
        let start = self.page * PAGE;
        for i in 0..PAGE {
            let lineno = start + i + 1;
            if lineno <= total {
                let line  = &self.lines[lineno - 1];
                let max_w = WIDTH.saturating_sub(ndig + 5);
                let disp  = truncate(line, max_w);
                if self.cursor == lineno {
                    println!("{CUR_BG}{YLW}►{RST}{CUR_BG} {:>ndig$} {CYAN}│{RST}{CUR_BG} {WHT}{}{RST}",
                        lineno, disp);
                } else {
                    println!("  {DIM}{:>ndig$}{RST} {CYAN}│{RST} {}", lineno, disp);
                }
            } else {
                // vim-style empty row — tilde at gutter position
                println!("  {DIM}{:>ndig$}   {MAG}~{RST}", "");
            }
        }

        // ── Bottom bar ────────────────────────────────────────────────────
        println!("{DIM}{}{RST}", "─".repeat(WIDTH));
        if !self.message.is_empty() {
            let col  = if self.msg_ok { GRN } else { RED };
            let clip = if !self.clipboard.is_empty() {
                format!("  {DIM}[clip: {} ln]{RST}", self.clipboard.len())
            } else { String::new() };
            println!("  {col}► {}{RST}{}", self.message, clip);
        } else {
            let undo = if self.undo_stack.is_empty() { String::new() }
                       else { format!("  undo:{}", self.undo_stack.len()) };
            let clip = if !self.clipboard.is_empty() {
                format!("  clip:{}", self.clipboard.len())
            } else { String::new() };
            println!("  {DIM}{} lines{}{}  ·  ? help{RST}", total, undo, clip);
        }
        println!("{DIM}{}{RST}", "─".repeat(WIDTH));
        println!("  {DIM}Ctrl+O{RST} Open  {DIM}Ctrl+S{RST} Save  \
                   {DIM}Ctrl+R{RST} Run  {DIM}Ctrl+F{RST} Find  \
                   {DIM}Ctrl+H{RST} Replace  {DIM}Ctrl+Q{RST} Quit");
        println!("{DIM}{}{RST}", "─".repeat(WIDTH));
    }

    fn display_help(&self) {
        print!("{CLR}");
        println!("{CYAN}{}{RST}", "━".repeat(70));
        println!("  {BOLD}{SAFFRON}LIPI{RST} {BOLD}Editor v2 — Command Reference{RST}");
        println!("{CYAN}{}{RST}", "━".repeat(70));
        println!();
        println!("  {YLW}{BOLD}NAVIGATION{RST}");
        println!("  {GRN}n{RST} / {GRN}p{RST}             Next / Previous page");
        println!("  {GRN}g<n>{RST}              Go to line n");
        println!("  {GRN}<n>{RST}               Jump to line n (type number only)");
        println!();
        println!("  {YLW}{BOLD}EDITING{RST}");
        println!("  {GRN}a <text>{RST}           Append line at end");
        println!("  {GRN}i<n> <text>{RST}        Insert after line n  (i0 = before line 1)");
        println!("  {GRN}e<n> <text>{RST}        Edit (replace) line n");
        println!("  {GRN}d<n>{RST}               Delete line n");
        println!("  {GRN}D<n>-<m>{RST}           Delete lines n through m");
        println!("  {GRN}dup<n>{RST}             Duplicate line n");
        println!("  {GRN}u{RST}                 Undo last change  (up to 50 levels)");
        println!();
        println!("  {YLW}{BOLD}SEARCH & REPLACE{RST}");
        println!("  {GRN}/<text>{RST}            Find first — from top  (Ctrl+F)");
        println!("  {GRN}N{RST}                 Find next occurrence");
        println!("  {GRN}h/<old>/<new>{RST}      Replace first in cursor line  (Ctrl+H)");
        println!("  {GRN}H/<old>/<new>{RST}      Replace ALL occurrences in file");
        println!();
        println!("  {YLW}{BOLD}CLIPBOARD{RST}");
        println!("  {GRN}c<n>{RST}               Copy line n to clipboard");
        println!("  {GRN}x<n>{RST}               Cut line n (copy + delete)");
        println!("  {GRN}v{RST} / {GRN}v<n>{RST}         Paste after cursor / after line n");
        println!();
        println!("  {YLW}{BOLD}FILE{RST}");
        println!("  {GRN}s{RST}                 Save file  (Ctrl+S)");
        println!("  {GRN}o <file>{RST}           Open another file  (Ctrl+O)");
        println!("  {GRN}r{RST}                 Save + Run  (Ctrl+R)");
        println!("  {GRN}q{RST}                 Quit  (Ctrl+Q)");
        println!();
        println!("{CYAN}{}{RST}", "━".repeat(70));
        print!("  {DIM}[Enter — editor में वापस]{RST}  ");
        io::stdout().flush().unwrap();
        let mut _buf = String::new();
        io::stdin().lock().read_line(&mut _buf).ok();
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        loop {
            self.display();
            print!("{GRN}:>{RST} ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    let cmd = line.trim().to_string();
                    if !self.exec(&cmd) { break; }
                }
            }
        }
    }

    fn exec(&mut self, raw: &str) -> bool {
        self.message.clear();
        self.msg_ok = true;

        // Pure number → jump to line
        if let Ok(n) = raw.trim().parse::<usize>() {
            if n >= 1 { self.jump(n); return true; }
        }

        match raw {
            "" => {}

            "?" | "help" => { self.display_help(); }

            // ── Navigation ───────────────────────────────────────────────
            "n" => {
                if self.page + 1 < self.total_pages() { self.page += 1; }
                else { self.err("अंतिम पृष्ठ"); }
            }
            "p" => {
                if self.page > 0 { self.page -= 1; }
                else { self.err("पहला पृष्ठ"); }
            }

            // ── Undo ─────────────────────────────────────────────────────
            "u" => {
                if let Some(prev) = self.undo_stack.pop() {
                    self.lines    = prev;
                    self.modified = true;
                    self.ok("↩ पूर्ववत");
                } else {
                    self.err("और पूर्ववत नहीं");
                }
            }

            // ── Save ─────────────────────────────────────────────────────
            "s" => { self.save(); }

            // ── Find next ────────────────────────────────────────────────
            "N" => {
                if self.find_query.is_empty() { self.err("पहले /text से खोजें"); }
                else { self.find_next(); }
            }

            // ── Run ──────────────────────────────────────────────────────
            "r" => {
                self.save();
                print!("{CLR}");
                println!("{CYAN}{}{RST}", "━".repeat(WIDTH));
                println!("  {BOLD}► Running: lipi {}{RST}", self.fname());
                println!("{CYAN}{}{RST}\n", "━".repeat(WIDTH));
                self.run_file();
                println!("\n{CYAN}{}{RST}", "━".repeat(WIDTH));
                println!("{DIM}  [Enter — editor में वापस]{RST}");
                io::stdout().flush().unwrap();
                let mut _buf = String::new();
                io::stdin().lock().read_line(&mut _buf).ok();
            }

            // ── Quit ─────────────────────────────────────────────────────
            "q" => {
                if self.modified {
                    print!("{CLR}");
                    print!("  {YLW}बिना सहेजे बाहर निकलें? (ह/न):{RST} ");
                    io::stdout().flush().unwrap();
                    let mut ans = String::new();
                    io::stdin().lock().read_line(&mut ans).ok();
                    let a = ans.trim();
                    if a == "ह" || a.eq_ignore_ascii_case("h") { return false; }
                } else {
                    return false;
                }
            }

            // ── Paste  v  or  v<n> ───────────────────────────────────────
            cmd if cmd.starts_with('v') => {
                if self.clipboard.is_empty() {
                    self.err("क्लिपबोर्ड खाली — c<n> या x<n> से कॉपी करें");
                } else {
                    let after = if cmd.len() > 1 {
                        cmd[1..].trim().parse::<usize>()
                            .unwrap_or(self.cursor.max(1))
                            .min(self.lines.len())
                    } else {
                        self.cursor.max(1).min(self.lines.len())
                    };
                    self.snapshot();
                    let count = self.clipboard.len();
                    for (i, l) in self.clipboard.iter().enumerate() {
                        self.lines.insert(after + i, l.clone());
                    }
                    self.modified = true;
                    self.jump(after + 1);
                    self.ok(&format!("{} लाइन पेस्ट (लाइन {} के बाद)", count, after));
                }
            }

            // ── Append ───────────────────────────────────────────────────
            cmd if cmd.starts_with('a') => {
                let text = if cmd.len() > 1 { cmd[1..].trim_start() } else { "" };
                self.snapshot();
                self.lines.push(text.to_string());
                self.modified = true;
                let n = self.lines.len();
                self.jump(n);
                self.ok(&format!("जोड़ा: लाइन {}", n));
            }

            // ── Duplicate  dup<n>  (must come before 'd') ────────────────
            cmd if cmd.starts_with("dup") => {
                match cmd[3..].trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= self.lines.len() => {
                        self.snapshot();
                        let copy = self.lines[n - 1].clone();
                        self.lines.insert(n, copy);
                        self.modified = true;
                        self.jump(n + 1);
                        self.ok(&format!("लाइन {} डुप्लिकेट → लाइन {}", n, n + 1));
                    }
                    _ => self.err(&format!("dup<n> — 1 से {} तक", self.lines.len().max(1))),
                }
            }

            // ── Delete range  D<n>-<m> ───────────────────────────────────
            cmd if cmd.starts_with('D') => {
                match parse_range(&cmd[1..]) {
                    Some((n, m)) if n >= 1 && m >= n && m <= self.lines.len() => {
                        self.snapshot();
                        self.lines.drain((n - 1)..m);
                        self.modified = true;
                        self.jump(n.min(self.lines.len().max(1)));
                        self.ok(&format!("लाइन {} से {} हटाई ({} लाइनें)", n, m, m - n + 1));
                    }
                    _ => self.err("उपयोग: D<n>-<m>  जैसे  D3-7"),
                }
            }

            // ── Delete single  d<n> ──────────────────────────────────────
            cmd if cmd.starts_with('d') => {
                match cmd[1..].trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= self.lines.len() => {
                        self.snapshot();
                        self.lines.remove(n - 1);
                        self.modified = true;
                        self.jump(n.min(self.lines.len().max(1)));
                        self.ok(&format!("लाइन {} हटाई", n));
                    }
                    _ => self.err(&format!("d<n> — 1 से {} तक", self.lines.len().max(1))),
                }
            }

            // ── Edit  e<n> <text> ────────────────────────────────────────
            cmd if cmd.starts_with('e') => {
                match parse_n_rest(&cmd[1..]) {
                    Some((n, text)) if n >= 1 => {
                        self.snapshot();
                        while self.lines.len() < n { self.lines.push(String::new()); }
                        self.lines[n - 1] = text.to_string();
                        self.modified = true;
                        self.jump(n);
                        self.ok(&format!("लाइन {} बदली", n));
                    }
                    _ => self.err("उपयोग: e<n> <text>  जैसे  e3 बताओ \"नमस्ते\""),
                }
            }

            // ── Insert  i<n> <text> ──────────────────────────────────────
            cmd if cmd.starts_with('i') => {
                match parse_n_rest(&cmd[1..]) {
                    Some((n, text)) => {
                        let pos = n.min(self.lines.len());
                        self.snapshot();
                        self.lines.insert(pos, text.to_string());
                        self.modified = true;
                        self.jump(pos + 1);
                        self.ok(&format!("लाइन {} के बाद जोड़ी", pos));
                    }
                    _ => self.err("उपयोग: i<n> <text>  (i0 = शुरुआत में)"),
                }
            }

            // ── Copy  c<n> ───────────────────────────────────────────────
            cmd if cmd.starts_with('c') => {
                match cmd[1..].trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= self.lines.len() => {
                        self.clipboard = vec![self.lines[n - 1].clone()];
                        self.cursor    = n;
                        self.ok(&format!("लाइन {} क्लिपबोर्ड में", n));
                    }
                    _ => self.err(&format!("c<n> — 1 से {} तक", self.lines.len().max(1))),
                }
            }

            // ── Cut  x<n> ────────────────────────────────────────────────
            cmd if cmd.starts_with('x') => {
                match cmd[1..].trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= self.lines.len() => {
                        self.snapshot();
                        self.clipboard = vec![self.lines.remove(n - 1)];
                        self.modified  = true;
                        self.jump(n.min(self.lines.len().max(1)));
                        self.ok(&format!("लाइन {} कट", n));
                    }
                    _ => self.err(&format!("x<n> — 1 से {} तक", self.lines.len().max(1))),
                }
            }

            // ── Replace all  H/<old>/<new> ───────────────────────────────
            cmd if cmd.starts_with('H') => {
                match parse_replace(&cmd[1..]) {
                    Some((old, new)) => {
                        let snap = self.lines.clone();
                        let mut count = 0usize;
                        for line in self.lines.iter_mut() {
                            if line.contains(&old) {
                                *line = line.replace(&old, &new);
                                count += 1;
                            }
                        }
                        if count > 0 {
                            self.undo_stack.push(snap);
                            self.modified = true;
                            self.ok(&format!("{} लाइनों में '{}' → '{}' बदला", count, old, new));
                        } else {
                            self.err(&format!("'{}' फ़ाइल में नहीं मिला", old));
                        }
                    }
                    None => self.err("उपयोग: H/<पुराना>/<नया>  जैसे  H/foo/bar"),
                }
            }

            // ── Replace in cursor line  h/<old>/<new> ────────────────────
            cmd if cmd.starts_with('h') => {
                match parse_replace(&cmd[1..]) {
                    Some((old, new)) => {
                        let ln = self.cursor.max(1);
                        if ln > self.lines.len() {
                            self.err("पहले किसी लाइन पर जाएं (<n> या g<n>)");
                        } else if self.lines[ln - 1].contains(&old) {
                            self.snapshot();
                            let replaced = self.lines[ln - 1].replacen(&old, &new, 1);
                            self.lines[ln - 1] = replaced;
                            self.modified = true;
                            self.ok(&format!("लाइन {} में '{}' → '{}'", ln, old, new));
                        } else {
                            self.err(&format!("'{}' लाइन {} में नहीं मिला", old, ln));
                        }
                    }
                    None => self.err("उपयोग: h/<पुराना>/<नया>  जैसे  h/foo/bar"),
                }
            }

            // ── Search ───────────────────────────────────────────────────
            cmd if cmd.starts_with('/') => {
                let query = cmd[1..].to_string();
                if query.is_empty() {
                    self.err("उपयोग: /<खोज>  जैसे  /नमस्ते");
                } else {
                    self.find_query = query;
                    self.find_from  = 0;
                    self.find_next();
                }
            }

            // ── Goto  g<n> ───────────────────────────────────────────────
            cmd if cmd.starts_with('g') => {
                match cmd[1..].trim().parse::<usize>() {
                    Ok(n) if n >= 1 => { self.jump(n); }
                    _ => self.err("g<n> — लाइन नंबर दें"),
                }
            }

            // ── Open  o <file> ───────────────────────────────────────────
            cmd if cmd.starts_with("o ") => {
                let new_path = cmd[2..].trim();
                if new_path.is_empty() {
                    self.err("उपयोग: o <filename>");
                } else {
                    if self.modified {
                        print!("{CLR}");
                        print!("  {YLW}सहेजे बिना बंद करें? (ह/न):{RST} ");
                        io::stdout().flush().unwrap();
                        let mut ans = String::new();
                        io::stdin().lock().read_line(&mut ans).ok();
                        let a = ans.trim();
                        if a != "ह" && !a.eq_ignore_ascii_case("h") { return true; }
                    }
                    let content = std::fs::read_to_string(new_path).unwrap_or_default();
                    self.lines = if content.is_empty() { Vec::new() }
                                 else { content.lines().map(|l| l.to_string()).collect() };
                    self.path       = new_path.to_string();
                    self.page       = 0;
                    self.cursor     = if self.lines.is_empty() { 0 } else { 1 };
                    self.modified   = false;
                    self.undo_stack.clear();
                    self.find_query.clear();
                    self.find_from  = 0;
                    self.ok(&format!("खोला: {} ({} lines)", new_path, self.lines.len()));
                }
            }

            other => self.err(&format!("अज्ञात आदेश: '{}'  (? = help)", other)),
        }
        true
    }

    fn find_next(&mut self) {
        let q     = self.find_query.clone();
        let total = self.lines.len();
        if total == 0 { self.err("फ़ाइल खाली"); return; }
        let start = self.find_from;
        let found = (start..total).chain(0..start)
            .find(|&i| self.lines[i].contains(&q));
        match found {
            Some(idx) => {
                self.page      = idx / PAGE;
                self.cursor    = idx + 1;
                self.find_from = (idx + 1) % total;
                self.ok(&format!("लाइन {} पर मिला: '{}'", idx + 1, q));
            }
            None => self.err(&format!("'{}' नहीं मिला", q)),
        }
    }

    fn total_pages(&self) -> usize {
        (self.lines.len().max(1) + PAGE - 1) / PAGE
    }

    fn jump(&mut self, n: usize) {
        if n >= 1 {
            self.page   = (n - 1) / PAGE;
            self.cursor = n.min(self.lines.len().max(1));
        }
    }

    fn ok(&mut self, msg: &str)  { self.message = msg.to_string(); self.msg_ok = true;  }
    fn err(&mut self, msg: &str) { self.message = msg.to_string(); self.msg_ok = false; }

    fn save(&mut self) {
        let content = self.lines.join("\n");
        match std::fs::write(&self.path, &content) {
            Ok(()) => {
                self.modified = false;
                self.ok(&format!("saved: {}", self.fname()));
            }
            Err(e) => self.err(&format!("save error: {e}")),
        }
    }

    fn run_file(&self) {
        match std::fs::read_to_string(&self.path) {
            Ok(src) => {
                let tokens = crate::lexer::tokenize(&src);
                match crate::parser::parse(tokens) {
                    Ok(stmts) => {
                        let prog = crate::compiler::Compiler::compile_program(&stmts);
                        let mut vm = crate::lvm::LVM::new();
                        if let Err(e) = vm.run(&prog) { eprintln!("LVM error: {e}"); }
                    }
                    Err(e) => eprintln!("parse error: {e}"),
                }
            }
            Err(e) => eprintln!("file error: {e}"),
        }
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars { return s.to_string(); }
    let head: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", head)
}

fn parse_n_rest(s: &str) -> Option<(usize, &str)> {
    let s   = s.trim_start();
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if end == 0 { return None; }
    let n: usize = s[..end].parse().ok()?;
    let rest = s[end..].trim_start();
    Some((n, rest))
}

fn parse_range(s: &str) -> Option<(usize, usize)> {
    let s   = s.trim();
    let mid = s.find('-')?;
    let n: usize = s[..mid].trim().parse().ok()?;
    let m: usize = s[mid + 1..].trim().parse().ok()?;
    Some((n, m))
}

fn parse_replace(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let mut chars = s.chars();
    let sep  = chars.next()?;
    let rest = &s[sep.len_utf8()..];
    let mid  = rest.find(sep)?;
    let old  = rest[..mid].to_string();
    let new  = rest[mid + sep.len_utf8()..].to_string();
    if old.is_empty() { return None; }
    Some((old, new))
}
