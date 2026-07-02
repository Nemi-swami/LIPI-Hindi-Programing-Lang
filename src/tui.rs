// LIPI TUI Editor v3 — Sublime Text-inspired terminal editor

use std::io::{self, Read, Write};

// ── ANSI ─────────────────────────────────────────────────────────────────────
const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";
const CLR:  &str = "\x1b[2J\x1b[H";
const RST:  &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM:  &str = "\x1b[2m";
const CYAN: &str = "\x1b[96m";
const GRN:  &str = "\x1b[92m";
const RED:  &str = "\x1b[91m";
const YLW:  &str = "\x1b[93m";
const CLRL: &str = "\x1b[2K";
const BG_BLU: &str = "\x1b[44m\x1b[97m";  // blue bg white text — title bar
const BG_GRN: &str = "\x1b[42m\x1b[30m";  // green bg — success
const BG_RED: &str = "\x1b[41m\x1b[97m";  // red bg — error

fn at(r: u16, c: u16) -> String { format!("\x1b[{};{}H", r, c) }
fn pad(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w { s.chars().take(w).collect() }
    else { format!("{s}{}", " ".repeat(w - n)) }
}

// ── Windows console API ───────────────────────────────────────────────────────

#[cfg(windows)]
#[repr(C)] struct Coord  { x: i16, y: i16 }
#[cfg(windows)]
#[repr(C)] struct SRect  { l: i16, t: i16, r: i16, b: i16 }
#[cfg(windows)]
#[repr(C)] struct SBInfo { sz: Coord, cur: Coord, attr: u16, win: SRect, max: Coord }

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(n: u32) -> isize;
    fn GetConsoleMode(h: isize, m: *mut u32) -> i32;
    fn SetConsoleMode(h: isize, m: u32) -> i32;
    fn GetConsoleScreenBufferInfo(h: isize, i: *mut SBInfo) -> i32;
}

#[derive(Clone, Copy)]
pub struct Modes {
    #[cfg(windows)] in_m:  u32,
    #[cfg(windows)] out_m: u32,
}

pub fn enter_raw() -> Modes {
    #[cfg(windows)]
    unsafe {
        let hi = GetStdHandle(0xFFFF_FFF6);
        let ho = GetStdHandle(0xFFFF_FFF5);
        let (mut im, mut om) = (0u32, 0u32);
        GetConsoleMode(hi, &mut im);
        GetConsoleMode(ho, &mut om);
        SetConsoleMode(hi, (im & !0x0007) | 0x0200);
        SetConsoleMode(ho, om | 0x0004);
        return Modes { in_m: im, out_m: om };
    }
    #[cfg(not(windows))]
    Modes {}
}

pub fn leave_raw(m: Modes) {
    #[cfg(windows)]
    unsafe {
        SetConsoleMode(GetStdHandle(0xFFFF_FFF6), m.in_m);
        SetConsoleMode(GetStdHandle(0xFFFF_FFF5), m.out_m);
    }
    #[cfg(not(windows))]
    let _ = m;
}

fn term_size() -> (u16, u16) {
    #[cfg(windows)]
    unsafe {
        let h = GetStdHandle(0xFFFF_FFF5);
        let mut i = SBInfo {
            sz: Coord{x:0,y:0}, cur: Coord{x:0,y:0}, attr: 0,
            win: SRect{l:0,t:0,r:0,b:0}, max: Coord{x:0,y:0},
        };
        GetConsoleScreenBufferInfo(h, &mut i);
        return ((i.win.b - i.win.t + 1).max(10) as u16,
                (i.win.r - i.win.l + 1).max(40) as u16);
    }
    #[cfg(not(windows))]
    (24, 80)
}

// ── Key reading ───────────────────────────────────────────────────────────────

enum Key {
    Char(char), Up, Down, Left, Right, Home, End, PgUp, PgDn,
    Bs, Del, Enter, Tab, Esc,
    CtrlA,                  // 0x01 — start of file
    CtrlD,                  // 0x04 — duplicate line
    CtrlE,                  // 0x05 — end of file
    CtrlF,                  // 0x06 — find
    CtrlG,                  // 0x07 — go to line
    CtrlK,                  // 0x0B — delete/cut line
    CtrlO,                  // 0x0F — open file
    CtrlQ,                  // 0x03|0x11 — quit
    CtrlR,                  // 0x12 — run
    CtrlS,                  // 0x13 — save
    CtrlV,                  // 0x16 — paste
    CtrlY,                  // 0x19 — redo
    CtrlZ,                  // 0x1A — undo
    CtrlSlash,              // 0x1F — comment toggle
    Other,
}

