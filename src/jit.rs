//! A native-code JIT for LIPI (Phase 19 F11).
//!
//! Compiles a restricted class of user functions — a single `फल <expr>` whose body
//! is `+`/`-`/`*` arithmetic over numeric parameters and number literals — to actual
//! x86-64 machine code, executed directly. The bytecode VM remains the fallback tier
//! for everything else (loops, calls, strings, division, …), so correctness always
//! has the interpreter to fall back to.
//!
//! Pure Rust, no crates: it emits SSE2 machine code by hand into an executable page
//! (VirtualAlloc on Windows, mmap on Unix) and calls it as `extern "C" fn(*const f64)
//! -> f64`. Division/modulo are deliberately excluded so LIPI's runtime error
//! semantics are never bypassed.
//!
//! **x86-64 only.** The emitted bytes are x86-64 SSE2 instructions, so the JIT is a
//! no-op on other architectures (e.g. Apple Silicon / arm64): `compile_program`
//! returns an empty map and every function runs on the bytecode VM. This keeps the
//! public API (`JitFn`, `call_raw`, `compile_program`) identical on all native
//! targets, so `lvm.rs`/`main.rs` need no `cfg` of their own.
//!
//! Codegen model: an operand stack lives on the native stack. Each value is 8 bytes.
//!   push const c : mov rax,imm64; movq xmm0,rax; sub rsp,8; movsd [rsp],xmm0
//!   load param i : movsd xmm0,[argptr + i*8]; sub rsp,8; movsd [rsp],xmm0
//!   a <op> b     : movsd xmm1,[rsp]; add rsp,8; movsd xmm0,[rsp]; <op>sd xmm0,xmm1; movsd [rsp],xmm0
//!   return       : movsd xmm0,[rsp]; add rsp,8; ret
//! The argument pointer is in rcx (Windows) / rdi (System V).

#![cfg(not(target_arch = "wasm32"))]

use crate::ast::Stmt;
use std::collections::HashMap;

#[cfg(target_arch = "x86_64")]
use crate::ast::{unwrap_located, BinOp, Expr};

/// A compiled native function. Owns its executable page; frees it on drop.
/// On non-x86-64 targets this is never constructed (the JIT is inert there).
pub struct JitFn {
    #[allow(dead_code)]
    ptr: usize,
    pub arity: usize,
}
impl JitFn {
    #[allow(dead_code)]
    pub fn ptr(&self) -> usize { self.ptr }
}

