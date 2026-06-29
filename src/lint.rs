/// LIPI linter (Phase 17D) — `lipi lint`.
///
/// Walks the parsed AST and reports, per function scope:
///   * assigned-but-never-read local variables,
///   * use of a variable never assigned in scope.
///
/// Deliberately conservative: it would rather miss a warning than emit a false
/// one. Undefined-variable checks only consider value-position identifiers
/// (function-call names are skipped, since imported stdlib functions look like
/// bare names), and `available` sets are over-approximated.
///
/// Findings print as `पंक्ति N: चेतावनी — <message>`. Exit code is always 0
/// (warnings, not errors); a parse error exits 2.

use std::collections::HashSet;
use crate::ast::{self, Stmt, Expr};

/// Entry point used by `main.rs`.
pub fn lint_source(src: &str) {
    let tokens = crate::lexer::tokenize(src);
    let stmts = match crate::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("व्याकरण त्रुटि: {e}");
            std::process::exit(2);
        }
    };

    let builtins = builtins();

    // Names visible everywhere: builtins + every top-level declaration
    // (variables, functions, classes, enums). Functions can read globals at
    // call time regardless of definition order, so we gather them all up front.
    let mut top_avail = builtins.clone();
    visible_decls(&stmts, &mut top_avail);

    let mut warns: Vec<(usize, String)> = Vec::new();
    lint_scope(&stmts, &top_avail, &builtins, &mut warns);

    warns.sort_by_key(|(l, _)| *l);
    warns.dedup();
    if warns.is_empty() {
        println!("कोई समस्या नहीं");
    } else {
        for (l, m) in &warns {
            println!("पंक्ति {}: चेतावनी — {}", l, m);
        }
    }
}

/// Lint one lexical scope (a function body or the top level).
/// `outer_avail` already contains builtins + enclosing-scope names + params.
fn lint_scope(
    body: &[Stmt],
    outer_avail: &HashSet<String>,
    builtins: &HashSet<String>,
    warns: &mut Vec<(usize, String)>,
) {
    // Names available within this scope.
    let mut avail = outer_avail.clone();
    visible_decls(body, &mut avail);

    // ---- undefined-variable check ----
    let mut reads: Vec<(String, usize)> = Vec::new();
    shallow_reads(body, 0, &mut reads);
    for (name, line) in &reads {
        if !avail.contains(name) && !builtins.contains(name) {
            warns.push((*line, format!("चर '{}' पढ़ा गया पर कभी निर्धारित नहीं", name)));
        }
    }

    // ---- assigned-but-never-read check ----
    // Deep reads include nested function / lambda bodies (closures capture
    // outer locals), so a variable used only inside a nested closure is NOT
    // flagged.
    let mut deep: HashSet<String> = HashSet::new();
    deep_reads_block(body, &mut deep);

    let mut assigns: Vec<(String, usize)> = Vec::new();
    let mut global_names: HashSet<String> = HashSet::new();
    collect_assignments(body, 0, &mut assigns, &mut global_names);
    let mut reported: HashSet<(String, usize)> = HashSet::new();
    for (name, line) in &assigns {
        if name.starts_with('_') { continue; }          // intentionally-unused convention
        if global_names.contains(name) { continue; }    // global write — read elsewhere
        if deep.contains(name) { continue; }
        if reported.insert((name.clone(), *line)) {
            warns.push((*line, format!("चर '{}' को मान दिया गया पर कभी पढ़ा नहीं गया", name)));
        }
    }

    // ---- recurse into nested scopes ----
    recurse_scopes(body, &avail, builtins, warns);
}