fn read_key() -> Key {
    let mut b = [0u8; 1];
    loop {
        if io::stdin().lock().read(&mut b).unwrap_or(0) == 0 { continue; }
        return match b[0] {
            0x01        => Key::CtrlA,
            0x03 | 0x11 => Key::CtrlQ,
            0x04        => Key::CtrlD,
            0x05        => Key::CtrlE,
            0x06        => Key::CtrlF,
            0x07        => Key::CtrlG,
            0x08 | 0x7F => Key::Bs,
            0x0B        => Key::CtrlK,
            0x0D | 0x0A => Key::Enter,
            0x09        => Key::Tab,
            0x0F        => Key::CtrlO,
            0x12        => Key::CtrlR,
            0x13        => Key::CtrlS,
            0x16        => Key::CtrlV,
            0x19        => Key::CtrlY,
            0x1A        => Key::CtrlZ,
            0x1B        => esc_seq(),
            0x1F        => Key::CtrlSlash,
            c if c >= 0x20 => utf8_char(c),
            _           => Key::Other,
        };
    }
}

fn esc_seq() -> Key {
    let mut b = [0u8; 1];
    if io::stdin().lock().read(&mut b).unwrap_or(0) == 0 { return Key::Esc; }
    if b[0] != b'[' { return Key::Esc; }
    if io::stdin().lock().read(&mut b).unwrap_or(0) == 0 { return Key::Esc; }
    match b[0] {
        b'A' => Key::Up,
        b'B' => Key::Down,
        b'C' => Key::Right,
        b'D' => Key::Left,
        b'H' => Key::Home,
        b'F' => Key::End,
        b'3' => { io::stdin().lock().read(&mut b).ok(); Key::Del }
        b'5' => { io::stdin().lock().read(&mut b).ok(); Key::PgUp }
        b'6' => { io::stdin().lock().read(&mut b).ok(); Key::PgDn }
        _    => Key::Other,
    }
}

fn utf8_char(first: u8) -> Key {
    if first < 0x80 { return Key::Char(first as char); }
    let extra = if first >= 0xF0 { 3 } else if first >= 0xE0 { 2 } else { 1 };
    let mut bytes = vec![first];
    for _ in 0..extra {
        let mut b = [0u8; 1];
        if io::stdin().lock().read(&mut b).unwrap_or(0) == 1 { bytes.push(b[0]); }
    }
    std::str::from_utf8(&bytes).ok()
        .and_then(|s| s.chars().next())
        .map(Key::Char)
        .unwrap_or(Key::Other)
}

// ── Syntax highlighting ───────────────────────────────────────────────────────

const KW_COLOR:  &str = "\x1b[38;5;75m";   // blue     — keywords
const STR_COLOR: &str = "\x1b[38;5;214m";  // orange   — strings
const NUM_COLOR: &str = "\x1b[38;5;120m";  // green    — numbers
const CMT_COLOR: &str = "\x1b[38;5;242m";  // grey     — comments
const HI_COLOR:  &str = "\x1b[30;43m";     // on-yellow — search match
const CUR_BG:    &str = "\x1b[48;5;236m";  // dark bg  — cursor line
const FN_COLOR:  &str = "\x1b[38;5;183m";  // lavender — function names
const OP_COLOR:  &str = "\x1b[38;5;203m";  // red      — operators

static KEYWORDS: &[&str] = &[
    // Output
    "बताओ","लिखो",
    // Control flow
    "यदि","अन्यथा","अन्य","जब","तक","बंद","करो","अगला",
    // Functions
    "विधि","फल","लाम्डा","शुद्ध",
    // Assignment / types
    "है","शून्य","सत्य","असत्य",
    // Loop
    "बार","के","लिए","में",
    // OOP
    "वर्ग","यह","बनाओ",
    // Import / error
    "आयात","कोशिश","पकड़ो","त्रुटि",
    // Phase 13
    "और","या","नहीं","वैश्विक",
    // Phase 15
    "विकल्प","मिलाओ",
    // Phase 16
    "जाँचो","स्थिर",
    // Comparison
    "से","अधिक","कम","बराबर",
    // Ternary
    "तो",
];