// ── x86-64 implementation ─────────────────────────────────────────────────────
#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::*;

    // ModRM for `movsd xmm0, [argreg + disp8]` — 0x41 = [rcx+d8], 0x47 = [rdi+d8].
    #[cfg(windows)]
    const ARG_MODRM: u8 = 0x41;
    #[cfg(not(windows))]
    const ARG_MODRM: u8 = 0x47;

    fn push_xmm0(c: &mut Vec<u8>) {
        c.extend_from_slice(&[0x48, 0x83, 0xEC, 0x08]);       // sub rsp, 8
        c.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x04, 0x24]); // movsd [rsp], xmm0
    }

    fn emit(c: &mut Vec<u8>, e: &Expr, params: &[String]) -> Option<()> {
        match e {
            Expr::Number(n) => {
                c.extend_from_slice(&[0x48, 0xB8]);                     // mov rax, imm64
                c.extend_from_slice(&n.to_bits().to_le_bytes());
                c.extend_from_slice(&[0x66, 0x48, 0x0F, 0x6E, 0xC0]);   // movq xmm0, rax
                push_xmm0(c);
                Some(())
            }
            Expr::Ident(name) => {
                let idx = params.iter().position(|p| p == name)?;
                if idx >= 16 { return None; } // keep disp within a signed byte
                c.extend_from_slice(&[0xF2, 0x0F, 0x10, ARG_MODRM, (idx * 8) as u8]); // movsd xmm0,[arg+d8]
                push_xmm0(c);
                Some(())
            }
            Expr::Binary { left, op, right } => {
                let opcode: [u8; 4] = match op {
                    BinOp::Add => [0xF2, 0x0F, 0x58, 0xC1], // addsd xmm0, xmm1
                    BinOp::Sub => [0xF2, 0x0F, 0x5C, 0xC1], // subsd xmm0, xmm1
                    BinOp::Mul => [0xF2, 0x0F, 0x59, 0xC1], // mulsd xmm0, xmm1
                    _ => return None, // Div/Mod/etc. left to the VM
                };
                emit(c, left, params)?;
                emit(c, right, params)?;
                c.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x0C, 0x24]); // movsd xmm1, [rsp]  (b)
                c.extend_from_slice(&[0x48, 0x83, 0xC4, 0x08]);       // add rsp, 8
                c.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x04, 0x24]); // movsd xmm0, [rsp]  (a)
                c.extend_from_slice(&opcode);
                c.extend_from_slice(&[0xF2, 0x0F, 0x11, 0x04, 0x24]); // movsd [rsp], xmm0  (result)
                Some(())
            }
            _ => None,
        }
    }

    /// Machine code for a qualifying function body, or None if it can't be JIT'd.
    pub(super) fn codegen(params: &[String], body: &[Stmt]) -> Option<Vec<u8>> {
        // Exactly one statement, a `फल <expr>`.
        let mut stmts = body.iter().map(unwrap_located);
        let first = stmts.next()?;
        if stmts.next().is_some() { return None; }
        let expr = match first {
            Stmt::Fal(e) => e,
            _ => return None,
        };
        let mut code = Vec::new();
        emit(&mut code, expr, params)?;
        code.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x04, 0x24]); // movsd xmm0, [rsp]
        code.extend_from_slice(&[0x48, 0x83, 0xC4, 0x08]);       // add rsp, 8
        code.push(0xC3);                                         // ret
        Some(code)
    }

    // ── executable memory ─────────────────────────────────────────────────────
    #[cfg(windows)]
    pub(super) mod mem {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn VirtualAlloc(addr: *mut core::ffi::c_void, size: usize, typ: u32, protect: u32) -> *mut core::ffi::c_void;
            fn VirtualFree(addr: *mut core::ffi::c_void, size: usize, typ: u32) -> i32;
        }
        pub unsafe fn alloc(code: &[u8]) -> usize {
            unsafe {
                let p = VirtualAlloc(core::ptr::null_mut(), code.len(), 0x3000, 0x40); // COMMIT|RESERVE, EXECUTE_READWRITE
                if p.is_null() { return 0; }
                core::ptr::copy_nonoverlapping(code.as_ptr(), p as *mut u8, code.len());
                p as usize
            }
        }
        pub unsafe fn free(ptr: usize) { if ptr != 0 { unsafe { VirtualFree(ptr as *mut core::ffi::c_void, 0, 0x8000); } } }
    }

    #[cfg(unix)]
    pub(super) mod mem {
        unsafe extern "C" {
            fn mmap(addr: *mut core::ffi::c_void, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut core::ffi::c_void;
            fn munmap(addr: *mut core::ffi::c_void, len: usize) -> i32;
        }
        #[cfg(target_os = "macos")] const MAP_ANON: i32 = 0x1000;
        #[cfg(not(target_os = "macos"))] const MAP_ANON: i32 = 0x20;
        pub unsafe fn alloc(code: &[u8]) -> usize {
            unsafe {
                // PROT_READ|WRITE|EXEC = 7, MAP_PRIVATE = 2
                let p = mmap(core::ptr::null_mut(), code.len(), 7, 2 | MAP_ANON, -1, 0);
                if p as isize == -1 { return 0; }
                core::ptr::copy_nonoverlapping(code.as_ptr(), p as *mut u8, code.len());
                p as usize
            }
        }
        pub unsafe fn free(ptr: usize) { if ptr != 0 { unsafe { munmap(ptr as *mut core::ffi::c_void, 0); } } }
    }
}

