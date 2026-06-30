//! Static type checker for LIPI — `lipi check` (Phase 18 #7).
//!
//! A gradual checker: it walks the parsed AST and flags only *concrete*,
//! high-confidence type mismatches. Unannotated values, `कुछ_भी` (Any), and
//! nominal class types (`Named`) are treated permissively and never flagged, so
//! existing untyped programs pass cleanly. The checker never executes the program
//! and is completely independent of the compiler and VM.
//!
//! What it catches:
//!   1. A non-number operand to `- * / // % & | ^ << >>` (e.g. `"x" - 5`).
//!   2. A call argument whose concrete type disagrees with the parameter's
//!      declared type.
//!   3. A `फल` whose concrete type disagrees with the function's `-> प्रकार`.
//!   4. An annotated variable initialised or reassigned with an incompatible
//!      concrete value.
//!
//! `+` is deliberately never flagged: LIPI's `+` coerces (string concatenation
//! with numbers is idiomatic), so a Str operand simply yields a Str result.

use std::collections::HashMap;

use crate::ast::{self, BinOp, Expr, Stmt, TypeHint};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub message: String,
}

struct FuncSig {
    params: Vec<TypeHint>,
    ret: TypeHint,
}

struct Checker {
    diags: Vec<Diagnostic>,
    /// Stack of lexical scopes mapping an *annotated* variable to its declared
    /// type. Unannotated variables are never recorded — they stay dynamic.
    scopes: Vec<HashMap<String, TypeHint>>,
    funcs: HashMap<String, FuncSig>,
    cur_line: u32,
    /// Expected return type of the function currently being checked, if annotated.
    cur_ret: Option<TypeHint>,
}

/// A type we are confident about and can flag mismatches on (i.e. a primitive,
/// not the gradual `Any`/`Named` escape hatches).
fn is_concrete(t: &TypeHint) -> bool {
    !matches!(t, TypeHint::Any | TypeHint::Named(_))
}

/// Static entry point: returns all diagnostics for a parsed program.
pub fn check(stmts: &[Stmt]) -> Vec<Diagnostic> {
    let mut c = Checker {
        diags: Vec::new(),
        scopes: vec![HashMap::new()],
        funcs: HashMap::new(),
        cur_line: 0,
        cur_ret: None,
    };
    c.collect_signatures(stmts);
    c.check_block(stmts);
    c.diags
}

