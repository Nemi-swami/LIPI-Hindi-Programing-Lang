# Installing LIPI

LIPI is **pure Rust** with zero runtime dependencies (the only optional build
dependency is `wasm-bindgen`, used solely for the WebAssembly build). Once built,
the `lipi` binary is fully self-contained.

## Prerequisites

- A Rust toolchain (`rustup` + `cargo`). LIPI is developed against the
  **GNU** toolchain (`stable-x86_64-pc-windows-gnu`) on Windows, but the code is
  portable Rust and builds on Linux/macOS with the default toolchain.
- On Windows with the GNU toolchain: an MSYS2/MinGW-w64 `gcc` for linking.

## Build — Windows (GNU toolchain, as developed)

```powershell
$env:RUSTUP_HOME = "D:\Rust\rustup"
$env:CARGO_HOME  = "D:\Rust\cargo"
$env:PATH = "D:\Rust\cargo\bin;D:\msys64\mingw64\bin;$env:PATH"
cargo build --release --target x86_64-pc-windows-gnu
```

Binary: `target\x86_64-pc-windows-gnu\release\lipi.exe`

## Build — Linux / macOS

```bash
cargo build --release
```

Binary: `target/release/lipi`

## Add `lipi` to your PATH

- **Windows:** copy `lipi.exe` somewhere on your `PATH` (e.g. `D:\Rust\cargo\bin`),
  or use the bundled `install.bat`.
- **Linux/macOS:** `cargo install --path .`, or copy `target/release/lipi` into
  `~/.local/bin`.

Verify:

```
lipi --version    # or just: lipi   (starts the REPL)
echo 'बताओ "नमस्ते"' > hi.swami && lipi hi.swami
```

## Browser playground (WebAssembly)

```powershell
wasm-pack build --target web --out-dir web/pkg --features wasm
```

Then serve `web/` over HTTP (e.g. `play.bat`) and open the page. The whole LVM runs
client-side in the browser.

## First steps

```
lipi hello.swami         # compile + run a source file
lipi ide                 # launch LIPI Studio (Monaco-based browser IDE)
lipi                     # REPL (persistent state, multiline blocks)
lipi roman hello.roman   # QWERTY Roman keywords → Devanagari, then run
```

See [LANGUAGE.md](LANGUAGE.md) for the full language + standard-library reference.
