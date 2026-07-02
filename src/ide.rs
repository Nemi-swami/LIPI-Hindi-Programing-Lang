use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const PORT: u16 = 8790;

pub fn run(open_browser: bool) {
    let root = locate_web_root();
    let root = match root {
        Some(r) => r,
        None => {
            eprintln!("LIPI Studio: web/ फ़ोल्डर नहीं मिला (web/studio/index.html अपेक्षित)");
            eprintln!("WASM बनाएँ: wasm-pack build --target web --out-dir web/pkg --features wasm");
            std::process::exit(2);
        }
    };
    let addr = format!("127.0.0.1:{PORT}");
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => { eprintln!("LIPI Studio: पोर्ट {PORT} नहीं खुला — {e}"); std::process::exit(2); }
    };
    let url = format!("http://{addr}/studio/");
    println!("LIPI Studio चल रहा है: {url}");
    println!("रोकने के लिए Ctrl+C दबाएँ।");
    if open_browser { open_url(&url); }

    for stream in listener.incoming() {
        match stream {
            Ok(s) => { let _ = serve(s, &root); }
            Err(_) => {}
        }
    }
}

fn locate_web_root() -> Option<PathBuf> {
    for cand in ["web", "../web", "../../web"] {
        let p = Path::new(cand);
        if p.join("studio").join("index.html").exists() || p.join("pkg").join("lipi.js").exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

fn serve(mut stream: TcpStream, root: &Path) -> std::io::Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req.lines().next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");
    let path = path.split('?').next().unwrap_or("/");

    let rel = if path == "/" || path == "/studio" || path == "/studio/" {
        "studio/index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };

    let safe = !rel.contains("..");
    let full = root.join(&rel);
    if safe && full.is_file() {
        match std::fs::read(&full) {
            Ok(body) => {
                let ct = content_type(&rel);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes())?;
                stream.write_all(&body)?;
                return Ok(());
            }
            Err(_) => {}
        }
    }
    let body = b"404 not found";
    let header = format!("HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".html") { "text/html; charset=utf-8" }
    else if path.ends_with(".js") { "text/javascript; charset=utf-8" }
    else if path.ends_with(".css") { "text/css; charset=utf-8" }
    else if path.ends_with(".wasm") { "application/wasm" }
    else if path.ends_with(".json") { "application/json; charset=utf-8" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".ts") { "text/plain; charset=utf-8" }
    else { "application/octet-stream" }
}

#[cfg(target_os = "windows")]
fn open_url(url: &str) {
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for exe in candidates {
        if std::path::Path::new(exe).exists() {
            if std::process::Command::new(exe)
                .arg(format!("--app={url}"))
                .arg("--window-size=1280,800")
                .spawn().is_ok() {
                return;
            }
        }
    }
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
}

#[cfg(not(target_os = "windows"))]
fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
