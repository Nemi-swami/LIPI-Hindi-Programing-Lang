//! भारत.डाक — send email over SMTP (Phase 19 F10).
//!
//! Sends via the system `curl` (present on Windows 10+, Linux, and macOS), which
//! handles SMTP/SMTPS + STARTTLS + auth — so this stays pure-Rust with no TLS/crypto
//! code of its own and works with real providers (Gmail, Outlook, SES, …).
//!
//!   डाक_भेजो(विन्यास)   → सत्य on success; a catchable error otherwise
//!
//! `विन्यास` (a Dict) fields:
//!   सर्वर        smtps URL, e.g. "smtps://smtp.gmail.com:465"
//!   उपयोगकर्ता   login user
//!   पासवर्ड      login password / app-password
//!   से           From address
//!   को           To address
//!   विषय         Subject
//!   संदेश        body text
//!
//! NOTE: built + wired but not exercised against a live mail server here (needs real
//! credentials). It is a thin, well-defined wrapper over curl's SMTP support.

use crate::bharat_stdlib::{NativeFn, Registry};
use crate::interpreter::Value;

#[cfg(not(target_arch = "wasm32"))]
fn field(d: &std::collections::HashMap<String, Value>, key: &str) -> Result<String, String> {
    match d.get(key) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{n}") }),
        _ => Err(format!("डाक_भेजो(): '{key}' चाहिए (वाक्य)")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send(args: Vec<Value>) -> Result<Value, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let d = match args.first() {
        Some(Value::Dict(m)) => m.clone(),
        _ => return Err("डाक_भेजो(): एक विन्यास कोश (Dict) चाहिए".to_string()),
    };
    let server = field(&d, "सर्वर")?;
    let user = field(&d, "उपयोगकर्ता")?;
    let pass = field(&d, "पासवर्ड")?;
    let from = field(&d, "से")?;
    let to = field(&d, "को")?;
    let subject = field(&d, "विषय")?;
    let body = field(&d, "संदेश")?;

    // RFC 822 message with a UTF-8 subject/body.
    let message = format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{body}\r\n"
    );

    let mut child = Command::new("curl")
        .arg("-s").arg("-S").arg("--ssl-reqd")
        .arg(&server)
        .arg("--mail-from").arg(&from)
        .arg("--mail-rcpt").arg(&to)
        .arg("--user").arg(format!("{user}:{pass}"))
        .arg("-T").arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("डाक_भेजो(): curl नहीं चला ({e}) — curl इंस्टॉल करें"))?;

    child.stdin.take()
        .ok_or_else(|| "डाक_भेजो(): stdin नहीं खुला".to_string())?
        .write_all(message.as_bytes())
        .map_err(|e| format!("डाक_भेजो(): संदेश भेजने में त्रुटि — {e}"))?;

    let out = child.wait_with_output().map_err(|e| format!("डाक_भेजो(): {e}"))?;
    if out.status.success() {
        Ok(Value::Bool(true))
    } else {
        Err(format!("डाक_भेजो(): SMTP विफल — {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

#[cfg(target_arch = "wasm32")]
fn send_unavailable(_args: Vec<Value>) -> Result<Value, String> {
    Err("डाक: SMTP WASM में उपलब्ध नहीं है".to_string())
}

pub fn daak_registry() -> Registry {
    #[cfg(not(target_arch = "wasm32"))]
    { let list: Vec<(&'static str, NativeFn)> = vec![("डाक_भेजो", send)]; list }
    #[cfg(target_arch = "wasm32")]
    { let list: Vec<(&'static str, NativeFn)> = vec![("डाक_भेजो", send_unavailable)]; list }
}