fn highlight(line: &str, width: usize, find: &str, is_cursor_line: bool) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len() * 4);
    let mut i = 0;
    let mut col = 0;

    if is_cursor_line { out.push_str(CUR_BG); }

    while i < chars.len() && col < width {
        // Comment
        if chars[i] == '#' || chars[i] == '।' {
            let rest: String = chars[i..].iter().take(width - col).collect();
            out.push_str(CMT_COLOR);
            out.push_str(&rest);
            out.push_str(RST);
            if is_cursor_line { out.push_str(CUR_BG); }
            break;
        }

        // String literal
        if chars[i] == '"' {
            out.push_str(STR_COLOR);
            out.push('"'); col += 1; i += 1;
            while i < chars.len() && col < width {
                let c = chars[i];
                out.push(c); col += 1; i += 1;
                if c == '"' { break; }
            }
            out.push_str(RST);
            if is_cursor_line { out.push_str(CUR_BG); }
            continue;
        }

        // Number
        if chars[i].is_ascii_digit() {
            out.push_str(NUM_COLOR);
            while i < chars.len() && col < width && (chars[i].is_ascii_digit() || chars[i] == '.') {
                out.push(chars[i]); col += 1; i += 1;
            }
            out.push_str(RST);
            if is_cursor_line { out.push_str(CUR_BG); }
            continue;
        }

        // Word — check keyword / function
        let is_word_start = chars[i] > '\u{0900}' || chars[i].is_alphabetic() || chars[i] == '_';
        if is_word_start {
            let start = i;
            while i < chars.len() && (chars[i] > '\u{0900}' || chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let wlen = word.chars().count();
            if col + wlen > width { break; }

            let is_kw = KEYWORDS.contains(&word.as_str());
            // Check if next non-space is '(' → function call
            let next_nonspace = chars[i..].iter().find(|&&c| c != ' ');
            let is_fn = !is_kw && next_nonspace == Some(&'(');

            let color = if is_kw { KW_COLOR } else if is_fn { FN_COLOR } else { "" };
            if !color.is_empty() { out.push_str(color); }

            if !find.is_empty() && word.contains(find) {
                let mut rest = word.as_str();
                while let Some(pos) = rest.find(find) {
                    out.push_str(&rest[..pos]);
                    out.push_str(HI_COLOR);
                    out.push_str(find);
                    if !color.is_empty() { out.push_str(color); }
                    else if is_cursor_line { out.push_str(CUR_BG); }
                    else { out.push_str(RST); }
                    rest = &rest[pos + find.len()..];
                }
                out.push_str(rest);
            } else {
                out.push_str(&word);
            }
            if !color.is_empty() { out.push_str(RST); if is_cursor_line { out.push_str(CUR_BG); } }
            col += wlen;
            continue;
        }

        // Operator coloring
        if "+-*/%&|^~<>=!".contains(chars[i]) {
            out.push_str(OP_COLOR);
            out.push(chars[i]); col += 1; i += 1;
            out.push_str(RST);
            if is_cursor_line { out.push_str(CUR_BG); }
            continue;
        }

        out.push(chars[i]); col += 1; i += 1;
    }

    if is_cursor_line { out.push_str(RST); }
    out
}

// ── Editor struct ─────────────────────────────────────────────────────────────

pub struct TuiEditor {
    path:       String,
    lines:      Vec<String>,
    row:        usize,
    col:        usize,
    scroll:     usize,
    modified:   bool,
    status:     String,
    st_ok:      bool,
    rows:       u16,
    cols:       u16,
    // Find
    find_query: String,
    find_mode:  bool,
    // Replace
    repl_mode:  u8,      // 0=off  1=typing find  2=typing replacement
    repl_query: String,
    repl_with:  String,
    // Go-to-line
    goto_mode:  bool,
    goto_buf:   String,
    // Undo / Redo
    undo_stack: Vec<(Vec<String>, usize, usize)>,
    redo_stack: Vec<(Vec<String>, usize, usize)>,
    // Internal clipboard (one or more lines)
    clipboard:  Vec<String>,
}

