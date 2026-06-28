/// LIPI documentation generator (Phase 17D) — `lipi doc`.
///
/// Extracts every top-level `विधि` (function) and `वर्ग` (class, with its
/// methods) and emits Markdown: a heading per function/class with its parameter
/// list, plus any comment lines (`# ...` or `। ...`) immediately preceding the
/// definition as its doc text.

use crate::ast::{self, Stmt, Param, Expr};

/// Produce a Markdown document for the given source. `title` is used in the
/// top heading (typically the file name).
pub fn generate(src: &str, title: &str) -> String {
    let src = src.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = src.split('\n').map(|l| l.trim_end_matches('\r')).collect();

    let tokens = crate::lexer::tokenize(src);
    let stmts = match crate::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => return format!("# {}\n\n**व्याकरण त्रुटि:** {}\n", title, e),
    };

    let mut out = String::new();
    out.push_str(&format!("# प्रलेखन — {}\n\n", title));

    let mut any = false;
    for stmt in &stmts {
        let line = stmt_line(stmt);
        match ast::unwrap_located(stmt) {
            Stmt::Vidhi { name, params, vararg, pure, .. } => {
                any = true;
                let sig = signature(name, params, vararg);
                let kw = if *pure { "शुद्ध विधि" } else { "विधि" };
                out.push_str(&format!("## {} `{}`\n\n", kw, sig));
                let doc = preceding_comments(&lines, line);
                if !doc.is_empty() {
                    out.push_str(&doc);
                    out.push_str("\n\n");
                }
            }
            Stmt::Varg { name, parent, methods, is_abstract } => {
                any = true;
                let kw = if *is_abstract { "सार वर्ग" } else { "वर्ग" };
                let heading = match parent {
                    Some(p) => format!("{} `{}` ( {} से )", kw, name, p),
                    None => format!("{} `{}`", kw, name),
                };
                out.push_str(&format!("## {}\n\n", heading));
                let doc = preceding_comments(&lines, line);
                if !doc.is_empty() {
                    out.push_str(&doc);
                    out.push_str("\n\n");
                }
                for m in methods {
                    let mline = stmt_line(m);
                    if let Stmt::Vidhi { name: mname, params, vararg, is_static, .. } = ast::unwrap_located(m) {
                        let sig = signature(mname, params, vararg);
                        let prefix = if *is_static { "साझा विधि" } else { "विधि" };
                        out.push_str(&format!("### {} `{}`\n\n", prefix, sig));
                        let mdoc = preceding_comments(&lines, mline);
                        if !mdoc.is_empty() {
                            out.push_str(&mdoc);
                            out.push_str("\n\n");
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if !any {
        out.push_str("_कोई विधि या वर्ग नहीं मिला।_\n");
    }
    out
}

/// Build a `name(param, param=default, *vararg)` signature string.
fn signature(name: &str, params: &[Param], vararg: &Option<String>) -> String {
    let mut parts: Vec<String> = params
        .iter()
        .map(|p| match &p.default {
            Some(d) => format!("{}={}", p.name, expr_repr(d)),
            None => p.name.clone(),
        })
        .collect();
    if let Some(v) = vararg {
        parts.push(format!("*{}", v));
    }
    format!("{}({})", name, parts.join(", "))
}

/// Render a constant default-value expression for a signature.
fn expr_repr(e: &Expr) -> String {
    match e {
        Expr::Number(n) => {
            if n.fract() == 0.0 && n.is_finite() { format!("{}", *n as i64) } else { format!("{}", n) }
        }
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Bool(b) => (if *b { "सत्य" } else { "असत्य" }).to_string(),
        Expr::BitNot(inner) => format!("~{}", expr_repr(inner)),
        Expr::Not(inner) => format!("नहीं {}", expr_repr(inner)),
        Expr::Binary { left, right, .. } => format!("{}…{}", expr_repr(left), expr_repr(right)),
        Expr::Ident(n) => n.clone(),
        _ => "…".to_string(),
    }
}

/// Collect comment lines immediately above `def_line` (1-based). Walks upward,
/// skipping decorator (`@...`) lines, gathering `#`/`।` comments until a blank
/// or code line is reached.
fn preceding_comments(lines: &[&str], def_line: usize) -> String {
    if def_line < 2 {
        return String::new();
    }
    let mut collected: Vec<String> = Vec::new();
    let mut idx = (def_line - 2) as isize; // line directly above (0-based)
    while idx >= 0 {
        let raw = lines[idx as usize].trim();
        if raw.is_empty() {
            break; // blank line ends the doc block
        }
        if raw.starts_with('@') {
            idx -= 1; // decorator — skip, keep scanning upward
            continue;
        }
        if let Some(rest) = raw.strip_prefix('#') {
            collected.push(rest.trim().to_string());
            idx -= 1;
            continue;
        }
        if raw.starts_with('।') {
            let text = raw.trim_matches('।').trim().to_string();
            collected.push(text);
            idx -= 1;
            continue;
        }
        break; // code line — stop
    }
    collected.reverse();
    // Drop empty trailing/leading entries but keep internal blanks minimal.
    let text: Vec<String> = collected.into_iter().filter(|l| !l.is_empty()).collect();
    text.join("  \n")
}

fn stmt_line(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Located { line, .. } => *line,
        _ => 0,
    }
}
