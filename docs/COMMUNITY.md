# Extending LIPI — Community Architecture

LIPI is designed so that **anyone can extend it without touching the core**. There
are three tiers of extension, from "write a `.swami` file" to "bridge any native
library on Earth." Together they mean LIPI has no hard ceiling: if a capability
exists as C code anywhere — a space-agency ephemeris library, a marine-sonar SDK,
a defense-grade crypto module, a GPU driver — LIPI can reach it.

```
Tier 1  Pure-LIPI packages      ← 90% of community work. No Rust, no build step.
Tier 2  Native addons (भारत.बाह्य) ← bind any C/C++ DLL. This is the "no limits" tier.
Tier 3  Stdlib contributions     ← merge a भारत.* module into the core (PR to this repo).
```

---

## Tier 1 — Pure-LIPI packages (`lipi pkg`)

The everyday path. A package is just LIPI source that others `आयात`. No compiler,
no Rust — publish by pushing a repo.

### Layout

```
my-package/
├── lipi.toml          # manifest (name, version, deps)
├── my-package.swami   # the module's public API (this file becomes importable)
├── README.md
└── examples/
```

### Publishing

There is no central registry yet (by design — pure-Rust core, no network host).
Distribution is **git-native**:

```powershell
lipi pkg init                                   # scaffold lipi.toml in a consumer project
lipi pkg add ganita https://github.com/you/lipi-ganita.git#v1.0   # add a git dep (optional #tag)
lipi pkg install                                # clones each dep into lipi_modules/
lipi pkg list
```

`lipi pkg install` shells out to the user's `git` (LIPI's own HTTP client is
http-only/no-TLS, so it can't reach GitHub directly). The clone lands at
`lipi_modules/<name>.swami`, after which `आयात "<name>"` resolves it — imports are
inlined at compile time into the shared instruction space (see `pkg.rs` and the
`ImportFile` note in `lvm.rs`).

### Versioning

Pin with a `#tag` or `#branch` ref in the git source. Follow semantic versioning
in your own `lipi.toml` `version` field so consumers can reason about upgrades.

---

## Tier 2 — Native addons via `भारत.बाह्य` (the no-limits tier)

`भारत.बाह्य` (FFI) is what removes LIPI's ceiling. Any function with a **C ABI**
in any shared library becomes callable. You do **not** need to modify or rebuild
the LIPI interpreter — you ship a `.dll` plus a thin `.swami` wrapper.

### The raw call

```lipi
आयात भारत.बाह्य

पुस्तकालय है बाह्य_पुस्तकालय("mylib.dll")
परिणाम है बाह्य_बुलाओ(पुस्तकालय, "compute", "dd:d", 3.0, 4.0)   # double compute(double,double)
बाह्य_बंद(पुस्तकालय)
```

Signature string is `"<args>:<ret>"` — one char per argument:

| Char | C type            | Notes |
|------|-------------------|-------|
| `i`  | 32-bit int        | LIPI Number ↔ int |
| `l`  | 64-bit int / ptr  | for handles, `size_t`, pointers |
| `d`  | double            | float64 |
| `s`  | `char*` (C string)| LIPI Str ↔ NUL-terminated UTF-8 |
| `v`  | void              | return only |

**One hard limitation** (pure Rust, no libffi): a single call is **either**
all-int/ptr/string **or** all-double — you can't mix `d` with `i/l/s`. Split the
call, or add a C shim in your DLL that takes only ints (passing doubles by
pointer). This covers the overwhelming majority of real C APIs.

### The addon convention

Wrap the raw FFI so consumers never see signature strings. Ship this as a Tier-1
package that depends on a bundled DLL:

```
lipi-blas/
├── lipi.toml
├── lipi-blas.swami        # idiomatic LIPI API, hides बाह्य_बुलाओ
├── native/
│   └── openblas.dll       # the native library (document how to obtain/build it)
└── README.md              # platform notes + provenance/licence of the DLL
```

`lipi-blas.swami`:

```lipi
आयात भारत.बाह्य

# Load once at module load; expose clean functions.
_पुस्तकालय है बाह्य_पुस्तकालय("native/openblas.dll")

विधि सदिश_बिंदु(n, x_ptr, y_ptr):
    फल बाह्य_बुलाओ(_पुस्तकालय, "cblas_ddot", "llll:d", n, x_ptr, 1, y_ptr, 1)
```

**Ground rules for native addons**

1. **State the ABI.** Only C-ABI exports work. C++ must be `extern "C"`.
2. **Ship or document the DLL.** Include provenance and licence — you are
   redistributing native code.
3. **Guard other platforms.** `भारत.बाह्य` is Windows-first today; on non-Windows
   and WASM it returns a catchable error. Wrap calls in `कोशिश`/`पकड़ो` and
   degrade gracefully.
4. **Memory is manual.** If a C function returns a pointer you must free, expose
   the matching free function and call it. LIPI's GC does not track foreign memory.

### Talking to hardware directly — `भारत.तंत्र`