impl Checker {
    /// Pre-pass: record every top-level function's signature so calls can be
    /// checked regardless of definition order.
    fn collect_signatures(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            if let Stmt::Vidhi { name, params, ret_type, .. } = ast::unwrap_located(stmt) {
                let ptypes = params
                    .iter()
                    .map(|p| p.type_hint.clone().unwrap_or(TypeHint::Any))
                    .collect();
                let ret = ret_type.clone().unwrap_or(TypeHint::Any);
                self.funcs.insert(name.clone(), FuncSig { params: ptypes, ret });
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare(&mut self, name: &str, t: TypeHint) {
        if let Some(s) = self.scopes.last_mut() {
            s.insert(name.to_string(), t);
        }
    }
    fn lookup(&self, name: &str) -> Option<TypeHint> {
        for s in self.scopes.iter().rev() {
            if let Some(t) = s.get(name) {
                return Some(t.clone());
            }
        }
        None
    }
    fn diag(&mut self, msg: String) {
        self.diags.push(Diagnostic { line: self.cur_line, message: msg });
    }

    fn check_block(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Located { line, inner } => {
                self.cur_line = *line as u32;
                self.check_stmt(inner);
            }

            Stmt::Assign { name, type_hint, value, .. } => {
                let vt = self.infer(value);
                match type_hint {
                    Some(declared) => {
                        if is_concrete(declared) && is_concrete(&vt) && !declared.accepts(&vt) {
                            self.diag(format!(
                                "चर '{}' का प्रकार {} घोषित है पर {} मान दिया गया",
                                name, declared.name(), vt.name()
                            ));
                        }
                        self.declare(name, declared.clone());
                    }
                    None => {
                        // Reassignment must respect an earlier annotation.
                        if let Some(declared) = self.lookup(name) {
                            if is_concrete(&declared) && is_concrete(&vt) && !declared.accepts(&vt) {
                                self.diag(format!(
                                    "चर '{}' ({}) को {} मान नहीं दिया जा सकता",
                                    name, declared.name(), vt.name()
                                ));
                            }
                        }
                    }
                }
            }

            Stmt::Vidhi { params, body, ret_type, .. } => {
                self.push_scope();
                for p in params {
                    if let Some(t) = &p.type_hint {
                        self.declare(&p.name, t.clone());
                    }
                }
                let saved_ret = self.cur_ret.take();
                self.cur_ret = ret_type.clone();
                self.check_block(body);
                self.cur_ret = saved_ret;
                self.pop_scope();
            }

            Stmt::Fal(expr) => {
                let vt = self.infer(expr);
                if let Some(ret) = self.cur_ret.clone() {
                    if is_concrete(&ret) && is_concrete(&vt) && !ret.accepts(&vt) {
                        self.diag(format!(
                            "{} प्रकार लौटाना है पर {} लौटाया गया",
                            ret.name(), vt.name()
                        ));
                    }
                }
            }

            Stmt::Print(e) | Stmt::ExprStmt(e) | Stmt::Yield(e) => {
                self.infer(e);
            }

            Stmt::Yadi { condition, then, otherwise } => {
                self.infer(condition);
                self.check_block(then);
                if let Some(els) = otherwise {
                    self.check_block(els);
                }
            }
            Stmt::JabTak { condition, body } => {
                self.infer(condition);
                self.check_block(body);
            }
            Stmt::BarKaro { count, body } => {
                self.infer(count);
                self.check_block(body);
            }
            Stmt::KeeLiye { iter, body, .. } => {
                self.infer(iter);
                // Loop variable is dynamic — left unannotated (Any).
                self.check_block(body);
            }
            Stmt::Varg { methods, .. } => {
                // Check method bodies; methods are not in the global sig table, so
                // their calls resolve to Any. यह.field access stays Any too.
                for m in methods {
                    self.check_stmt(m);
                }
            }

            // Everything else carries no annotations we can check.
            _ => {}
        }
    }

    /// Infer a best-effort type for an expression, recording diagnostics for any
    /// concrete mismatch found along the way.
    fn infer(&mut self, expr: &Expr) -> TypeHint {
        match expr {
            Expr::Number(_) => TypeHint::Number,
            Expr::Str(_) => TypeHint::Str,
            Expr::Bool(_) => TypeHint::Bool,
            Expr::List(_) | Expr::ListWithSpread(_) => TypeHint::List,
            Expr::Dict(_) => TypeHint::Dict,

            Expr::Ident(name) => {
                if name == "शून्य" {
                    TypeHint::Nil
                } else {
                    self.lookup(name).unwrap_or(TypeHint::Any)
                }
            }

            Expr::Binary { left, op, right } => {
                let lt = self.infer(left);
                let rt = self.infer(right);
                self.infer_binop(op, &lt, &rt)
            }

            Expr::Compare { left, right, .. } => {
                self.infer(left);
                self.infer(right);
                TypeHint::Bool
            }
            Expr::Not(e) => {
                self.infer(e);
                TypeHint::Bool
            }
            Expr::Membership { item, container, .. } => {
                self.infer(item);
                self.infer(container);
                TypeHint::Bool
            }
            Expr::BitNot(e) => {
                self.infer(e);
                TypeHint::Number
            }

            Expr::Call { name, args } => {
                self.check_call_args(name, args);
                self.funcs.get(name).map(|s| s.ret.clone()).unwrap_or(TypeHint::Any)
            }
            Expr::CallKw { name, args, kwargs } => {
                // Positional portion only; keyword reordering makes precise
                // matching not worth it. Still infer args to surface inner errors.
                for a in args {
                    self.infer(a);
                }
                for (_, v) in kwargs {
                    self.infer(v);
                }
                self.funcs.get(name).map(|s| s.ret.clone()).unwrap_or(TypeHint::Any)
            }

            Expr::Ternary { condition, then_val, else_val } => {
                self.infer(condition);
                let a = self.infer(then_val);
                let b = self.infer(else_val);
                if a == b { a } else { TypeHint::Any }
            }

            Expr::Slice { obj, start, end, step } => {
                let ot = self.infer(obj);
                for part in [start, end, step].into_iter().flatten() {
                    self.infer(part);
                }
                match ot {
                    TypeHint::List => TypeHint::List,
                    TypeHint::Str => TypeHint::Str,
                    _ => TypeHint::Any,
                }
            }

            Expr::Walrus { name, value } => {
                let vt = self.infer(value);
                // A walrus binding is dynamic unless the var was already annotated.
                if let Some(declared) = self.lookup(name) {
                    if is_concrete(&declared) && is_concrete(&vt) && !declared.accepts(&vt) {
                        self.diag(format!(
                            "चर '{}' ({}) को {} मान नहीं दिया जा सकता",
                            name, declared.name(), vt.name()
                        ));
                    }
                    declared
                } else {
                    vt
                }
            }

            Expr::Index { obj, idx } => {
                self.infer(obj);
                self.infer(idx);
                TypeHint::Any
            }
            Expr::MethodCall { object, args, .. } => {
                self.infer(object);
                for a in args {
                    self.infer(a);
                }
                TypeHint::Any
            }
            Expr::MethodCallKw { object, args, kwargs, .. } => {
                self.infer(object);
                for a in args {
                    self.infer(a);
                }
                for (_, v) in kwargs {
                    self.infer(v);
                }
                TypeHint::Any
            }
            Expr::Attr { obj, .. } => {
                self.infer(obj);
                TypeHint::Any
            }
            Expr::Await(e) => {
                self.infer(e);
                TypeHint::Any
            }
            Expr::Comprehension { .. } => TypeHint::List,
            Expr::Lambda { .. } => TypeHint::Any,
        }
    }

    fn infer_binop(&mut self, op: &BinOp, lt: &TypeHint, rt: &TypeHint) -> TypeHint {
        match op {
            // `+` is permissive in LIPI (string coercion). Never flagged.
            BinOp::Add => {
                if matches!(lt, TypeHint::Str) || matches!(rt, TypeHint::Str) {
                    TypeHint::Str
                } else if matches!(lt, TypeHint::List) && matches!(rt, TypeHint::List) {
                    TypeHint::List
                } else if matches!(lt, TypeHint::Number) && matches!(rt, TypeHint::Number) {
                    TypeHint::Number
                } else {
                    TypeHint::Any
                }
            }
            // Logical operators — operands may be anything truthy; result Bool.
            BinOp::And | BinOp::Or => TypeHint::Bool,
            // All remaining arithmetic / bitwise ops require numbers.
            _ => {
                for (side, t) in [("बायाँ", lt), ("दायाँ", rt)] {
                    if is_concrete(t) && !matches!(t, TypeHint::Number) {
                        self.diag(format!(
                            "अंकगणित '{}' को संख्या चाहिए पर {} पक्ष {} है",
                            binop_name(op), side, t.name()
                        ));
                    }
                }
                TypeHint::Number
            }
        }
    }

    fn check_call_args(&mut self, name: &str, args: &[Expr]) {
        // Infer every argument first (so nested errors surface even for unknown
        // callees), collecting their types.
        let arg_types: Vec<TypeHint> = args.iter().map(|a| self.infer(a)).collect();
        if let Some(sig) = self.funcs.get(name) {
            let params = sig.params.clone();
            for (i, at) in arg_types.iter().enumerate() {
                if let Some(pt) = params.get(i) {
                    if is_concrete(pt) && is_concrete(at) && !pt.accepts(at) {
                        let pos = i + 1;
                        self.diag(format!(
                            "'{}' का {}वाँ तर्क {} होना चाहिए पर {} दिया गया",
                            name, pos, pt.name(), at.name()
                        ));
                    }
                }
            }
        }
    }
}

fn binop_name(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::FloorDiv => "//",
        BinOp::Mod => "%",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::LShift => "<<",
        BinOp::RShift => ">>",
        BinOp::And => "और",
        BinOp::Or => "या",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostics(src: &str) -> Vec<Diagnostic> {
        let tokens = crate::lexer::tokenize(src);
        let stmts = crate::parser::parse(tokens).expect("parse");
        check(&stmts)
    }

    #[test]
    fn clean_annotated_program_passes() {
        let src = "विधि जोड़ो(अ: संख्या, ब: संख्या) -> संख्या:\n    फल अ + ब\nबताओ जोड़ो(2, 3)\n";
        assert!(diagnostics(src).is_empty());
    }

    #[test]
    fn untyped_program_never_flagged() {
        let src = "क है 5\nक है \"नमस्ते\"\nबताओ क\n";
        assert!(diagnostics(src).is_empty(), "untyped code must stay dynamic");
    }

    #[test]
    fn wrong_arg_type_flagged() {
        let src = "विधि दुगुना(अ: संख्या) -> संख्या:\n    फल अ * 2\nबताओ दुगुना(\"x\")\n";
        let d = diagnostics(src);
        assert_eq!(d.len(), 1, "expected one arg mismatch, got {d:?}");
    }

    #[test]
    fn wrong_return_type_flagged() {
        let src = "विधि f() -> संख्या:\n    फल \"नमस्ते\"\n";
        let d = diagnostics(src);
        assert_eq!(d.len(), 1, "expected one return mismatch, got {d:?}");
    }

    #[test]
    fn bad_annotated_assignment_flagged() {
        let src = "नाम: वाक्य है 42\n";
        let d = diagnostics(src);
        assert_eq!(d.len(), 1, "expected one assignment mismatch, got {d:?}");
    }

    #[test]
    fn string_minus_number_flagged() {
        let src = "बताओ \"x\" - 5\n";
        let d = diagnostics(src);
        assert_eq!(d.len(), 1, "expected one arithmetic mismatch, got {d:?}");
    }

    #[test]
    fn string_plus_number_allowed() {
        let src = "आयु: संख्या है 30\nबताओ \"उम्र: \" + आयु\n";
        assert!(diagnostics(src).is_empty(), "+ coercion must not be flagged");
    }

    #[test]
    fn any_escape_hatch_silences() {
        let src = "विधि f(अ: कुछ_भी) -> कुछ_भी:\n    फल अ - 1\nबताओ f(\"x\")\n";
        assert!(diagnostics(src).is_empty(), "कुछ_भी must silence checks");
    }
}
