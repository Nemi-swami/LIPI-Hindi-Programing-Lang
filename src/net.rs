//! TCP networking for LIPI — pure std::net, no external crates.
//!
//! Exposed as the stdlib module `भारत.संजाल`. Sockets are stateful, so handles
//! (opaque Number ids) are kept in a thread-local registry; the builtins look them
//! up by id. Single-threaded VM, so a thread-local is sufficient. On WASM there is
//! no real socket support, so every function returns a catchable error.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
use std::cell::{Cell, RefCell};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{TcpListener, TcpStream};

#[cfg(not(target_arch = "wasm32"))]
enum Handle {
    Stream(TcpStream),
    Listener(TcpListener),
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static HANDLES: RefCell<HashMap<u64, Handle>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

#[cfg(not(target_arch = "wasm32"))]
fn alloc_handle(h: Handle) -> u64 {
    let id = NEXT_ID.with(|n| { let v = n.get(); n.set(v + 1); v });
    HANDLES.with(|m| m.borrow_mut().insert(id, h));
    id
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_id(args: &[Value], fname: &str) -> Result<u64, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(*n as u64),
        _ => Err(format!("{fname}(): हैंडल (संख्या) अपेक्षित")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn arg_str(args: &[Value], idx: usize, fname: &str) -> Result<String, String> {
    match args.get(idx) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!("{fname}(): वाक्य अपेक्षित, मिला: {other}")),
        None => Err(format!("{fname}(): पर्याप्त तर्क नहीं")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn addr(args: &[Value], fname: &str) -> Result<String, String> {
    let host = arg_str(args, 0, fname)?;
    let port = match args.get(1) {
        Some(Value::Number(n)) => *n as u16,
        Some(Value::Str(s)) => s.trim().parse::<u16>().map_err(|_| format!("{fname}(): अमान्य पोर्ट"))?,
        _ => return Err(format!("{fname}(): पोर्ट (संख्या) अपेक्षित")),
    };
    Ok(format!("{host}:{port}"))
}

// ── Native functions ─────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn net_connect(args: Vec<Value>) -> Result<Value, String> {
    let a = addr(&args, "सॉकेट_जोड़ो")?;
    let stream = TcpStream::connect(&a).map_err(|e| format!("सॉकेट_जोड़ो(): जुड़ नहीं सका {a} — {e}"))?;
    Ok(Value::Number(alloc_handle(Handle::Stream(stream)) as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn net_listen(args: Vec<Value>) -> Result<Value, String> {
    let a = addr(&args, "सॉकेट_सुनो")?;
    let listener = TcpListener::bind(&a).map_err(|e| format!("सॉकेट_सुनो(): बाँध नहीं सका {a} — {e}"))?;
    Ok(Value::Number(alloc_handle(Handle::Listener(listener)) as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn net_accept(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सॉकेट_स्वीकारो")?;
    let stream = HANDLES.with(|m| {
        match m.borrow().get(&id) {
            Some(Handle::Listener(l)) => l.accept().map(|(s, _)| s)
                .map_err(|e| format!("सॉकेट_स्वीकारो(): {e}")),
            Some(_) => Err("सॉकेट_स्वीकारो(): हैंडल listener नहीं है".to_string()),
            None => Err("सॉकेट_स्वीकारो(): अमान्य हैंडल".to_string()),
        }
    })?;
    Ok(Value::Number(alloc_handle(Handle::Stream(stream)) as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn net_send(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सॉकेट_भेजो")?;
    let data = arg_str(&args, 1, "सॉकेट_भेजो")?;
    HANDLES.with(|m| {
        match m.borrow_mut().get_mut(&id) {
            Some(Handle::Stream(s)) => s.write_all(data.as_bytes())
                .map(|_| Value::Bool(true))
                .map_err(|e| format!("सॉकेट_भेजो(): {e}")),
            Some(_) => Err("सॉकेट_भेजो(): हैंडल stream नहीं है".to_string()),
            None => Err("सॉकेट_भेजो(): अमान्य हैंडल".to_string()),
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn net_recv(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सॉकेट_पढ़ो")?;
    let n = match args.get(1) {
        Some(Value::Number(n)) => *n as usize,
        None => 4096,
        _ => return Err("सॉकेट_पढ़ो(): बाइट संख्या अपेक्षित".to_string()),
    };
    HANDLES.with(|m| {
        match m.borrow_mut().get_mut(&id) {
            Some(Handle::Stream(s)) => {
                let mut buf = vec![0u8; n];
                let got = s.read(&mut buf).map_err(|e| format!("सॉकेट_पढ़ो(): {e}"))?;
                buf.truncate(got);
                Ok(Value::Str(String::from_utf8_lossy(&buf).into_owned()))
            }
            Some(_) => Err("सॉकेट_पढ़ो(): हैंडल stream नहीं है".to_string()),
            None => Err("सॉकेट_पढ़ो(): अमान्य हैंडल".to_string()),
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn net_close(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सॉकेट_बंद")?;
    let removed = HANDLES.with(|m| m.borrow_mut().remove(&id).is_some());
    Ok(Value::Bool(removed)) // dropping the handle closes the socket
}

// ── WASM stubs (no sockets in the browser) ───────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn net_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("संजाल: सॉकेट WASM में उपलब्ध नहीं हैं".to_string())
}

pub fn sanjaal_registry() -> Registry {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("सॉकेट_जोड़ो", net_connect),
            ("सॉकेट_सुनो", net_listen),
            ("सॉकेट_स्वीकारो", net_accept),
            ("सॉकेट_भेजो", net_send),
            ("सॉकेट_पढ़ो", net_recv),
            ("सॉकेट_बंद", net_close),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("सॉकेट_जोड़ो", net_unavailable),
            ("सॉकेट_सुनो", net_unavailable),
            ("सॉकेट_स्वीकारो", net_unavailable),
            ("सॉकेट_भेजो", net_unavailable),
            ("सॉकेट_पढ़ो", net_unavailable),
            ("सॉकेट_बंद", net_unavailable),
        ];
        list
    }
}
