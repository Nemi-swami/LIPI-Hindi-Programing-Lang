//! Parallel execution for LIPI — genuine OS threads via std::thread, no external crates.
//!
//! Exposed as the stdlib module `भारत.सूत्र`. `समानांतर` runs a list of independent
//! LIPI source-code strings, each in its own thread, and collects their captured
//! output in input order. On WASM there are no threads, so every function errors.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
fn run_source(src: &str) -> String {
    let toks = crate::lexer::tokenize(src);
    let stmts = match crate::parser::parse(toks) {
        Ok(s) => s,
        Err(e) => return format!("व्याकरण त्रुटि: {e}"),
    };
    let prog = crate::compiler::Compiler::compile_program(&stmts);
    let mut vm = crate::lvm::LVM::new_capturing();
    if let Err(e) = vm.run(&prog) {
        return format!("त्रुटि: {e}");
    }
    vm.output
}

#[cfg(not(target_arch = "wasm32"))]
fn samanantar(args: Vec<Value>) -> Result<Value, String> {
    let list = match args.into_iter().next() {
        Some(Value::List(l)) => l,
        _ => return Err("समानांतर(): सूची (वाक्य की) अपेक्षित".to_string()),
    };
    let mut sources: Vec<String> = Vec::with_capacity(list.len());
    for v in list {
        match v {
            Value::Str(s) => sources.push(s),
            other => return Err(format!("समानांतर(): हर तत्व वाक्य होना चाहिए, मिला: {other}")),
        }
    }
    let handles: Vec<_> = sources
        .into_iter()
        .map(|src| std::thread::spawn(move || run_source(&src)))
        .collect();
    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        match h.join() {
            Ok(s) => out.push(Value::Str(s.trim_end_matches('\n').to_string())),
            Err(_) => return Err("समानांतर(): सूत्र विफल".to_string()),
        }
    }
    Ok(Value::List(out))
}

#[cfg(not(target_arch = "wasm32"))]
fn sutra_ganana(_args: Vec<Value>) -> Result<Value, String> {
    let n = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(1);
    Ok(Value::Number(n as f64))
}

#[cfg(target_arch = "wasm32")]
fn sutra_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("सूत्र: समानांतर सूत्र WASM में उपलब्ध नहीं हैं".to_string())
}

pub fn sutra_registry() -> Registry {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("समानांतर", samanantar),
            ("सूत्र_गणना", sutra_ganana),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("समानांतर", sutra_unavailable),
            ("सूत्र_गणना", sutra_unavailable),
        ];
        list
    }
}