impl TuiEditor {
    pub fn open(path: &str) -> Self {
        let src = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = src.lines().map(|l| l.to_string()).collect();
        if lines.is_empty() { lines.push(String::new()); }
        let (rows, cols) = term_size();
        TuiEditor {
            path: path.to_string(), lines,
            row: 0, col: 0, scroll: 0,
            modified: false, status: String::new(), st_ok: true,
            rows, cols,
            find_query: String::new(), find_mode: false,
            repl_mode: 0, repl_query: String::new(), repl_with: String::new(),
            goto_mode: false, goto_buf: String::new(),
            undo_stack: Vec::new(), redo_stack: Vec::new(),
            clipboard: Vec::new(),
        }
    }

    fn edit_rows(&self) -> usize { self.rows.saturating_sub(4) as usize }

    // ── Render ────────────────────────────────────────────────────────────────

    fn render(&self) {
        let cols   = self.cols as usize;
        let edit_r = self.edit_rows();
        let ndig   = self.lines.len().to_string().len().max(3);
        let code_w = cols.saturating_sub(ndig + 4);

        let mut o = String::with_capacity(32768);
        o.push_str(HIDE);
        o.push_str(CLR);

        // ── Title bar ─────────────────────────────────────────────────────────
        let dot = if self.modified { format!(" {YLW}●{RST}{BG_BLU}") } else { String::new() };
        let undo_s = if !self.undo_stack.is_empty() {
            format!("  {DIM}undo:{}{RST}{BG_BLU}", self.undo_stack.len())
        } else { String::new() };
        let clip_s = if !self.clipboard.is_empty() { "  📋" } else { "" };
        let title = format!(" {BOLD}LIPI{RST}{BG_BLU}  {}{}{}  Ln {}  Col {}{}",
                            self.path, dot, undo_s, self.row + 1, self.col + 1, clip_s);
        o.push_str(&format!("{BG_BLU}{}{RST}\r\n", pad(&title, cols)));

        // ── Code lines ────────────────────────────────────────────────────────
        let fq = if self.find_mode || self.repl_mode > 0 { self.find_query.clone() }
                 else if !self.find_query.is_empty() { self.find_query.clone() }
                 else { String::new() };

        for i in 0..edit_r {
            o.push_str(CLRL);
            let li = self.scroll + i;
            if li < self.lines.len() {
                let line   = &self.lines[li];
                let is_cur = li == self.row;
                let h_off  = if is_cur && self.col > code_w.saturating_sub(4) {
                    self.col.saturating_sub(code_w / 2)
                } else { 0 };
                let sliced: String = line.chars().skip(h_off).collect();
                let nc = if is_cur { CYAN } else { DIM };
                let hl = highlight(&sliced, code_w, &fq, is_cur);
                o.push_str(&format!("{nc}{:>ndig$}{RST} │ {hl}", li + 1));
            } else {
                o.push_str(&format!("{DIM}{:>ndig$}  ~{RST}", ""));
            }
            o.push_str("\r\n");
        }

        // ── Divider ───────────────────────────────────────────────────────────
        o.push_str(&format!("{CYAN}{}{RST}\r\n", "─".repeat(cols)));

        // ── Status line ───────────────────────────────────────────────────────
        o.push_str(CLRL);
        if self.find_mode {
            o.push_str(&format!("{YLW} Find: {}{RST}  {DIM}Enter=Next  Tab=Replace  Esc=Cancel{RST}",
                                self.find_query));
        } else if self.repl_mode == 1 {
            o.push_str(&format!("{YLW} Replace › Find: {}{RST}  {DIM}Enter=Next  Esc=Cancel{RST}",
                                self.repl_query));
        } else if self.repl_mode == 2 {
            o.push_str(&format!("{YLW} Replace › With: {}{RST}  {DIM}Enter=Replace All  Esc=Cancel{RST}",
                                self.repl_with));
        } else if self.goto_mode {
            o.push_str(&format!("{YLW} Go to line: {}{RST}  {DIM}Enter=Jump  Esc=Cancel{RST}",
                                self.goto_buf));
        } else if !self.status.is_empty() {
            let c = if self.st_ok { GRN } else { RED };
            o.push_str(&format!("{c} {}{RST}", self.status));
        } else {
            o.push_str(&format!(
                "{DIM} Ctrl+S Save  Ctrl+R Run  Ctrl+F Find  Ctrl+Z Undo  Ctrl+D Dup  \
                 Ctrl+K Del  Ctrl+G Line  Ctrl+/ Cmt  Ctrl+Q Quit{RST}"));
        }
        o.push_str("\r\n");

        // ── Cursor reposition ─────────────────────────────────────────────────
        let h_off = if self.col > code_w.saturating_sub(4) {
            self.col.saturating_sub(code_w / 2)
        } else { 0 };
        let cr = (self.row.saturating_sub(self.scroll)) as u16 + 2;
        let cc = (self.col.saturating_sub(h_off)) as u16 + ndig as u16 + 4;
        o.push_str(&at(cr, cc));
        o.push_str(SHOW);

        print!("{o}");
        io::stdout().flush().unwrap();
    }

