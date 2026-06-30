//! Minimal HTTP server for LIPI — pure std::net, no external crates.
//!
//! Exposed as the stdlib module `भारत.सर्वर`. A server is stateful (a listener plus
//! its registered routes), so handles (opaque Number ids) are kept in a thread-local
//! registry. `सर्वर_चलाओ` blocks while it serves requests. On WASM there is no socket
//! support, so every function returns a catchable error.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
use std::cell::{Cell, RefCell};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead, BufReader, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::TcpListener;

#[cfg(not(target_arch = "wasm32"))]
struct Server {
    listener: TcpListener,
    routes: HashMap<String, String>,
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static SERVERS: RefCell<HashMap<u64, Server>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
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
fn server_create(args: Vec<Value>) -> Result<Value, String> {
    let port = match args.first() {
        Some(Value::Number(n)) => *n as u16,
        Some(Value::Str(s)) => s.trim().parse::<u16>().map_err(|_| "सर्वर_बनाओ(): अमान्य पोर्ट".to_string())?,
        _ => return Err("सर्वर_बनाओ(): पोर्ट (संख्या) अपेक्षित".to_string()),
    };
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("सर्वर_बनाओ(): पोर्ट {port} पर बाँध नहीं सका — {e}"))?;
    let id = NEXT_ID.with(|n| { let v = n.get(); n.set(v + 1); v });
    SERVERS.with(|m| m.borrow_mut().insert(id, Server { listener, routes: HashMap::new() }));
    Ok(Value::Number(id as f64))
}

#[cfg(not(target_arch = "wasm32"))]
fn server_route(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सर्वर_मार्ग")?;
    let path = arg_str(&args, 1, "सर्वर_मार्ग")?;
    let body = arg_str(&args, 2, "सर्वर_मार्ग")?;
    SERVERS.with(|m| {
        match m.borrow_mut().get_mut(&id) {
            Some(srv) => { srv.routes.insert(path, body); Ok(Value::Bool(true)) }
            None => Err("सर्वर_मार्ग(): अमान्य हैंडल".to_string()),
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn server_run(args: Vec<Value>) -> Result<Value, String> {
    let id = handle_id(&args, "सर्वर_चलाओ")?;
    let limit = match args.get(1) {
        Some(Value::Number(n)) => *n as u64,
        None => 0,
        _ => return Err("सर्वर_चलाओ(): अधिकतम (संख्या) अपेक्षित".to_string()),
    };
    // Pull the server out so we don't hold the RefCell borrow while blocking on accept.
    let server = SERVERS.with(|m| m.borrow_mut().remove(&id));
    let server = match server {
        Some(s) => s,
        None => return Err("सर्वर_चलाओ(): अमान्य हैंडल".to_string()),
    };
    let mut served: u64 = 0;
    for incoming in server.listener.incoming() {
        let stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        // Drain the rest of the request headers.
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => { if line == "\r\n" || line == "\n" { break; } }
                Err(_) => break,
            }
        }
        let raw_path = request_line.split_whitespace().nth(1).unwrap_or("/");
        // Browsers percent-encode non-ASCII (e.g. Devanagari) paths, so decode
        // before matching against the routes registered as raw UTF-8.
        let path = percent_decode(raw_path);
        let response = match server.routes.get(&path) {
            Some(body) => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.as_bytes().len(), body),
            None => {
                let body = "404 नहीं मिला";
                format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.as_bytes().len(), body)
            }
        };
        let mut stream = reader.into_inner();
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
        served += 1;
        if limit != 0 && served >= limit {
            break;
        }
    }
    Ok(Value::Number(served as f64))
}

/// Decode `%XX` percent-encoding into a UTF-8 string. Invalid escapes or bytes
/// that don't form valid UTF-8 fall back to the original raw text so a malformed
/// path never panics — it just won't match a route. `+` is left as-is (it only
/// means space in query strings, not path segments).
#[cfg(not(target_arch = "wasm32"))]
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| raw.to_string())
}

#[cfg(target_arch = "wasm32")]
fn server_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("सर्वर: HTTP सर्वर WASM में उपलब्ध नहीं है".to_string())
}

pub fn sarvar_registry() -> Registry {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("सर्वर_बनाओ", server_create),
            ("सर्वर_मार्ग", server_route),
            ("सर्वर_चलाओ", server_run),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("सर्वर_बनाओ", server_unavailable),
            ("सर्वर_मार्ग", server_unavailable),
            ("सर्वर_चलाओ", server_unavailable),
        ];
        list
    }
}