/// Recurse into nested function definitions, class methods, and lambdas,
/// linting each as its own scope.
fn recurse_scopes(
    body: &[Stmt],
    avail: &HashSet<String>,
    builtins: &HashSet<String>,
    warns: &mut Vec<(usize, String)>,
) {
    for stmt in body {
        match ast::unwrap_located(stmt) {
            Stmt::Vidhi { params, body, vararg, .. } => {
                let mut inner = avail.clone();
                for p in params { inner.insert(p.name.clone()); }
                if let Some(v) = vararg { inner.insert(v.clone()); }
                lint_scope(body, &inner, builtins, warns);
            }
            Stmt::Varg { methods, .. } => {
                for m in methods {
                    if let Stmt::Vidhi { params, body, vararg, is_static, .. } = ast::unwrap_located(m) {
                        let mut inner = avail.clone();
                        if !is_static { inner.insert("यह".to_string()); }
                        for p in params { inner.insert(p.name.clone()); }
                        if let Some(v) = vararg { inner.insert(v.clone()); }
                        lint_scope(body, &inner, builtins, warns);
                    }
                }
            }
            // Control-flow blocks belong to the current scope — descend so the
            // lambdas / nested functions they contain are reached.
            Stmt::Yadi { then, otherwise, .. } => {
                recurse_scopes(then, avail, builtins, warns);
                if let Some(o) = otherwise { recurse_scopes(o, avail, builtins, warns); }
            }
            Stmt::BarKaro { body, .. }
            | Stmt::KeeLiye { body, .. }
            | Stmt::JabTak { body, .. }
            | Stmt::Saath { body, .. }
            | Stmt::Parikshan { body, .. } => recurse_scopes(body, avail, builtins, warns),
            Stmt::TryCatch { body, clauses } => {
                recurse_scopes(body, avail, builtins, warns);
                for c in clauses { recurse_scopes(&c.body, avail, builtins, warns); }
            }
            Stmt::Milao { arms, .. } => {
                for a in arms { recurse_scopes(&a.body, avail, builtins, warns); }
            }
            _ => {}
        }
        // Lambdas embedded in expressions.
        for_each_lambda_in_stmt(stmt, &mut |params, lbody| {
            let mut inner = avail.clone();
            for p in params { inner.insert(p.clone()); }
            lint_scope(lbody, &inner, builtins, warns);
        });
    }
}

// ===== declaration collection (names visible in a scope) =====

/// Add every name a scope introduces: variable assignments, loop/catch/match
/// bindings, walrus targets, globals, and nested definition names. Descends
/// control-flow blocks but treats nested function / lambda bodies as opaque
/// (only their definition name is exported).
fn visible_decls(body: &[Stmt], out: &mut HashSet<String>) {
    for stmt in body {
        match ast::unwrap_located(stmt) {
            Stmt::Assign { name, value, .. } => { out.insert(name.clone()); walrus_in_expr(value, out); }
            Stmt::SthirDecl { name, value } => { out.insert(name.clone()); walrus_in_expr(value, out); }
            Stmt::MultiAssign { names, values } => {
                for n in names { out.insert(n.clone()); }
                for v in values { walrus_in_expr(v, out); }
            }
            Stmt::ChainAssign { names, value } => {
                for n in names { out.insert(n.clone()); }
                walrus_in_expr(value, out);
            }
            Stmt::KeeLiye { var, iter, body } => {
                out.insert(var.clone());
                walrus_in_expr(iter, out);
                visible_decls(body, out);
            }
            Stmt::BarKaro { count, body } => { walrus_in_expr(count, out); visible_decls(body, out); }
            Stmt::Yadi { condition, then, otherwise } => {
                walrus_in_expr(condition, out);
                visible_decls(then, out);
                if let Some(o) = otherwise { visible_decls(o, out); }
            }
            Stmt::JabTak { condition, body } => { walrus_in_expr(condition, out); visible_decls(body, out); }
            Stmt::TryCatch { body, clauses } => {
                visible_decls(body, out);
                for c in clauses { out.insert(c.var.clone()); visible_decls(&c.body, out); }
            }
            Stmt::Milao { subject, arms } => {
                walrus_in_expr(subject, out);
                for a in arms {
                    if let ast::MilaoPattern::Variant(_, binds) = &a.pattern {
                        for b in binds { out.insert(b.clone()); }
                    }
                    visible_decls(&a.body, out);
                }
            }
            Stmt::Saath { expr, var, body } => {
                walrus_in_expr(expr, out);
                out.insert(var.clone());
                visible_decls(body, out);
            }
            Stmt::Global(names) => { for n in names { out.insert(n.clone()); } }
            Stmt::Vidhi { name, .. } => { out.insert(name.clone()); }
            Stmt::Varg { name, .. } => { out.insert(name.clone()); }
            Stmt::ViKalp { name, .. } => { out.insert(name.clone()); }
            Stmt::Parikshan { body, .. } => visible_decls(body, out),
            _ => {}
        }
    }
}

