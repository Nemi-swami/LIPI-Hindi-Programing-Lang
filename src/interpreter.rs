/// LIPI 2.0 Tree-Walk Interpreter — Phase 1
/// Evaluates AST directly. Phase 2 will compile to LVM bytecode.

use std::collections::HashMap;
use crate::ast::*;
use crate::karaka::{Karaka, KarakaEnv};

// ===== VALUE =====

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Function { params: Vec<Param>, body: Vec<Stmt> },
    NativeFunction(fn(Vec<Value>) -> Result<Value, String>),
    Nil,
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    /// Class instance — वर्ग वस्तु (Phase 6)
    Instance { class: String, fields: HashMap<String, Value> },
    /// First-class function / closure — carries captured scope (Phase 10/11)
    Closure { func_name: String, captured: HashMap<String, Value> },
    /// Enum type definition — Value::EnumDef stored in globals (Phase 15)
    EnumDef { name: String, variants: HashMap<String, usize> },
    /// Enum instance — a specific variant with payload values (Phase 15)
    Enum { enum_name: String, variant: String, values: Vec<Value> },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Str(s)  => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{}", if *b { "सत्य" } else { "असत्य" }),
            Value::Nil     => write!(f, "शून्य"),
            Value::Function { .. }      => write!(f, "<विधि>"),
            Value::NativeFunction(_)    => write!(f, "<स्वदेशी-विधि>"),
            Value::Closure { func_name, .. } => write!(f, "<विधि:{func_name}>"),
            Value::EnumDef { name, .. } => write!(f, "<विकल्प:{name}>"),
            Value::Enum { enum_name, variant, values } => {
                if values.is_empty() {
                    write!(f, "{enum_name}::{variant}")
                } else {
                    write!(f, "{enum_name}::{variant}(")?;
                    for (i, v) in values.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", val_repr(v))?;
                    }
                    write!(f, ")")
                }
            }
            Value::List(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val_repr(item))?;
                }
                write!(f, "]")
            }
            Value::Dict(m) => {
                write!(f, "{{")?;
                let mut pairs: Vec<(&String, &Value)> = m.iter().collect();
                pairs.sort_by_key(|(k, _)| k.as_str());
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{k}\": {}", val_repr(v))?;
                }
                write!(f, "}}")
            }
            Value::Instance { class, fields } => {
                write!(f, "<{}", class)?;
                if !fields.is_empty() {
                    let mut pairs: Vec<(&String, &Value)> = fields.iter().collect();
                    pairs.sort_by_key(|(k, _)| k.as_str());
                    write!(f, " {{")?;
                    for (i, (k, v)) in pairs.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{k}: {}", val_repr(v))?;
                    }
                    write!(f, "}}")?;
                }
                write!(f, ">")
            }
        }
    }
}

/// Formats a value for use inside a collection (strings get quotes).
fn val_repr(v: &Value) -> String {
    match v {
        Value::Str(s) => format!("\"{s}\""),
        other => format!("{other}"),
    }
}

// ===== ENVIRONMENT =====

#[derive(Debug)]
struct Env {
    vals: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}

impl Env {
    fn new() -> Self { Env { vals: HashMap::new(), parent: None } }