    // ── Scroll / clamp ────────────────────────────────────────────────────────

    fn scroll_view(&mut self) {
        let er = self.edit_rows();
        if self.row < self.scroll { self.scroll = self.row; }
        else if self.row >= self.scroll + er { self.scroll = self.row + 1 - er; }
    }

    fn clamp_col(&mut self) {
        let max = self.lines[self.row].chars().count();
        if self.col > max { self.col = max; }
    }

    fn byte_idx(&self, row: usize, col: usize) -> usize {
        self.lines[row].char_indices().nth(col)
            .map(|(i, _)| i).unwrap_or(self.lines[row].len())
    }

    // ── Undo / Redo ───────────────────────────────────────────────────────────

    fn save_undo(&mut self) {
        let state = (self.lines.clone(), self.row, self.col);
        self.undo_stack.push(state);
        if self.undo_stack.len() > 200 { self.undo_stack.remove(0); }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some((lines, row, col)) = self.undo_stack.pop() {
            self.redo_stack.push((self.lines.clone(), self.row, self.col));
            self.lines = lines;
            self.row   = row.min(self.lines.len().saturating_sub(1));
            self.col   = col.min(self.lines[self.row].chars().count());
            self.modified = true;
            self.status = format!("Undo ({} left)", self.undo_stack.len());
            self.st_ok  = true;
        } else {
            self.status = "Nothing to undo".into();
            self.st_ok  = false;
        }
    }

    fn redo(&mut self) {
        if let Some((lines, row, col)) = self.redo_stack.pop() {
            self.undo_stack.push((self.lines.clone(), self.row, self.col));
            self.lines = lines;
            self.row   = row.min(self.lines.len().saturating_sub(1));
            self.col   = col.min(self.lines[self.row].chars().count());
            self.modified = true;
            self.status = format!("Redo ({} left)", self.redo_stack.len());
            self.st_ok  = true;
        } else {
            self.status = "Nothing to redo".into();
            self.st_ok  = false;
        }
    }

    // ── Editing ops ───────────────────────────────────────────────────────────

    fn insert(&mut self, c: char) {
        let bi = self.byte_idx(self.row, self.col);
        self.lines[self.row].insert(bi, c);
        self.col += 1;
        self.modified = true;
    }

    fn backspace(&mut self) {
        if self.col > 0 {
            self.save_undo();
            let bi = self.byte_idx(self.row, self.col - 1);
            let ei = self.byte_idx(self.row, self.col);
            self.lines[self.row].drain(bi..ei);
            self.col -= 1;
            self.modified = true;
        } else if self.row > 0 {
            self.save_undo();
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.lines[self.row].chars().count();
            self.lines[self.row].push_str(&line);
            self.modified = true;
        }
    }

    fn delete_fwd(&mut self) {
        let len = self.lines[self.row].chars().count();
        if self.col < len {
            self.save_undo();
            let bi = self.byte_idx(self.row, self.col);
            let ei = self.byte_idx(self.row, self.col + 1);
            self.lines[self.row].drain(bi..ei);
            self.modified = true;
        } else if self.row + 1 < self.lines.len() {
            self.save_undo();
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
            self.modified = true;
        }
    }

