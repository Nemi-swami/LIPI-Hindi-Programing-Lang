mod karaka;
mod types;
mod typecheck;
mod lexer;
mod ast;
mod parser;
mod interpreter;
mod bharat_stdlib;
mod regex_engine;
mod bignum;
mod net;
mod zip;
mod sql;
mod server;
mod threads;
mod matrak;
mod rekhiy;
mod niyantran;
mod disha;
mod suraksha;
mod antaral;
mod ffi;
mod tantra;
mod cbthunk;
mod https;
mod lognaad;
mod tomlparse;
mod yaml;
mod xmlparse;
mod argparse;
mod daak;
mod jit;
mod opcode;
mod compiler;
mod lvm;
mod serializer;
mod editor;
mod tui;
mod roman;
mod phonetic;
mod formatter;
mod lint;
mod docgen;
mod pkg;
mod lsp;
mod flame;
mod ide;

use std::io::{self, BufRead, Write};

// ── Core: lex → parse → compile → LVM ────────────────────────────────────────

fn run(source: &str) {
    let tokens = lexer::tokenize(source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new();
            vm.set_jit(jit::compile_program(&stmts));
            if let Err(e) = vm.run(&program) {
                eprintln!("LVM त्रुटि: {e}");
                show_error_line(source, &e);
            }
        }
        Err(e) => {
            eprintln!("व्याकरण त्रुटि: {e}");
            show_error_line(source, &e);
        }
    }
}

/// `lipi profile foo.swami` — run with opcode/function instrumentation, then
/// print a profile report to stderr (Phase 17D).
fn run_profile(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => {
            if path.ends_with(".roman") || path.ends_with(".r") { roman::roman_to_devanagari(&s) }
            else if path.ends_with(".vani") { phonetic::vani_to_devanagari(&s) }
            else { s }
        }
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new();
            if let Err(e) = vm.run_profiled(&program) {
                eprintln!("LVM त्रुटि: {e}");
                show_error_line(&source, &e);
            }
        }
        Err(e) => { eprintln!("व्याकरण त्रुटि: {e}"); show_error_line(&source, &e); }
    }
}

/// `lipi trace foo.swami` — print a JSON step trace (line + variables per step)
/// for the in-IDE debugger's replay view. Verifiable headless equivalent of the
/// browser stepper.
fn run_trace(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => {
            if path.ends_with(".roman") || path.ends_with(".r") { roman::roman_to_devanagari(&s) }
            else if path.ends_with(".vani") { phonetic::vani_to_devanagari(&s) }
            else { s }
        }
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new();
            println!("{}", vm.run_trace(&program));
        }
        Err(e) => { eprintln!("पार्स त्रुटि: {e}"); std::process::exit(2); }
    }
}

/// `lipi debug foo.swami` — run the program under the interactive line debugger.
fn run_debug(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => {
            if path.ends_with(".roman") || path.ends_with(".r") { roman::roman_to_devanagari(&s) }
            else if path.ends_with(".vani") { phonetic::vani_to_devanagari(&s) }
            else { s }
        }
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new();
            if let Err(e) = vm.run_debug(&program, &source) {
                eprintln!("LVM त्रुटि: {e}");
                show_error_line(&source, &e);
            }
        }
        Err(e) => { eprintln!("व्याकरण त्रुटि: {e}"); show_error_line(&source, &e); }
    }
}

fn run_flame(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => {
            if path.ends_with(".roman") || path.ends_with(".r") { roman::roman_to_devanagari(&s) }
            else if path.ends_with(".vani") { phonetic::vani_to_devanagari(&s) }
            else { s }
        }
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new_capturing();
            match vm.run_flame(&program) {
                Ok(folded) => {
                    print!("{}", flame::folded_to_svg(&folded));
                    eprintln!("\n--- folded stacks ---\n{}", flame::folded_to_text(&folded));
                }
                Err(e) => { eprintln!("LVM त्रुटि: {e}"); show_error_line(&source, &e); }
            }
        }
        Err(e) => { eprintln!("व्याकरण त्रुटि: {e}"); show_error_line(&source, &e); }
    }
}