    fn child(parent: Env) -> Self {
        Env { vals: HashMap::new(), parent: Some(Box::new(parent)) }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.vals.get(name).cloned()
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    fn set(&mut self, name: String, val: Value) {
        self.vals.insert(name, val);
    }

    fn set_existing(&mut self, name: &str, val: Value) -> bool {
        if self.vals.contains_key(name) {
            self.vals.insert(name.to_string(), val);
            true
        } else {
            self.parent.as_mut().map_or(false, |p| p.set_existing(name, val))
        }
    }
}

// ===== SIGNAL (for early return) =====

enum Signal { Return(Value) }

// ===== INTERPRETER =====

pub struct Interpreter {
    env: Env,
    karaka: KarakaEnv,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: Env::new(), karaka: KarakaEnv::new() }
    }

    pub fn exec_all(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            if let Some(Signal::Return(_)) = self.exec(stmt)? { break; }
        }
        Ok(())
    }

    fn exec(&mut self, stmt: &Stmt) -> Result<Option<Signal>, String> {
        match stmt {
            // Source-position wrapper (Phase 17) — tree-walk ignores the line
            Stmt::Located { inner, .. } => self.exec(inner),

            // परीक्षण blocks are skipped outside `lipi test`
            Stmt::Parikshan { .. } => Ok(None),

            // ── Assignment ──────────────────────────────────────────────────
            // अ, ब है 1, 2  — tuple unpacking (Phase 17)
            Stmt::MultiAssign { names, values } => {
                let vals: Vec<Value> = if values.len() == names.len() {
                    values.iter().map(|v| self.eval(v)).collect::<Result<_, _>>()?
                } else {
                    match self.eval(&values[0])? {
                        Value::List(items) if items.len() == names.len() => items,
                        Value::List(items) => return Err(format!(
                            "खोलना विफल: {} नाम हैं पर सूची में {} मान", names.len(), items.len())),
                        other => return Err(format!("खोलने के लिए सूची अपेक्षित, मिला: {}", val_repr(&other))),
                    }
                };
                for (name, val) in names.iter().zip(vals) {
                    if !self.env.set_existing(name, val.clone()) {
                        self.env.set(name.clone(), val);
                    }
                }
                Ok(None)
            }

            // अ है ब है 0  — chained assignment (Phase 17)
            Stmt::ChainAssign { names, value } => {
                let val = self.eval(value)?;
                for name in names {
                    if !self.env.set_existing(name, val.clone()) {
                        self.env.set(name.clone(), val.clone());
                    }
                }
                Ok(None)
            }

            Stmt::Assign { name, karaka, value } => {
                let val = self.eval(value)?;
                // Register Karaka role before storing value
                if let Some(k) = karaka {
                    self.karaka.annotate(name, k.clone());
                }
                if !self.env.set_existing(name, val.clone()) {
                    self.env.set(name.clone(), val);
                }
                Ok(None)
            }

            // ── Print ────────────────────────────────────────────────────────
            Stmt::Print(expr) => {
                println!("{}", self.eval(expr)?);
                Ok(None)
            }

            // ── Conditional ──────────────────────────────────────────────────
            Stmt::Yadi { condition, then, otherwise } => {
                if is_truthy(&self.eval(condition)?) {
                    self.exec_block(then)?;
                } else if let Some(els) = otherwise {
                    self.exec_block(els)?;
                }
                Ok(None)
            }

            // ── Repeat N times ───────────────────────────────────────────────
            Stmt::BarKaro { count, body } => {
                let n = self.eval_number(count)? as i64;
                for _ in 0..n.max(0) {
                    if let Some(sig) = self.exec_block(body)? {
                        return Ok(Some(sig));
                    }
                }
                Ok(None)
            }

            // ── For-each ─────────────────────────────────────────────────────
            Stmt::KeeLiye { var, iter, body } => {
                match self.eval(iter)? {
                    Value::Str(s) => {
                        for ch in s.chars() {
                            self.env.set(var.clone(), Value::Str(ch.to_string()));
                            if let Some(sig) = self.exec_block(body)? {
                                return Ok(Some(sig));
                            }
                        }
                    }
                    Value::Number(n) => {
                        for i in 0..(n as i64) {
                            self.env.set(var.clone(), Value::Number(i as f64));
                            if let Some(sig) = self.exec_block(body)? {
                                return Ok(Some(sig));
                            }
                        }
                    }
                    other => return Err(format!("के लिए: '{other}' पर iteration नहीं हो सकती")),
                }
                Ok(None)
            }

            // ── Function definition ──────────────────────────────────────────
            Stmt::Vidhi { name, params, body, decorators, .. } => {
                if !decorators.is_empty() {
                    return Err("सजावट (@) केवल LVM में समर्थित है".to_string());
                }
                self.env.set(name.clone(), Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                });
                Ok(None)
            }

            // ── Return ───────────────────────────────────────────────────────
            Stmt::Fal(expr) => {
                Ok(Some(Signal::Return(self.eval(expr)?)))
            }

            // ── Bare expression (e.g. function call as statement) ────────────
            Stmt::ExprStmt(expr) => {
                self.eval(expr)?;
                Ok(None)
            }

            // Phase 5 stub — tree-walk path
            Stmt::IndexAssign { obj, idx, val } => {
                let idx_val = self.eval(idx)?;
                let new_val = self.eval(val)?;
                let mut coll = self.env.get(obj)
                    .ok_or_else(|| format!("'{}' परिभाषित नहीं है", obj))?;
                match (&mut coll, idx_val) {
                    (Value::List(v), Value::Number(n)) => {
                        let i = n as usize;
                        if i < v.len() { v[i] = new_val; } else { return Err(format!("सूची सीमा से बाहर")); }
                    }
                    (Value::Dict(m), k) => { m.insert(format!("{k}"), new_val); }
                    _ => return Err("अनुक्रमण असाइनमेंट असमर्थित".into()),
                }
                self.env.set(obj.clone(), coll);
                Ok(None)
            }

            // Phase 6 stubs — tree-walk path (LVM is default)
            Stmt::Varg { .. } => Ok(None),
            Stmt::AttrAssign { .. } => Err("वर्ग-क्षेत्र असाइनमेंट LVM में चलाएं".into()),

            // Phase 7+ stubs
            Stmt::JabTak { .. } | Stmt::BandKaro | Stmt::Agla | Stmt::Likho(_)
            | Stmt::TryCatch { .. } | Stmt::Phenko(_) | Stmt::AayatFile(_) | Stmt::Global(_)
            | Stmt::ViKalp { .. } | Stmt::Milao { .. } | Stmt::Saath { .. }
            | Stmt::Jancho { .. } | Stmt::SthirDecl { .. } | Stmt::Yield(_) =>
                Err("यह सुविधा LVM में चलाएं".into()),

            // ── Import stdlib module ─────────────────────────────────────────
            Stmt::Aayat(module) => {
                let registry = match module.as_str() {
                    "भारत.पहचान"  => crate::bharat_stdlib::pehchaan_registry(),
                    "भारत.संख्या"  => crate::bharat_stdlib::sankhya_registry(),
                    "भारत.भुगतान"  => crate::bharat_stdlib::bhugtaan_registry(),
                    "भारत.भाषा"   => crate::bharat_stdlib::bhasha_registry(),
                    other => return Err(format!("अज्ञात मॉड्यूल: {}", other)),
                };
                for (fname, func) in registry {
                    self.env.set(fname.to_string(), Value::NativeFunction(func));
                }
                Ok(None)
            }
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Option<Signal>, String> {
        for stmt in stmts {
            if let Some(sig) = self.exec(stmt)? {
                return Ok(Some(sig));
            }
        }
        Ok(None)
    }

    // ===== EXPRESSION EVALUATION =====

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n)  => Ok(Value::Number(*n)),
            Expr::Str(s)     => Ok(Value::Str(s.clone())),
            Expr::Bool(b)    => Ok(Value::Bool(*b)),

            Expr::Ident(name) => {
                self.env.get(name)
                    .ok_or_else(|| format!("'{}' परिभाषित नहीं है", name))
            }

            Expr::Binary { left, op, right } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                eval_binary(l, op, r)
            }

            Expr::Compare { left, op, right } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                eval_compare(l, op, r)
            }

            Expr::Call { name, args } => self.call_fn(name, args),

            // item में_है container — membership (Phase 17)
            Expr::Membership { item, container, negated } => {
                let i = self.eval(item)?;
                let c = self.eval(container)?;
                let found = contains_value(&i, &c)?;
                Ok(Value::Bool(if *negated { !found } else { found }))
            }

            // नाम := expr — walrus (Phase 17)
            Expr::Walrus { name, value } => {
                let val = self.eval(value)?;
                if !self.env.set_existing(name, val.clone()) {
                    self.env.set(name.clone(), val.clone());
                }
                Ok(val)
            }

            // [expr के लिए var iter में यदि cond] — comprehension (Phase 17)
            Expr::Comprehension { expr, clauses, cond } => {
                let mut out: Vec<Value> = Vec::new();
                self.eval_comp(expr, clauses, 0, cond, &mut out)?;
                Ok(Value::List(out))
            }

            // obj[start:end:step]  — slice (Phase 17)
            Expr::Slice { obj, start, end, step } => {
                let o = self.eval(obj)?;
                let mut part = |p: &Option<Box<Expr>>| -> Result<Value, String> {
                    match p {
                        Some(e) => self.eval(e),
                        None    => Ok(Value::Nil),
                    }
                };
                let (s, e, st) = (part(start)?, part(end)?, part(step)?);
                slice_value(o, s, e, st)
            }

            // Keyword arguments are LVM-only (Phase 17) — the tree-walk
            // interpreter is legacy and no longer the default runtime.
            Expr::CallKw { .. } => Err(
                "कीवर्ड तर्क (नाम=मान) केवल LVM में समर्थित हैं".to_string()
            ),

            Expr::MethodCall { object, method, args } => {
                let obj = self.eval(object)?;
                let mut evaled = Vec::new();
                for a in args { evaled.push(self.eval(a)?); }
                call_method(obj, method, evaled)
            }

            // Phase 5 — tree-walk interpreter stubs (LVM is the default path)
            Expr::List(elems) => {
                let mut v = Vec::new();
                for e in elems { v.push(self.eval(e)?); }
                Ok(Value::List(v))
            }
            // Phase 17 — spread in list literals: [*अ, 99, *ब]
            Expr::ListWithSpread(elems) => {
                let mut v = Vec::new();
                for (is_spread, e) in elems {
                    let val = self.eval(e)?;
                    if *is_spread {
                        match val {
                            Value::List(items) => v.extend(items),
                            other => return Err(format!(
                                "फैलाव (*) केवल सूची पर हो सकता है, मिला: {other}"
                            )),
                        }
                    } else {
                        v.push(val);
                    }
                }
                Ok(Value::List(v))
            }
            Expr::Dict(pairs) => {
                let mut m = HashMap::new();
                for (k, v) in pairs {
                    m.insert(format!("{}", self.eval(k)?), self.eval(v)?);
                }
                Ok(Value::Dict(m))
            }
            Expr::Index { obj, idx } => {
                let o = self.eval(obj)?;
                let i = self.eval(idx)?;
                match (o, i) {
                    (Value::List(v), Value::Number(n)) =>
                        v.get(n as usize).cloned().ok_or_else(|| format!("सूची सीमा से बाहर")),
                    (Value::Dict(m), k) =>
                        m.get(&format!("{k}")).cloned().ok_or_else(|| format!("कुंजी नहीं मिली")),
                    (o, _) => Err(format!("'{}' पर अनुक्रमण नहीं होता", o)),
                }
            }
            // Phase 6 stub
            Expr::Attr { .. } => Err("वर्ग-क्षेत्र पहुंच LVM में चलाएं".into()),
            // Phase 10 stub — lambdas only run in LVM
            Expr::Lambda { .. }   => Err("लाम्डा LVM में चलाएं".into()),
            // Phase 12 stubs
            Expr::Ternary { .. }  => Err("त्रिचर LVM में चलाएं".into()),
            Expr::BitNot(_)       => Err("बिटवाइज़ LVM में चलाएं".into()),
            // Phase 13 stubs
            Expr::Not(_)          => Err("यह सुविधा LVM में चलाएं".into()),
        }
    }

    /// Recursive comprehension eval — one clause per nesting level; the
    /// trailing यदि filter guards the innermost append. Iterates List
    /// elements, Str chars, Number ranges, and Dict keys (sorted), matching
    /// the LVM's GetIterLen/IterNext semantics.
    fn eval_comp(
        &mut self,
        expr: &Expr,
        clauses: &[(String, Expr)],
        depth: usize,
        cond: &Option<Box<Expr>>,
        out: &mut Vec<Value>,
    ) -> Result<(), String> {
        if depth == clauses.len() {
            if let Some(c) = cond {
                if !is_truthy(&self.eval(c)?) {
                    return Ok(());
                }
            }
            out.push(self.eval(expr)?);
            return Ok(());
        }
        let (var, iter) = &clauses[depth];
        let items: Vec<Value> = match self.eval(iter)? {
            Value::List(l) => l,
            Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(),
            Value::Number(n) => (0..n as i64).map(|i| Value::Number(i as f64)).collect(),
            Value::Dict(d) => {
                let mut ks: Vec<String> = d.keys().cloned().collect();
                ks.sort();
                ks.into_iter().map(Value::Str).collect()
            }
            other => return Err(format!("के लिए: '{other}' पर iteration नहीं हो सकती")),
        };
        for item in items {
            self.env.set(var.clone(), item);
            self.eval_comp(expr, clauses, depth + 1, cond, out)?;
        }
        Ok(())
    }

    fn call_fn(&mut self, name: &str, arg_exprs: &[Expr]) -> Result<Value, String> {
        // ── Built-in functions ────────────────────────────────────────────────
        match name {
            "लम्बाई" => {
                let v = self.eval(arg_exprs.first().ok_or("लम्बाई() को एक तर्क चाहिए")?)?;
                return match v {
                    Value::Str(s) => Ok(Value::Number(s.chars().count() as f64)),
                    _ => Err("लम्बाई() केवल वाक्य पर काम करता है".to_string()),
                };
            }
            "पूर्णांक" => {
                let v = self.eval(arg_exprs.first().ok_or("पूर्णांक() को एक तर्क चाहिए")?)?;
                return match v {
                    Value::Number(n) => Ok(Value::Number(n.floor())),
                    Value::Str(s) => s.trim().parse::<f64>()
                        .map(|n| Value::Number(n))
                        .map_err(|_| format!("'{}' को संख्या में नहीं बदल सका", s)),
                    _ => Err("पूर्णांक() को संख्या या वाक्य चाहिए".to_string()),
                };
            }
            _ => {}
        }

        // ── Extract variable names for Karaka checking ────────────────────────
        let arg_var_names: Vec<Option<String>> = arg_exprs.iter().map(|e| {
            if let Expr::Ident(n) = e { Some(n.clone()) } else { None }
        }).collect();

        // ── Look up user-defined or native function ───────────────────────────
        let func = self.env.get(name)
            .ok_or_else(|| format!("विधि '{}' परिभाषित नहीं है", name))?;

        // Native stdlib functions bypass Karaka checking and scope creation
        if let Value::NativeFunction(f) = func {
            let mut args = Vec::new();
            for e in arg_exprs { args.push(self.eval(e)?); }
            return f(args);
        }

        let (params, body) = match func {
            Value::Function { params, body } => (params, body),
            _ => return Err(format!("'{}' एक विधि नहीं है", name)),
        };

        // Argument count check
        if arg_exprs.len() != params.len() {
            return Err(format!(
                "'{}' को {} तर्क चाहिए, {} दिए गए",
                name, params.len(), arg_exprs.len()
            ));
        }

        // ── Karaka role check (soft warning) ──────────────────────────────────
        let param_roles: Vec<(String, Option<Karaka>)> = params.iter()
            .map(|p| (p.name.clone(), p.karaka.clone()))
            .collect();
        self.karaka.check_call(name, &param_roles, &arg_var_names);

        // ── Evaluate arguments in current scope ───────────────────────────────
        let mut arg_vals = Vec::new();
        for e in arg_exprs { arg_vals.push(self.eval(e)?); }

        // ── Execute function in child scope ───────────────────────────────────
        let parent_env = std::mem::replace(&mut self.env, Env::new());
        self.env = Env::child(parent_env);

        for (param, val) in params.iter().zip(arg_vals) {
            self.env.set(param.name.clone(), val);
        }

        let result = self.exec_block(&body)?;

        // Restore parent scope
        let child_env = std::mem::replace(&mut self.env, Env::new());
        self.env = *child_env.parent.expect("function scope has parent");

        Ok(match result {
            Some(Signal::Return(v)) => v,
            None => Value::Nil,
        })
    }

    fn eval_number(&mut self, expr: &Expr) -> Result<f64, String> {
        match self.eval(expr)? {
            Value::Number(n) => Ok(n),
            other => Err(format!("संख्या अपेक्षित, मिला: {other}")),
        }
    }
}