    fn newline(&mut self) {
        self.save_undo();
        let bi   = self.byte_idx(self.row, self.col);
        let rest = self.lines[self.row][bi..].to_string();
        self.lines[self.row].truncate(bi);
        // Auto-indent: match current line's leading spaces; add 4 more after ':'
        let indent = self.lines[self.row].chars().take_while(|&c| c == ' ').count();
        let extra  = if self.lines[self.row].trim_end().ends_with(':') { 4 } else { 0 };
        self.row += 1;
        self.col  = indent + extra;
        self.lines.insert(self.row, format!("{}{}", " ".repeat(indent + extra), rest));
        self.modified = true;
    }

    fn duplicate_line(&mut self) {
        self.save_undo();
        let line = self.lines[self.row].clone();
        self.lines.insert(self.row + 1, line);
        self.row += 1;
        self.modified = true;
        self.status = "Line duplicated (Ctrl+Z to undo)".into();
        self.st_ok  = true;
    }

    fn delete_line(&mut self) {
        self.save_undo();
        self.clipboard = vec![self.lines[self.row].clone()];
        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.col = 0;
        } else {
            self.lines.remove(self.row);
            if self.row >= self.lines.len() { self.row = self.lines.len() - 1; }
            self.clamp_col();
        }
        self.modified = true;
        self.status = "Line cut → Ctrl+V to paste".into();
        self.st_ok  = true;
    }

    fn paste_clipboard(&mut self) {
        if self.clipboard.is_empty() { return; }
        self.save_undo();
        for (i, line) in self.clipboard.iter().enumerate() {
            let ins_at = self.row + 1 + i;
            self.lines.insert(ins_at, line.clone());
        }
        self.row += self.clipboard.len();
        self.col = 0;
        self.modified = true;
        self.status = format!("{} line(s) pasted", self.clipboard.len());
        self.st_ok  = true;
    }

    fn toggle_comment(&mut self) {
        self.save_undo();
        let line = self.lines[self.row].clone();
        let trimmed = line.trim_start();
        if trimmed.starts_with("# ") {
            // Remove comment marker
            let offset = line.len() - trimmed.len();
            self.lines[self.row] = format!("{}{}", &line[..offset], &trimmed[2..]);
        } else if trimmed.starts_with('#') {
            let offset = line.len() - trimmed.len();
            self.lines[self.row] = format!("{}{}", &line[..offset], &trimmed[1..]);
        } else {
            // Add comment
            let spaces: String = line.chars().take_while(|&c| c == ' ').collect();
            let rest = line.trim_start();
            self.lines[self.row] = format!("{}# {}", spaces, rest);
        }
        self.modified = true;
    }

    fn replace_all(&mut self) {
        if self.repl_query.is_empty() { return; }
        self.save_undo();
        let mut count = 0usize;
        for line in &mut self.lines {
            if line.contains(&self.repl_query) {
                *line = line.replace(&self.repl_query, &self.repl_with);
                count += 1;
            }
        }
        self.status  = format!("Replaced in {} line(s)", count);
        self.st_ok   = count > 0;
        self.repl_mode = 0;
        if count > 0 { self.modified = true; }
    }

    // ── Find ─────────────────────────────────────────────────────────────────

    fn find_next(&mut self) {
        if self.find_query.is_empty() { return; }
        let q = self.find_query.clone();
        let n = self.lines.len();
        for offset in 1..=n {
            let r = (self.row + offset) % n;
            if let Some(pos) = self.lines[r].find(&q) {
                self.row = r;
                self.col = self.lines[r][..pos].chars().count();
                self.scroll_view();
                self.status = format!("Found: \"{}\" (line {})", q, r + 1);
                self.st_ok  = true;
                return;
            }
        }
        self.status = format!("\"{}\" not found", q);
        self.st_ok  = false;
    }

    // ── Save ─────────────────────────────────────────────────────────────────

    fn save(&mut self) {
        match std::fs::write(&self.path, self.lines.join("\n")) {
            Ok(()) => {
                self.modified = false;
                self.status   = format!("✓ Saved: {}", self.path);
                self.st_ok    = true;
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
                self.st_ok  = false;
            }
        }
    }

    // ── Run ──────────────────────────────────────────────────────────────────

    fn run_program(&mut self) {
        self.save();
        print!("{CLR}");
        println!("{CYAN}{}{RST}", "─".repeat(64));
        println!("  {BOLD}► Running: {}{RST}", self.path);
        println!("{CYAN}{}{RST}\n", "─".repeat(64));
        io::stdout().flush().unwrap();

        let src = std::fs::read_to_string(&self.path).unwrap_or_default();
        let tokens = crate::lexer::tokenize(&src);
        match crate::parser::parse(tokens) {
            Ok(stmts) => {
                let prog = crate::compiler::Compiler::compile_program(&stmts);
                let mut vm = crate::lvm::LVM::new();
                if let Err(e) = vm.run(&prog) { eprintln!("\n  {RED}त्रुटि:{RST} {e}"); }
            }
            Err(e) => eprintln!("  {RED}व्याकरण त्रुटि:{RST} {e}"),
        }

        println!("\n{CYAN}{}{RST}", "─".repeat(64));
        print!("{DIM}  Press Enter to return to editor…{RST}");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).ok();
        self.status.clear();
        self.st_ok = true;
    }

    // ── Open file ─────────────────────────────────────────────────────────────

    fn open_file(&mut self) {
        print!("{CLR}");
        println!("{BG_BLU}  LIPI — Open File  {RST}\n");

        let mut files: Vec<String> = Vec::new();
        let search_dirs = ["D:\\Projects\\lipi-lang\\examples", "."];
        for dir in &search_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                let mut batch: Vec<String> = entries.flatten()
                    .filter_map(|e| {
                        let n = e.file_name().to_string_lossy().to_string();
                        if n.ends_with(".swami") || n.ends_with(".roman") || n.ends_with(".vani") {
                            Some(format!("{}\\{}", dir, n))
                        } else { None }
                    }).collect();
                batch.sort();
                for f in batch { if !files.contains(&f) { files.push(f); } }
            }
        }

        if files.is_empty() {
            println!("  {RED}No .swami/.roman/.vani files found{RST}");
            let mut b = String::new(); io::stdin().read_line(&mut b).ok();
            return;
        }

        for (i, f) in files.iter().enumerate() {
            println!("  {CYAN}[{}]{RST}  {}", i + 1, f);
        }
        println!("\n  {DIM}[N] New file{RST}");
        print!("  Number or filename: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_string();

        if input.eq_ignore_ascii_case("n") {
            print!("  Filename (.swami/.vani): ");
            io::stdout().flush().unwrap();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();
            if !name.is_empty() {
                let path = if name.contains('.') { name } else { format!("{}.swami", name) };
                self.load_path(&path);
            }
        } else if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= files.len() { self.load_path(&files[n - 1].clone()); }
        } else if !input.is_empty() {
            self.load_path(&input);
        }
    }

    fn load_path(&mut self, path: &str) {
        let src = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = src.lines().map(|l| l.to_string()).collect();
        if lines.is_empty() { lines.push(String::new()); }
        self.path     = path.to_string();
        self.lines    = lines;
        self.row      = 0; self.col = 0; self.scroll = 0;
        self.modified = false;
        self.undo_stack.clear(); self.redo_stack.clear();
        self.status   = format!("Opened: {}", path);
        self.st_ok    = true;
    }

    // ── Main loop ─────────────────────────────────────────────────────────────

    pub fn run(&mut self) {
        let mut modes = enter_raw();

        loop {
            let (r, c) = term_size();
            self.rows = r; self.cols = c;
            self.scroll_view();
            self.render();

            if self.status.starts_with('✓') { self.status.clear(); }

            let key = read_key();

            // ── Goto-line mode ────────────────────────────────────────────────
            if self.goto_mode {
                match key {
                    Key::Esc => {
                        self.goto_mode = false; self.goto_buf.clear(); self.status.clear();
                    }
                    Key::Enter => {
                        if let Ok(n) = self.goto_buf.trim().parse::<usize>() {
                            let target = n.saturating_sub(1).min(self.lines.len() - 1);
                            self.row = target; self.col = 0; self.scroll_view();
                            self.status = format!("Jumped to line {}", target + 1);
                            self.st_ok = true;
                        }
                        self.goto_mode = false; self.goto_buf.clear();
                    }
                    Key::Bs => { self.goto_buf.pop(); }
                    Key::Char(c) if c.is_ascii_digit() => { self.goto_buf.push(c); }
                    _ => {}
                }
                continue;
            }

            // ── Replace mode ──────────────────────────────────────────────────
            if self.repl_mode > 0 {
                match key {
                    Key::Esc => { self.repl_mode = 0; self.status.clear(); }
                    Key::Enter if self.repl_mode == 1 => { self.repl_mode = 2; self.repl_with.clear(); }
                    Key::Enter if self.repl_mode == 2 => { self.replace_all(); }
                    Key::Bs if self.repl_mode == 1 => { self.repl_query.pop(); }
                    Key::Bs if self.repl_mode == 2 => { self.repl_with.pop(); }
                    Key::Char(c) if self.repl_mode == 1 => { self.repl_query.push(c); }
                    Key::Char(c) if self.repl_mode == 2 => { self.repl_with.push(c); }
                    _ => {}
                }
                continue;
            }

            // ── Find mode ─────────────────────────────────────────────────────
            if self.find_mode {
                match key {
                    Key::Esc | Key::CtrlF => { self.find_mode = false; self.status.clear(); }
                    Key::Enter => { self.find_next(); }
                    Key::Tab => {
                        // Switch to Replace mode
                        self.find_mode  = false;
                        self.repl_mode  = 1;
                        self.repl_query = self.find_query.clone();
                    }
                    Key::Bs => { self.find_query.pop(); }
                    Key::Char(c) => { self.find_query.push(c); }
                    _ => {}
                }
                continue;
            }

            // ── Normal mode ───────────────────────────────────────────────────
            match key {

                // Quit
                Key::CtrlQ => {
                    if !self.modified { break; }
                    self.status = "Unsaved changes! Ctrl+Q again = quit without saving".into();
                    self.st_ok = false;
                    self.scroll_view(); self.render();
                    if let Key::CtrlQ = read_key() { break; }
                    self.status.clear(); self.st_ok = true;
                }

                // Save / Run / Open
                Key::CtrlS => self.save(),
                Key::CtrlR => { leave_raw(modes); self.run_program(); modes = enter_raw(); }
                Key::CtrlO => { leave_raw(modes); self.open_file();   modes = enter_raw(); }

                // Find
                Key::CtrlF => { self.find_mode = true; self.find_query.clear(); }

                // Go to line
                Key::CtrlG => { self.goto_mode = true; self.goto_buf.clear(); }

                // Undo / Redo
                Key::CtrlZ => self.undo(),
                Key::CtrlY => self.redo(),

                // Duplicate line
                Key::CtrlD => self.duplicate_line(),

                // Delete/cut line → Ctrl+K
                Key::CtrlK => self.delete_line(),

                // Paste
                Key::CtrlV => self.paste_clipboard(),

                // Comment toggle
                Key::CtrlSlash => self.toggle_comment(),

                // Start / end of file
                Key::CtrlA => { self.row = 0; self.col = 0; }
                Key::CtrlE => { self.row = self.lines.len() - 1; self.col = self.lines[self.row].chars().count(); }

                // Navigation
                Key::Up => {
                    if self.row > 0 { self.row -= 1; self.clamp_col(); }
                }
                Key::Down => {
                    if self.row + 1 < self.lines.len() { self.row += 1; self.clamp_col(); }
                }
                Key::Left => {
                    if self.col > 0 { self.col -= 1; }
                    else if self.row > 0 {
                        self.row -= 1;
                        self.col = self.lines[self.row].chars().count();
                    }
                }
                Key::Right => {
                    let len = self.lines[self.row].chars().count();
                    if self.col < len { self.col += 1; }
                    else if self.row + 1 < self.lines.len() { self.row += 1; self.col = 0; }
                }
                Key::Home => { self.col = 0; }
                Key::End  => { self.col = self.lines[self.row].chars().count(); }
                Key::PgUp => {
                    let n = self.edit_rows();
                    self.row = self.row.saturating_sub(n); self.clamp_col();
                }
                Key::PgDn => {
                    let n = self.edit_rows();
                    self.row = (self.row + n).min(self.lines.len() - 1); self.clamp_col();
                }

                // Editing
                Key::Enter    => self.newline(),
                Key::Bs       => self.backspace(),
                Key::Del      => self.delete_fwd(),
                Key::Tab      => { self.save_undo(); for _ in 0..4 { self.insert(' '); } }
                Key::Char(c)  => self.insert(c),

                Key::Esc | Key::Other => {}
            }
        }

        leave_raw(modes);
        print!("{CLR}{SHOW}");
        io::stdout().flush().unwrap();
    }
}