/// Variable assignments at this scope that participate in the unused check.
/// Limited to clear `name है expr` style assignments (Assign / SthirDecl /
/// MultiAssign / ChainAssign) so loop/catch bindings are never flagged.
/// `global_names` collects `वैश्विक` declarations (writes that are read
/// elsewhere — never flagged as unused).
fn collect_assignments(
    body: &[Stmt],
    _depth: usize,
    out: &mut Vec<(String, usize)>,
    global_names: &mut HashSet<String>,
) {
    for stmt in body {
        let line = stmt_line(stmt);
        match ast::unwrap_located(stmt) {
            Stmt::Assign { name, .. } => out.push((name.clone(), line)),
            Stmt::SthirDecl { name, .. } => out.push((name.clone(), line)),
            Stmt::MultiAssign { names, .. } => for n in names { out.push((n.clone(), line)); },
            Stmt::ChainAssign { names, .. } => for n in names { out.push((n.clone(), line)); },
            Stmt::Global(names) => for n in names { global_names.insert(n.clone()); },
            // Descend control flow (same scope), skip nested fn/lambda scopes.
            Stmt::Yadi { then, otherwise, .. } => {
                collect_assignments(then, _depth, out, global_names);
                if let Some(o) = otherwise { collect_assignments(o, _depth, out, global_names); }
            }
            Stmt::BarKaro { body, .. }
            | Stmt::KeeLiye { body, .. }
            | Stmt::JabTak { body, .. }
            | Stmt::Saath { body, .. }
            | Stmt::Parikshan { body, .. } => collect_assignments(body, _depth, out, global_names),
            Stmt::TryCatch { body, clauses } => {
                collect_assignments(body, _depth, out, global_names);
                for c in clauses { collect_assignments(&c.body, _depth, out, global_names); }
            }
            Stmt::Milao { arms, .. } => {
                for a in arms { collect_assignments(&a.body, _depth, out, global_names); }
            }
            _ => {}
        }
    }
}

// ===== read collection =====

/// Value-position identifier reads in this scope, with the line of the
/// enclosing statement. Descends control flow but NOT nested fn/lambda bodies.
fn shallow_reads(body: &[Stmt], _depth: usize, out: &mut Vec<(String, usize)>) {
    for stmt in body {
        let line = stmt_line(stmt);
        let push = |names: Vec<String>, out: &mut Vec<(String, usize)>| {
            for n in names { out.push((n, line)); }
        };
        match ast::unwrap_located(stmt) {
            Stmt::Assign { value, .. } => push(value_idents(value), out),
            Stmt::SthirDecl { value, .. } => push(value_idents(value), out),
            Stmt::Print(e) | Stmt::Likho(e) | Stmt::Fal(e) | Stmt::ExprStmt(e) | Stmt::Phenko(e) =>
                push(value_idents(e), out),
            Stmt::Jancho { expr, message } => {
                push(value_idents(expr), out);
                if let Some(m) = message { push(value_idents(m), out); }
            }
            Stmt::MultiAssign { values, .. } => for v in values { push(value_idents(v), out); },
            Stmt::ChainAssign { value, .. } => push(value_idents(value), out),
            Stmt::IndexAssign { obj, idx, val } => {
                out.push((obj.clone(), line));
                push(value_idents(idx), out);
                push(value_idents(val), out);
            }
            Stmt::AttrAssign { obj, val, .. } => {
                out.push((obj.clone(), line));
                push(value_idents(val), out);
            }
            Stmt::KeeLiye { iter, body, .. } => { push(value_idents(iter), out); shallow_reads(body, _depth, out); }
            Stmt::BarKaro { count, body } => { push(value_idents(count), out); shallow_reads(body, _depth, out); }
            Stmt::Yadi { condition, then, otherwise } => {
                push(value_idents(condition), out);
                shallow_reads(then, _depth, out);
                if let Some(o) = otherwise { shallow_reads(o, _depth, out); }
            }
            Stmt::JabTak { condition, body } => { push(value_idents(condition), out); shallow_reads(body, _depth, out); }
            Stmt::TryCatch { body, clauses } => {
                shallow_reads(body, _depth, out);
                for c in clauses { shallow_reads(&c.body, _depth, out); }
            }
            Stmt::Milao { subject, arms } => {
                push(value_idents(subject), out);
                for a in arms {
                    if let Some(g) = &a.guard { push(value_idents(g), out); }
                    shallow_reads(&a.body, _depth, out);
                }
            }
            Stmt::Saath { expr, body, .. } => { push(value_idents(expr), out); shallow_reads(body, _depth, out); }
            Stmt::Parikshan { body, .. } => shallow_reads(body, _depth, out),
            _ => {}
        }
    }
}

