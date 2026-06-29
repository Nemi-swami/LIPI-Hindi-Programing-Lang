/// LIPI source formatter (Phase 17D) — `lipi fmt`.
///
/// A line-based reindenter. It does NOT round-trip through the AST (that would
/// drop comments); instead it re-derives indentation from the *relative* block
/// structure of the source — exactly the way the lexer derives INDENT/DEDENT
/// from leading whitespace — and rewrites each level to a clean 4-space indent.
///
/// Guarantees:
///   * Behavior-preserving — the token stream is identical, because we only
///     touch leading/trailing whitespace (which the lexer already trims) and we
///     reproduce the lexer's indent-stack popping logic verbatim, so the
///     INDENT/DEDENT sequence is unchanged.
///   * Idempotent — formatted output uses indents that are exact multiples of 4,
///     which map back through the same stack to the same depths.
///
/// Comments (`# ...` and `। ...`) and string contents are preserved verbatim.

const INDENT_UNIT: usize = 4;

/// Reformat a full source string. Returns the formatted source (always ends
/// with a single trailing newline unless the input was empty).
pub fn format_source(src: &str) -> String {
    // Strip a leading UTF-8 BOM the same way the lexer does, so the output is
    // clean and re-lexes identically.
    let src = src.trim_start_matches('\u{feff}');

    // Mirror of the lexer's `indent_stack` (original indent widths). Depth of a
    // line = stack.len() - 1.
    let mut indent_stack: Vec<usize> = vec![0];

    // Open-bracket depth across lines (multiline collections / calls). While
    // > 0 we are on a continuation line: the lexer ignores its indentation.
    let mut bracket_depth: i32 = 0;
    // Block depth of the line that opened the still-open brackets.
    let mut cont_base_depth: usize = 0;

    let mut out: Vec<String> = Vec::new();

    for raw in src.lines() {
        // trim_end removes a trailing '\r' (CRLF) plus trailing spaces/tabs,
        // matching the lexer's `trim_end()`.
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        // ---- Continuation line (inside open brackets) ----
        if bracket_depth > 0 {
            if trimmed.is_empty() {
                out.push(String::new());
                continue;
            }
            // Closing-bracket-led lines dedent back to the opening line's depth;
            // everything else sits one level in.
            let starts_close = matches!(trimmed.chars().next(), Some(')') | Some(']') | Some('}'));
            let depth = if starts_close { cont_base_depth } else { cont_base_depth + 1 };
            out.push(format!("{}{}", " ".repeat(depth * INDENT_UNIT), trimmed));
            bracket_depth += bracket_delta(trimmed);
            if bracket_depth < 0 { bracket_depth = 0; }
            continue;
        }

        // ---- Blank line ----
        if trimmed.is_empty() {
            out.push(String::new());
            continue;
        }

        // ---- Comment-only line ----
        // The lexer skips these entirely, so they do NOT affect the indent
        // stack. We render them at the current block depth.
        if code_part(trimmed).trim().is_empty() {
            let depth = indent_stack.len() - 1;
            out.push(format!("{}{}", " ".repeat(depth * INDENT_UNIT), trimmed));
            continue;
        }

        // ---- Code line ---- update the indent stack exactly like the lexer.
        let indent = leading_indent(line);
        let cur = *indent_stack.last().unwrap();
        if indent > cur {
            indent_stack.push(indent);
        } else if indent < cur {
            while *indent_stack.last().unwrap() > indent {
                indent_stack.pop();
            }
        }
        let depth = indent_stack.len() - 1;
        out.push(format!("{}{}", " ".repeat(depth * INDENT_UNIT), trimmed));

        // Track multiline collections: a positive net bracket delta opens a
        // continuation region anchored at this line's depth.
        let delta = bracket_delta(trimmed);
        if delta > 0 {
            bracket_depth = delta;
            cont_base_depth = depth;
        }
    }

    let mut result = out.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

/// Net open-bracket delta of a line's code portion. Brackets inside string
/// literals or after a `#` / `।` comment marker are ignored — matching the
/// lexer, which counts brackets from tokens (strings already folded, comments
/// stripped).
fn bracket_delta(s: &str) -> i32 {
    let mut d = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for c in s.chars() {
        if in_str {
            if esc { esc = false; }
            else if c == '\\' { esc = true; }
            else if c == '"' { in_str = false; }
            continue;
        }
        match c {
            '"' => in_str = true,
            '#' | '।' => break,
            '(' | '[' | '{' => d += 1,
            ')' | ']' | '}' => d -= 1,
            _ => {}
        }
    }
    d
}

/// The code portion of a line, with any trailing `#`/`।` comment removed.
/// String-aware so a `#` or `।` inside a string literal is kept. Mirrors the
/// lexer's `strip_comment`.
fn code_part(s: &str) -> &str {
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in s.char_indices() {
        if in_str {
            if esc { esc = false; }
            else if c == '\\' { esc = true; }
            else if c == '"' { in_str = false; }
        } else if c == '"' {
            in_str = true;
        } else if c == '#' || c == '।' {
            return &s[..i];
        }
    }
    s
}

/// Leading-indent width: spaces count 1, tabs count 4 — identical to the
/// lexer's `leading_indent`.
fn leading_indent(line: &str) -> usize {
    let mut n = 0usize;
    for c in line.chars() {
        match c {
            ' ' => n += 1,
            '\t' => n += 4,
            _ => break,
        }
    }
    n
}
