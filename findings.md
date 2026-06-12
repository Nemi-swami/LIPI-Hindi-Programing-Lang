# LIPI — Findings & Research

This file captures discoveries, benchmarks, design decisions, and research relevant to LIPI development.

---

## Competitive Analysis (2026-06-11)

### Where LIPI stands vs Top 5

| Dimension | Python | JS | Java | Rust | LIPI |
|-----------|--------|-----|------|------|------|
| Stdlib modules | 200+ | 40+ | 300+ | 80+ | 17 |
| Packages | 500k+ | 2M+ | 400k+ | 150k+ | 0 |
| Community users | Millions | Billions | Millions | Hundreds of thousands | 1 |
| Type system | Gradual | Optional (TS) | Static | Static | None |
| Async | Full | Full | Full | Full | None |
| Regex | Built-in | Built-in | Built-in | Built-in | None |
| JIT | PyPy | V8 | JVM | N/A (compiled) | None |
| Debugger | Full | Full | Full | Full | None |
| Package manager | pip | npm | Maven/Gradle | Cargo | None |

**LIPI overall capability vs Python: ~3–5%**  
**LIPI for Indian-context scripting: ~70–80%** (no competitor exists)

---

## Key Insight: Language vs Ecosystem

Python won not because of language quality but because of:
1. NumPy (1995/2006) — scientific computing
2. Django (2005) — web development
3. pip + PyPI (2008) — package distribution
4. Jupyter (2014) — data science UX
5. Machine learning libraries (2015+)

**LIPI's strategy:** Carve a specific niche (Indian-language education, Devanagari computing, government/CBSE tools) rather than trying to out-Python Python in general purpose.

---

## Genuine Competitive Advantages LIPI Has

1. **Only language with Devanagari-native syntax** — no direct competitor
2. **Phonetic input (.vani)** — write programs without Devanagari keyboard
3. **Ancient Indian mathematics built-in** — Kuttaka, Virahanka, Brahmagupta
4. **Indian financial primitives** — Aadhaar validation, UPI, Indian comma formatting
5. **WASM playground** — runs in browser, no install needed

---

## Technical Findings

### VM Architecture
- Current: Stack VM (LVM), bytecode compiled, no JIT
- Copy-on-write semantics for List/Dict means mutation is O(n) — needs GC for large data
- `f64` for all numbers — loses precision above 2^53 — needs integer type separation

### Parser
- SOV (Subject-Object-Verb) grammar — Hindi sentence order
- INDENT/DEDENT tokenization (Python-style)
- Precedence chain: expression → logical_or → logical_and → bitwise_or → ... → unary → primary

### Opcode space
- Tags 0x00–0x40 used (Assert=0x3F, DeclareConst=0x40)
- Next free: 0x41 onward for Phase 17+ opcodes

### Serializer
- `.libc` binary format, magic `b"LIPI"` version 2
- Next version should be 3 when adding new opcode tags

---

## Library Research

### JSON (for Phase 17B)
- Options: hand-write pure Rust JSON parser OR use `serde_json` crate
- WASM concern: `serde_json` works in WASM — safe to add
- Recommendation: add `serde_json` as optional dependency (not default), gate behind `full` feature

### Regex (for Phase 17A)
- Options: `regex` crate (fast, WASM-safe) or hand-write
- `regex` crate is WASM-compatible — safe to add
- Recommendation: `regex` crate, expose as `भारत.पाठ` module

### HTTP Client (for Phase 17B)
- `ureq` — simple, no async, WASM-incompatible (needs native TCP)
- `minreq` — minimal, no async, WASM-incompatible
- For WASM: use JS `fetch` via `web-sys`
- Recommendation: `ureq` for native, `web-sys fetch` for WASM, feature-gated

### Async Runtime (for Phase 18A)
- Options: `tokio` (heavy), `async-std`, or hand-write a simple event loop
- `tokio` is NOT WASM-compatible in full — need feature gates
- Recommendation: hand-write minimal event loop for LIPI's use case

---

## Findings To Add Here As Work Progresses

- [ ] Benchmark: LVM performance vs CPython on fibonacci, sorting, string ops
- [ ] Benchmark: WASM vs native LVM execution speed
- [ ] Survey: what do Hindi-speaking students want from a programming language?
- [ ] Research: existing Devanagari programming languages (any prior art?)
- [ ] Research: CBSE CS curriculum — what language do they currently teach?
- [ ] Research: government digitization initiatives that could adopt LIPI
