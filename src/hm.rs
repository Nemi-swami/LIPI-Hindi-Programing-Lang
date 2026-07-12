use std::collections::{HashMap, BTreeMap};
use crate::ast::{Expr, Stmt, BinOp, unwrap_located};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Num,
    Str,
    Bool,
    Nil,
    List(Box<Type>),
    Dict(Box<Type>),
    Fun(Vec<Type>, Box<Type>),
    Class(String),
    Instance(String),
    Var(u32),
    Any,
    Record { fields: BTreeMap<String, Type>, rest: Option<u32> },
}

impl Type {
    pub fn pretty(&self) -> String {
        match self {
            Type::Num => "संख्या".into(),
            Type::Str => "वाक्य".into(),
            Type::Bool => "तर्क".into(),
            Type::Nil => "शून्य".into(),
            Type::List(t) => format!("सूची<{}>", t.pretty()),
            Type::Dict(t) => format!("कोश<{}>", t.pretty()),
            Type::Fun(args, ret) => {
                let a: Vec<String> = args.iter().map(|t| t.pretty()).collect();
                format!("({}) -> {}", a.join(", "), ret.pretty())
            }
            Type::Class(name) => format!("वर्ग:{}", name),
            Type::Instance(name) => name.clone(),
            Type::Var(id) => format!("τ{}", id),
            Type::Any => "कुछ_भी".into(),
            Type::Record { fields, rest } => {
                let parts: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.pretty()))
                    .collect();
                match rest {
                    Some(r) => format!("{{{} | τ{}}}", parts.join(", "), r),
                    None => format!("{{{}}}", parts.join(", ")),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheme {
    pub vars: Vec<u32>,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    pub methods: HashMap<String, Scheme>,
    pub parents: Vec<String>,
}

pub struct Inferrer {
    next_var: u32,
    subst: HashMap<u32, Type>,
    pub errors: Vec<TypeError>,
    cur_line: usize,
    pub classes: HashMap<String, ClassInfo>,
}

impl Inferrer {
    pub fn new() -> Self {
        Self {
            next_var: 0,
            subst: HashMap::new(),
            errors: Vec::new(),
            cur_line: 0,
            classes: HashMap::new(),
        }
    }

    fn fresh(&mut self) -> Type {
        let v = self.next_var;
        self.next_var += 1;
        Type::Var(v)
    }

    fn err(&mut self, msg: String) {
        self.errors.push(TypeError { line: self.cur_line, message: msg });
    }

    pub fn apply(&self, t: &Type) -> Type {
        match t {
            Type::Var(id) => match self.subst.get(id) {
                Some(inner) => self.apply(&inner.clone()),
                None => t.clone(),
            },
            Type::List(t) => Type::List(Box::new(self.apply(t))),
            Type::Dict(t) => Type::Dict(Box::new(self.apply(t))),
            Type::Fun(args, ret) => Type::Fun(
                args.iter().map(|a| self.apply(a)).collect(),
                Box::new(self.apply(ret)),
            ),
            Type::Record { fields, rest } => {
                let applied_fields: BTreeMap<String, Type> = fields.iter()
                    .map(|(k, v)| (k.clone(), self.apply(v)))
                    .collect();
                match rest {
                    Some(r) => match self.subst.get(r) {
                        Some(inner) => {
                            let applied_rest = self.apply(&inner.clone());
                            match applied_rest {
                                Type::Record { fields: rf, rest: rr } => {
                                    let mut merged = applied_fields;
                                    for (k, v) in rf {
                                        merged.entry(k).or_insert(v);
                                    }
                                    Type::Record { fields: merged, rest: rr }
                                }
                                Type::Var(id) => Type::Record { fields: applied_fields, rest: Some(id) },
                                _ => Type::Record { fields: applied_fields, rest: Some(*r) },
                            }
                        }
                        None => Type::Record { fields: applied_fields, rest: Some(*r) },
                    }
                    None => Type::Record { fields: applied_fields, rest: None },
                }
            }
            _ => t.clone(),
        }
    }

    fn occurs(&self, v: u32, t: &Type) -> bool {
        match self.apply(t) {
            Type::Var(id) => id == v,
            Type::List(t) | Type::Dict(t) => self.occurs(v, &t),
            Type::Fun(args, ret) => args.iter().any(|a| self.occurs(v, a)) || self.occurs(v, &ret),
            Type::Record { fields, rest } => {
                fields.values().any(|t| self.occurs(v, t))
                    || matches!(rest, Some(r) if r == v)
            }
            _ => false,
        }
    }

    fn unify(&mut self, a: &Type, b: &Type) -> Result<(), String> {
        let a = self.apply(a);
        let b = self.apply(b);
        match (a, b) {
            (Type::Any, _) | (_, Type::Any) => Ok(()),
            (Type::Num, Type::Num) | (Type::Str, Type::Str) |
            (Type::Bool, Type::Bool) | (Type::Nil, Type::Nil) => Ok(()),
            (Type::Instance(a), Type::Instance(b)) => {
                if a == b { Ok(()) }
                else if self.is_ancestor(&a, &b) || self.is_ancestor(&b, &a) { Ok(()) }
                else { Err(format!("प्रकार मेल नहीं: वस्तु {} vs वस्तु {}", a, b)) }
            }
            (Type::Class(a), Type::Class(b)) if a == b => Ok(()),
            (Type::Var(x), Type::Var(y)) if x == y => Ok(()),
            (Type::Var(x), other) | (other, Type::Var(x)) => {
                if self.occurs(x, &other) {
                    Err(format!("प्रकार अनंत — τ{} occurs in {}", x, other.pretty()))
                } else {
                    self.subst.insert(x, other);
                    Ok(())
                }
            }
            (Type::List(a), Type::List(b)) => self.unify(&a, &b),
            (Type::Dict(a), Type::Dict(b)) => self.unify(&a, &b),
            (Type::Fun(a_args, a_ret), Type::Fun(b_args, b_ret)) => {
                if a_args.len() != b_args.len() {
                    return Err(format!("तर्क संख्या मेल नहीं: {} vs {}", a_args.len(), b_args.len()));
                }
                for (x, y) in a_args.iter().zip(b_args.iter()) { self.unify(x, y)?; }
                self.unify(&a_ret, &b_ret)
            }
            (Type::Record { fields: af, rest: ar }, Type::Record { fields: bf, rest: br }) => {
                self.unify_records(af, ar, bf, br)
            }
            (a, b) => Err(format!("प्रकार मेल नहीं: {} vs {}", a.pretty(), b.pretty())),
        }
    }

    fn unify_records(
        &mut self,
        af: BTreeMap<String, Type>,
        ar: Option<u32>,
        bf: BTreeMap<String, Type>,
        br: Option<u32>,
    ) -> Result<(), String> {
        for (k, av) in af.iter() {
            if let Some(bv) = bf.get(k) { self.unify(av, bv)?; }
        }
        let a_only: BTreeMap<String, Type> = af.iter()
            .filter(|(k, _)| !bf.contains_key(*k))
            .map(|(k, v)| (k.clone(), v.clone())).collect();
        let b_only: BTreeMap<String, Type> = bf.iter()
            .filter(|(k, _)| !af.contains_key(*k))
            .map(|(k, v)| (k.clone(), v.clone())).collect();
        match (ar, br) {
            (None, None) => {
                if !a_only.is_empty() || !b_only.is_empty() {
                    let mut missing = Vec::new();
                    for k in a_only.keys() { missing.push(k.clone()); }
                    for k in b_only.keys() { missing.push(k.clone()); }
                    Err(format!("अभिलेख क्षेत्र मेल नहीं: {}", missing.join(", ")))
                } else { Ok(()) }
            }
            (None, Some(rb)) => {
                if !b_only.is_empty() {
                    return Err(format!("क्षेत्र नहीं है: {}", b_only.keys().cloned().collect::<Vec<_>>().join(", ")));
                }
                let new_rec = Type::Record { fields: a_only, rest: None };
                if self.occurs(rb, &new_rec) { return Err(format!("प्रकार अनंत — τ{} occurs in record", rb)); }
                self.subst.insert(rb, new_rec);
                Ok(())
            }
            (Some(ra), None) => {
                if !a_only.is_empty() {
                    return Err(format!("क्षेत्र नहीं है: {}", a_only.keys().cloned().collect::<Vec<_>>().join(", ")));
                }
                let new_rec = Type::Record { fields: b_only, rest: None };
                if self.occurs(ra, &new_rec) { return Err(format!("प्रकार अनंत — τ{} occurs in record", ra)); }
                self.subst.insert(ra, new_rec);
                Ok(())
            }
            (Some(ra), Some(rb)) => {
                if ra == rb {
                    if !a_only.is_empty() || !b_only.is_empty() {
                        return Err("अभिलेख पंक्ति चर मेल नहीं".into());
                    }
                    return Ok(());
                }
                let rho = self.next_var; self.next_var += 1;
                let ra_val = Type::Record { fields: b_only, rest: Some(rho) };
                let rb_val = Type::Record { fields: a_only, rest: Some(rho) };
                if self.occurs(ra, &ra_val) || self.occurs(rb, &rb_val) {
                    return Err(format!("प्रकार अनंत — record row"));
                }
                self.subst.insert(ra, ra_val);
                self.subst.insert(rb, rb_val);
                Ok(())
            }
        }
    }

    fn is_ancestor(&self, ancestor: &str, child: &str) -> bool {
        let mut stack: Vec<String> = vec![child.to_string()];
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        while let Some(c) = stack.pop() {
            if !seen.insert(c.clone()) { continue; }
            if c == ancestor { return true; }
            if let Some(info) = self.classes.get(&c) {
                for p in &info.parents { stack.push(p.clone()); }
            }
        }
        false
    }

    fn unify_or_err(&mut self, a: &Type, b: &Type) {
        if let Err(e) = self.unify(a, b) { self.err(e); }
    }

    fn generalize(&self, env: &HashMap<String, Scheme>, t: &Type) -> Scheme {
        let free_in_t = free_vars(&self.apply(t));
        let mut env_vars: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for sc in env.values() {
            let ty = self.apply(&sc.ty);
            for v in free_vars(&ty) {
                if !sc.vars.contains(&v) { env_vars.insert(v); }
            }
        }
        let quantified: Vec<u32> = free_in_t.into_iter().filter(|v| !env_vars.contains(v)).collect();
        Scheme { vars: quantified, ty: self.apply(t) }
    }

    fn instantiate(&mut self, sc: &Scheme) -> Type {
        let mut renaming: HashMap<u32, Type> = HashMap::new();
        for v in &sc.vars { renaming.insert(*v, self.fresh()); }
        rename(&sc.ty, &renaming)
    }

    fn infer_expr(&mut self, e: &Expr, env: &HashMap<String, Scheme>) -> Type {
        match e {
            Expr::Number(_) => Type::Num,
            Expr::Str(_) => Type::Str,
            Expr::Bool(_) => Type::Bool,
            Expr::Ident(name) if name == "शून्य" => Type::Nil,
            Expr::Ident(name) if name == "सत्य" || name == "असत्य" => Type::Bool,
            Expr::Ident(name) => {
                if let Some(sc) = env.get(name) {
                    self.instantiate(&sc.clone())
                } else if self.classes.contains_key(name) {
                    Type::Class(name.clone())
                } else {
                    Type::Any
                }
            }
            Expr::Binary { left, op, right } => {
                let lt = self.infer_expr(left, env);
                let rt = self.infer_expr(right, env);
                match op {
                    BinOp::And | BinOp::Or => Type::Bool,
                    BinOp::Add => {
                        let l = self.apply(&lt);
                        let r = self.apply(&rt);
                        if matches!(l, Type::Str) || matches!(r, Type::Str) {
                            Type::Str
                        } else if matches!(l, Type::List(_)) && matches!(r, Type::List(_)) {
                            self.unify_or_err(&l, &r);
                            l
                        } else {
                            self.unify_or_err(&lt, &Type::Num);
                            self.unify_or_err(&rt, &Type::Num);
                            Type::Num
                        }
                    }
                    _ => {
                        self.unify_or_err(&lt, &Type::Num);
                        self.unify_or_err(&rt, &Type::Num);
                        Type::Num
                    }
                }
            }
            Expr::Compare { left, op: _, right } => {
                let lt = self.infer_expr(left, env);
                let rt = self.infer_expr(right, env);
                let _ = self.unify(&lt, &rt);
                Type::Bool
            }
            Expr::Not(e) => { self.infer_expr(e, env); Type::Bool }
            Expr::BitNot(e) => {
                let t = self.infer_expr(e, env);
                self.unify_or_err(&t, &Type::Num);
                Type::Num
            }
            Expr::List(items) => {
                let elt = self.fresh();
                for item in items {
                    let t = self.infer_expr(item, env);
                    self.unify_or_err(&elt, &t);
                }
                Type::List(Box::new(elt))
            }
            Expr::ListWithSpread(items) => {
                let elt = self.fresh();
                for (is_spread, expr) in items {
                    let t = self.infer_expr(expr, env);
                    if *is_spread {
                        self.unify_or_err(&t, &Type::List(Box::new(elt.clone())));
                    } else {
                        self.unify_or_err(&elt, &t);
                    }
                }
                Type::List(Box::new(elt))
            }
            Expr::Dict(pairs) => {
                let all_str_keys = !pairs.is_empty() && pairs.iter().all(|(k, _)| matches!(k, Expr::Str(_)));
                if all_str_keys {
                    let mut fields: BTreeMap<String, Type> = BTreeMap::new();
                    for (k, v) in pairs {
                        if let Expr::Str(key) = k {
                            let vt = self.infer_expr(v, env);
                            fields.insert(key.clone(), vt);
                        }
                    }
                    Type::Record { fields, rest: None }
                } else {
                    let elt = self.fresh();
                    for pair in pairs {
                        self.infer_expr(&pair.0, env);
                        let t = self.infer_expr(&pair.1, env);
                        self.unify_or_err(&elt, &t);
                    }
                    Type::Dict(Box::new(elt))
                }
            }
            Expr::Call { name, args } => {
                if self.classes.contains_key(name) {
                    for a in args { self.infer_expr(a, env); }
                    return Type::Instance(name.clone());
                }
                let fun_ty = match env.get(name) {
                    Some(sc) => self.instantiate(&sc.clone()),
                    None => return Type::Any,
                };
                let arg_tys: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                let ret = self.fresh();
                let expected = Type::Fun(arg_tys, Box::new(ret.clone()));
                self.unify_or_err(&fun_ty, &expected);
                ret
            }
            Expr::CallKw { name, args, kwargs } => {
                for a in args { self.infer_expr(a, env); }
                for (_, v) in kwargs { self.infer_expr(v, env); }
                if self.classes.contains_key(name) { return Type::Instance(name.clone()); }
                match env.get(name) {
                    Some(sc) => match self.instantiate(&sc.clone()) {
                        Type::Fun(_, ret) => *ret,
                        _ => Type::Any,
                    },
                    None => Type::Any,
                }
            }
            Expr::Ternary { condition, then_val, else_val } => {
                self.infer_expr(condition, env);
                let a = self.infer_expr(then_val, env);
                let b = self.infer_expr(else_val, env);
                self.unify_or_err(&a, &b);
                a
            }
            Expr::Index { obj, idx } => {
                let ot = self.infer_expr(obj, env);
                self.infer_expr(idx, env);
                let applied = self.apply(&ot);
                if let Type::Record { fields, .. } = &applied {
                    if let Expr::Str(key) = idx.as_ref() {
                        if let Some(t) = fields.get(key) { return t.clone(); }
                    }
                }
                match applied {
                    Type::List(inner) => *inner,
                    Type::Dict(inner) => *inner,
                    Type::Str => Type::Str,
                    _ => Type::Any,
                }
            }
            Expr::Slice { obj, start, end, step } => {
                let t = self.infer_expr(obj, env);
                for e in [start, end, step].iter().copied().flatten() {
                    let et = self.infer_expr(e, env);
                    self.unify_or_err(&et, &Type::Num);
                }
                t
            }
            Expr::Attr { obj, field } => {
                let ot = self.infer_expr(obj, env);
                let applied = self.apply(&ot);
                if let Type::Record { fields, .. } = &applied {
                    if let Some(t) = fields.get(field) { return t.clone(); }
                }
                if matches!(applied, Type::Record { .. } | Type::Var(_)) {
                    let ft = self.fresh();
                    let rest_id = self.next_var;
                    self.next_var += 1;
                    let mut fmap = BTreeMap::new();
                    fmap.insert(field.clone(), ft.clone());
                    let record_ty = Type::Record { fields: fmap, rest: Some(rest_id) };
                    self.unify_or_err(&ot, &record_ty);
                    return ft;
                }
                Type::Any
            }
            Expr::MethodCall { object, method, args } => {
                let ot = self.infer_expr(object, env);
                let arg_tys: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                if let Type::Instance(cls) = self.apply(&ot) {
                    if let Some(sc) = self.method_lookup(&cls, method) {
                        let inst = self.instantiate(&sc);
                        if let Type::Fun(params, ret) = inst {
                            let expected_arg_count = params.len().saturating_sub(1);
                            if expected_arg_count == arg_tys.len() {
                                for (p, a) in params.iter().skip(1).zip(arg_tys.iter()) {
                                    self.unify_or_err(p, a);
                                }
                                return *ret;
                            }
                        }
                    }
                }
                Type::Any
            }
            Expr::MethodCallKw { object, args, kwargs, .. } => {
                self.infer_expr(object, env);
                for a in args { self.infer_expr(a, env); }
                for (_, v) in kwargs { self.infer_expr(v, env); }
                Type::Any
            }
            Expr::Lambda { params, body, .. } => {
                let mut lam_env = env.clone();
                let param_tys: Vec<Type> = params.iter().map(|_| self.fresh()).collect();
                for (p, t) in params.iter().zip(param_tys.iter()) {
                    lam_env.insert(p.clone(), Scheme { vars: vec![], ty: t.clone() });
                }
                let ret = self.infer_body(body, &mut lam_env);
                Type::Fun(param_tys, Box::new(ret))
            }
            Expr::Await(e) => self.infer_expr(e, env),
            Expr::Walrus { name: _, value } => self.infer_expr(value, env),
            Expr::Membership { item, container, .. } => {
                self.infer_expr(item, env);
                self.infer_expr(container, env);
                Type::Bool
            }
            Expr::Comprehension { expr, clauses, cond } => {
                let mut inner_env = env.clone();
                for (var, iter) in clauses {
                    let it = self.infer_expr(iter, env);
                    let elt = match self.apply(&it) {
                        Type::List(t) => *t,
                        Type::Str => Type::Str,
                        Type::Dict(t) => *t,
                        _ => Type::Any,
                    };
                    inner_env.insert(var.clone(), Scheme { vars: vec![], ty: elt });
                }
                if let Some(c) = cond { self.infer_expr(c, &inner_env); }
                let elt = self.infer_expr(expr, &inner_env);
                Type::List(Box::new(elt))
            }
        }
    }

    fn method_lookup(&self, class: &str, method: &str) -> Option<Scheme> {
        let mut stack: Vec<String> = vec![class.to_string()];
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        while let Some(c) = stack.pop() {
            if !seen.insert(c.clone()) { continue; }
            if let Some(info) = self.classes.get(&c) {
                if let Some(sc) = info.methods.get(method) { return Some(sc.clone()); }
                for p in &info.parents { stack.push(p.clone()); }
            }
        }
        None
    }

    pub fn infer_program(&mut self, stmts: &[Stmt]) -> HashMap<String, Scheme> {
        let mut env = builtin_env();
        for s in stmts {
            if let Stmt::Varg { name, parents, .. } = unwrap_located(s) {
                let info = ClassInfo { methods: HashMap::new(), parents: parents.clone() };
                self.classes.insert(name.clone(), info);
            }
        }
        for s in stmts {
            self.set_line(s);
            self.infer_stmt(s, &mut env);
        }
        env
    }

    fn set_line(&mut self, s: &Stmt) {
        if let Stmt::Located { line, .. } = s { self.cur_line = *line; }
    }

    fn infer_stmt(&mut self, s: &Stmt, env: &mut HashMap<String, Scheme>) {
        self.set_line(s);
        match unwrap_located(s) {
            Stmt::Vidhi { name, params, body, .. } => {
                let mut fn_env = env.clone();
                let param_tys: Vec<Type> = params.iter().map(|_| self.fresh()).collect();
                for (p, t) in params.iter().zip(param_tys.iter()) {
                    fn_env.insert(p.name.clone(), Scheme { vars: vec![], ty: t.clone() });
                }
                let ret_ty = self.fresh();
                let placeholder = Scheme {
                    vars: vec![],
                    ty: Type::Fun(param_tys.clone(), Box::new(ret_ty.clone())),
                };
                fn_env.insert(name.clone(), placeholder);
                let body_ret = self.infer_body(body, &mut fn_env);
                self.unify_or_err(&ret_ty, &body_ret);
                let fun_ty = self.apply(&Type::Fun(param_tys, Box::new(ret_ty)));
                let sc = self.generalize(env, &fun_ty);
                env.insert(name.clone(), sc);
            }
            Stmt::Assign { name, value, .. } => {
                let t = self.infer_expr(value, env);
                let sc = self.generalize(env, &t);
                env.insert(name.clone(), sc);
            }
            Stmt::ChainAssign { names, value } => {
                let t = self.infer_expr(value, env);
                let sc = self.generalize(env, &t);
                for n in names { env.insert(n.clone(), sc.clone()); }
            }
            Stmt::MultiAssign { names, values } => {
                if values.len() == 1 {
                    let t = self.infer_expr(&values[0], env);
                    let elt = match self.apply(&t) {
                        Type::List(t) => *t,
                        _ => Type::Any,
                    };
                    for n in names {
                        env.insert(n.clone(), Scheme { vars: vec![], ty: elt.clone() });
                    }
                } else {
                    for (n, v) in names.iter().zip(values.iter()) {
                        let t = self.infer_expr(v, env);
                        env.insert(n.clone(), Scheme { vars: vec![], ty: t });
                    }
                }
            }
            Stmt::SthirDecl { name, value } => {
                let t = self.infer_expr(value, env);
                env.insert(name.clone(), Scheme { vars: vec![], ty: t });
            }
            Stmt::Fal(e) => { self.infer_expr(e, env); }
            Stmt::Yield(e) => { self.infer_expr(e, env); }
            Stmt::Print(e) | Stmt::Likho(e) => { self.infer_expr(e, env); }
            Stmt::ExprStmt(e) => { self.infer_expr(e, env); }
            Stmt::Yadi { condition, then, otherwise } => {
                self.infer_expr(condition, env);
                for st in then { self.infer_stmt(st, env); }
                if let Some(otherwise) = otherwise {
                    for st in otherwise { self.infer_stmt(st, env); }
                }
            }
            Stmt::JabTak { condition, body } => {
                self.infer_expr(condition, env);
                for st in body { self.infer_stmt(st, env); }
            }
            Stmt::BarKaro { count, body } => {
                let ct = self.infer_expr(count, env);
                self.unify_or_err(&ct, &Type::Num);
                for st in body { self.infer_stmt(st, env); }
            }
            Stmt::KeeLiye { var, iter, body } => {
                let it = self.infer_expr(iter, env);
                let elt = match self.apply(&it) {
                    Type::List(t) => *t,
                    Type::Str => Type::Str,
                    Type::Dict(t) => *t,
                    Type::Num => Type::Num,
                    _ => Type::Any,
                };
                let mut body_env = env.clone();
                body_env.insert(var.clone(), Scheme { vars: vec![], ty: elt });
                for st in body { self.infer_stmt(st, &mut body_env); }
            }
            Stmt::IndexAssign { obj, idx, val } => {
                let ot = env.get(obj).map(|sc| sc.ty.clone()).unwrap_or(Type::Any);
                let it = self.infer_expr(idx, env);
                let vt = self.infer_expr(val, env);
                match self.apply(&ot) {
                    Type::List(inner) => {
                        self.unify_or_err(&it, &Type::Num);
                        self.unify_or_err(&*inner, &vt);
                    }
                    Type::Dict(inner) => { self.unify_or_err(&*inner, &vt); }
                    _ => {}
                }
            }
            Stmt::SliceAssign { obj, start, end, step, val } => {
                let _ = env.get(obj);
                for e in [start, end, step].iter().copied().flatten() {
                    let et = self.infer_expr(e, env);
                    self.unify_or_err(&et, &Type::Num);
                }
                self.infer_expr(val, env);
            }
            Stmt::AttrAssign { obj: _, field: _, val } => { self.infer_expr(val, env); }
            Stmt::Varg { name, methods, .. } => {
                let mut method_schemes: HashMap<String, Scheme> = HashMap::new();
                for m in methods {
                    if let Stmt::Vidhi { name: mname, params, body, is_static, .. } = unwrap_located(m) {
                        let mut m_env = env.clone();
                        let mut all_params: Vec<Type> = Vec::new();
                        if !*is_static {
                            let self_ty = Type::Instance(name.clone());
                            m_env.insert("यह".into(), Scheme { vars: vec![], ty: self_ty.clone() });
                            all_params.push(self_ty);
                        }
                        for p in params {
                            let t = self.fresh();
                            m_env.insert(p.name.clone(), Scheme { vars: vec![], ty: t.clone() });
                            all_params.push(t);
                        }
                        let body_ret = self.infer_body(body, &mut m_env);
                        let fun_ty = self.apply(&Type::Fun(all_params, Box::new(body_ret)));
                        let sc = self.generalize(env, &fun_ty);
                        method_schemes.insert(mname.clone(), sc);
                    }
                }
                if let Some(info) = self.classes.get_mut(name) {
                    info.methods = method_schemes;
                }
            }
            Stmt::TryCatch { body, clauses } => {
                for st in body { self.infer_stmt(st, env); }
                for c in clauses {
                    let mut c_env = env.clone();
                    c_env.insert(c.var.clone(), Scheme { vars: vec![], ty: Type::Any });
                    for st in &c.body { self.infer_stmt(st, &mut c_env); }
                }
            }
            Stmt::Phenko(e) => { self.infer_expr(e, env); }
            Stmt::Saath { expr, var, body } => {
                let t = self.infer_expr(expr, env);
                let mut b_env = env.clone();
                b_env.insert(var.clone(), Scheme { vars: vec![], ty: t });
                for st in body { self.infer_stmt(st, &mut b_env); }
            }
            Stmt::Milao { subject, arms } => {
                self.infer_expr(subject, env);
                for arm in arms {
                    for st in &arm.body { self.infer_stmt(st, env); }
                }
            }
            Stmt::Jancho { expr, .. } => { self.infer_expr(expr, env); }
            Stmt::Parikshan { body, .. } => {
                for st in body { self.infer_stmt(st, env); }
            }
            Stmt::Located { inner, .. } => self.infer_stmt(inner, env),
            _ => {}
        }
    }

    fn infer_body(&mut self, body: &[Stmt], env: &mut HashMap<String, Scheme>) -> Type {
        let mut ret = Type::Nil;
        for s in body {
            self.set_line(s);
            match unwrap_located(s) {
                Stmt::Fal(e) => {
                    ret = self.infer_expr(e, env);
                    return ret;
                }
                other => self.infer_stmt(other, env),
            }
        }
        ret
    }
}

fn builtin_env() -> HashMap<String, Scheme> {
    let mut env = HashMap::new();
    let poly = |arity: usize, ret_is_arg: bool| {
        let vars: Vec<u32> = (0..arity as u32).collect();
        let args: Vec<Type> = vars.iter().map(|v| Type::Var(*v)).collect();
        let ret = if ret_is_arg && !args.is_empty() { args[0].clone() } else { Type::Any };
        Scheme { vars, ty: Type::Fun(args, Box::new(ret)) }
    };
    let mono = |args: Vec<Type>, ret: Type| Scheme {
        vars: vec![], ty: Type::Fun(args, Box::new(ret))
    };
    env.insert("लम्बाई".into(), Scheme {
        vars: vec![0], ty: Type::Fun(vec![Type::Var(0)], Box::new(Type::Num))
    });
    env.insert("पूर्णांक".into(), mono(vec![Type::Any], Type::Num));
    env.insert("दशमलव".into(), mono(vec![Type::Any], Type::Num));
    env.insert("वाक्य".into(), mono(vec![Type::Any], Type::Str));
    env.insert("बूल".into(), mono(vec![Type::Any], Type::Bool));
    env.insert("पढ़ो".into(), mono(vec![], Type::Str));
    env.insert("यादृच्छिक".into(), mono(vec![Type::Num], Type::Num));
    env.insert("निरपेक्ष".into(), mono(vec![Type::Num], Type::Num));
    env.insert("घात".into(), mono(vec![Type::Num, Type::Num], Type::Num));
    env.insert("वर्गमूल".into(), mono(vec![Type::Num], Type::Num));
    env.insert("गोल".into(), mono(vec![Type::Num], Type::Num));
    env.insert("प्रकार".into(), mono(vec![Type::Any], Type::Str));
    env.insert("वर्ग_का".into(), mono(vec![Type::Any], Type::Str));
    env.insert("है_उदाहरण".into(), mono(vec![Type::Any, Type::Str], Type::Bool));
    env.insert("विशेषताएँ".into(), mono(vec![Type::Any], Type::List(Box::new(Type::Str))));
    env.insert("विधियाँ_का".into(), mono(vec![Type::Any], Type::List(Box::new(Type::Str))));
    env.insert("यूआईडी".into(), mono(vec![], Type::Str));
    env.insert("मानचित्र".into(), poly(2, false));
    env.insert("छानो".into(), poly(2, false));
    env.insert("मोड़ो".into(), poly(3, false));
    env
}

fn free_vars(t: &Type) -> Vec<u32> {
    let mut out = Vec::new();
    fn walk(t: &Type, out: &mut Vec<u32>) {
        match t {
            Type::Var(id) => if !out.contains(id) { out.push(*id); },
            Type::List(t) | Type::Dict(t) => walk(t, out),
            Type::Fun(args, ret) => {
                for a in args { walk(a, out); }
                walk(ret, out);
            }
            Type::Record { fields, rest } => {
                for v in fields.values() { walk(v, out); }
                if let Some(r) = rest {
                    if !out.contains(r) { out.push(*r); }
                }
            }
            _ => {}
        }
    }
    walk(t, &mut out);
    out
}

fn rename(t: &Type, m: &HashMap<u32, Type>) -> Type {
    match t {
        Type::Var(id) => m.get(id).cloned().unwrap_or_else(|| t.clone()),
        Type::List(t) => Type::List(Box::new(rename(t, m))),
        Type::Dict(t) => Type::Dict(Box::new(rename(t, m))),
        Type::Fun(args, ret) => Type::Fun(
            args.iter().map(|a| rename(a, m)).collect(),
            Box::new(rename(ret, m)),
        ),
        Type::Record { fields, rest } => {
            let new_fields: BTreeMap<String, Type> = fields.iter()
                .map(|(k, v)| (k.clone(), rename(v, m)))
                .collect();
            let new_rest = match rest {
                Some(r) => match m.get(r) {
                    Some(Type::Var(id)) => Some(*id),
                    _ => *rest,
                },
                None => None,
            };
            Type::Record { fields: new_fields, rest: new_rest }
        }
        _ => t.clone(),
    }
}

pub fn infer_file(source: &str) -> (Vec<(String, String)>, Vec<TypeError>) {
    let tokens = crate::lexer::tokenize(source);
    let stmts = match crate::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => return (Vec::new(), vec![TypeError { line: 0, message: format!("व्याकरण त्रुटि: {}", e) }]),
    };
    let mut inf = Inferrer::new();
    let env = inf.infer_program(&stmts);
    let builtin_names: std::collections::HashSet<String> = builtin_env().into_keys().collect();
    let mut results: Vec<(String, String)> = env.into_iter()
        .filter(|(n, _)| !builtin_names.contains(n))
        .map(|(name, sc)| (name, inf.apply(&sc.ty).pretty()))
        .collect();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    (results, inf.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn infer_ok(src: &str) -> HashMap<String, String> {
        let (r, e) = infer_file(src);
        assert!(e.is_empty(), "unexpected errors: {:?}", e);
        r.into_iter().collect()
    }

    #[test]
    fn add_returns_num() {
        let r = infer_ok("विधि f(अ, ब):\n    फल अ + ब\n");
        assert!(r["f"].contains("संख्या"), "got: {}", r["f"]);
    }

    #[test]
    fn identity_is_polymorphic() {
        let r = infer_ok("विधि पहचान(x):\n    फल x\n");
        assert!(r["पहचान"].contains("τ"), "got: {}", r["पहचान"]);
    }

    #[test]
    fn string_plus_number_gives_string() {
        let r = infer_ok("विधि f(x):\n    फल \"नमस्ते\" + x\n");
        assert!(r["f"].contains("वाक्य"), "got: {}", r["f"]);
    }

    #[test]
    fn number_minus_string_errors() {
        let (_r, e) = infer_file("विधि f(x):\n    फल x - \"नमस्ते\"\n");
        assert!(!e.is_empty());
    }

    #[test]
    fn list_of_num_inferred() {
        let r = infer_ok("क है [1, 2, 3]\n");
        assert_eq!(r["क"], "सूची<संख्या>");
    }

    #[test]
    fn mixed_list_errors() {
        let (_r, e) = infer_file("क है [1, \"नमस्ते\"]\n");
        assert!(!e.is_empty());
    }

    #[test]
    fn class_instance_type() {
        let r = infer_ok("वर्ग व्यक्ति:\n    विधि बनाओ(n):\n        यह.नाम है n\np है व्यक्ति(\"राम\")\n");
        assert_eq!(r["p"], "व्यक्ति");
    }

    #[test]
    fn method_call_return_type() {
        let src = "वर्ग बॉक्स:\n    विधि बनाओ(v):\n        यह.v है v\n    विधि आयु():\n        फल 42\nb है बॉक्स(1)\nx है b.आयु()\n";
        let r = infer_ok(src);
        assert_eq!(r["x"], "संख्या");
    }

    #[test]
    fn if_else_body_types_flow() {
        let r = infer_ok("विधि f(x):\n    यदि x से अधिक 0:\n        फल 1\n    फल 0\n");
        assert!(r["f"].contains("संख्या"));
    }

    #[test]
    fn comprehension_element_type() {
        let r = infer_ok("क है [i * 2 के लिए i 5 में]\n");
        assert_eq!(r["क"], "सूची<संख्या>");
    }

    #[test]
    fn lambda_type() {
        let r = infer_ok("g है लाम्डा(a, b): a + b\n");
        assert!(r["g"].contains("संख्या"));
    }

    #[test]
    fn subclass_instance_unifies_with_parent() {
        let src = "वर्ग A:\n    विधि बनाओ():\n        यह.x है 1\nवर्ग B(A):\n    विधि बनाओ():\n        यह.x है 2\na है A()\nb है B()\n";
        let r = infer_ok(src);
        assert_eq!(r["a"], "A");
        assert_eq!(r["b"], "B");
    }

    #[test]
    fn builtin_length_returns_num() {
        let r = infer_ok("क है लम्बाई([1, 2, 3])\n");
        assert_eq!(r["क"], "संख्या");
    }

    #[test]
    fn error_carries_line_number() {
        let (_, e) = infer_file("क है 1\nख है \"hi\"\nग है क - ख\n");
        assert!(!e.is_empty());
        assert!(e[0].line > 0, "line should be set, got: {}", e[0].line);
    }

    #[test]
    fn dict_literal_becomes_record() {
        let r = infer_ok("क है {\"नाम\": \"राम\", \"आयु\": 30}\n");
        assert!(r["क"].contains("नाम: वाक्य"), "got: {}", r["क"]);
        assert!(r["क"].contains("आयु: संख्या"), "got: {}", r["क"]);
        assert!(r["क"].starts_with("{") && r["क"].ends_with("}"), "got: {}", r["क"]);
    }

    #[test]
    fn record_field_via_string_index() {
        let src = "व्यक्ति है {\"नाम\": \"राम\", \"आयु\": 30}\nजन्म है व्यक्ति[\"नाम\"]\n";
        let r = infer_ok(src);
        assert_eq!(r["जन्म"], "वाक्य");
    }

    #[test]
    fn row_polymorphic_field_access_in_function() {
        let src = "विधि नाम_का(व्यक्ति):\n    फल व्यक्ति.नाम\n";
        let r = infer_ok(src);
        assert!(r["नाम_का"].contains("->"), "got: {}", r["नाम_का"]);
        assert!(r["नाम_का"].contains("नाम:"), "got: {}", r["नाम_का"]);
    }

    #[test]
    fn closed_record_missing_field_errors() {
        let src = "व्यक्ति है {\"नाम\": \"राम\", \"आयु\": 30}\nक है व्यक्ति.शहर\n";
        let (_r, e) = infer_file(src);
        assert!(!e.is_empty(), "expected a type error for missing field");
    }

    #[test]
    fn comprehension_dict_still_uniform() {
        let r = infer_ok("क है [i * 2 के लिए i 3 में]\n");
        assert_eq!(r["क"], "सूची<संख्या>");
    }
}
