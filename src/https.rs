//! भारत.सुरक्षित — HTTPS/TLS client (Phase 19 F4).
//!
//! LIPI's original `भारत.http` client is plain-text `http://` only (pure std::net
//! has no TLS). This module adds real HTTPS by linking Windows' system `winhttp.dll`
//! — the same "link a system library" technique the interpreter already uses for
//! `kernel32`. That removes the single biggest limitation for talking to modern web
//! APIs: LLM endpoints (api.anthropic.com, api.openai.com), payment gateways, cloud
//! services — anything behind TLS.
//!
//!   https_पाओ(url [, शीर्षक])          → GET  → {स्थिति, सामग्री}
//!   https_भेजो(url, शरीर [, शीर्षक])    → POST → {स्थिति, सामग्री}
//!
//! `शीर्षक` (headers) is an optional Dict of name→value. Returns a Dict with
//! `स्थिति` (HTTP status Number) and `सामग्री` (response body Str). Transport/TLS
//! failures are catchable errors; a 404/500 is a normal return (check स्थिति).
//! https:// only. WASM and non-Windows return a catchable error.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;
use std::collections::HashMap;

#[cfg(all(not(target_arch = "wasm32"), windows))]
mod win {
    use std::os::raw::c_void;

    pub const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
    pub const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
    pub const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;