fn show_error_line(source: &str, msg: &str) {
    // Extract line number from "(line N)" (parse errors) or
    // "(पंक्ति N)" (runtime errors, Phase 17) in the error message
    let marker = if msg.contains("(line ") { "(line " } else { "(पंक्ति " };
    if let Some(start) = msg.find(marker) {
        let rest = &msg[start + marker.len()..];
        if let Some(end) = rest.find(')') {
            if let Ok(n) = rest[..end].trim().parse::<usize>() {
                if let Some(line) = source.lines().nth(n.saturating_sub(1)) {
                    eprintln!("  {:>4} │ {}", n, line);
                    eprintln!("       │ {}", "^".repeat(line.trim().chars().count().max(1)));
                }
            }
        }
    }
}

/// `lipi test foo.swami` — Phase 17 test framework.
/// Each परीक्षण "नाम": block runs in a fresh VM. The file's non-test statements
/// (functions, classes, imports, setup) are re-run before every test so tests
/// stay isolated. जाँचो/runtime failures mark the test failed; the run continues.
fn run_tests(src_path: &str) {
    let source = match std::fs::read_to_string(src_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", src_path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    let stmts = match parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("व्याकरण त्रुटि: {e}");
            show_error_line(&source, &e);
            std::process::exit(2);
        }
    };

    // Partition: परीक्षण blocks vs everything else (shared setup)
    let mut setup: Vec<ast::Stmt> = Vec::new();
    let mut tests: Vec<(String, Vec<ast::Stmt>)> = Vec::new();
    for stmt in stmts {
        if let ast::Stmt::Parikshan { name, body } = ast::unwrap_located(&stmt) {
            tests.push((name.clone(), body.clone()));
        } else {
            setup.push(stmt);
        }
    }

    if tests.is_empty() {
        println!("'{}' में कोई परीक्षण नहीं — परीक्षण \"नाम\": ब्लॉक जोड़ें", src_path);
        return;
    }

    println!("परीक्षण: {} ({} परीक्षण)\n", src_path, tests.len());
    let mut passed = 0usize;
    let mut failed = 0usize;
    let started = std::time::Instant::now();

    for (name, body) in &tests {
        let mut prog_stmts = setup.clone();
        prog_stmts.extend(body.iter().cloned());
        let program = compiler::Compiler::compile_program(&prog_stmts);
        let mut vm = lvm::LVM::new();
        match vm.run(&program) {
            Ok(()) => {
                passed += 1;
                println!("  \x1b[32m✓\x1b[0m {}", name);
            }
            Err(e) => {
                failed += 1;
                println!("  \x1b[31m✗\x1b[0m {}", name);
                for l in e.lines() {
                    println!("      {}", l);
                }
            }
        }
    }

    let ms = started.elapsed().as_millis();
    println!();
    if failed == 0 {
        println!("\x1b[32mसभी {} परीक्षण सफल\x1b[0m ({} ms)", passed, ms);
    } else {
        println!("\x1b[31m{} विफल\x1b[0m, {} सफल — कुल {} ({} ms)", failed, passed, passed + failed, ms);
        std::process::exit(1);
    }
}

fn compile_to_file(src_path: &str) {
    match std::fs::read_to_string(src_path) {
        Ok(source) => {
            let tokens = lexer::tokenize(&source);
            match parser::parse(tokens) {
                Ok(stmts) => {
                    let program = compiler::Compiler::compile_program(&stmts);
                    let out = src_path.replace(".swami", ".libc");
                    match serializer::save(&program, &out) {
                        Ok(()) => println!("✓ {} → {}", src_path, out),
                        Err(e) => eprintln!("सहेज त्रुटि: {e}"),
                    }
                }
                Err(e) => eprintln!("व्याकरण त्रुटि: {e}"),
            }
        }
        Err(e) => eprintln!("फ़ाइल नहीं खुली '{}': {e}", src_path),
    }
}

fn run_libc(path: &str) {
    match serializer::load(path) {
        Ok(program) => {
            let mut vm = lvm::LVM::new();
            if let Err(e) = vm.run(&program) {
                eprintln!("LVM त्रुटि: {e}");
            }
        }
        Err(e) => eprintln!("लोड त्रुटि: {e}"),
    }
}

