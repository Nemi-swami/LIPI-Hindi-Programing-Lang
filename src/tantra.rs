//! भारत.तंत्र — raw machine memory & memory-mapped I/O (Phase 19 F2).
//!
//! The bare-metal / hardware tier. Two capabilities:
//!
//! 1. **Managed buffers** (`स्मृति_*`) — allocate raw bytes on the heap, read/write
//!    them as bytes / 64-bit ints / doubles / strings. This is how you build the
//!    C `struct`s, arrays and out-parameters that `भारत.बाह्य` (FFI) functions
//!    expect: allocate a buffer, fill it, pass its pointer to a driver/SDK, read
//!    the result back. Bounds-checked against the allocation.
//!
//! 2. **Raw volatile access** (`कच्चा_*`) — peek/poke an ABSOLUTE address, 8- and
//!    32-bit, with `read_volatile`/`write_volatile`. This is exactly how software
//!    talks to memory-mapped hardware registers — the transistors behind a device
//!    are exposed to the CPU as addresses; writing a register flips them. Pair with
//!    a kernel driver (reached via FFI) that maps physical device memory into the
//!    process. UNCHECKED by design — a bad address crashes the process, same as C.
//!
//! Pure Rust, no crates. Meaningless inside the WASM sandbox → every function there
//! returns a catchable error.
//!
//!   स्मृति_आवंटन(आकार)                 → allocate zeroed bytes, returns a pointer
//!   स्मृति_मुक्त(सूचक)                  → free it
//!   स्मृति_आकार(सूचक)                   → allocation size
//!   स्मृति_लिखो_बाइट/पूर्ण/दशमलव/वाक्य   → write u8 / i64 / f64 / UTF-8+NUL at offset
//!   स्मृति_पढ़ो_बाइट/पूर्ण/दशमलव         → read them back
//!   स्मृति_वाक्य(सूचक)                  → read a C string at any pointer (for FFI returns)
//!   कच्चा_पढ़ो/लिखो(पता[,मान])          → volatile 8-bit MMIO
//!   कच्चा_पढ़ो३२/लिखो३२(पता[,मान])      → volatile 32-bit MMIO

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    // pointer (heap address) → owning buffer. Keeps the allocation alive and
    // records its size so स्मृति_* accesses can be bounds-checked.
    static BUFS: RefCell<HashMap<usize, Vec<u8>>> = RefCell::new(HashMap::new());
}

#[cfg(not(target_arch = "wasm32"))]
fn num(args: &[Value], i: usize, fname: &str) -> Result<i64, String> {
    match args.get(i) {
        Some(Value::Number(n)) => Ok(*n as i64),
        _ => Err(format!("{fname}(): तर्क {} संख्या होना चाहिए", i + 1)),
    }
}