/// All referenced names anywhere in a block — value idents AND call names,
/// descending into every nested scope (functions, lambdas). Used for the
/// unused-assignment check so closures count as readers.
fn deep_reads_block(body: &[Stmt], out: &mut HashSet<String>) {
    for stmt in body {
        deep_reads_stmt(stmt, out);
    }
}

fn deep_reads_stmt(stmt: &Stmt, out: &mut HashSet<String>) {
    match ast::unwrap_located(stmt) {
        Stmt::Assign { value, .. } | Stmt::SthirDecl { value, .. } => all_names(value, out),
        Stmt::Print(e) | Stmt::Likho(e) | Stmt::Fal(e) | Stmt::ExprStmt(e) | Stmt::Phenko(e) => all_names(e, out),
        Stmt::Jancho { expr, message } => {
            all_names(expr, out);
            if let Some(m) = message { all_names(m, out); }
        }
        Stmt::MultiAssign { values, .. } => for v in values { all_names(v, out); },
        Stmt::ChainAssign { value, .. } => all_names(value, out),
        Stmt::IndexAssign { obj, idx, val } => { out.insert(obj.clone()); all_names(idx, out); all_names(val, out); }
        Stmt::AttrAssign { obj, val, .. } => { out.insert(obj.clone()); all_names(val, out); }
        Stmt::KeeLiye { iter, body, .. } => { all_names(iter, out); deep_reads_block(body, out); }
        Stmt::BarKaro { count, body } => { all_names(count, out); deep_reads_block(body, out); }
        Stmt::Yadi { condition, then, otherwise } => {
            all_names(condition, out);
            deep_reads_block(then, out);
            if let Some(o) = otherwise { deep_reads_block(o, out); }
        }
        Stmt::JabTak { condition, body } => { all_names(condition, out); deep_reads_block(body, out); }
        Stmt::TryCatch { body, clauses } => {
            deep_reads_block(body, out);
            for c in clauses { deep_reads_block(&c.body, out); }
        }
        Stmt::Milao { subject, arms } => {
            all_names(subject, out);
            for a in arms {
                if let Some(g) = &a.guard { all_names(g, out); }
                deep_reads_block(&a.body, out);
            }
        }
        Stmt::Saath { expr, body, .. } => { all_names(expr, out); deep_reads_block(body, out); }
        Stmt::Vidhi { body, .. } => deep_reads_block(body, out),
        Stmt::Varg { methods, .. } => for m in methods { deep_reads_stmt(m, out); },
        Stmt::Parikshan { body, .. } => deep_reads_block(body, out),
        _ => {}
    }
}

// ===== expression walkers =====

/// Value-position identifiers in an expression. Does NOT descend into lambda
/// bodies (separate scope) and does NOT collect call names.
fn value_idents(e: &Expr) -> Vec<String> {
    let mut v = Vec::new();
    vi(e, &mut v);
    v
}