    #[link(name = "winhttp")]
    unsafe extern "system" {
        pub fn WinHttpOpen(agent: *const u16, access: u32, proxy: *const u16, bypass: *const u16, flags: u32) -> *mut c_void;
        pub fn WinHttpConnect(session: *mut c_void, server: *const u16, port: u16, reserved: u32) -> *mut c_void;
        pub fn WinHttpOpenRequest(connect: *mut c_void, verb: *const u16, object: *const u16, version: *const u16, referrer: *const u16, accept: *const *const u16, flags: u32) -> *mut c_void;
        pub fn WinHttpSendRequest(req: *mut c_void, headers: *const u16, headers_len: u32, optional: *const u8, optional_len: u32, total_len: u32, context: usize) -> i32;
        pub fn WinHttpReceiveResponse(req: *mut c_void, reserved: *mut c_void) -> i32;
        pub fn WinHttpQueryHeaders(req: *mut c_void, info_level: u32, name: *const u16, buffer: *mut c_void, buffer_len: *mut u32, index: *mut u32) -> i32;
        pub fn WinHttpReadData(req: *mut c_void, buffer: *mut u8, to_read: u32, read: *mut u32) -> i32;
        pub fn WinHttpCloseHandle(h: *mut c_void) -> i32;
    }
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Split `https://host[:port]/path` → (host, port, path). https only.
#[cfg(all(not(target_arch = "wasm32"), windows))]
fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url.strip_prefix("https://")
        .ok_or_else(|| format!("सुरक्षित: केवल https:// समर्थित है (मिला '{url}')"))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().map_err(|_| "सुरक्षित: अमान्य पोर्ट".to_string())?),
        None => (authority.to_string(), 443),
    };
    if host.is_empty() { return Err("सुरक्षित: होस्ट खाली है".to_string()); }
    Ok((host, port, path.to_string()))
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn build_headers(extra: Option<&Value>, fname: &str) -> Result<String, String> {
    let mut out = String::new();
    if let Some(Value::Dict(m)) = extra {
        for (k, v) in m {
            let vs = match v {
                Value::Str(s) => s.clone(),
                Value::Number(n) => if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{n}") },
                _ => return Err(format!("{fname}(): शीर्षक मान वाक्य/संख्या होना चाहिए")),
            };
            out.push_str(&format!("{k}: {vs}\r\n"));
        }
    } else if extra.is_some() && !matches!(extra, Some(Value::Nil)) {
        return Err(format!("{fname}(): शीर्षक एक कोश (Dict) होना चाहिए"));
    }
    Ok(out)
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn request(verb: &str, url: &str, body: &[u8], headers: String, fname: &str) -> Result<Value, String> {
    use win::*;
    use std::os::raw::c_void;
    use std::ptr::{null, null_mut};

    let (host, port, path) = parse_url(url)?;
    let mut status = 0u32;
    let mut content: Vec<u8> = Vec::new();

    unsafe {
        let session = WinHttpOpen(wide("LIPI/1.0").as_ptr(), 0, null(), null(), 0);
        if session.is_null() { return Err(format!("{fname}(): WinHttp सत्र शुरू नहीं हुआ")); }

        // Ensure handles are always closed, even on early error.
        struct Guard(*mut c_void);
        impl Drop for Guard { fn drop(&mut self) { if !self.0.is_null() { unsafe { WinHttpCloseHandle(self.0); } } } }
        let _gs = Guard(session);

        let connect = WinHttpConnect(session, wide(&host).as_ptr(), port, 0);
        if connect.is_null() { return Err(format!("{fname}(): '{host}' से जुड़ नहीं सका")); }
        let _gc = Guard(connect);

        let req = WinHttpOpenRequest(connect, wide(verb).as_ptr(), wide(&path).as_ptr(), null(), null(), null(), WINHTTP_FLAG_SECURE);
        if req.is_null() { return Err(format!("{fname}(): अनुरोध नहीं बना")); }
        let _gr = Guard(req);

        let hdr_w = wide(&headers);
        let hdr_len = if headers.is_empty() { 0 } else { (hdr_w.len() - 1) as u32 };
        let ok = WinHttpSendRequest(req, hdr_w.as_ptr(), hdr_len, body.as_ptr(), body.len() as u32, body.len() as u32, 0);
        if ok == 0 { return Err(format!("{fname}(): अनुरोध भेजा नहीं जा सका (TLS/नेटवर्क विफल)")); }

        if WinHttpReceiveResponse(req, null_mut()) == 0 {
            return Err(format!("{fname}(): प्रतिक्रिया प्राप्त नहीं हुई"));
        }

        let mut len = 4u32;
        let mut idx = 0u32;
        WinHttpQueryHeaders(req, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, null(), &mut status as *mut u32 as *mut c_void, &mut len, &mut idx);

        let mut buf = [0u8; 8192];
        loop {
            let mut read = 0u32;
            if WinHttpReadData(req, buf.as_mut_ptr(), buf.len() as u32, &mut read) == 0 {
                return Err(format!("{fname}(): प्रतिक्रिया पढ़ने में त्रुटि"));
            }
            if read == 0 { break; }
            content.extend_from_slice(&buf[..read as usize]);
            if content.len() > 64 * 1024 * 1024 { return Err(format!("{fname}(): प्रतिक्रिया बहुत बड़ी (>64MB)")); }
        }
    }

    let mut d = HashMap::new();
    d.insert("स्थिति".to_string(), Value::Number(status as f64));
    d.insert("सामग्री".to_string(), Value::Str(String::from_utf8_lossy(&content).into_owned()));
    Ok(Value::Dict(d))
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn https_get(args: Vec<Value>) -> Result<Value, String> {
    let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("https_पाओ(): पहला तर्क url (वाक्य) होना चाहिए".to_string()) };
    let headers = build_headers(args.get(1), "https_पाओ")?;
    request("GET", &url, &[], headers, "https_पाओ")
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn https_post(args: Vec<Value>) -> Result<Value, String> {
    let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("https_भेजो(): पहला तर्क url (वाक्य) होना चाहिए".to_string()) };
    let body = match args.get(1) { Some(Value::Str(s)) => s.clone().into_bytes(), _ => return Err("https_भेजो(): दूसरा तर्क शरीर (वाक्य) होना चाहिए".to_string()) };
    let mut headers = build_headers(args.get(2), "https_भेजो")?;
    if !headers.to_lowercase().contains("content-type") {
        headers.push_str("content-type: application/json\r\n");
    }
    request("POST", &url, &body, headers, "https_भेजो")
}

// ── Unix backend: shell out to curl (universally present; provides TLS) ───────

#[cfg(all(not(target_arch = "wasm32"), unix))]
fn curl_request(verb: &str, url: &str, body: Option<&str>, extra: Option<&Value>, fname: &str) -> Result<Value, String> {
    use std::process::Command;
    if !url.starts_with("https://") {
        return Err(format!("{fname}(): केवल https:// समर्थित है"));
    }
    let mut cmd = Command::new("curl");
    cmd.arg("-s").arg("-S").arg("-X").arg(verb);
    let mut has_ct = false;
    if let Some(Value::Dict(m)) = extra {
        for (k, v) in m {
            if k.eq_ignore_ascii_case("content-type") { has_ct = true; }
            let vs = match v {
                Value::Str(s) => s.clone(),
                Value::Number(n) => if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{n}") },
                _ => return Err(format!("{fname}(): शीर्षक मान वाक्य/संख्या होना चाहिए")),
            };
            cmd.arg("-H").arg(format!("{k}: {vs}"));
        }
    } else if extra.is_some() && !matches!(extra, Some(Value::Nil)) {
        return Err(format!("{fname}(): शीर्षक एक कोश (Dict) होना चाहिए"));
    }
    if let Some(b) = body {
        if !has_ct { cmd.arg("-H").arg("content-type: application/json"); }
        cmd.arg("--data-binary").arg(b);
    }
    cmd.arg("-w").arg("\nLIPISTATUS:%{http_code}");
    cmd.arg(url);

    let out = cmd.output().map_err(|e| format!("{fname}(): curl नहीं चला ({e}) — curl इंस्टॉल करें"))?;
    let s = String::from_utf8_lossy(&out.stdout);
    let (content, status) = match s.rfind("\nLIPISTATUS:") {
        Some(i) => (s[..i].to_string(), s[i + 12..].trim().parse::<f64>().unwrap_or(0.0)),
        None => (s.into_owned(), 0.0),
    };
    if status == 0.0 {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{fname}(): अनुरोध विफल — {}", err.trim()));
    }
    let mut d = HashMap::new();
    d.insert("स्थिति".to_string(), Value::Number(status));
    d.insert("सामग्री".to_string(), Value::Str(content));
    Ok(Value::Dict(d))
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
fn https_get(args: Vec<Value>) -> Result<Value, String> {
    let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("https_पाओ(): पहला तर्क url (वाक्य) होना चाहिए".to_string()) };
    curl_request("GET", &url, None, args.get(1), "https_पाओ")
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
fn https_post(args: Vec<Value>) -> Result<Value, String> {
    let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("https_भेजो(): पहला तर्क url (वाक्य) होना चाहिए".to_string()) };
    let body = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("https_भेजो(): दूसरा तर्क शरीर (वाक्य) होना चाहिए".to_string()) };
    curl_request("POST", &url, Some(&body), args.get(2), "https_भेजो")
}