// ===== METHOD DISPATCH =====

fn call_method(obj: Value, method: &str, _args: Vec<Value>) -> Result<Value, String> {
    match (&obj, method) {
        (Value::Str(s), "लम्बाई")     => Ok(Value::Number(s.chars().count() as f64)),
        (Value::Str(s), "रोमन_में")   => Ok(Value::Str(romanize(s))),
        (Value::Str(s), "बड़े_अक्षर") => Ok(Value::Str(s.to_uppercase())),
        (Value::Str(s), "छोटे_अक्षर") => Ok(Value::Str(s.to_lowercase())),
        (Value::Str(s), "उलटा")       => Ok(Value::Str(s.chars().rev().collect())),
        (Value::Number(n), "पूर्णांक") => Ok(Value::Number(n.floor())),
        _ => Err(format!("'{method}' विधि '{obj}' पर उपलब्ध नहीं है")),
    }
}

// ===== BINARY OPERATIONS =====

fn eval_binary(l: Value, op: &BinOp, r: Value) -> Result<Value, String> {
    match op {
        BinOp::Add => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::Str(a), Value::Str(b))       => Ok(Value::Str(a + &b)),
            (Value::Str(a), Value::Number(b))    => Ok(Value::Str(format!("{a}{}", fmt_num(b)))),
            (Value::Number(a), Value::Str(b))    => Ok(Value::Str(format!("{}{b}", fmt_num(a)))),
            (Value::Str(a), Value::Bool(b))      => Ok(Value::Str(format!("{a}{}", if b { "सत्य" } else { "असत्य" }))),
            (l, r) => Err(format!("जोड़ नहीं हो सकता: '{l}' + '{r}'")),
        },
        BinOp::Sub => num2(l, r, |a, b| a - b, "-"),
        BinOp::Mul => num2(l, r, |a, b| a * b, "*"),
        BinOp::Div => {
            if let (_, Value::Number(b)) = (&l, &r) {
                if *b == 0.0 { return Err("शून्य से भाग नहीं होता".to_string()); }
            }
            num2(l, r, |a, b| a / b, "/")
        }
        BinOp::FloorDiv => {
            if let (_, Value::Number(b)) = (&l, &r) {
                if *b == 0.0 { return Err("शून्य से भाग नहीं होता".to_string()); }
            }
            num2(l, r, |a, b| (a / b).floor(), "//")
        }
        BinOp::Mod    => num2(l, r, |a, b| a % b, "%"),
        BinOp::BitAnd => num2(l, r, |a, b| (a as i64 & b as i64) as f64, "&"),
        BinOp::BitOr  => num2(l, r, |a, b| (a as i64 | b as i64) as f64, "|"),
        BinOp::BitXor => num2(l, r, |a, b| (a as i64 ^ b as i64) as f64, "^"),
        BinOp::LShift => num2(l, r, |a, b| ((a as i64) << (b as u32)) as f64, "<<"),
        BinOp::RShift => num2(l, r, |a, b| ((a as i64) >> (b as u32)) as f64, ">>"),
        BinOp::And => {
            let lb = matches!(l, Value::Bool(true)) || matches!(l, Value::Number(n) if n != 0.0);
            let rb = matches!(r, Value::Bool(true)) || matches!(r, Value::Number(n) if n != 0.0);
            Ok(Value::Bool(lb && rb))
        }
        BinOp::Or => {
            let lb = matches!(l, Value::Bool(true)) || matches!(l, Value::Number(n) if n != 0.0);
            let rb = matches!(r, Value::Bool(true)) || matches!(r, Value::Number(n) if n != 0.0);
            Ok(Value::Bool(lb || rb))
        }
    }
}