/// Run a source file, auto-detecting .roman / .vani translation by extension.
fn run_source_file(path: &str) {
    match std::fs::read_to_string(path) {
        Ok(src) => {
            if path.ends_with(".roman") || path.ends_with(".r") {
                run(&roman::roman_to_devanagari(&src));
            } else if path.ends_with(".vani") {
                run(&phonetic::vani_to_devanagari(&src));
            } else {
                run(&src);
            }
        }
        Err(e) => eprintln!("फ़ाइल नहीं खुली: {e}"),
    }
}

// ── Phase 17D tooling: fmt / lint / doc ────────────────────────────────────────

/// `lipi fmt foo.swami`         → print formatted source to stdout
/// `lipi fmt --write foo.swami` → reformat the file in place
fn run_fmt(path: &str, write: bool) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let formatted = formatter::format_source(&source);
    if write {
        match std::fs::write(path, &formatted) {
            Ok(()) => println!("✓ स्वरूपित: {}", path),
            Err(e) => { eprintln!("लिख त्रुटि '{}': {e}", path); std::process::exit(2); }
        }
    } else {
        print!("{}", formatted);
    }
}

/// `lipi lint foo.swami` → report linter warnings (exit 0)
fn run_lint(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    lint::lint_source(&source);
}

/// `lipi check foo.swami` — static type checker (Phase 18 #7). Reports type
/// mismatches without running the program. Exit 0 if clean, 1 if any mismatch,
/// 2 on parse/file error.
fn run_check(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let tokens = lexer::tokenize(&source);
    let stmts = match parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => { eprintln!("व्याकरण त्रुटि: {e}"); show_error_line(&source, &e); std::process::exit(2); }
    };
    let diags = typecheck::check(&stmts);
    if diags.is_empty() {
        println!("✓ कोई प्रकार-त्रुटि नहीं ({})", path);
        return;
    }
    let lines: Vec<&str> = source.lines().collect();
    for d in &diags {
        eprintln!("प्रकार-त्रुटि (पंक्ति {}): {}", d.line, d.message);
        if d.line >= 1 {
            if let Some(line) = lines.get(d.line as usize - 1) {
                eprintln!("  {:>4} │ {}", d.line, line);
            }
        }
    }
    eprintln!("\n{} प्रकार-त्रुटि मिलीं", diags.len());
    std::process::exit(1);
}

/// `lipi doc foo.swami` → emit Markdown documentation to stdout
fn run_doc(path: &str) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("फ़ाइल नहीं खुली '{}': {e}", path); std::process::exit(2); }
    };
    let title = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    print!("{}", docgen::generate(&source, title));
}

// ── REPL ──────────────────────────────────────────────────────────────────────

/// Path to the persistent REPL history file (~/.lipi_history).
fn history_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::Path::new(&home).join(".lipi_history"))
}