// ── other-platform fallback (non-Windows, non-Unix) ──────────────────────────

#[cfg(all(not(target_arch = "wasm32"), not(windows), not(unix)))]
fn https_unsupported(_args: Vec<Value>) -> Result<Value, String> {
    Err("सुरक्षित: इस मंच पर HTTPS बैकएंड उपलब्ध नहीं है".to_string())
}

#[cfg(target_arch = "wasm32")]
fn https_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("सुरक्षित: HTTPS WASM में उपलब्ध नहीं है".to_string())
}

pub fn surakshit_registry() -> Registry {
    // Windows (WinHTTP) and Unix (curl) both provide https_पाओ/https_भेजो.
    #[cfg(all(not(target_arch = "wasm32"), any(windows, unix)))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("https_पाओ", https_get),
            ("https_भेजो", https_post),
        ];
        list
    }
    #[cfg(all(not(target_arch = "wasm32"), not(windows), not(unix)))]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("https_पाओ", https_unsupported),
            ("https_भेजो", https_unsupported),
        ];
        list
    }
    #[cfg(target_arch = "wasm32")]
    {
        let list: Vec<(&'static str, NativeFn)> = vec![
            ("https_पाओ", https_unavailable),
            ("https_भेजो", https_unavailable),
        ];
        list
    }
}
