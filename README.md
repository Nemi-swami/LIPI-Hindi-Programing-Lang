# LIPI — लिपि

**India's own programming language.** Write code in Devanagari script with Hindi/Sanskrit
keywords, compiled to a bytecode VM, written in **100% pure Rust** (no external crates).

```lipi
विधि नमस्ते(नाम):
    फल "नमस्ते " + नाम + "!"

बताओ नमस्ते("दुनिया")        # नमस्ते दुनिया!
```

---

## ✨ Highlights

- **Full language** — functions, closures, classes + inheritance, enums + pattern matching,
  generators (`उत्पन्न`), async/await (`प्रतीक्षा`), decorators, comprehensions, operator
  overloading, typed exceptions, modules with namespaces.
- **Bytecode VM (LVM)** — compiles to `.libc` bytecode; tail-call optimization; runs on
  desktop and in the **browser via WebAssembly**.
- **Batteries-included stdlib (`भारत.*`)** — JSON, CSV, regex, datetime, crypto (sha256/md5/
  base64), HTTP client + server, sockets, threads, ZIP, SQL, big integers, statistics — plus a unique
  **mission-critical computing suite** (units/dimensional-analysis, linear algebra,
  quaternions, PID + Kalman control, geodesy, Hamming ECC, interval arithmetic).
- **Tooling** — formatter, linter, doc generator, test framework, profiler (+ SVG flame
  graphs), debugger, LSP server, package manager, and **LIPI Studio**, a full Monaco-based
  (VSCode-engine) IDE in the browser.
- **No Hindi keyboard? No problem.** Type on a normal QWERTY keyboard — LIPI Studio
  transliterates Roman → Devanagari live as you type (like Google Input Tools), or use
  `.roman` / `.vani` files.

## 🚀 Quick start

```powershell
# Build (Windows, MSYS2 + Rust GNU toolchain)
$env:RUSTUP_HOME = "D:\Rust\rustup"
$env:CARGO_HOME  = "D:\Rust\cargo"
$env:PATH = "D:\Rust\cargo\bin;D:\msys64\mingw64\bin;$env:PATH"
cargo build --release --target x86_64-pc-windows-gnu
```

The binary is `target\x86_64-pc-windows-gnu\release\lipi.exe`. See [INSTALL.md](INSTALL.md)
for other platforms and adding `lipi` to your PATH.

```
lipi hello.swami         # compile + run
lipi ide                 # launch LIPI Studio (browser IDE)
lipi test foo.swami      # run परीक्षण test blocks
lipi fmt / lint / doc    # tooling
lipi profile foo.swami   # profiler   (--flame for an SVG flame graph)
lipi debug foo.swami     # step debugger
```

## 📝 A taste of LIPI

```lipi
# Classes + inheritance
वर्ग व्यक्ति:
    विधि बनाओ(नाम, आयु):
        यह.नाम है नाम
        यह.आयु है आयु
    विधि परिचय():
        बताओ यह.नाम + " की आयु " + यह.आयु

# Lazy generators
विधि फिबोनाची():
    अ है 0
    ब है 1
    जब तक सत्य:
        उत्पन्न अ
        temp है अ + ब
        अ है ब
        ब है temp

# Async / await
विधि काम(n):
    प्रतीक्षा सोओ(100)
    फल n * 2
बताओ चलाओ(काम(21))        # 42

# Mission-critical: dimensional analysis (catches the Mars Orbiter bug class)
आयात भारत.मात्रक
जोर है जोड़_मात्रा(मात्रा(500, "न्यूटन"), मात्रा(100, "पाउंड_बल"))
बताओ मात्रा_वाक्य(जोर)    # auto-converts; mixing force + length would error
```

## 📚 Language reference

The full syntax + stdlib reference lives in [LANGUAGE.md](LANGUAGE.md). Runnable examples are in
[`examples/`](examples/).

## 🏗️ Architecture

```
source.swami → lexer → parser → compiler → LVM (bytecode VM) → output
```

| Layer | File |
|-------|------|
| Tokenizer (Devanagari, INDENT/DEDENT) | `src/lexer.rs` |
| Parser (SOV grammar) | `src/parser.rs` |
| Compiler (AST → bytecode) | `src/compiler.rs` |
| Virtual machine | `src/lvm.rs` |
| Standard library | `src/bharat_stdlib.rs` + `src/{bignum,net,zip,sql,matrak,rekhiy,…}.rs` |
| Browser IDE | `web/studio/` (served by `src/ide.rs`) |

## 🌐 Input modes

| Extension | How you type |
|-----------|--------------|
| `.swami` | Devanagari (default) |
| `.roman` | Roman keywords (`batao`, `yadi`, …) — identifiers stay as typed |
| `.vani` | Full phonetic transliteration (keywords **and** identifiers) |

In **LIPI Studio**, toggle live Roman input and type Devanagari from a QWERTY keyboard.

## 📦 Status

Pure Rust, zero runtime dependencies. Phases 1–18 complete: a usable language with a real
standard library, tooling, a browser IDE, and a mission-critical computing suite.

## 📄 License

MIT — see [LICENSE](LICENSE).

---

*LIPI — भारत की अपनी प्रोग्रामिंग भाषा.*