/// Count of unclosed ( [ { brackets in `s`, ignoring bracket chars inside string
/// literals. Used to keep reading continuation lines for multiline REPL input.
fn open_bracket_depth(s: &str) -> i32 {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut prev = '\0';
    for c in s.chars() {
        if in_str {
            if c == '"' && prev != '\\' { in_str = false; }
        } else {
            match c {
                '"' => in_str = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }
        prev = c;
    }
    depth
}

fn repl() {
    println!("LIPI 3.0 — LVM Edition | भारत की पहली स्वदेशी प्रोग्रामिंग भाषा");
    println!("Paninian Grammar · LVM Bytecode · भारत stdlib");
    println!("बाहर निकलने के लिए 'बाहर' या Ctrl+C दबाएं · :सहायता मदद के लिए\n");

    // Load persisted history
    let mut history: Vec<String> = Vec::new();
    if let Some(hp) = history_path() {
        if let Ok(text) = std::fs::read_to_string(&hp) {
            history = text.lines().map(|l| l.to_string()).collect();
            if !history.is_empty() {
                println!("({} पूर्व इतिहास पंक्तियाँ लोड हुईं — :इतिहास देखें)\n", history.len());
            }
        }
    }

    // Persistent session: all successfully-run inputs are accumulated so state
    // (variables, functions, classes) carries across prompts. We replay the whole
    // session in a capturing VM each turn and print only the new output delta.
    let mut session = String::new();
    let mut prev_output = String::new();

    let stdin = io::stdin();
    loop {
        print!("lipi> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        match stdin.lock().read_line(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let mut input = buf.trim_end_matches(['\n', '\r']).to_string();
        let trimmed = input.trim();

        // REPL meta-commands
        if trimmed == "बाहर" || trimmed == "exit" || trimmed == "quit" {
            println!("नमस्ते!");
            break;
        }
        if trimmed == ":इतिहास" || trimmed == ":history" {
            let start = history.len().saturating_sub(20);
            for (i, h) in history[start..].iter().enumerate() {
                println!("  {:>3}  {}", start + i + 1, h);
            }
            continue;
        }
        if trimmed == ":सहायता" || trimmed == ":help" {
            println!("  बाहर / exit      — REPL बंद करें");
            println!("  :इतिहास          — पिछली पंक्तियाँ दिखाएँ");
            println!("  :रीसेट           — सत्र अवस्था साफ़ करें");
            println!("  बहु-पंक्ति: ':' से समाप्त पंक्ति या खुले कोष्ठक → खाली पंक्ति तक पढ़ता है");
            continue;
        }
        if trimmed == ":रीसेट" || trimmed == ":reset" {
            session.clear();
            prev_output.clear();
            println!("(सत्र अवस्था साफ़)");
            continue;
        }
        if trimmed.is_empty() { continue; }

        // Multiline continuation: keep reading while the block opener ':' was used
        // or brackets are still open. An empty line ends the block.
        while input.trim_end().ends_with(':') || open_bracket_depth(&input) > 0 {
            print!("..... ");
            io::stdout().flush().unwrap();
            let mut cont = String::new();
            if stdin.lock().read_line(&mut cont).unwrap_or(0) == 0 { break; }
            let cont = cont.trim_end_matches(['\n', '\r']);
            if cont.trim().is_empty() { break; }
            input.push('\n');
            input.push_str(cont);
        }

        // Compile + run the accumulated session plus this input in a capturing VM,
        // then print only the new output. Commit to the session on success.
        let candidate = if session.is_empty() { input.clone() } else { format!("{session}\n{input}") };
        let tokens = lexer::tokenize(&candidate);
        match parser::parse(tokens) {
            Ok(stmts) => {
                let program = compiler::Compiler::compile_program(&stmts);
                let mut vm = lvm::LVM::new_capturing();
                match vm.run(&program) {
                    Ok(()) => {
                        let full = vm.output;
                        let delta = if full.starts_with(&prev_output) { &full[prev_output.len()..] } else { &full[..] };
                        if !delta.is_empty() { println!("{delta}"); }
                        prev_output = full;
                        session = candidate;
                        // persist history
                        history.push(input.replace('\n', " ↵ "));
                        if let Some(hp) = history_path() {
                            let _ = std::fs::write(&hp, history.join("\n"));
                        }
                    }
                    Err(e) => {
                        eprintln!("LVM त्रुटि: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("व्याकरण त्रुटि: {e}");
            }
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.as_slice() {
        // No args → REPL
        [_] => repl(),

        // lipi build foo.swami  → compile to foo.libc
        [_, cmd, path] if cmd == "build" => compile_to_file(path),

        // lipi run foo.libc [a b c]  → execute precompiled bytecode (args → तर्क())
        [_, cmd, path, rest @ ..] if cmd == "run" => {
            lvm::set_script_args(rest.to_vec());
            run_libc(path);
        }

        // lipi edit foo.swami  → open TUI editor
        [_, cmd, path] if cmd == "edit" => tui::TuiEditor::open(path).run(),

        // lipi test foo.swami  → run परीक्षण blocks (Phase 17 test framework)
        [_, cmd, path] if cmd == "test" => run_tests(path),

        // lipi fmt --write foo.swami → reformat the file in place
        [_, cmd, flag, path] if cmd == "fmt" && flag == "--write" => run_fmt(path, true),

        // lipi fmt foo.swami  → print formatted source to stdout
        [_, cmd, path] if cmd == "fmt" => run_fmt(path, false),

        // lipi lint foo.swami → report linter warnings (Phase 17D)
        [_, cmd, path] if cmd == "lint" => run_lint(path),

        // lipi check foo.swami → static type checker (Phase 18 #7)
        [_, cmd, path] if cmd == "check" => run_check(path),

        // lipi doc foo.swami  → emit Markdown documentation (Phase 17D)
        [_, cmd, path] if cmd == "doc" => run_doc(path),

        // lipi profile --flame foo.swami → emit an SVG flame graph to stdout (Phase 18)
        [_, cmd, flag, path] if cmd == "profile" && flag == "--flame" => run_flame(path),

        // lipi profile foo.swami → run with opcode/function profiling (Phase 17D)
        [_, cmd, path] if cmd == "profile" => run_profile(path),

        // lipi pkg <sub> [args] → package manager (Phase 17D)
        [_, cmd, rest @ ..] if cmd == "pkg" => pkg::run(rest),

        // lipi lsp → Language Server Protocol over stdio (Phase 17D)
        [_, cmd] if cmd == "lsp" => lsp::run(),

        // lipi ide [--no-open] → launch LIPI Studio (browser IDE) (Phase 18)
        [_, cmd] if cmd == "ide" => ide::run(true),
        [_, cmd, flag] if cmd == "ide" && flag == "--no-open" => ide::run(false),

        // lipi debug foo.swami → interactive line debugger (Phase 17D)
        [_, cmd, path] if cmd == "debug" => run_debug(path),

        // lipi trace foo.swami → JSON step trace for the in-IDE debugger (Phase I)
        [_, cmd, path] if cmd == "trace" => run_trace(path),

        // lipi foo.libc [a b c] → execute precompiled bytecode (args → तर्क())
        [_, path, rest @ ..] if path.ends_with(".libc") => {
            lvm::set_script_args(rest.to_vec());
            run_libc(path);
        }

        // lipi roman foo.roman  → translate Roman → Devanagari, then run
        [_, cmd, path] if cmd == "roman" => {
            match std::fs::read_to_string(path) {
                Ok(src) => run(&roman::roman_to_devanagari(&src)),
                Err(e)  => eprintln!("फ़ाइल नहीं खुली '{}': {e}", path),
            }
        }

        // lipi roman-show foo.roman → print translated source (for debugging)
        [_, cmd, path] if cmd == "roman-show" => {
            match std::fs::read_to_string(path) {
                Ok(src) => print!("{}", roman::roman_to_devanagari(&src)),
                Err(e)  => eprintln!("फ़ाइल नहीं खुली '{}': {e}", path),
            }
        }

        // lipi phonetic foo.vani → phonetic Roman → Devanagari, then run
        [_, cmd, path] if cmd == "phonetic" => {
            match std::fs::read_to_string(path) {
                Ok(src) => run(&phonetic::vani_to_devanagari(&src)),
                Err(e)  => eprintln!("फ़ाइल नहीं खुली '{}': {e}", path),
            }
        }

        // lipi phonetic-show foo.vani → print phonetically translated source
        [_, cmd, path] if cmd == "phonetic-show" => {
            match std::fs::read_to_string(path) {
                Ok(src) => print!("{}", phonetic::vani_to_devanagari(&src)),
                Err(e)  => eprintln!("फ़ाइल नहीं खुली '{}': {e}", path),
            }
        }

        // lipi foo.swami  → compile + run (auto-detect .roman / .vani too)
        [_, path] => run_source_file(path),

        // lipi foo.swami a b c → run with script args available via तर्क()
        [_, path, rest @ ..]
            if path.ends_with(".swami")
                || path.ends_with(".roman")
                || path.ends_with(".r")
                || path.ends_with(".vani") =>
        {
            lvm::set_script_args(rest.to_vec());
            run_source_file(path);
        }

        _ => eprintln!("उपयोग: lipi [build|run|edit|test|fmt|fmt --write|lint|doc|profile|debug|pkg|lsp|roman|roman-show|phonetic|phonetic-show <फ़ाइल>] | [फ़ाइल.swami|.roman|.vani]"),
    }
}
