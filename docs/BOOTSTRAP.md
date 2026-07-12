# LIPI Self-Hosting — Bootstrap Plan

**Status (2026-07-13):** proof-of-concept lexer works (`examples/selfhost_lexer.swami`). The remaining stages are months of solo work; this document lays out the staircase.

## What "self-hosting" means

The current LIPI implementation lives in `src/*.rs` — LIPI source → tokens → AST → bytecode → LVM, all written in Rust. Self-hosting means rewriting this pipeline **in LIPI itself**, then bootstrapping:

1. Rust LIPI (`lipi.exe`) compiles the LIPI-written LIPI compiler (`lipi.swami`) into bytecode (`lipi.libc`).
2. `lipi.libc` runs on the Rust-hosted LVM and compiles any subsequent LIPI source.
3. Eventually the LVM itself gets replaced (WASM target, LIPI-generated C, etc.) and the Rust code retires.

## Bootstrap staircase

### Stage 0 — Rust LIPI (DONE)
The current interpreter. Everything below builds on top of what `lipi.exe` accepts today.

### Stage 1 — Lexer (PROOF-OF-CONCEPT DONE)
`examples/selfhost_lexer.swami` — tokenizes arithmetic expressions with numbers, identifiers, and `+ - * / = ( )`.

Missing to be production-grade:
- Devanagari identifier range check (U+0900–U+097F + halant + nukta)
- String literals with escapes (`\n`, `\t`, `\"`, `\\`, `\uXXXX`)
- All comparison operators (`< > <= >= == !=`)
- Comments: `#`, `।...।`, block `।।...।।`
- Keywords table (`यदि`, `अन्यथा`, `विधि`, `फल`, `जब तक`, `के लिए`, `में`, `बताओ`, ...)
- Indent/Dedent tracking (Python-style block sensitivity)
- Multi-line strings `"""..."""`
- Scientific notation, radix literals (`0x`, `0b`, `0o`)
- Line number tracking per token

**Effort:** ~1 week solo. ~500 lines of LIPI.

### Stage 2 — Parser
Recursive-descent parser mirroring `src/parser.rs`. Produces AST as nested Dicts (LIPI has no ADTs, but Dicts + a `प्रकार` tag work fine).

Missing:
- Full expression grammar (precedence chain: assignment → ternary → or → and → not → comparison → shift → additive → multiplicative → unary → power → primary)
- All statement forms (`यदि / अन्यथा`, `जब तक`, `के लिए`, `बार करो`, `विधि`, `वर्ग`, `कोशिश / पकड़ो`, `साथ`, `मिलाओ`, ...)
- Error recovery (mirror `parse_recover`)
- Slice syntax, list/dict/set literals, comprehensions, spread, walrus
- Decorator handling

**Effort:** ~4–6 weeks solo. ~1500 lines of LIPI.

### Stage 3 — Compiler (AST → bytecode)
Walk the AST and emit bytecode as a list of `[Opcode, arg1, arg2, ...]` tuples. Match the opcode set in `src/opcode.rs` (currently ~80 opcodes including Phase 20 additions like `SetSlice`).

Missing:
- Bytecode serialization (`.libc` format, tags per opcode)
- Line-number table (for error diagnostics)
- Class hierarchy table (multi-parent support from mixins work)
- Constant pool / interning

**Effort:** ~6–8 weeks solo. ~2000 lines of LIPI.

### Stage 4 — Bootstrap round-trip
Prove that `lipi.libc` (LIPI compiler compiled by Rust LIPI) can recompile its own source and produce an identical `lipi.libc'`. Fixed-point achieved.

**Effort:** ~2 weeks debugging + fuzzing.

### Stage 5 — Runtime replacement (optional)
Rewrite the LVM in LIPI too, targeting some other backend (LLVM IR, C, WASM). At this point Rust is no longer needed for LIPI to build itself.

**Effort:** ~6–12 months.

## Total honest estimate

**Stages 1–4: 3–5 months of focused solo work.**
Stage 5: another 6–12 months.

## Practical incremental value

Even without full self-hosting, this exercise pays dividends:

- **Stage 1 done** proves the language is powerful enough to write text-manipulation-heavy code.
- **Stage 2 partial** would enable macro-style AST transformation from LIPI code — user-defined language extensions.
- **Stage 3 partial** would enable custom bytecode emitters (DSLs compiled by LIPI to LIPI bytecode).

## Why we're not doing all of it now

This document sits alongside a working Stage 1 as an honest artifact: LIPI is capable enough to reach self-hosting, and the path is understood. Committing to the full 3–5-month roadmap is a separate decision from adding it to the plan.
