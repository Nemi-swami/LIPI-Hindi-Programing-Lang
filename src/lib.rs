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

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Pre-load stdin lines for पढ़ो() — must be called before run_source() in web mode.
/// Each line in `input` is consumed in order by each पढ़ो() call.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn set_stdin(input: &str) {
    let lines: Vec<String> = input.lines().map(|l| l.to_string()).collect();
    lvm::set_stdin_buffer(lines);
}

/// Run a LIPI source string and return all output as a single String.
/// Called directly from the WASM playground; also usable in native integration tests.
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn run_source(src: &str) -> String {
    let tokens = lexer::tokenize(src);
    match parser::parse(tokens) {
        Ok(stmts) => {
            let program = compiler::Compiler::compile_program(&stmts);
            let mut vm = lvm::LVM::new_capturing();
            match vm.run(&program) {
                Ok(()) => vm.output,
                Err(e) => format!("LVM त्रुटि: {e}"),
            }
        }
        Err(e) => format!("व्याकरण त्रुटि: {e}"),
    }
}