/// Membership test (में_है) — Phase 17. Shared by LVM and tree-walk.
/// List → element equality, Str → substring, Dict → key existence.
pub fn contains_value(item: &Value, container: &Value) -> Result<bool, String> {
    match container {
        Value::List(items) => Ok(items.iter().any(|v| membership_eq(v, item))),
        Value::Str(s) => match item {
            Value::Str(sub) => Ok(s.contains(sub.as_str())),
            other => Err(format!("वाक्य में केवल वाक्य खोजा जा सकता है, मिला: {}", other)),
        },
        Value::Dict(map) => Ok(map.contains_key(&format!("{}", item))),
        other => Err(format!("में_है केवल सूची, वाक्य या कोश पर हो सकता है, मिला: {}", other)),
    }
}

fn membership_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::Str(x), Value::Str(y))       => x == y,
        (Value::Bool(x), Value::Bool(y))     => x == y,
        (Value::Nil, Value::Nil)             => true,
        _ => false,
    }
}

/// Slice a List or Str with Python semantics (Phase 17).
/// `start`/`end`/`step` are Nil when omitted. Shared by LVM and tree-walk.
pub fn slice_value(obj: Value, start: Value, end: Value, step: Value) -> Result<Value, String> {
    fn bound(v: Value, what: &str) -> Result<Option<i64>, String> {
        match v {
            Value::Nil       => Ok(None),
            Value::Number(n) => Ok(Some(n as i64)),
            other => Err(format!("स्लाइस {} संख्या होनी चाहिए, मिला: {}", what, other)),
        }
    }
    let step_i = bound(step, "चरण")?.unwrap_or(1);
    if step_i == 0 {
        return Err("स्लाइस चरण शून्य नहीं हो सकता".to_string());
    }
    let s = bound(start, "आरंभ")?;
    let e = bound(end, "अंत")?;
    match obj {
        Value::List(items) => {
            let idxs = slice_indices(items.len() as i64, s, e, step_i);
            Ok(Value::List(idxs.into_iter().map(|i| items[i as usize].clone()).collect()))
        }
        Value::Str(st) => {
            let chars: Vec<char> = st.chars().collect();
            let idxs = slice_indices(chars.len() as i64, s, e, step_i);
            Ok(Value::Str(idxs.into_iter().map(|i| chars[i as usize]).collect()))
        }
        other => Err(format!("स्लाइस केवल सूची या वाक्य पर हो सकता है, मिला: {}", other)),
    }
}

