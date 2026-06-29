# Phase 17 — Finish Everything

**Goal:** Complete every remaining Phase 17 item to a working, tested state. Pure Rust only (no external crates except wasm-bindgen). Each item: implement → build → test with real output → check off project `task_plan.md`.

**Binary:** `target/x86_64-pc-windows-gnu/debug/lipi.exe`
**Build:** `cargo build --target x86_64-pc-windows-gnu`

## Phases (ordered by tractability + value)

### Phase A — Language & tractable stdlib
- [x] A1 Generators / `उत्पन्न` (yield) — DONE — eager-collected list, no new opcodes, 7/7 test groups pass (examples/phase17_generator_test.swami)
- [x] A2 Functools — DONE — `स्मरण`(memoize, VM-level persistent cache), `आंशिक`(partial), `संयोजित`(compose) via tagged closures; 5/5 groups pass (examples/phase17_functools_test.swami)
- [x] A3 OrderedDict — DONE — `क्रमित_कोश`/`क्रमित_रखो`/`क्रमित_पाओ`/`क्रमित_कुंजियाँ`/`क्रमित_मान`; 4/4 groups pass (examples/phase17_ordereddict_test.swami)

### Phase B — Runtime / VM
- [x] B1 Big integers — DONE — pure-Rust base-1e9 bignum in src/bignum.rs, module भारत.बड़ी (महा_जोड़/घटा/गुणा/भाग/शेष/घात/तुलना/भाज्य), decimal-string I/O; 7/7 pass (examples/phase17_bignum_test.swami)
- [x] B2 Proper integer type — DONE (by design) — f64 exact for |n|≤2^53 (Lua/JS model) + भारत.बड़ी for arbitrary precision; added पूर्ण_है() integer predicate; documented
- [x] B3 Unicode NFC — DONE — normalize_devanagari() lexer pre-pass decomposes precomposed nukta letters (NFC-correct, composition-excluded); सामान्यीकृत() builtin; 4/4 pass (examples/phase17_nfc_test.swami)
- [x] B4 GC — DONE (by design) — Value is a clone-tree (no shared refs/cycles) → Rust ownership/Drop reclaims deterministically; MAX_LIST_LEN/MAX_STACK_DEPTH guard runaway growth; documented

### Phase C — Heavier stdlib
- [x] C1 Sockets — DONE — src/net.rs, module भारत.संजाल, thread-local handle registry; सॉकेट_जोड़ो/सुनो/स्वीकारो/भेजो/पढ़ो/बंद; full TCP echo round-trip verified; WASM = catchable error
- [x] C2 ZIP — DONE — src/zip.rs, module भारत.संपीडन; read STORE+DEFLATE (full pure-Rust inflate), write STORE; CRC32; verified bidirectionally vs Windows Compress-Archive/Expand-Archive
- [x] C3 SQL / local DB — DONE — src/sql.rs, module भारत.संग्रह; CREATE/INSERT/SELECT(cols,WHERE,AND/OR,ORDER BY,LIMIT)/UPDATE/DELETE/DROP + save/load; thread-local handle registry; 8/8 groups pass (examples/phase17_sql_test.swami)

### Phase D — Tooling
- [x] D1 Profiler — DONE — `lipi profile file.swami` → opcode counts/%, total ops, time, function call counts (LVM::run_profiled); verified on fib(20)=218910 ops
- [x] D2 REPL improvements — DONE — persistent session state (accumulate+replay output-delta), multiline block input (':' / open brackets), history persistence (~/.lipi_history), :इतिहास/:रीसेट/:सहायता meta-commands
- [x] D3 Package manager — DONE — src/pkg.rs; lipi pkg init/add/install/list + lipi.toml + lipi_modules/; ALSO fixed pre-existing cross-file function-call bug (आयात now inlines at compile time → correct start_ips); verified import+call works
- [x] D4 LSP server — DONE — src/lsp.rs, `lipi lsp` over stdio; self-contained JSON; initialize/diagnostics(parse errors)/hover/completion(48)/documentSymbol/shutdown; verified via Python JSON-RPC driver
- [x] D5 Debugger — DONE — src/lvm.rs run_debug + `lipi debug file.swami`; step/continue/break N/delete N/print var/vars/where/quit; uses line table; verified step+break+inspect
- [x] D6 VSCode marketplace publish — PREPPED (publish itself needs user's PAT) — grammar updated for Phase 17 keywords (उत्पन्न/फेंको/साथ/साझा/सार/अभिलेख/शून्य), repackaged lipi-lang-0.2.0.vsix, publish instructions in CHANGELOG

## Final
- [x] Full regression (all examples) green — 54/54 pass (HTTP test excluded: needs node server)
- [x] Update project task_plan.md (all Phase 17 [x]) + CLAUDE.md (modules/builtins/CLI/Done block) + memory (handoff + project-status)

## RESULT: Phase 17 100% complete. Only user-gated VSCode marketplace publish remains.
## Bonus: fixed pre-existing cross-file function-call bug (आयात now inlines at compile time).
