/// LIPI 2.0 Parser — SOV grammar, INDENT/DEDENT blocks, Karaka type annotations

use crate::lexer::{Token, TokenKind};
use crate::ast::*;
use crate::karaka::Karaka;

// ── String interpolation helper ────────────────────────────────────────────────
fn fmt_target_expr(inner: &str) -> Expr {
    let s = inner.trim();
    // Support obj.field inside {}
    if let Some(dot) = s.find('.') {
        let obj   = s[..dot].trim().to_string();
        let field = s[dot+1..].trim().to_string();
        Expr::Attr { obj: Box::new(Expr::Ident(obj)), field }
    } else {
        Expr::Ident(s.to_string())
    }
}

fn fmt_part_expr(inner: &str) -> Expr {
    Expr::Call { name: "वाक्य".to_string(), args: vec![fmt_target_expr(inner)] }
}

/// Named placeholder with a format spec: `{नाम:.2}` desugars to स्वरूप("{:.2}", नाम).
/// The spec must start with a character that can begin a स्वरूप spec
/// (digit, '.', '%', ',', '₹') so colon-bearing literal text (JSON, times
/// like "10:30") stays literal.
fn named_spec_expr(inner: &str) -> Option<Expr> {
    let (target, spec) = inner.split_once(':')?;
    let target = target.trim();
    let spec = spec.trim();
    if !is_interp_target(target) || spec.is_empty() {
        return None;
    }
    let first = spec.chars().next().unwrap();
    if !(first.is_ascii_digit() || first == '.' || first == '%' || first == ',' || first == '₹') {
        return None;
    }
    Some(Expr::Call {
        name: "स्वरूप".to_string(),
        args: vec![Expr::Str(format!("{{:{spec}}}")), fmt_target_expr(target)],
    })
}

/// A `{...}` interior is interpolated only if it's a plain identifier —
/// Devanagari/ASCII word chars, not starting with a digit. Anything else
/// (e.g. raw JSON text inside a string) is kept literal.
fn is_interp_ident(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with(|c: char| c.is_ascii_digit())
        && s.chars().all(|c| {
            ('\u{0900}'..='\u{097F}').contains(&c) || c.is_ascii_alphanumeric() || c == '_'
        })
}

fn is_interp_target(s: &str) -> bool {
    match s.split_once('.') {
        Some((obj, field)) => is_interp_ident(obj.trim()) && is_interp_ident(field.trim()),
        None => is_interp_ident(s),
    }
}

fn parse_fmt_string(s: &str) -> Expr {
    let mut parts: Vec<Expr> = Vec::new();
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        if open > 0 { parts.push(Expr::Str(rest[..open].to_string())); }
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            let inner = &after[..close];
            if is_interp_target(inner.trim()) {
                parts.push(fmt_part_expr(inner));
            } else if let Some(e) = named_spec_expr(inner.trim()) {
                // {नाम:.2} — named placeholder with format spec
                parts.push(e);
            } else {
                // {} / {:spec} placeholders for स्वरूप(), or non-identifier
                // text (e.g. JSON) — keep as literal
                parts.push(Expr::Str(format!("{{{}}}", inner)));
            }
            rest = &after[close + 1..];
        } else {
            parts.push(Expr::Str(rest[open..].to_string()));
            rest = "";
            break;
        }
    }
    if !rest.is_empty() { parts.push(Expr::Str(rest.to_string())); }
    if parts.is_empty() { return Expr::Str(String::new()); }
    let mut expr = parts.remove(0);
    for part in parts {
        expr = Expr::Binary { left: Box::new(expr), op: BinOp::Add, right: Box::new(part) };
    }
    expr
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

/// Build a Call (no keyword args) or CallKw (Phase 17) expression.
fn make_call(name: String, args: Vec<Expr>, kwargs: Vec<(String, Expr)>) -> Expr {
    if kwargs.is_empty() {
        Expr::Call { name, args }
    } else {
        Expr::CallKw { name, args, kwargs }
    }
}

fn make_method_call(object: Expr, method: String, args: Vec<Expr>, kwargs: Vec<(String, Expr)>) -> Expr {
    if kwargs.is_empty() {
        Expr::MethodCall { object: Box::new(object), method, args }
    } else {
        Expr::MethodCallKw { object: Box::new(object), method, args, kwargs }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 };
    p.program()
}

pub fn parse_recover(tokens: Vec<Token>) -> (Program, Vec<String>) {
    let mut p = Parser { tokens, pos: 0 };
    let mut stmts = Vec::new();
    let mut errors = Vec::new();
    p.skip_newlines();
    while !p.at_eof() {
        match p.statement() {
            Ok(s) => stmts.push(s),
            Err(e) => {
                errors.push(e);
                let mut depth: i32 = 0;
                while !p.at_eof() {
                    match p.peek().kind {
                        TokenKind::Newline | TokenKind::Dedent if depth <= 0 => break,
                        TokenKind::Indent | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                        TokenKind::Dedent | TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
                        _ => {}
                    }
                    p.advance();
                }
            }
        }
        p.skip_newlines();
    }
    (stmts, errors)
}

