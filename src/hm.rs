use std::collections::HashMap;
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
    Var(u32),
    Any,
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
            Type::Var(id) => format!("τ{}", id),
            Type::Any => "कुछ_भी".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheme {
    pub vars: Vec<u32>,
    pub ty: Type,
}

pub struct Inferrer {
    next_var: u32,
    subst: HashMap<u32, Type>,
    pub errors: Vec<String>,
}

impl Inferrer {
    pub fn new() -> Self { Self { next_var: 0, subst: HashMap::new(), errors: Vec::new() } }

    fn fresh(&mut self) -> Type {
        let v = self.next_var;
        self.next_var += 1;
        Type::Var(v)
    }

    fn apply(&self, t: &Type) -> Type {
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
            _ => t.clone(),
        }
    }

    fn occurs(&self, v: u32, t: &Type) -> bool {
        match self.apply(t) {
            Type::Var(id) => id == v,
            Type::List(t) | Type::Dict(t) => self.occurs(v, &t),
            Type::Fun(args, ret) => args.iter().any(|a| self.occurs(v, a)) || self.occurs(v, &ret),
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
            (a, b) => Err(format!("प्रकार मेल नहीं: {} vs {}", a.pretty(), b.pretty())),
        }
    }

    fn generalize(&self, env: &HashMap<String, Scheme>, t: &Type) -> Scheme {
        let free_in_t = free_vars(&self.apply(t));
        let mut env_vars = std::collections::HashSet::new();
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
            Expr::Ident(name) => match env.get(name) {
                Some(sc) => self.instantiate(&sc.clone()),
                None => Type::Any,
            },
            Expr::Binary { left, op, right } => {
                let lt = self.infer_expr(left, env);
                let rt = self.infer_expr(right, env);
                match op {
                    BinOp::And | BinOp::Or => {
                        Type::Bool
                    }
                    _ => {
                        if let Err(e) = self.unify(&lt, &Type::Num) { self.errors.push(e); }
                        if let Err(e) = self.unify(&rt, &Type::Num) { self.errors.push(e); }
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
            Expr::List(items) => {
                let elt = self.fresh();
                for item in items {
                    let t = self.infer_expr(item, env);
                    if let Err(e) = self.unify(&elt, &t) { self.errors.push(e); }
                }
                Type::List(Box::new(elt))
            }
            Expr::Dict(pairs) => {
                let elt = self.fresh();
                for pair in pairs {
                    let t = self.infer_expr(&pair.1, env);
                    if let Err(e) = self.unify(&elt, &t) { self.errors.push(e); }
                }
                Type::Dict(Box::new(elt))
            }
            Expr::Call { name, args } => {
                let fun_ty = match env.get(name) {
                    Some(sc) => self.instantiate(&sc.clone()),
                    None => return Type::Any,
                };
                let arg_tys: Vec<Type> = args.iter().map(|a| self.infer_expr(a, env)).collect();
                let ret = self.fresh();
                let expected = Type::Fun(arg_tys, Box::new(ret.clone()));
                if let Err(e) = self.unify(&fun_ty, &expected) { self.errors.push(e); }
                ret
            }
            Expr::Ternary { condition, then_val, else_val } => {
                self.infer_expr(condition, env);
                let a = self.infer_expr(then_val, env);
                let b = self.infer_expr(else_val, env);
                if let Err(e) = self.unify(&a, &b) { self.errors.push(e); }
                a
            }
            _ => Type::Any,
        }
    }

    pub fn infer_program(&mut self, stmts: &[Stmt]) -> HashMap<String, Scheme> {
        let mut env: HashMap<String, Scheme> = HashMap::new();
        for s in stmts {
            match unwrap_located(s) {
                Stmt::Vidhi { name, params, body, .. } => {
                    let mut fn_env = env.clone();
                    let mut param_tys: Vec<Type> = Vec::new();
                    for p in params {
                        let t = self.fresh();
                        fn_env.insert(p.name.clone(), Scheme { vars: vec![], ty: t.clone() });
                        param_tys.push(t);
                    }
                    let ret_ty = self.fresh();
                    let placeholder = Scheme {
                        vars: vec![],
                        ty: Type::Fun(param_tys.clone(), Box::new(ret_ty.clone())),
                    };
                    fn_env.insert(name.clone(), placeholder);
                    let body_ret = self.infer_body(body, &mut fn_env);
                    if let Err(e) = self.unify(&ret_ty, &body_ret) { self.errors.push(e); }
                    let fun_ty = self.apply(&Type::Fun(param_tys, Box::new(ret_ty)));
                    let sc = self.generalize(&env, &fun_ty);
                    env.insert(name.clone(), sc);
                }
                Stmt::Assign { name, value, .. } => {
                    let t = self.infer_expr(value, &env);
                    let sc = self.generalize(&env, &t);
                    env.insert(name.clone(), sc);
                }
                _ => {}
            }
        }
        env
    }

    fn infer_body(&mut self, body: &[Stmt], env: &mut HashMap<String, Scheme>) -> Type {
        let mut ret = Type::Nil;
        for s in body {
            match unwrap_located(s) {
                Stmt::Fal(e) => {
                    ret = self.infer_expr(e, env);
                    return ret;
                }
                Stmt::Assign { name, value, .. } => {
                    let t = self.infer_expr(value, env);
                    env.insert(name.clone(), Scheme { vars: vec![], ty: t });
                }
                Stmt::ExprStmt(e) => { self.infer_expr(e, env); }
                _ => {}
            }
        }
        ret
    }
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
        _ => t.clone(),
    }
}

pub fn infer_file(source: &str) -> (Vec<(String, String)>, Vec<String>) {
    let tokens = crate::lexer::tokenize(source);
    let stmts = match crate::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => return (Vec::new(), vec![format!("व्याकरण त्रुटि: {}", e)]),
    };
    let mut inf = Inferrer::new();
    let env = inf.infer_program(&stmts);
    let mut results: Vec<(String, String)> = env.into_iter()
        .map(|(name, sc)| (name, inf.apply(&sc.ty).pretty()))
        .collect();
    results.sort_by(|a, b| a.0.cmp(&b.0));
    (results, inf.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_add_returns_num() {
        let src = "विधि f(अ, ब):\n    फल अ + ब\n";
        let (results, errors) = infer_file(src);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        let ty = results.iter().find(|(n, _)| n == "f").map(|(_, t)| t.clone()).unwrap();
        assert!(ty.contains("संख्या"), "got: {}", ty);
    }

    #[test]
    fn infer_identity_is_polymorphic() {
        let src = "विधि पहचान(x):\n    फल x\n";
        let (results, errors) = infer_file(src);
        assert!(errors.is_empty());
        let ty = results.iter().find(|(n, _)| n == "पहचान").map(|(_, t)| t.clone()).unwrap();
        assert!(ty.contains("τ"), "expected type var, got: {}", ty);
    }

    #[test]
    fn number_string_mismatch_errors() {
        let src = "विधि f(x):\n    फल x + \"नमस्ते\"\n";
        let (_r, errors) = infer_file(src);
        assert!(!errors.is_empty(), "expected an error");
    }
}
