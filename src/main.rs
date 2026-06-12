mod karaka;
mod lexer;
mod ast;
mod parser;
mod interpreter;
mod bharat_stdlib;
mod regex_engine;
mod opcode;
mod compiler;
mod lvm;
mod serializer;
mod editor;
mod tui;
mod roman;
mod phonetic;

use std::io::{self, BufRead, Write};

// ── Core: lex → parse → compile → LVM ────────────────────────────────────────

fn run(source: &str) {
    let tokens = lexer::tokenize(source);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new();
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

// ── REPL ──────────────────────────────────────────────────────────────────────

fn repl() {
    println!("LIPI 3.0 — LVM Edition | भारत की पहली स्वदेशी प्रोग्रामिंग भाषा");
    println!("Paninian Grammar · LVM Bytecode · भारत stdlib");
    println!("बाहर निकलने के लिए 'बाहर' या Ctrl+C दबाएं\n");

    let stdin = io::stdin();
    loop {
        print!("lipi> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed == "बाहर" || trimmed == "exit" || trimmed == "quit" {
                    println!("नमस्ते!");
                    break;
                }
                if !trimmed.is_empty() {
                    run(trimmed);
                }
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

        _ => eprintln!("उपयोग: lipi [build|run|edit|roman|roman-show|phonetic|phonetic-show <फ़ाइल>] | [फ़ाइल.swami|.roman|.vani]"),
    }
}
