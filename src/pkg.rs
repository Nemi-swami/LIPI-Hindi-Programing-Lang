//! LIPI package manager — pure Rust, no external crates.
//!
//! A minimal local package manager built around a `lipi.toml` manifest and a
//! `lipi_modules/` install directory. Dependencies are local paths (there is no
//! central registry / network host). Installed packages become importable by name
//! (`आयात "नाम"` resolves to `lipi_modules/नाम.swami` — see ImportFile in lvm.rs).
//!
//!   lipi pkg init                 — scaffold a lipi.toml
//!   lipi pkg add <नाम> <पथ>       — add a path dependency to lipi.toml
//!   lipi pkg install              — copy each dependency into lipi_modules/
//!   lipi pkg list                 — show declared dependencies

use std::collections::BTreeMap;
use std::path::Path;

const MANIFEST: &str = "lipi.toml";
const MODULES_DIR: &str = "lipi_modules";

/// A tiny manifest: package name/version + name→path dependencies. We parse only
/// the subset of TOML we emit (sections, `key = "value"`).
struct Manifest {
    name: String,
    version: String,
    deps: BTreeMap<String, String>,
}

impl Manifest {
    fn default() -> Manifest {
        Manifest { name: "मेरा_पैकेज".to_string(), version: "0.1.0".to_string(), deps: BTreeMap::new() }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();
        s.push_str("[package]\n");
        s.push_str(&format!("name = \"{}\"\n", self.name));
        s.push_str(&format!("version = \"{}\"\n\n", self.version));
        s.push_str("[dependencies]\n");
        for (k, v) in &self.deps {
            s.push_str(&format!("{k} = \"{v}\"\n"));
        }
        s
    }

    fn parse(text: &str) -> Manifest {
        let mut m = Manifest::default();
        let mut section = String::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_string();
                continue;
            }
            if let Some(eq) = line.find('=') {
                let key = line[..eq].trim().to_string();
                let mut val = line[eq + 1..].trim().to_string();
                if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    val = val[1..val.len() - 1].to_string();
                }
                match section.as_str() {
                    "package" => match key.as_str() {
                        "name" => m.name = val,
                        "version" => m.version = val,
                        _ => {}
                    },
                    "dependencies" => { m.deps.insert(key, val); }
                    _ => {}
                }
            }
        }
        m
    }

    fn load() -> Result<Manifest, String> {
        let text = std::fs::read_to_string(MANIFEST)
            .map_err(|_| format!("{MANIFEST} नहीं मिला — पहले 'lipi pkg init' चलाएँ"))?;
        Ok(Manifest::parse(&text))
    }

    fn save(&self) -> Result<(), String> {
        std::fs::write(MANIFEST, self.to_toml()).map_err(|e| format!("{MANIFEST} लिख नहीं सका: {e}"))
    }
}

/// Entry point for the `lipi pkg <sub> [args]` command family.
pub fn run(args: &[String]) {
    match args {
        [] | [_] if args.first().map(|s| s.as_str()) == Some("list") => list(),
        [sub] if sub == "init" => init(),
        [sub] if sub == "install" => install(),
        [sub] if sub == "list" => list(),
        [sub, name, path] if sub == "add" => add(name, path),
        _ => eprintln!("उपयोग: lipi pkg [init | install | list | add <नाम> <पथ>]"),
    }
}

fn init() {
    if Path::new(MANIFEST).exists() {
        eprintln!("{MANIFEST} पहले से मौजूद है");
        return;
    }
    let m = Manifest::default();
    match m.save() {
        Ok(()) => println!("✓ {MANIFEST} बनाया\n  [package] name = \"{}\"  version = \"{}\"", m.name, m.version),
        Err(e) => eprintln!("{e}"),
    }
}

fn add(name: &str, path: &str) {
    let mut m = match Manifest::load() { Ok(m) => m, Err(e) => { eprintln!("{e}"); return; } };
    if !Path::new(path).exists() {
        eprintln!("चेतावनी: निर्भरता पथ '{path}' अभी मौजूद नहीं है");
    }
    m.deps.insert(name.to_string(), path.to_string());
    match m.save() {
        Ok(()) => println!("✓ निर्भरता जोड़ी: {name} = \"{path}\""),
        Err(e) => eprintln!("{e}"),
    }
}

fn list() {
    let m = match Manifest::load() { Ok(m) => m, Err(e) => { eprintln!("{e}"); return; } };
    println!("पैकेज: {} v{}", m.name, m.version);
    if m.deps.is_empty() {
        println!("  (कोई निर्भरता नहीं)");
    } else {
        println!("निर्भरताएँ:");
        for (k, v) in &m.deps {
            let installed = Path::new(&format!("{MODULES_DIR}/{k}.swami")).exists();
            let mark = if installed { "✓" } else { "·" };
            println!("  {mark} {k} = \"{v}\"");
        }
    }
}

fn install() {
    let m = match Manifest::load() { Ok(m) => m, Err(e) => { eprintln!("{e}"); return; } };
    if m.deps.is_empty() {
        println!("कोई निर्भरता नहीं — कुछ इंस्टॉल नहीं करना");
        return;
    }
    if let Err(e) = std::fs::create_dir_all(MODULES_DIR) {
        eprintln!("{MODULES_DIR} नहीं बना: {e}");
        return;
    }
    let mut ok = 0;
    let mut fail = 0;
    for (name, src) in &m.deps {
        let src_path = Path::new(src);
        let dest = format!("{MODULES_DIR}/{name}.swami");
        let result = if src_path.is_dir() {
            // directory dependency: prefer <dir>/<name>.swami, else <dir>/lib.swami
            let cand1 = src_path.join(format!("{name}.swami"));
            let cand2 = src_path.join("lib.swami");
            let chosen = if cand1.exists() { Some(cand1) } else if cand2.exists() { Some(cand2) } else { None };
            match chosen {
                Some(p) => std::fs::copy(&p, &dest).map(|_| ()),
                None => { eprintln!("✗ {name}: '{src}' में {name}.swami या lib.swami नहीं मिला"); fail += 1; continue; }
            }
        } else {
            std::fs::copy(src_path, &dest).map(|_| ())
        };
        match result {
            Ok(()) => { println!("✓ {name} → {dest}"); ok += 1; }
            Err(e) => { eprintln!("✗ {name}: {e}"); fail += 1; }
        }
    }
    println!("\nइंस्टॉल पूर्ण: {ok} सफल, {fail} विफल");
    if fail > 0 { std::process::exit(1); }
}