fn vi(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Ident(n) => out.push(n.clone()),
        Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) => {}
        Expr::Binary { left, right, .. } | Expr::Compare { left, right, .. } => { vi(left, out); vi(right, out); }
        Expr::Call { args, .. } => for a in args { vi(a, out); },
        Expr::CallKw { args, kwargs, .. } => {
            for a in args { vi(a, out); }
            for (_, v) in kwargs { vi(v, out); }
        }
        Expr::MethodCall { object, args, .. } => { vi(object, out); for a in args { vi(a, out); } }
        Expr::List(items) => for i in items { vi(i, out); },
        Expr::ListWithSpread(items) => for (_, i) in items { vi(i, out); },
        Expr::Dict(pairs) => for (k, val) in pairs { vi(k, out); vi(val, out); },
        Expr::Index { obj, idx } => { vi(obj, out); vi(idx, out); }
        Expr::Slice { obj, start, end, step } => {
            vi(obj, out);
            if let Some(s) = start { vi(s, out); }
            if let Some(s) = end { vi(s, out); }
            if let Some(s) = step { vi(s, out); }
        }
        Expr::Attr { obj, .. } => vi(obj, out),
        Expr::Lambda { .. } => {}                 // separate scope
        Expr::Ternary { condition, then_val, else_val } => { vi(condition, out); vi(then_val, out); vi(else_val, out); }
        Expr::BitNot(x) | Expr::Not(x) => vi(x, out),
        Expr::Membership { item, container, .. } => { vi(item, out); vi(container, out); }
        Expr::Walrus { value, .. } => vi(value, out),   // name is a target, not a read
        Expr::Comprehension { expr, clauses, cond } => {
            // The comprehension binds its own clause variables; its expr/cond
            // reads are handled in the comprehension's own scope sense. Treat
            // the source iterables as reads but skip expr/cond idents that are
            // clause variables to avoid false positives.
            let bound: HashSet<&String> = clauses.iter().map(|(v, _)| v).collect();
            for (_, src) in clauses { vi(src, out); }
            let mut tmp = Vec::new();
            vi(expr, &mut tmp);
            if let Some(c) = cond { vi(c, &mut tmp); }
            for n in tmp { if !bound.contains(&n) { out.push(n); } }
        }
    }
}

/// All referenced names: value idents + call names, descending into lambdas.
fn all_names(e: &Expr, out: &mut HashSet<String>) {
    match e {
        Expr::Ident(n) => { out.insert(n.clone()); }
        Expr::Number(_) | Expr::Str(_) | Expr::Bool(_) => {}
        Expr::Binary { left, right, .. } | Expr::Compare { left, right, .. } => { all_names(left, out); all_names(right, out); }
        Expr::Call { name, args } => { out.insert(name.clone()); for a in args { all_names(a, out); } }
        Expr::CallKw { name, args, kwargs } => {
            out.insert(name.clone());
            for a in args { all_names(a, out); }
            for (_, v) in kwargs { all_names(v, out); }
        }
        Expr::MethodCall { object, args, .. } => { all_names(object, out); for a in args { all_names(a, out); } }
        Expr::List(items) => for i in items { all_names(i, out); },
        Expr::ListWithSpread(items) => for (_, i) in items { all_names(i, out); },
        Expr::Dict(pairs) => for (k, val) in pairs { all_names(k, out); all_names(val, out); },
        Expr::Index { obj, idx } => { all_names(obj, out); all_names(idx, out); }
        Expr::Slice { obj, start, end, step } => {
            all_names(obj, out);
            if let Some(s) = start { all_names(s, out); }
            if let Some(s) = end { all_names(s, out); }
            if let Some(s) = step { all_names(s, out); }
        }
        Expr::Attr { obj, .. } => all_names(obj, out),
        Expr::Lambda { body, .. } => for s in body { deep_reads_stmt(s, out); },
        Expr::Ternary { condition, then_val, else_val } => { all_names(condition, out); all_names(then_val, out); all_names(else_val, out); }
        Expr::BitNot(x) | Expr::Not(x) => all_names(x, out),
        Expr::Membership { item, container, .. } => { all_names(item, out); all_names(container, out); }
        Expr::Walrus { value, .. } => all_names(value, out),
        Expr::Comprehension { expr, clauses, cond } => {
            all_names(expr, out);
            for (_, src) in clauses { all_names(src, out); }
            if let Some(c) = cond { all_names(c, out); }
        }
    }
}