impl Parser {
    fn program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_eof() {
            stmts.push(self.statement()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        let line = self.peek().line;
        let stmt = self.statement_inner()?;
        Ok(Stmt::Located { line, inner: Box::new(stmt) })
    }

    fn statement_inner(&mut self) -> Result<Stmt, String> {
        match self.peek().kind.clone() {
            TokenKind::Batao   => self.stmt_print(),
            TokenKind::Yadi    => self.stmt_yadi(),
            TokenKind::Vidhi   => self.stmt_vidhi(),
            TokenKind::Fal     => self.stmt_fal(),
            TokenKind::Utpann  => { self.advance(); Ok(Stmt::Yield(self.expression()?)) }
            TokenKind::Aayat   => self.stmt_aayat(),
            TokenKind::Varg    => self.stmt_varg(),
            TokenKind::Sar     => self.stmt_varg(),
            TokenKind::Sajha   => self.stmt_static_vidhi(),
            TokenKind::JabTak  => self.stmt_jab_tak(),
            TokenKind::BandKaro => { self.advance(); Ok(Stmt::BandKaro) }
            TokenKind::Agla    => { self.advance(); Ok(Stmt::Agla) }
            TokenKind::Likho   => { self.advance(); Ok(Stmt::Likho(self.expression()?)) }
            TokenKind::Koshish   => self.stmt_koshish(),
            TokenKind::Phenko    => { self.advance(); Ok(Stmt::Phenko(self.expression()?)) }
            TokenKind::Vaishvik  => self.stmt_vaishvik(),
            TokenKind::Vikalp    => self.stmt_vikalp(),
            TokenKind::Milao     => self.stmt_milao(),
            TokenKind::Jancho    => self.stmt_jancho(),
            TokenKind::Sthir     => self.stmt_sthir(),
            TokenKind::Shuddha   => self.stmt_shuddha_vidhi(),
            TokenKind::At        => self.stmt_decorated(),
            TokenKind::Parikshan => self.stmt_parikshan(),
            TokenKind::Saath     => self.stmt_saath(),
            TokenKind::Abhilekh  => self.stmt_abhilekh(),
            TokenKind::Number(_) if matches!(self.peek_at(1), TokenKind::BarKaro) =>
                self.stmt_bar_karo(),
            TokenKind::Ident(_) => self.stmt_ident_lead(),
            _ => Ok(Stmt::ExprStmt(self.expression()?)),
        }
    }

    // ===== STATEMENT PARSERS =====

    /// बताओ <expr>
    fn stmt_print(&mut self) -> Result<Stmt, String> {
        self.advance();
        Ok(Stmt::Print(self.expression()?))
    }

    /// <name> [karaka] है <expr>
    /// <name>(<args>)
    /// <name> के लिए <iter> में: <block>
    /// <name>[idx] है <expr>
    fn stmt_ident_lead(&mut self) -> Result<Stmt, String> {
        let name = self.expect_ident()?;

        // Check for Karaka annotation before है
        // Syntax: नाम कर्ता है "राम"
        let karaka = self.parse_optional_karaka();

        // Phase 18 #7: optional `: प्रकार` type annotation — `क: संख्या है 5`
        let type_hint = self.parse_optional_type()?;

        match self.peek().kind.clone() {
            TokenKind::Hai => {
                self.advance();
                // Chained assignment: अ है ब है 0 — `Ident है` lookahead.
                if matches!(self.peek().kind, TokenKind::Ident(_))
                    && matches!(self.peek_at(1), TokenKind::Hai)
                {
                    let mut names = vec![name];
                    while matches!(self.peek().kind, TokenKind::Ident(_))
                        && matches!(self.peek_at(1), TokenKind::Hai)
                    {
                        names.push(self.expect_ident()?);
                        self.advance(); // consume है
                    }
                    return Ok(Stmt::ChainAssign { names, value: self.expression()? });
                }
                Ok(Stmt::Assign { name, karaka, type_hint, value: self.expression()? })
            }
            TokenKind::LParen => {
                self.advance();
                let (args, kwargs) = self.arg_list_kw()?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stmt::ExprStmt(make_call(name, args, kwargs)))
            }
            TokenKind::Comma => {
                // अ, ब है 1, 2  — tuple unpacking (Phase 17)
                let mut names = vec![name];
                while self.check_kind(TokenKind::Comma) {
                    self.advance();
                    names.push(self.expect_ident()?);
                }
                let line = self.peek().line;
                self.expect_kind(TokenKind::Hai)?;
                let mut values = vec![self.expression()?];
                while self.check_kind(TokenKind::Comma) {
                    self.advance();
                    values.push(self.expression()?);
                }
                if values.len() != 1 && values.len() != names.len() {
                    return Err(format!(
                        "बाएँ {} नाम हैं पर दाएँ {} मान (line {})",
                        names.len(), values.len(), line
                    ));
                }
                Ok(Stmt::MultiAssign { names, values })
            }
            TokenKind::KeeLiye => {
                self.advance();
                let iter = self.expression()?;
                self.expect_kind(TokenKind::Mein)?;
                self.expect_kind(TokenKind::Colon)?;
                self.skip_newlines();
                let body = self.block()?;
                Ok(Stmt::KeeLiye { var: name, iter, body })
            }
            TokenKind::LBracket => {
                // name[idx] है val   → IndexAssign
                // name[a:b:c] है val → SliceAssign
                // name[expr]         → bare index expression (statement)
                self.advance();
                // A bracket that starts with `:` is a slice with omitted start.
                let start = if self.check_kind(TokenKind::Colon) {
                    None
                } else {
                    Some(self.expression()?)
                };
                if self.check_kind(TokenKind::RBracket) {
                    // Plain index — no colon seen. `start` must be Some here.
                    let idx = start.unwrap();
                    self.advance();
                    if self.check_kind(TokenKind::Hai) {
                        self.advance();
                        let val = self.expression()?;
                        return Ok(Stmt::IndexAssign { obj: name, idx, val });
                    }
                    return Ok(Stmt::ExprStmt(Expr::Index {
                        obj: Box::new(Expr::Ident(name)),
                        idx: Box::new(idx),
                    }));
                }
                // Slice path — consume the first `:`, then parse end / step.
                self.expect_kind(TokenKind::Colon)?;
                let end = if self.check_kind(TokenKind::Colon) || self.check_kind(TokenKind::RBracket) {
                    None
                } else {
                    Some(self.expression()?)
                };
                let step = if self.check_kind(TokenKind::Colon) {
                    self.advance();
                    if self.check_kind(TokenKind::RBracket) { None } else { Some(self.expression()?) }
                } else {
                    None
                };
                self.expect_kind(TokenKind::RBracket)?;
                if self.check_kind(TokenKind::Hai) {
                    self.advance();
                    let val = self.expression()?;
                    return Ok(Stmt::SliceAssign { obj: name, start, end, step, val });
                }
                Ok(Stmt::ExprStmt(Expr::Slice {
                    obj: Box::new(Expr::Ident(name)),
                    start: start.map(Box::new),
                    end: end.map(Box::new),
                    step: step.map(Box::new),
                }))
            }
            TokenKind::Dot => {
                // name.field है val  — attribute assignment
                // name.method(args)  — method call as statement
                self.advance();
                let field = self.expect_field_name()?;
                if self.check_kind(TokenKind::Hai) {
                    self.advance();
                    let val = self.expression()?;
                    Ok(Stmt::AttrAssign { obj: name, field, val })
                } else if self.check_kind(TokenKind::LParen) {
                    self.advance();
                    let (args, kwargs) = self.arg_list_kw()?;
                    self.expect_kind(TokenKind::RParen)?;
                    Ok(Stmt::ExprStmt(make_method_call(Expr::Ident(name), field, args, kwargs)))
                } else {
                    Ok(Stmt::ExprStmt(Expr::Attr {
                        obj: Box::new(Expr::Ident(name)),
                        field,
                    }))
                }
            }
            _ => Ok(Stmt::ExprStmt(Expr::Ident(name))),
        }
    }