FFI reaches library *functions*; `भारत.तंत्र` reaches *memory* — the other half of
systems programming. It gives you:

- **Managed buffers** — `स्मृति_आवंटन(बाइट्स)` returns a pointer; write/read it as
  bytes, 64-bit ints, doubles, or NUL-terminated strings (`स्मृति_लिखो_पूर्ण`,
  `स्मृति_पढ़ो_दशमलव`, `स्मृति_लिखो_वाक्य`, …), bounds-checked. This is how you build
  the C `struct`s and out-parameters an FFI call needs, then read results back.
- **Volatile MMIO** — `कच्चा_लिखो३२(पता, मान)` / `कच्चा_पढ़ो३२(पता)` do
  `write_volatile`/`read_volatile` at an absolute address: exactly how you drive a
  **memory-mapped hardware register**. The transistors behind a device are exposed
  to the CPU as addresses; writing the register flips them. Combine with a kernel
  driver (loaded via `भारत.बाह्य`) that maps the device's physical memory into your
  process, and LIPI is doing register-level device I/O.

```lipi
आयात भारत.तंत्र
# Build a 2-float struct to hand a native physics kernel:
सं है स्मृति_आवंटन(16)
स्मृति_लिखो_दशमलव(सं, 0, 9.81)
स्मृति_लिखो_दशमलव(सं, 8, 0.0)
# ... बाह्य_बुलाओ(lib, "integrate", "l:v", सं) ...
स्मृति_मुक्त(सं)
```

`कच्चा_*` is unchecked by design — a bad address crashes the process, just like C.
That is the price of unrestricted hardware access.

### Callbacks — when the library calls YOU (`बाह्य_कॉलबैक`)

Many C APIs are callback-driven: `qsort` wants a comparator, a GUI toolkit wants an
event handler, a driver wants a completion routine. `बाह्य_कॉलबैक(क्लोजर, "ll:i")`
turns a LIPI closure into a real C function pointer you can pass into such APIs.
When the native code invokes it, LIPI runs your closure and returns its result to C.

```lipi
विधि तुलना(अ, ब):                       # अ, ब are C pointers to the two elements
    फल कच्चा_पढ़ो३२(अ) - कच्चा_पढ़ो३२(ब)
सीएमपी है बाह्य_कॉलबैक(तुलना, "ll:i")     # → a C function pointer
बाह्य_बुलाओ(lib, "qsort", "llll:v", आधार, गिनती, 4, सीएमपी)
बाह्य_कॉलबैक_मुक्त(सीएमपी)               # free the slot when C is done with it
```

Limits: callback arguments are integer/pointer only (`i`/`l`), at most 4 (covers
comparators, window procs, interrupt handlers); use `भारत.तंत्र` to read any struct
the pointers refer to. A callback that errors returns 0 to C.

### Why this reaches every domain

| Domain | Existing C libraries LIPI can now drive via `भारत.बाह्य` |
|--------|----------------------------------------------------------|
| Space / astrodynamics | SPICE (NAIF), SGP4, ephemeris libs |
| Marine / sonar | vendor sonar SDKs, NMEA parsers, GDAL |
| Defense / crypto | OpenSSL, libsodium, hardware HSM drivers |
| Scientific / ML | BLAS/LAPACK, FFTW, ONNX Runtime, CUDA driver API |
| Graphics / apps | OpenGL, SDL2, WebView2, Win32 |

The mission-critical `भारत.*` modules (units, linear algebra, Kalman, geodesy,
Hamming ECC, intervals) give correctness primitives in-language; `भारत.बाह्य`
gives reach to everything else.

---

## Tier 3 — Stdlib contributions (`भारत.*`)

If a capability is broadly useful and pure-Rust-implementable, contribute it to
the core as a new `भारत.*` module. This is a PR to this repository.

### Recipe (follow any existing module, e.g. `src/antaral.rs`)

1. Create `src/yourmod.rs` exposing `pub fn yourmod_registry() -> Registry` —
   a `Vec<(&'static str, NativeFn)>` of `fn(Vec<Value>) -> Result<Value, String>`.
2. `mod yourmod;` in **both** `src/main.rs` and `src/lib.rs` (WASM root).
3. Add the module-name arm in `src/lvm.rs` where imports are resolved
   (`"भारत.यौरमॉड" => crate::yourmod::yourmod_registry()`).
4. Add an `examples/..._test.swami` with `जाँचो` assertions.
5. Document it in `CLAUDE.md` (source-layout table + stdlib registry table).

### Constraints (non-negotiable for core)

- **Pure Rust, no external crates** (the sole exception is `wasm-bindgen`, WASM
  feature only). Hand-write parsers/algorithms as the existing modules do.
- **WASM-safe.** If a feature needs the OS (sockets, clock, FFI), gate it behind
  `#[cfg(not(target_arch = "wasm32"))]` and return a catchable error on WASM.
- **Errors are catchable Hindi strings**, never panics.

Native addons (Tier 2) have **none** of these constraints — that is the point.
Use Tier 3 for portable, pure primitives; use Tier 2 for everything else.