// ── managed buffers ──────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn mem_alloc(args: Vec<Value>) -> Result<Value, String> {
    let size = num(&args, 0, "स्मृति_आवंटन")?;
    if size <= 0 || size > (1 << 30) {
        return Err("स्मृति_आवंटन(): आकार 1..1GB के बीच होना चाहिए".to_string());
    }
    let mut buf = vec![0u8; size as usize];
    let ptr = buf.as_mut_ptr() as usize; // heap address is stable; we never grow it
    BUFS.with(|m| m.borrow_mut().insert(ptr, buf));
    Ok(Value::Number(ptr as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_free(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_मुक्त")? as usize;
    Ok(Value::Bool(BUFS.with(|m| m.borrow_mut().remove(&ptr)).is_some()))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_size(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_आकार")? as usize;
    BUFS.with(|m| m.borrow().get(&ptr).map(|b| Value::Number(b.len() as f64)))
        .ok_or_else(|| "स्मृति_आकार(): अज्ञात सूचक (क्या यह स्मृति_आवंटन से आया?)".to_string())
}

/// Run `f` over the buffer bytes at `[off, off+width)`, bounds-checked.
#[cfg(not(target_arch = "wasm32"))]
fn with_slice_mut<R>(ptr: usize, off: usize, width: usize, fname: &str, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, String> {
    BUFS.with(|m| {
        let mut map = m.borrow_mut();
        let buf = map.get_mut(&ptr).ok_or_else(|| format!("{fname}(): अज्ञात सूचक"))?;
        if off + width > buf.len() {
            return Err(format!("{fname}(): ऑफसेट {off}+{width} बफ़र आकार {} से बाहर", buf.len()));
        }
        Ok(f(&mut buf[off..off + width]))
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_write_byte(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_लिखो_बाइट")? as usize;
    let off = num(&args, 1, "स्मृति_लिखो_बाइट")? as usize;
    let val = num(&args, 2, "स्मृति_लिखो_बाइट")? as u8;
    with_slice_mut(ptr, off, 1, "स्मृति_लिखो_बाइट", |s| s[0] = val)?;
    Ok(Value::Bool(true))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_read_byte(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_पढ़ो_बाइट")? as usize;
    let off = num(&args, 1, "स्मृति_पढ़ो_बाइट")? as usize;
    let b = with_slice_mut(ptr, off, 1, "स्मृति_पढ़ो_बाइट", |s| s[0])?;
    Ok(Value::Number(b as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_write_int(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_लिखो_पूर्ण")? as usize;
    let off = num(&args, 1, "स्मृति_लिखो_पूर्ण")? as usize;
    let val = num(&args, 2, "स्मृति_लिखो_पूर्ण")?;
    with_slice_mut(ptr, off, 8, "स्मृति_लिखो_पूर्ण", |s| s.copy_from_slice(&val.to_le_bytes()))?;
    Ok(Value::Bool(true))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_read_int(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_पढ़ो_पूर्ण")? as usize;
    let off = num(&args, 1, "स्मृति_पढ़ो_पूर्ण")? as usize;
    let v = with_slice_mut(ptr, off, 8, "स्मृति_पढ़ो_पूर्ण", |s| {
        let mut b = [0u8; 8]; b.copy_from_slice(s); i64::from_le_bytes(b)
    })?;
    Ok(Value::Number(v as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_write_float(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_लिखो_दशमलव")? as usize;
    let off = num(&args, 1, "स्मृति_लिखो_दशमलव")? as usize;
    let val = match args.get(2) { Some(Value::Number(n)) => *n, _ => return Err("स्मृति_लिखो_दशमलव(): तीसरा तर्क संख्या".to_string()) };
    with_slice_mut(ptr, off, 8, "स्मृति_लिखो_दशमलव", |s| s.copy_from_slice(&val.to_le_bytes()))?;
    Ok(Value::Bool(true))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_read_float(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_पढ़ो_दशमलव")? as usize;
    let off = num(&args, 1, "स्मृति_पढ़ो_दशमलव")? as usize;
    let v = with_slice_mut(ptr, off, 8, "स्मृति_पढ़ो_दशमलव", |s| {
        let mut b = [0u8; 8]; b.copy_from_slice(s); f64::from_le_bytes(b)
    })?;
    Ok(Value::Number(v))
}

#[cfg(not(target_arch = "wasm32"))]
fn mem_write_str(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_लिखो_वाक्य")? as usize;
    let off = num(&args, 1, "स्मृति_लिखो_वाक्य")? as usize;
    let s = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("स्मृति_लिखो_वाक्य(): तीसरा तर्क वाक्य".to_string()) };
    let mut bytes = s.into_bytes();
    bytes.push(0); // NUL terminate — ready to pass as C char*
    with_slice_mut(ptr, off, bytes.len(), "स्मृति_लिखो_वाक्य", |dst| dst.copy_from_slice(&bytes))?;
    Ok(Value::Bool(true))
}

/// Read a NUL-terminated C string at any pointer — including one returned by an
/// FFI call, so it need not be a यंत्र-managed buffer.
#[cfg(not(target_arch = "wasm32"))]
fn mem_cstr(args: Vec<Value>) -> Result<Value, String> {
    let ptr = num(&args, 0, "स्मृति_वाक्य")? as usize;
    if ptr == 0 { return Ok(Value::Str(String::new())); }
    let s = unsafe {
        let p = ptr as *const u8;
        let mut len = 0usize;
        while len < (1 << 20) && *p.add(len) != 0 { len += 1; }
        String::from_utf8_lossy(std::slice::from_raw_parts(p, len)).into_owned()
    };
    Ok(Value::Str(s))
}

// ── raw volatile MMIO (absolute addresses, unchecked) ────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn raw_read(args: Vec<Value>) -> Result<Value, String> {
    let addr = num(&args, 0, "कच्चा_पढ़ो")? as usize;
    let v = unsafe { (addr as *const u8).read_volatile() };
    Ok(Value::Number(v as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn raw_write(args: Vec<Value>) -> Result<Value, String> {
    let addr = num(&args, 0, "कच्चा_लिखो")? as usize;
    let val = num(&args, 1, "कच्चा_लिखो")? as u8;
    unsafe { (addr as *mut u8).write_volatile(val) };
    Ok(Value::Bool(true))
}

#[cfg(not(target_arch = "wasm32"))]
fn raw_read32(args: Vec<Value>) -> Result<Value, String> {
    let addr = num(&args, 0, "कच्चा_पढ़ो३२")? as usize;
    let v = unsafe { (addr as *const u32).read_volatile() };
    Ok(Value::Number(v as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn raw_write32(args: Vec<Value>) -> Result<Value, String> {
    let addr = num(&args, 0, "कच्चा_लिखो३२")? as usize;
    let val = num(&args, 1, "कच्चा_लिखो३२")? as u32;
    unsafe { (addr as *mut u32).write_volatile(val) };
    Ok(Value::Bool(true))
}

// ── WASM stubs ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn tantra_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("तंत्र: कच्ची स्मृति WASM सैंडबॉक्स में उपलब्ध नहीं है".to_string())
}

pub fn tantra_registry() -> Registry {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("स्मृति_आवंटन", mem_alloc),
            ("स्मृति_मुक्त", mem_free),
            ("स्मृति_आकार", mem_size),
            ("स्मृति_लिखो_बाइट", mem_write_byte),
            ("स्मृति_पढ़ो_बाइट", mem_read_byte),
            ("स्मृति_लिखो_पूर्ण", mem_write_int),
            ("स्मृति_पढ़ो_पूर्ण", mem_read_int),
            ("स्मृति_लिखो_दशमलव", mem_write_float),
            ("स्मृति_पढ़ो_दशमलव", mem_read_float),
            ("स्मृति_लिखो_वाक्य", mem_write_str),
            ("स्मृति_वाक्य", mem_cstr),
            ("कच्चा_पढ़ो", raw_read),
            ("कच्चा_लिखो", raw_write),
            ("कच्चा_पढ़ो३२", raw_read32),
            ("कच्चा_लिखो३२", raw_write32),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let names = [
            "स्मृति_आवंटन", "स्मृति_मुक्त", "स्मृति_आकार",
            "स्मृति_लिखो_बाइट", "स्मृति_पढ़ो_बाइट", "स्मृति_लिखो_पूर्ण", "स्मृति_पढ़ो_पूर्ण",
            "स्मृति_लिखो_दशमलव", "स्मृति_पढ़ो_दशमलव", "स्मृति_लिखो_वाक्य", "स्मृति_वाक्य",
            "कच्चा_पढ़ो", "कच्चा_लिखो", "कच्चा_पढ़ो३२", "कच्चा_लिखो३२",
        ];
        names.iter().map(|&n| (n, tantra_unavailable as NativeFn)).collect()
    }
}