    /// यदि <cond>: <block> [अन्यथा: <block>]
    fn stmt_yadi(&mut self) -> Result<Stmt, String> {
        self.advance();
        let condition = self.expression()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let then = self.block()?;
        let otherwise = if self.check_kind(TokenKind::Anyatha) {
            self.advance();
            if self.check_kind(TokenKind::Yadi) {
                // अन्यथा यदि — else-if chain (no colon after अन्यथा)
                Some(vec![self.stmt_yadi()?])
            } else {
                self.expect_kind(TokenKind::Colon)?;
                self.skip_newlines();
                Some(self.block()?)
            }
        } else {
            None
        };
        Ok(Stmt::Yadi { condition, then, otherwise })
    }

    /// <number> बार करो: <block>
    fn stmt_bar_karo(&mut self) -> Result<Stmt, String> {
        let count = self.expression()?;
        self.expect_kind(TokenKind::BarKaro)?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::BarKaro { count, body })
    }

    /// विधि <name>(<params>): <block>
    fn stmt_vidhi(&mut self) -> Result<Stmt, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let (params, vararg) = self.param_list()?;
        self.expect_kind(TokenKind::RParen)?;
        let ret_type = self.parse_optional_return_type()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::Vidhi { name, params, body, vararg, pure: false, decorators: vec![], is_static: false, ret_type })
    }

    /// साझा विधि name(params): body  — static (shared) class method (Phase 17).
    /// No implicit यह; called as ClassName.method(args).
    fn stmt_static_vidhi(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume साझा
        self.expect_kind(TokenKind::Vidhi)?;
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let (params, vararg) = self.param_list()?;
        self.expect_kind(TokenKind::RParen)?;
        let ret_type = self.parse_optional_return_type()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::Vidhi { name, params, body, vararg, pure: false, decorators: vec![], is_static: true, ret_type })
    }

    /// @सजावट / @कारखाना(आर्ग) lines, then a विधि (or शुद्ध विधि) definition.
    /// Outermost decorator first: @अ @ब विधि f → f है अ(ब(f))
    fn stmt_decorated(&mut self) -> Result<Stmt, String> {
        let mut decorators = Vec::new();
        while self.check_kind(TokenKind::At) {
            let line = self.peek().line;
            self.advance(); // consume @
            let dname = self.expect_ident()?;
            let deco = if self.check_kind(TokenKind::LParen) {
                self.advance();
                let (args, kwargs) = self.arg_list_kw()?;
                self.expect_kind(TokenKind::RParen)?;
                make_call(dname, args, kwargs)
            } else {
                Expr::Ident(dname)
            };
            if !matches!(self.peek().kind, TokenKind::Newline | TokenKind::Eof) {
                return Err(format!("सजावट '@' के बाद नई पंक्ति अपेक्षित (line {})", line));
            }
            decorators.push(deco);
            self.skip_newlines();
        }
        let mut stmt = match self.peek().kind {
            TokenKind::Vidhi => self.stmt_vidhi()?,
            TokenKind::Shuddha => self.stmt_shuddha_vidhi()?,
            _ => return Err(format!(
                "सजावट '@' के बाद 'विधि' अपेक्षित (line {})", self.peek().line
            )),
        };
        if let Stmt::Vidhi { decorators: d, .. } = &mut stmt {
            *d = decorators;
        }
        Ok(stmt)
    }

    /// परीक्षण "नाम": <block>  — test definition (Phase 17)
    fn stmt_parikshan(&mut self) -> Result<Stmt, String> {
        let line = self.peek().line;
        self.advance(); // consume परीक्षण
        let name = match self.peek().kind.clone() {
            TokenKind::Str(s) => { self.advance(); s }
            _ => return Err(format!("परीक्षण के बाद \"नाम\" (वाक्य) अपेक्षित (line {})", line)),
        };
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::Parikshan { name, body })
    }

    /// अभिलेख Name(field1, field2, ...)  — record / dataclass (Phase 17).
    /// Desugars to a class with an auto-generated बनाओ that stores each field.
    fn stmt_abhilekh(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume अभिलेख
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let mut fields = Vec::new();
        if !self.check_kind(TokenKind::RParen) {
            loop {
                fields.push(self.expect_ident()?);
                if !self.check_kind(TokenKind::Comma) { break; }
                self.advance();
            }
        }
        self.expect_kind(TokenKind::RParen)?;

        let params: Vec<Param> = fields.iter()
            .map(|f| Param { name: f.clone(), karaka: None, default: None, type_hint: None })
            .collect();
        let body: Vec<Stmt> = fields.iter()
            .map(|f| Stmt::AttrAssign {
                obj: "यह".to_string(),
                field: f.clone(),
                val: Expr::Ident(f.clone()),
            })
            .collect();
        let ctor = Stmt::Vidhi {
            name: "बनाओ".to_string(),
            params,
            body,
            vararg: None,
            pure: false,
            decorators: vec![],
            is_static: false,
            ret_type: None,
        };
        Ok(Stmt::Varg { name, parent: None, methods: vec![ctor], is_abstract: false })
    }

    /// साथ <expr> के_रूप_में <नाम>: <block>  — context manager (Phase 17)
    fn stmt_saath(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume साथ
        let expr = self.expression()?;
        self.expect_kind(TokenKind::KeRupMein)?;
        let var = self.expect_ident()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::Saath { expr, var, body })
    }

    /// फल <expr>
    fn stmt_fal(&mut self) -> Result<Stmt, String> {
        self.advance();
        Ok(Stmt::Fal(self.expression()?))
    }

    /// जब तक <cond>: <block>
    fn stmt_jab_tak(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume जब तक
        let condition = self.expression()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::JabTak { condition, body })
    }

    /// [सार] वर्ग Name[(Parent)]: INDENT [विधि ...] DEDENT
    fn stmt_varg(&mut self) -> Result<Stmt, String> {
        let is_abstract = if self.check_kind(TokenKind::Sar) {
            self.advance(); // consume सार
            true
        } else { false };
        self.expect_kind(TokenKind::Varg)?; // consume वर्ग
        let name = self.expect_ident()?;
        let parent = if self.check_kind(TokenKind::LParen) {
            self.advance();
            let p = self.expect_ident()?;
            self.expect_kind(TokenKind::RParen)?;
            Some(p)
        } else { None };
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let methods = self.block()?;
        Ok(Stmt::Varg { name, parent, methods, is_abstract })
    }

    /// कोशिश: body + one or more पकड़ो clauses (Phase 17A typed exceptions).
    ///   पकड़ो त्रुटि:        catch-all, binds त्रुटि   (back-compat)
    ///   पकड़ो त्रुटि ई:      catch-all, binds ई
    ///   पकड़ो ClassName:     typed, binds त्रुटि
    ///   पकड़ो ClassName ई:   typed, binds ई
    fn stmt_koshish(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume कोशिश
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        self.skip_newlines();
        if !self.check_kind(TokenKind::Pakdo) {
            return Err(format!("कोशिश के बाद 'पकड़ो' अपेक्षित (line {})", self.peek().line));
        }
        let mut clauses: Vec<CatchClause> = Vec::new();
        while self.check_kind(TokenKind::Pakdo) {
            self.advance(); // consume पकड़ो
            let first = self.expect_ident()?;
            // Single ident → catch-all binding that name (back-compat: पकड़ो गलती:).
            // The compiler upgrades it to a typed clause if the name is a known
            // error class. Two idents → typed: पकड़ो ClassName var:
            let (class, var) = if self.check_kind(TokenKind::Colon) {
                (None, first)
            } else {
                let second = self.expect_ident()?;
                if first == "त्रुटि" { (None, second) }
                else { (Some(first), second) }
            };
            self.expect_kind(TokenKind::Colon)?;
            self.skip_newlines();
            let handler = self.block()?;
            clauses.push(CatchClause { class, var, body: handler });
            self.skip_newlines();
        }
        Ok(Stmt::TryCatch { body, clauses })
    }

    /// वैश्विक नाम1, नाम2  — declare names as global (Phase 13)
    fn stmt_vaishvik(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume वैश्विक
        let mut names = vec![self.expect_ident()?];
        while self.check_kind(TokenKind::Comma) {
            self.advance();
            names.push(self.expect_ident()?);
        }
        Ok(Stmt::Global(names))
    }

    /// जाँचो expr [, "message"]  — assert (Phase 16, Nyaya Pratijna)
    fn stmt_jancho(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume जाँचो
        let expr = self.expression()?;
        let message = if self.check_kind(TokenKind::Comma) {
            self.advance();
            Some(self.expression()?)
        } else {
            None
        };
        Ok(Stmt::Jancho { expr, message })
    }

    /// स्थिर name है expr  — immutable constant (Phase 16, Samkhya)
    fn stmt_sthir(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume स्थिर
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::Hai)?;
        let value = self.expression()?;
        Ok(Stmt::SthirDecl { name, value })
    }

    /// शुद्ध विधि name(...): body  — pure function (Phase 16, Gita karma yoga)
    fn stmt_shuddha_vidhi(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume शुद्ध
        if !matches!(self.peek().kind, TokenKind::Vidhi) {
            return Err(format!("'शुद्ध' के बाद 'विधि' अपेक्षित (line {})", self.peek().line));
        }
        self.advance(); // consume विधि
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let (params, vararg) = self.param_list()?;
        self.expect_kind(TokenKind::RParen)?;
        let ret_type = self.parse_optional_return_type()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.block()?;
        Ok(Stmt::Vidhi { name, params, body, vararg, pure: true, decorators: vec![], is_static: false, ret_type })
    }

    /// विकल्प Name: INDENT variant_line* DEDENT
    /// variant_line: Name  OR  Name(field1, field2, ...)
    fn stmt_vikalp(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume विकल्प
        let name = self.expect_ident()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let mut variants = Vec::new();
        if self.check_kind(TokenKind::Indent) {
            self.advance();
            self.skip_newlines();
            while !self.at_eof() && !self.check_kind(TokenKind::Dedent) {
                let vname = self.expect_ident()?;
                let fields = if self.check_kind(TokenKind::LParen) {
                    self.advance();
                    let mut flds = Vec::new();
                    while !self.check_kind(TokenKind::RParen) && !self.at_eof() {
                        flds.push(self.expect_ident()?);
                        if self.check_kind(TokenKind::Comma) { self.advance(); }
                    }
                    self.expect_kind(TokenKind::RParen)?;
                    flds
                } else {
                    Vec::new()
                };
                variants.push(crate::ast::ViKalpVariant { name: vname, fields });
                self.skip_newlines();
            }
            if self.check_kind(TokenKind::Dedent) { self.advance(); }
        }
        Ok(Stmt::ViKalp { name, variants })
    }

    /// मिलाओ <expr>: INDENT arm* DEDENT
    /// arm: VariantName[(bound, ...)]:  body  OR  अन्यथा: body
    fn stmt_milao(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume मिलाओ
        let subject = self.expression()?;
        self.expect_kind(TokenKind::Colon)?;
        self.skip_newlines();
        let mut arms = Vec::new();
        if self.check_kind(TokenKind::Indent) {
            self.advance();
            self.skip_newlines();
            while !self.at_eof() && !self.check_kind(TokenKind::Dedent) {
                let pattern = if self.check_kind(TokenKind::Anyatha) {
                    self.advance();
                    crate::ast::MilaoPattern::Wildcard
                } else {
                    let vname = self.expect_ident()?;
                    let binds = if self.check_kind(TokenKind::LParen) {
                        self.advance();
                        let mut bs = Vec::new();
                        while !self.check_kind(TokenKind::RParen) && !self.at_eof() {
                            bs.push(self.expect_ident()?);
                            if self.check_kind(TokenKind::Comma) { self.advance(); }
                        }
                        self.expect_kind(TokenKind::RParen)?;
                        bs
                    } else {
                        Vec::new()
                    };
                    crate::ast::MilaoPattern::Variant(vname, binds)
                };
                // Optional guard: VariantName(binds) यदि <cond>:  (Phase 17)
                let guard = if self.check_kind(TokenKind::Yadi) {
                    self.advance();
                    Some(self.expression()?)
                } else {
                    None
                };
                self.expect_kind(TokenKind::Colon)?;
                self.skip_newlines();
                let body = self.block()?;
                arms.push(crate::ast::MilaoArm { pattern, guard, body });
                self.skip_newlines();
            }
            if self.check_kind(TokenKind::Dedent) { self.advance(); }
        }
        Ok(Stmt::Milao { subject, arms })
    }

    /// आयात भारत.पहचान  OR  आयात "file.swami"
    fn stmt_aayat(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume आयात
        // File import: आयात "path.swami"
        if let TokenKind::Str(path) = self.peek().kind.clone() {
            self.advance();
            if matches!(self.peek().kind, TokenKind::KeRupMein) {
                self.advance();
                let alias = self.expect_ident()?;
                return Ok(Stmt::AayatFileAs { path, alias });
            }
            return Ok(Stmt::AayatFile(path));
        }
        let prefix = self.expect_ident()?;
        self.expect_kind(TokenKind::Dot)?;
        let module = self.expect_ident()?;
        Ok(Stmt::Aayat(format!("{}.{}", prefix, module)))
    }

    // ===== BLOCK PARSING =====

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        if self.check_kind(TokenKind::Indent) {
            self.advance();
            let mut stmts = Vec::new();
            self.skip_newlines();
            while !self.at_eof() && !self.check_kind(TokenKind::Dedent) {
                stmts.push(self.statement()?);
                self.skip_newlines();
            }
            if self.check_kind(TokenKind::Dedent) { self.advance(); }
            Ok(stmts)
        } else {
            Ok(vec![self.statement()?])
        }
    }

    // ===== EXPRESSION PARSERS =====

    fn expression(&mut self) -> Result<Expr, String> {
        self.logical_or()
    }

    // Logical OR — lowest precedence (या)
    fn logical_or(&mut self) -> Result<Expr, String> {
        let mut left = self.logical_and()?;
        while self.check_kind(TokenKind::Ya) {
            self.advance();
            let right = self.logical_and()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    // Logical AND (और)
    fn logical_and(&mut self) -> Result<Expr, String> {
        let mut left = self.bitwise_or()?;
        while self.check_kind(TokenKind::Aur) {
            self.advance();
            let right = self.bitwise_or()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::And, right: Box::new(right) };
        }
        Ok(left)
    }

    // Bitwise OR has lowest precedence among bitwise ops
    fn bitwise_or(&mut self) -> Result<Expr, String> {
        let mut left = self.bitwise_xor()?;
        while self.check_kind(TokenKind::BitOr) {
            self.advance();
            let right = self.bitwise_xor()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::BitOr, right: Box::new(right) };
        }
        Ok(left)
    }

    fn bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut left = self.bitwise_and()?;
        while self.check_kind(TokenKind::BitXor) {
            self.advance();
            let right = self.bitwise_and()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::BitXor, right: Box::new(right) };
        }
        Ok(left)
    }

    fn bitwise_and(&mut self) -> Result<Expr, String> {
        let mut left = self.comparison()?;
        while self.check_kind(TokenKind::BitAnd) {
            self.advance();
            let right = self.comparison()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::BitAnd, right: Box::new(right) };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.shift()?;

        // item में_है container / item नहीं_है container — membership (Phase 17)
        if matches!(self.peek().kind, TokenKind::MeinHai | TokenKind::NahinHai) {
            let negated = matches!(self.peek().kind, TokenKind::NahinHai);
            self.advance();
            let container = self.shift()?;
            return Ok(Expr::Membership {
                item: Box::new(left),
                container: Box::new(container),
                negated,
            });
        }

        // Chained comparisons (Phase 17): a < b < c  →  (a < b) और (b < c).
        // The middle expression is compiled twice — side-effecting middle
        // expressions (function calls) run twice in a chain.
        let mut acc: Option<Expr> = None;
        loop {
            let op = match self.peek().kind {
                TokenKind::SeAdhik => CmpOp::SeAdhik,
                TokenKind::SeKam   => CmpOp::SeKam,
                TokenKind::Barabar => CmpOp::Eq,
                TokenKind::EqEq    => CmpOp::Eq,
                TokenKind::NotEq   => CmpOp::NotEq,
                TokenKind::Lt      => CmpOp::Lt,
                TokenKind::Gt      => CmpOp::Gt,
                TokenKind::LtEq    => CmpOp::LtEq,
                TokenKind::GtEq    => CmpOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.shift()?;
            let cmp = Expr::Compare {
                left: Box::new(left.clone()),
                op,
                right: Box::new(right.clone()),
            };
            acc = Some(match acc {
                None       => cmp,
                Some(prev) => Expr::Binary {
                    left: Box::new(prev),
                    op: BinOp::And,
                    right: Box::new(cmp),
                },
            });
            left = right;
        }
        Ok(acc.unwrap_or(left))
    }

    fn shift(&mut self) -> Result<Expr, String> {
        let mut left = self.additive()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::LShift => BinOp::LShift,
                TokenKind::RShift => BinOp::RShift,
                _ => break,
            };
            self.advance();
            let right = self.additive()?;
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    /// SOV condition: subject यदि condition
    fn condition_expr(&mut self) -> Result<Expr, String> {
        let left = self.additive()?;
        let op = match self.peek().kind {
            TokenKind::SeAdhik => { self.advance(); CmpOp::SeAdhik }
            TokenKind::SeKam   => { self.advance(); CmpOp::SeKam }
            TokenKind::Barabar => { self.advance(); CmpOp::Eq }
            TokenKind::EqEq    => { self.advance(); CmpOp::Eq }
            TokenKind::NotEq   => { self.advance(); CmpOp::NotEq }
            TokenKind::GtEq    => { self.advance(); CmpOp::GtEq }
            TokenKind::LtEq    => { self.advance(); CmpOp::LtEq }
            TokenKind::Lt      => { self.advance(); CmpOp::Lt }
            TokenKind::Gt      => { self.advance(); CmpOp::Gt }
            _ => return Ok(left),
        };
        let right = self.additive()?;
        Ok(Expr::Compare { left: Box::new(left), op, right: Box::new(right) })
    }

    fn additive(&mut self) -> Result<Expr, String> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Plus  => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.multiplicative()?;
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Star       => BinOp::Mul,
                TokenKind::Slash      => BinOp::Div,
                TokenKind::SlashSlash => BinOp::FloorDiv,
                TokenKind::Percent    => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.unary()?;
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.check_kind(TokenKind::Minus) {
            self.advance();
            return Ok(Expr::Binary {
                left: Box::new(Expr::Number(0.0)),
                op: BinOp::Sub,
                right: Box::new(self.primary()?),
            });
        }
        if self.check_kind(TokenKind::BitNot) {
            self.advance();
            return Ok(Expr::BitNot(Box::new(self.primary()?)));
        }
        if self.check_kind(TokenKind::Nahin) {
            self.advance();
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary_atom()?;
        // Postfix: chained  expr[idx]  and  expr.method()  and  expr.field
        loop {
            if self.check_kind(TokenKind::LBracket) {
                self.advance();
                expr = self.bracket_suffix(expr)?;
            } else if self.check_kind(TokenKind::Dot) {
                self.advance();
                let field = self.expect_field_name()?;
                if self.check_kind(TokenKind::LParen) {
                    self.advance();
                    let (args, kwargs) = self.arg_list_kw()?;
                    self.expect_kind(TokenKind::RParen)?;
                    expr = make_method_call(expr, field, args, kwargs);
                } else {
                    expr = Expr::Attr { obj: Box::new(expr), field };
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary_atom(&mut self) -> Result<Expr, String> {
        if matches!(self.peek().kind, TokenKind::Pratiksha) {
            self.advance();
            return Ok(Expr::Await(Box::new(self.unary()?)));
        }
        let tok = self.advance();
        match tok.kind {
            TokenKind::Number(n) => Ok(Expr::Number(n)),
            TokenKind::Str(s)    => {
                if s.contains('{') && s.contains('}') {
                    Ok(parse_fmt_string(&s))
                } else {
                    Ok(Expr::Str(s))
                }
            }
            TokenKind::Bool(b)   => Ok(Expr::Bool(b)),

            TokenKind::Ident(name) => {
                if self.check_kind(TokenKind::ColonEq) {
                    // नाम := expr — walrus (Phase 17). RHS is a full expression
                    // (lowest precedence, Python semantics) — parenthesize for
                    // `(न := f()) से अधिक 5`.
                    self.advance();
                    let value = self.expression()?;
                    Ok(Expr::Walrus { name, value: Box::new(value) })
                } else if self.check_kind(TokenKind::LParen) {
                    self.advance();
                    let (args, kwargs) = self.arg_list_kw()?;
                    self.expect_kind(TokenKind::RParen)?;
                    Ok(make_call(name, args, kwargs))
                } else if self.check_kind(TokenKind::Dot) {
                    self.advance();
                    let field = self.expect_field_name()?;
                    if self.check_kind(TokenKind::LParen) {
                        self.advance();
                        let (args, kwargs) = self.arg_list_kw()?;
                        self.expect_kind(TokenKind::RParen)?;
                        Ok(make_method_call(Expr::Ident(name), field, args, kwargs))
                    } else {
                        Ok(Expr::Attr { obj: Box::new(Expr::Ident(name)), field })
                    }
                } else {
                    Ok(Expr::Ident(name))
                }
            }

            // Grouped expression
            TokenKind::LParen => {
                let e = self.expression()?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(e)
            }

            // यदि cond तो a अन्यथा b  — ternary expression (Phase 12)
            TokenKind::Yadi => {
                let condition = self.expression()?;
                self.expect_kind(TokenKind::Toh)?;
                let then_val = self.expression()?;
                self.expect_kind(TokenKind::Anyatha)?;
                let else_val = self.expression()?;
                Ok(Expr::Ternary {
                    condition: Box::new(condition),
                    then_val:  Box::new(then_val),
                    else_val:  Box::new(else_val),
                })
            }

            // लाम्डा(params): expr  — anonymous function (Phase 10)
            TokenKind::Lambda => {
                self.expect_kind(TokenKind::LParen)?;
                let mut params = Vec::new();
                let mut vararg: Option<String> = None;
                if !self.check_kind(TokenKind::RParen) {
                    loop {
                        if self.check_kind(TokenKind::Star) {
                            self.advance();
                            vararg = Some(self.expect_ident()?);
                            break;
                        }
                        params.push(self.expect_ident()?);
                        if !self.check_kind(TokenKind::Comma) { break; }
                        self.advance();
                    }
                }
                self.expect_kind(TokenKind::RParen)?;
                self.expect_kind(TokenKind::Colon)?;
                let body = if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Eof) {
                    self.skip_newlines();
                    self.block()?
                } else {
                    vec![Stmt::Fal(self.expression()?)]
                };
                Ok(Expr::Lambda { params, vararg, body })
            }

            // सूची literal  [e1, e2, ...]  — elements may start with * (spread, Phase 17)
            TokenKind::LBracket => {
                let mut elems: Vec<(bool, Expr)> = Vec::new();
                let mut has_spread = false;
                if !self.check_kind(TokenKind::RBracket) {
                    loop {
                        let is_spread = if self.check_kind(TokenKind::Star) {
                            self.advance();
                            has_spread = true;
                            true
                        } else {
                            false
                        };
                        let e = self.expression()?;
                        // [expr के लिए var iter में ...] — comprehension (Phase 17)
                        if elems.is_empty() && !is_spread && self.check_kind(TokenKind::KeeLiye) {
                            return self.comprehension_tail(e);
                        }
                        elems.push((is_spread, e));
                        if !self.check_kind(TokenKind::Comma) { break; }
                        self.advance();
                    }
                }
                self.expect_kind(TokenKind::RBracket)?;
                if has_spread {
                    Ok(Expr::ListWithSpread(elems))
                } else {
                    Ok(Expr::List(elems.into_iter().map(|(_, e)| e).collect()))
                }
            }

            // कोश literal  {"key": val, ...}
            TokenKind::LBrace => {
                let mut pairs = Vec::new();
                if !self.check_kind(TokenKind::RBrace) {
                    loop {
                        let key = self.expression()?;
                        self.expect_kind(TokenKind::Colon)?;
                        let val = self.expression()?;
                        pairs.push((key, val));
                        if !self.check_kind(TokenKind::Comma) { break; }
                        self.advance();
                    }
                }
                self.expect_kind(TokenKind::RBrace)?;
                Ok(Expr::Dict(pairs))
            }

            other => Err(format!("अपेक्षित मान नहीं मिला (line {}), मिला: {:?}", tok.line, other)),
        }
    }

    /// Rest of a list comprehension, after `[expr` with के लिए peeked:
    /// one or more `के लिए <var> <iter> में` clauses, optional `यदि <cond>`, `]`.
    fn comprehension_tail(&mut self, expr: Expr) -> Result<Expr, String> {
        let mut clauses: Vec<(String, Expr)> = Vec::new();
        while self.check_kind(TokenKind::KeeLiye) {
            self.advance();
            let var = self.expect_ident()?;
            let iter = self.expression()?;
            self.expect_kind(TokenKind::Mein)?;
            clauses.push((var, iter));
        }
        let cond = if self.check_kind(TokenKind::Yadi) {
            self.advance();
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        self.expect_kind(TokenKind::RBracket)?;
        Ok(Expr::Comprehension { expr: Box::new(expr), clauses, cond })
    }

    // ===== KARAKA PARSING =====

    /// Try to consume a Karaka keyword at current position.
    /// Returns Some(Karaka) if a karaka token is found, None otherwise.
    fn parse_optional_karaka(&mut self) -> Option<Karaka> {
        match self.peek().kind {
            TokenKind::Karta     => { self.advance(); Some(Karaka::Karta) }
            TokenKind::Karma     => { self.advance(); Some(Karaka::Karma) }
            TokenKind::Karan     => { self.advance(); Some(Karaka::Karana) }
            TokenKind::Sampradan => { self.advance(); Some(Karaka::Sampradan) }
            TokenKind::Apadan    => { self.advance(); Some(Karaka::Apadan) }
            TokenKind::Adhikaran => { self.advance(); Some(Karaka::Adhikaran) }
            _ => None,
        }
    }

    // ===== PARAM & ARG LISTS =====

    /// Returns (regular_params, vararg_name)
    fn param_list(&mut self) -> Result<(Vec<Param>, Option<String>), String> {
        let mut params = Vec::new();
        let mut vararg = None;
        if self.check_kind(TokenKind::RParen) { return Ok((params, vararg)); }
        loop {
            if self.check_kind(TokenKind::Star) {
                // *नाम  — vararg: collect remaining args into a list
                self.advance();
                vararg = Some(self.expect_ident()?);
                break; // vararg must be last
            }
            let name = self.expect_ident()?;
            let karaka = self.parse_optional_karaka();
            // Phase 18 #7: optional `: प्रकार` type annotation — `अ: संख्या`
            let type_hint = self.parse_optional_type()?;
            // Phase 17: optional constant default — `ब=0`
            let default = if self.check_kind(TokenKind::Assign) {
                self.advance();
                Some(self.param_default()?)
            } else {
                if params.iter().any(|p: &Param| p.default.is_some()) {
                    return Err(format!(
                        "डिफ़ॉल्ट मान वाले पैरामीटर के बाद '{}' का भी डिफ़ॉल्ट मान होना चाहिए (line {})",
                        name, self.peek().line
                    ));
                }
                None
            };
            params.push(Param { name, karaka, default, type_hint });
            if !self.check_kind(TokenKind::Comma) { break; }
            self.advance();
        }
        Ok((params, vararg))
    }

    /// Phase 18 #7 — optional `: प्रकार` annotation. Consumes `: <typename>` and
    /// maps it to a `TypeHint`; returns `None` if there is no colon. Gradual, so
    /// the annotation is always optional.
    fn parse_optional_type(&mut self) -> Result<Option<TypeHint>, String> {
        if self.check_kind(TokenKind::Colon) {
            self.advance();
            let tname = self.expect_ident()?;
            Ok(Some(TypeHint::from_name(&tname)))
        } else {
            Ok(None)
        }
    }

    /// Phase 18 #7 — optional `-> प्रकार` return annotation after a `)`.
    fn parse_optional_return_type(&mut self) -> Result<Option<TypeHint>, String> {
        if self.check_kind(TokenKind::Arrow) {
            self.advance();
            let tname = self.expect_ident()?;
            Ok(Some(TypeHint::from_name(&tname)))
        } else {
            Ok(None)
        }
    }

    /// Constant default value for a parameter: number, string, bool, or negative number.
    fn param_default(&mut self) -> Result<Expr, String> {
        let line = self.peek().line;
        let neg = if self.check_kind(TokenKind::Minus) { self.advance(); true } else { false };
        match self.peek().kind.clone() {
            TokenKind::Number(n) => { self.advance(); Ok(Expr::Number(if neg { -n } else { n })) }
            TokenKind::Str(s) if !neg => { self.advance(); Ok(Expr::Str(s)) }
            TokenKind::Bool(b) if !neg => { self.advance(); Ok(Expr::Bool(b)) }
            _ => Err(format!(
                "डिफ़ॉल्ट मान केवल स्थिरांक हो सकता है — संख्या, वाक्य या सत्य/असत्य (line {})", line
            )),
        }
    }

    /// After consuming `[`, parse a plain index `[i]` or a slice
    /// `[start:end:step]` (all parts optional) — Phase 17.
    fn bracket_suffix(&mut self, obj: Expr) -> Result<Expr, String> {
        let start = if self.check_kind(TokenKind::Colon) {
            None
        } else {
            Some(self.expression()?)
        };
        if let Some(idx) = start {
            if self.check_kind(TokenKind::RBracket) {
                self.advance();
                return Ok(Expr::Index { obj: Box::new(obj), idx: Box::new(idx) });
            }
            self.expect_kind(TokenKind::Colon)?;
            return self.slice_rest(obj, Some(idx));
        }
        self.expect_kind(TokenKind::Colon)?;
        self.slice_rest(obj, None)
    }

    /// Parse the `end[:step]]` part of a slice — called just after the first `:`.
    fn slice_rest(&mut self, obj: Expr, start: Option<Expr>) -> Result<Expr, String> {
        let end = if self.check_kind(TokenKind::Colon) || self.check_kind(TokenKind::RBracket) {
            None
        } else {
            Some(self.expression()?)
        };
        let step = if self.check_kind(TokenKind::Colon) {
            self.advance();
            if self.check_kind(TokenKind::RBracket) { None } else { Some(self.expression()?) }
        } else {
            None
        };
        self.expect_kind(TokenKind::RBracket)?;
        Ok(Expr::Slice {
            obj: Box::new(obj),
            start: start.map(Box::new),
            end: end.map(Box::new),
            step: step.map(Box::new),
        })
    }

    /// Returns (positional_args, keyword_args) — Phase 17 keyword arguments.
    /// Keyword args (`नाम=मान`) must come after all positional args.
    fn arg_list_kw(&mut self) -> Result<(Vec<Expr>, Vec<(String, Expr)>), String> {
        let mut args = Vec::new();
        let mut kwargs: Vec<(String, Expr)> = Vec::new();
        if self.check_kind(TokenKind::RParen) { return Ok((args, kwargs)); }
        loop {
            let line = self.peek().line;
            let is_kw = matches!(self.peek().kind, TokenKind::Ident(_))
                && matches!(self.peek_at(1), TokenKind::Assign);
            if is_kw {
                let name = self.expect_ident()?;
                self.advance(); // consume '='
                if kwargs.iter().any(|(n, _)| n == &name) {
                    return Err(format!("कीवर्ड तर्क '{}' दो बार दिया गया (line {})", name, line));
                }
                kwargs.push((name, self.expression()?));
            } else {
                if !kwargs.is_empty() {
                    return Err(format!(
                        "कीवर्ड तर्क के बाद स्थान-आधारित तर्क नहीं आ सकता (line {})", line
                    ));
                }
                args.push(self.expression()?);
            }
            if !self.check_kind(TokenKind::Comma) { break; }
            self.advance();
        }
        Ok((args, kwargs))
    }

    // ===== HELPERS =====

    fn expect_ident(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(s) => Ok(s),
            other => Err(format!("नाम अपेक्षित (line {}), मिला: {:?}", tok.line, other)),
        }
    }

    /// Like expect_ident, but also accepts keywords as field/method names (after a dot).
    /// Needed because list method .मिलाओ() shares its name with the match keyword.
    fn expect_field_name(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(s) => Ok(s),
            // Keywords that double as method names
            TokenKind::Milao    => Ok("मिलाओ".into()),
            TokenKind::Batao    => Ok("बताओ".into()),
            TokenKind::Likho    => Ok("लिखो".into()),
            TokenKind::Aur      => Ok("और".into()),
            TokenKind::Ya       => Ok("या".into()),
            TokenKind::Agla     => Ok("अगला".into()),
            TokenKind::Vikalp   => Ok("विकल्प".into()),
            other => Err(format!("नाम अपेक्षित (line {}), मिला: {:?}", tok.line, other)),
        }
    }

    fn expect_kind(&mut self, expected: TokenKind) -> Result<(), String> {
        let tok = self.advance();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(format!(
                "अपेक्षित {:?} (line {}), मिला {:?}",
                expected, tok.line, tok.kind
            ))
        }
    }

    fn check_kind(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_at(&self, offset: usize) -> TokenKind {
        self.tokens[(self.pos + offset).min(self.tokens.len() - 1)].kind.clone()
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        tok
    }

    fn skip_newlines(&mut self) {
        while self.check_kind(TokenKind::Newline) { self.advance(); }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
}
