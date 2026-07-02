//! भारत.बाह्य — Foreign Function Interface (Phase 19 F1).
//!
//! Call any C-ABI function in any native shared library (`.dll` on Windows) from
//! LIPI. This is the "no limits" escape hatch: with FFI, LIPI can drive OpenGL,
//! BLAS/LAPACK, hardware SDKs, OS APIs — anything with a C entry point — so the
//! language is no longer bounded by its own stdlib. Same idea as Python's ctypes.
//!
//! Pure Rust — links only `kernel32` (a system library `std` already links), no
//! external crates. WASM has no dynamic loader, so every function errors there.
//!
//!   बाह्य_पुस्तकालय(पथ)              → load a library, returns an opaque handle id
//!   बाह्य_बुलाओ(हैंडल, नाम, ढांचा, …)  → resolve `नाम`, call it, return the result
//!   बाह्य_बंद(हैंडल)                → unload the library
//!
//! ── The ढांचा (signature) string ────────────────────────────────────────────
//! `"<args>:<ret>"` — one char per argument, then `:` and one return char.
//!   i  32-bit int        l  64-bit int / pointer
//!   d  double (f64)      s  string (C `char*`)
//!   v  void (return only)
//! Examples:  "ii:i"  int f(int,int)   "s:l"  void* f(char*)   "d:d"  double f(double)
//!
//! LIMITATION (no libffi, pure Rust): a single call is EITHER all integer/pointer/
//! string args, OR all-double args — you cannot mix `d` with `i/l/s` in one call,
//! because integers and floats travel in different CPU registers and dispatching a
//! runtime-built mixed signature needs assembly. Split such calls or wrap in a
//! shim. Covers the overwhelming majority of real C APIs.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
use std::cell::{Cell, RefCell};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(all(not(target_arch = "wasm32"), windows))]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(name: *const u8) -> *mut core::ffi::c_void;
    fn GetProcAddress(module: *mut core::ffi::c_void, name: *const u8) -> *mut core::ffi::c_void;
    fn FreeLibrary(module: *mut core::ffi::c_void) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static LIBS: RefCell<HashMap<u64, usize>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

#[cfg(not(target_arch = "wasm32"))]
fn cstr(s: &str) -> Vec<u8> {
    let mut b = s.as_bytes().to_vec();
    b.push(0);
    b
}

// ── library load / unload ────────────────────────────────────────────────────

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn lib_open(args: Vec<Value>) -> Result<Value, String> {
    let path = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("बाह्य_पुस्तकालय(): पथ (वाक्य) अपेक्षित".to_string()),
    };
    let cpath = cstr(&path);
    let handle = unsafe { LoadLibraryA(cpath.as_ptr()) };
    if handle.is_null() {
        return Err(format!("बाह्य_पुस्तकालय(): लोड नहीं हुआ '{path}' (क्या फ़ाइल मौजूद है?)"));
    }
    let id = NEXT_ID.with(|n| { let v = n.get(); n.set(v + 1); v });
    LIBS.with(|m| m.borrow_mut().insert(id, handle as usize));
    Ok(Value::Number(id as f64))
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn lib_close(args: Vec<Value>) -> Result<Value, String> {
    let id = match args.first() {
        Some(Value::Number(n)) => *n as u64,
        _ => return Err("बाह्य_बंद(): हैंडल (संख्या) अपेक्षित".to_string()),
    };
    let removed = LIBS.with(|m| m.borrow_mut().remove(&id));
    match removed {
        Some(h) => { unsafe { FreeLibrary(h as *mut core::ffi::c_void); } Ok(Value::Bool(true)) }
        None => Ok(Value::Bool(false)),
    }
}

// ── the call dispatcher ──────────────────────────────────────────────────────