/// Collect walrus assignment targets in an expression (they introduce names).
fn walrus_in_expr(e: &Expr, out: &mut HashSet<String>) {
    match e {
        Expr::Walrus { name, value } => { out.insert(name.clone()); walrus_in_expr(value, out); }
        Expr::Binary { left, right, .. } | Expr::Compare { left, right, .. } => { walrus_in_expr(left, out); walrus_in_expr(right, out); }
        Expr::Call { args, .. } => for a in args { walrus_in_expr(a, out); },
        Expr::CallKw { args, kwargs, .. } => { for a in args { walrus_in_expr(a, out); } for (_, v) in kwargs { walrus_in_expr(v, out); } }
        Expr::MethodCall { object, args, .. } => { walrus_in_expr(object, out); for a in args { walrus_in_expr(a, out); } }
        Expr::List(items) => for i in items { walrus_in_expr(i, out); },
        Expr::ListWithSpread(items) => for (_, i) in items { walrus_in_expr(i, out); },
        Expr::Dict(pairs) => for (k, v) in pairs { walrus_in_expr(k, out); walrus_in_expr(v, out); },
        Expr::Index { obj, idx } => { walrus_in_expr(obj, out); walrus_in_expr(idx, out); }
        Expr::Slice { obj, start, end, step } => {
            walrus_in_expr(obj, out);
            if let Some(s) = start { walrus_in_expr(s, out); }
            if let Some(s) = end { walrus_in_expr(s, out); }
            if let Some(s) = step { walrus_in_expr(s, out); }
        }
        Expr::Attr { obj, .. } => walrus_in_expr(obj, out),
        Expr::Ternary { condition, then_val, else_val } => { walrus_in_expr(condition, out); walrus_in_expr(then_val, out); walrus_in_expr(else_val, out); }
        Expr::BitNot(x) | Expr::Not(x) => walrus_in_expr(x, out),
        Expr::Membership { item, container, .. } => { walrus_in_expr(item, out); walrus_in_expr(container, out); }
        _ => {}
    }
}

/// Invoke `f(params, body)` for each lambda found inside a statement's
/// expressions (top-level lambdas of this statement, not nested control flow).
fn for_each_lambda_in_stmt(stmt: &Stmt, f: &mut dyn FnMut(&[String], &[Stmt])) {
    let visit_expr = |e: &Expr, f: &mut dyn FnMut(&[String], &[Stmt])| lambdas_in_expr(e, f);
    match ast::unwrap_located(stmt) {
        Stmt::Assign { value, .. } | Stmt::SthirDecl { value, .. } => visit_expr(value, f),
        Stmt::Print(e) | Stmt::Likho(e) | Stmt::Fal(e) | Stmt::ExprStmt(e) | Stmt::Phenko(e) => visit_expr(e, f),
        Stmt::Jancho { expr, message } => { visit_expr(expr, f); if let Some(m) = message { visit_expr(m, f); } }
        Stmt::MultiAssign { values, .. } => for v in values { visit_expr(v, f); },
        Stmt::ChainAssign { value, .. } => visit_expr(value, f),
        Stmt::IndexAssign { idx, val, .. } => { visit_expr(idx, f); visit_expr(val, f); }
        Stmt::AttrAssign { val, .. } => visit_expr(val, f),
        Stmt::KeeLiye { iter, .. } => visit_expr(iter, f),
        Stmt::BarKaro { count, .. } => visit_expr(count, f),
        Stmt::Yadi { condition, .. } => visit_expr(condition, f),
        Stmt::JabTak { condition, .. } => visit_expr(condition, f),
        Stmt::Milao { subject, .. } => visit_expr(subject, f),
        Stmt::Saath { expr, .. } => visit_expr(expr, f),
        _ => {}
    }
}