/// Python slice index resolution: negative values count from the end,
/// out-of-range values clamp, never error.
fn slice_indices(len: i64, start: Option<i64>, end: Option<i64>, step: i64) -> Vec<i64> {
    let norm = |v: i64| if v < 0 { v + len } else { v };
    let mut out = Vec::new();
    if step > 0 {
        let mut i  = norm(start.unwrap_or(0)).clamp(0, len);
        let stop   = norm(end.unwrap_or(len)).clamp(0, len);
        while i < stop { out.push(i); i += step; }
    } else {
        let mut i = match start { Some(v) => norm(v).min(len - 1), None => len - 1 };
        let stop  = match end   { Some(v) => norm(v).max(-1),      None => -1 };
        while i > stop { out.push(i); i += step; }
    }
    out
}

fn num2<F: Fn(f64, f64) -> f64>(l: Value, r: Value, f: F, sym: &str) -> Result<Value, String> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        (l, r) => Err(format!("'{sym}' के लिए संख्याएं चाहिए, मिला: '{l}' और '{r}'")),
    }
}

// ===== COMPARISON =====

fn eval_compare(l: Value, op: &CmpOp, r: Value) -> Result<Value, String> {
    let result = match op {
        CmpOp::Eq => values_equal(&l, &r),
        CmpOp::NotEq => !values_equal(&l, &r),
        CmpOp::SeAdhik | CmpOp::Gt  => cmp_num(l, r, |a, b| a > b)?,
        CmpOp::SeKam   | CmpOp::Lt  => cmp_num(l, r, |a, b| a < b)?,
        CmpOp::GtEq                 => cmp_num(l, r, |a, b| a >= b)?,
        CmpOp::LtEq                 => cmp_num(l, r, |a, b| a <= b)?,
    };
    Ok(Value::Bool(result))
}