#[cfg(all(not(target_arch = "wasm32"), windows))]
unsafe fn call_int(f: usize, a: &[usize]) -> usize {
    unsafe {
        use core::mem::transmute;
        match a.len() {
            0 => (transmute::<usize, extern "C" fn() -> usize>(f))(),
            1 => (transmute::<usize, extern "C" fn(usize) -> usize>(f))(a[0]),
            2 => (transmute::<usize, extern "C" fn(usize, usize) -> usize>(f))(a[0], a[1]),
            3 => (transmute::<usize, extern "C" fn(usize, usize, usize) -> usize>(f))(a[0], a[1], a[2]),
            4 => (transmute::<usize, extern "C" fn(usize, usize, usize, usize) -> usize>(f))(a[0], a[1], a[2], a[3]),
            5 => (transmute::<usize, extern "C" fn(usize, usize, usize, usize, usize) -> usize>(f))(a[0], a[1], a[2], a[3], a[4]),
            _ => (transmute::<usize, extern "C" fn(usize, usize, usize, usize, usize, usize) -> usize>(f))(a[0], a[1], a[2], a[3], a[4], a[5]),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
unsafe fn call_double(f: usize, a: &[f64]) -> f64 {
    unsafe {
        use core::mem::transmute;
        match a.len() {
            0 => (transmute::<usize, extern "C" fn() -> f64>(f))(),
            1 => (transmute::<usize, extern "C" fn(f64) -> f64>(f))(a[0]),
            2 => (transmute::<usize, extern "C" fn(f64, f64) -> f64>(f))(a[0], a[1]),
            3 => (transmute::<usize, extern "C" fn(f64, f64, f64) -> f64>(f))(a[0], a[1], a[2]),
            _ => (transmute::<usize, extern "C" fn(f64, f64, f64, f64) -> f64>(f))(a[0], a[1], a[2], a[3]),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
unsafe fn read_cstr(ptr: usize) -> String {
    if ptr == 0 { return String::new(); }
    unsafe {
        let p = ptr as *const u8;
        let mut len = 0usize;
        while len < (1 << 20) && *p.add(len) != 0 { len += 1; }
        let slice = std::slice::from_raw_parts(p, len);
        String::from_utf8_lossy(slice).into_owned()
    }
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn lib_call(args: Vec<Value>) -> Result<Value, String> {
    let id = match args.first() {
        Some(Value::Number(n)) => *n as u64,
        _ => return Err("बाह्य_बुलाओ(): पहला तर्क हैंडल (संख्या) होना चाहिए".to_string()),
    };
    let fname = match args.get(1) {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("बाह्य_बुलाओ(): दूसरा तर्क फलन-नाम (वाक्य) होना चाहिए".to_string()),
    };
    let spec = match args.get(2) {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("बाह्य_बुलाओ(): तीसरा तर्क ढांचा (वाक्य) होना चाहिए, जैसे \"ii:i\"".to_string()),
    };
    let (arg_spec, ret_spec) = match spec.split_once(':') {
        Some((a, r)) => (a.to_string(), r.chars().next().unwrap_or('v')),
        None => (spec.clone(), 'v'),
    };
    let call_args = &args[3.min(args.len())..];
    if call_args.len() != arg_spec.chars().count() {
        return Err(format!("बाह्य_बुलाओ(): ढांचा '{}' को {} तर्क चाहिए, मिले {}", arg_spec, arg_spec.chars().count(), call_args.len()));
    }

    let handle = LIBS.with(|m| m.borrow().get(&id).copied());
    let handle = match handle {
        Some(h) => h,
        None => return Err(format!("बाह्य_बुलाओ(): अमान्य हैंडल {id} (पहले बाह्य_पुस्तकालय() से लोड करें)")),
    };
    let cname = cstr(&fname);
    let proc = unsafe { GetProcAddress(handle as *mut core::ffi::c_void, cname.as_ptr()) };
    if proc.is_null() {
        return Err(format!("बाह्य_बुलाओ(): फलन '{fname}' पुस्तकालय में नहीं मिला"));
    }
    let proc = proc as usize;

    let is_double = arg_spec.chars().any(|c| c == 'd') || ret_spec == 'd';

    if is_double {
        // all-double path (math libraries, physics kernels)
        if !arg_spec.chars().all(|c| c == 'd') {
            return Err("बाह्य_बुलाओ(): double (d) को पूर्णांक/वाक्य तर्कों के साथ नहीं मिला सकते — एक ही कॉल में सभी तर्क 'd' हों (pure-Rust FFI सीमा)".to_string());
        }
        if arg_spec.chars().count() > 4 {
            return Err("बाह्य_बुलाओ(): double मोड में अधिकतम 4 तर्क".to_string());
        }
        let mut dargs: Vec<f64> = Vec::new();
        for (i, v) in call_args.iter().enumerate() {
            match v {
                Value::Number(n) => dargs.push(*n),
                _ => return Err(format!("बाह्य_बुलाओ(): तर्क {} 'd' है पर संख्या नहीं मिली", i + 1)),
            }
        }
        let r = unsafe { call_double(proc, &dargs) };
        return Ok(Value::Number(r));
    }

    // integer / pointer / string path
    if arg_spec.chars().count() > 6 {
        return Err("बाह्य_बुलाओ(): पूर्णांक मोड में अधिकतम 6 तर्क".to_string());
    }
    // string buffers must outlive the call — keep them here
    let mut keep: Vec<Vec<u8>> = Vec::new();
    let mut iargs: Vec<usize> = Vec::new();
    for (i, (c, v)) in arg_spec.chars().zip(call_args.iter()).enumerate() {
        match c {
            'i' | 'l' => match v {
                Value::Number(n) => iargs.push(*n as i64 as usize),
                Value::Bool(b) => iargs.push(*b as usize),
                _ => return Err(format!("बाह्य_बुलाओ(): तर्क {} '{}' है पर संख्या नहीं मिली", i + 1, c)),
            },
            's' => match v {
                Value::Str(s) => {
                    let buf = cstr(s);
                    iargs.push(buf.as_ptr() as usize);
                    keep.push(buf);
                }
                _ => return Err(format!("बाह्य_बुलाओ(): तर्क {} 's' है पर वाक्य नहीं मिला", i + 1)),
            },
            other => return Err(format!("बाह्य_बुलाओ(): अज्ञात तर्क-प्रकार '{other}' ढांचे में")),
        }
    }
    let r = unsafe { call_int(proc, &iargs) };
    drop(keep);

    Ok(match ret_spec {
        'v' => Value::Nil,
        'i' => Value::Number(r as u32 as i32 as f64),
        'l' => Value::Number(r as i64 as f64),
        's' => Value::Str(unsafe { read_cstr(r) }),
        other => return Err(format!("बाह्य_बुलाओ(): अज्ञात रिटर्न-प्रकार '{other}'")),
    })
}

// ── platform fallbacks ───────────────────────────────────────────────────────

#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
fn ffi_unsupported(_args: Vec<Value>) -> Result<Value, String> {
    Err("बाह्य: यह FFI बैकएंड अभी केवल Windows पर उपलब्ध है".to_string())
}

#[cfg(target_arch = "wasm32")]
fn ffi_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("बाह्य: FFI WASM में उपलब्ध नहीं है (कोई dynamic loader नहीं)".to_string())
}

pub fn bahya_registry() -> Registry {
    #[cfg(all(not(target_arch = "wasm32"), windows))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("बाह्य_पुस्तकालय", lib_open),
            ("बाह्य_बुलाओ", lib_call),
            ("बाह्य_बंद", lib_close),
        ];
        list
    }
    #[cfg(all(not(target_arch = "wasm32"), not(windows)))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("बाह्य_पुस्तकालय", ffi_unsupported),
            ("बाह्य_बुलाओ", ffi_unsupported),
            ("बाह्य_बंद", ffi_unsupported),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("बाह्य_पुस्तकालय", ffi_unavailable),
            ("बाह्य_बुलाओ", ffi_unavailable),
            ("बाह्य_बंद", ffi_unavailable),
        ];
        list
    }
}