fn lambdas_in_expr(e: &Expr, f: &mut dyn FnMut(&[String], &[Stmt])) {
    match e {
        Expr::Lambda { params, body } => f(params, body),
        Expr::Binary { left, right, .. } | Expr::Compare { left, right, .. } => { lambdas_in_expr(left, f); lambdas_in_expr(right, f); }
        Expr::Call { args, .. } => for a in args { lambdas_in_expr(a, f); },
        Expr::CallKw { args, kwargs, .. } => { for a in args { lambdas_in_expr(a, f); } for (_, v) in kwargs { lambdas_in_expr(v, f); } }
        Expr::MethodCall { object, args, .. } => { lambdas_in_expr(object, f); for a in args { lambdas_in_expr(a, f); } }
        Expr::List(items) => for i in items { lambdas_in_expr(i, f); },
        Expr::ListWithSpread(items) => for (_, i) in items { lambdas_in_expr(i, f); },
        Expr::Dict(pairs) => for (k, v) in pairs { lambdas_in_expr(k, f); lambdas_in_expr(v, f); },
        Expr::Index { obj, idx } => { lambdas_in_expr(obj, f); lambdas_in_expr(idx, f); }
        Expr::Slice { obj, start, end, step } => {
            lambdas_in_expr(obj, f);
            if let Some(s) = start { lambdas_in_expr(s, f); }
            if let Some(s) = end { lambdas_in_expr(s, f); }
            if let Some(s) = step { lambdas_in_expr(s, f); }
        }
        Expr::Attr { obj, .. } => lambdas_in_expr(obj, f),
        Expr::Ternary { condition, then_val, else_val } => { lambdas_in_expr(condition, f); lambdas_in_expr(then_val, f); lambdas_in_expr(else_val, f); }
        Expr::BitNot(x) | Expr::Not(x) => lambdas_in_expr(x, f),
        Expr::Membership { item, container, .. } => { lambdas_in_expr(item, f); lambdas_in_expr(container, f); }
        Expr::Walrus { value, .. } => lambdas_in_expr(value, f),
        Expr::Comprehension { expr, clauses, cond } => {
            lambdas_in_expr(expr, f);
            for (_, s) in clauses { lambdas_in_expr(s, f); }
            if let Some(c) = cond { lambdas_in_expr(c, f); }
        }
        _ => {}
    }
}

// ===== helpers =====

/// Source line of a statement (from its Located wrapper, 0 if absent).
fn stmt_line(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Located { line, .. } => *line,
        _ => 0,
    }
}

/// Built-in functions, constants and special identifiers that are always
/// considered defined.
fn builtins() -> HashSet<String> {
    [
        // builtins
        "लम्बाई", "पूर्णांक", "वाक्य", "__padho__", "यादृच्छिक", "निर्गम",
        "निरपेक्ष", "घात", "वर्गमूल", "गोल", "संचिका_सामग्री", "संचिका_लिखो",
        "संचिका_है", "प्रकार", "मानचित्र", "छानो", "मोड़ो", "तर्क", "पथ_जोड़ो",
        "फोल्डर_सूची", "फोल्डर_बनाओ", "फाइल_हटाओ", "फाइल_कॉपी", "पर्यावरण",
        "वर्तमान_फोल्डर", "स्वरूप", "यूआईडी", "युग्म", "गणना", "श्रृंखला",
        "गिनती_कोश", "कार्तीय", "सर्व_संयोजन",
        // constants / special (pre-loaded globals in LVM::new)
        "पाई", "अनंत", "ऋण_अनंत", "शून्य", "यह", "सत्य", "असत्य", "त्रुटि",
        "अरब", "खरब", "नील", "शंख", "पद्म",
        "आर्यभट_पाई", "आर्यभट_कोण", "आर्यभट_ज्या_गणना", "नक्षत्र_संख्या",
        "तिथि_संख्या", "युग_वर्ष", "ब्रह्मगुप्त_शून्य",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