fn values_equal(l: &Value, r: &Value) -> bool {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Str(a), Value::Str(b))       => a == b,
        (Value::Bool(a), Value::Bool(b))     => a == b,
        _ => false,
    }
}

fn cmp_num<F: Fn(f64, f64) -> bool>(l: Value, r: Value, f: F) -> Result<bool, String> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(f(a, b)),
        (l, r) => Err(format!("तुलना के लिए संख्याएं चाहिए, मिला: '{l}' और '{r}'")),
    }
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b)          => *b,
        Value::Number(n)        => *n != 0.0,
        Value::Str(s)           => !s.is_empty(),
        Value::Nil              => false,
        Value::Function{..}     => true,
        Value::NativeFunction(_)=> true,
        Value::List(v)          => !v.is_empty(),
        Value::Dict(m)          => !m.is_empty(),
        Value::Instance { .. }  => true,
        Value::Closure { .. }   => true,
        Value::EnumDef { .. }   => true,
        Value::Enum { .. }      => true,
    }
}

// ===== HELPERS =====

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) }
    else { format!("{n}") }
}

/// Minimal Devanagari → Latin romanization (IAST-lite)
fn romanize(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let r = match ch {
            'अ' => "a",  'आ' => "ā",  'इ' => "i",  'ई' => "ī",
            'उ' => "u",  'ऊ' => "ū",  'ए' => "e",  'ओ' => "o",
            'ऐ' => "ai", 'औ' => "au", 'ऋ' => "ṛ",
            'क' => "k",  'ख' => "kh", 'ग' => "g",  'घ' => "gh", 'ङ' => "ṅ",
            'च' => "ch", 'छ' => "chh",'ज' => "j",  'झ' => "jh", 'ञ' => "ñ",
            'ट' => "ṭ",  'ठ' => "ṭh", 'ड' => "ḍ",  'ढ' => "ḍh", 'ण' => "ṇ",
            'त' => "t",  'थ' => "th", 'द' => "d",  'ध' => "dh", 'न' => "n",
            'प' => "p",  'फ' => "ph", 'ब' => "b",  'भ' => "bh", 'म' => "m",
            'य' => "y",  'र' => "r",  'ल' => "l",  'व' => "v",
            'श' => "sh", 'ष' => "ṣ",  'स' => "s",  'ह' => "h",
            // matras (vowel diacritics)
            'ा' => "ā", 'ि' => "i", 'ी' => "ī",
            'ु' => "u", 'ू' => "ū", 'े' => "e", 'ो' => "o",
            'ै' => "ai",'ौ' => "au",'ं' => "ṃ",
            '्' => "",   // virama — suppress inherent vowel
            ' ' => " ",
            other => { out.push(other); continue; }
        };
        out.push_str(r);
    }
    out
}