#[cfg(target_arch = "x86_64")]
impl Drop for JitFn {
    fn drop(&mut self) { unsafe { x86::mem::free(self.ptr); } }
}
#[cfg(not(target_arch = "x86_64"))]
impl Drop for JitFn {
    fn drop(&mut self) {}
}

/// Call JIT'd code at `ptr` with f64 args. UNSAFE: `ptr` must be a page this module
/// produced for exactly `args.len()` parameters. Only reachable on x86-64 (elsewhere
/// `compile_program` yields no functions, so this is never invoked).
#[cfg(target_arch = "x86_64")]
pub unsafe fn call_raw(ptr: usize, args: &[f64]) -> f64 {
    let f: unsafe extern "C" fn(*const f64) -> f64 = unsafe { core::mem::transmute(ptr) };
    unsafe { f(args.as_ptr()) }
}
#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn call_raw(_ptr: usize, _args: &[f64]) -> f64 {
    unreachable!("LIPI JIT is x86-64 only; no functions are compiled on this target")
}

/// JIT every qualifying top-level function in the program (x86-64 only).
#[cfg(target_arch = "x86_64")]
pub fn compile_program(stmts: &[Stmt]) -> HashMap<String, JitFn> {
    let mut out = HashMap::new();
    for s in stmts {
        if let Stmt::Vidhi { name, params, body, vararg, decorators, is_static, .. } = unwrap_located(s) {
            if vararg.is_some() || !decorators.is_empty() || *is_static { continue; }
            if params.iter().any(|p| p.default.is_some()) { continue; }
            let pnames: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            if let Some(code) = x86::codegen(&pnames, body) {
                let ptr = unsafe { x86::mem::alloc(&code) };
                if ptr != 0 {
                    out.insert(name.clone(), JitFn { ptr, arity: pnames.len() });
                }
            }
        }
    }
    out
}
/// On non-x86-64 targets the JIT is inert — nothing is compiled, the VM runs everything.
#[cfg(not(target_arch = "x86_64"))]
pub fn compile_program(_stmts: &[Stmt]) -> HashMap<String, JitFn> {
    HashMap::new()
}

// Tests execute emitted x86-64 machine code, so they only build/run on x86-64.
#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::x86::{codegen, mem};
    use super::call_raw;
    use crate::ast::{BinOp, Expr, Stmt};
    fn expr_num(n: f64) -> Expr { Expr::Number(n) }
    fn ident(s: &str) -> Expr { Expr::Ident(s.into()) }
    fn bin(l: Expr, op: BinOp, r: Expr) -> Expr { Expr::Binary { left: Box::new(l), op, right: Box::new(r) } }

    #[test]
    fn jit_arithmetic() {
        // f(a, b) = a*a + b*b
        let params = vec!["अ".to_string(), "ब".to_string()];
        let body = vec![Stmt::Fal(bin(
            bin(ident("अ"), BinOp::Mul, ident("अ")),
            BinOp::Add,
            bin(ident("ब"), BinOp::Mul, ident("ब")),
        ))];
        let code = codegen(&params, &body).expect("should JIT");
        let ptr = unsafe { mem::alloc(&code) };
        assert!(ptr != 0);
        let r = unsafe { call_raw(ptr, &[3.0, 4.0]) };
        unsafe { mem::free(ptr); }
        assert_eq!(r, 25.0);
    }

    #[test]
    fn jit_const_and_params() {
        // g(x) = x * 2 - 1
        let params = vec!["x".to_string()];
        let body = vec![Stmt::Fal(bin(bin(ident("x"), BinOp::Mul, expr_num(2.0)), BinOp::Sub, expr_num(1.0)))];
        let code = codegen(&params, &body).unwrap();
        let ptr = unsafe { mem::alloc(&code) };
        let r = unsafe { call_raw(ptr, &[10.0]) };
        unsafe { mem::free(ptr); }
        assert_eq!(r, 19.0);
    }

    #[test]
    fn rejects_division() {
        let params = vec!["a".to_string()];
        let body = vec![Stmt::Fal(bin(ident("a"), BinOp::Div, Expr::Number(2.0)))];
        assert!(codegen(&params, &body).is_none());
    }
}
