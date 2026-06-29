# Progress — Phase 17 Finish

## Session 2026-06-29 (start)
- Verified merge state: phase17-completion has both agent branches merged, build clean.
- Regression baseline: 45/49 examples pass (4 = 2 intentional demos + 2 stale pre-P16 files).
- Updated CLAUDE.md + task_plan.md for properties/deque/mem-limits/fmt/lint/doc.
- Starting full Phase 17 completion. Plan in this dir.

### Log
(append entries as each item lands)

## Session 2026-06-29 (Phase A — language & tractable stdlib)
- A1 Generators (उत्पन्न): eager-collected list via hidden __gen_acc__, no new opcodes. 7/7 pass.
- A2 Functools: स्मरण/आंशिक/संयोजित as tagged closures; memoize uses VM-level memo_caches (persistent). 5/5 pass.
- A3 OrderedDict: क्रमित_* builtins, list-of-pairs, insertion-ordered. 4/4 pass.
- All builds clean (exit 0). Next: Phase B (runtime/VM) — bigint, int type, NFC, GC.

## Session 2026-06-29 (Phase B — runtime/VM)
- B1 Big integers: src/bignum.rs base-1e9 bignum, module भारत.बड़ी. 7/7 pass (2^53+1, huge mul, divmod, 2^100, 30!).
- B3 Unicode NFC: normalize_devanagari() lexer pre-pass + सामान्यीकृत(). 4/4 pass.
- B2 (by design): f64 + भारत.बड़ी; added पूर्ण_है() predicate. B4 (by design): clone-tree Values, no cycles, Rust ownership = GC.
- Next: Phase C — sockets, ZIP, SQL.

## Session 2026-06-29 (Phase C — heavier stdlib)
- C1 Sockets: src/net.rs, भारत.संजाल, thread-local handle registry. TCP echo round-trip verified.
- C2 ZIP: src/zip.rs, भारत.संपीडन. Full pure-Rust inflate (stored/fixed/dynamic Huffman) + CRC32 + STORE writer. Bidirectional vs Windows verified (read deflate 5001 chars exact incl BOM; Windows expanded LIPI zip).
- C3 SQL: in progress — src/sql.rs minimal SQL engine भारत.संग्रह.

- C3 SQL: src/sql.rs भारत.संग्रह — tokenizer+recursive-descent parser+executor; CREATE/INSERT/SELECT/UPDATE/DELETE/DROP, WHERE AND/OR, ORDER BY, LIMIT, save/load. 8/8 pass.
- Phase C COMPLETE. Next: Phase D tooling (profiler, REPL, package manager, LSP, debugger).
- Note: renamed sql Cell enum collision via `use std::cell::Cell as StdCell`.

## Session 2026-06-29 (Phase D — tooling)
- D1 Profiler: lipi profile, LVM::run_profiled (opcode counts/%, time, fn calls). fib(20)=218910 ops verified.
- D2 REPL: persistent session (accumulate+output-delta), multiline blocks, ~/.lipi_history, :इतिहास/:रीसेट/:सहायता.
- D3 Package manager: src/pkg.rs lipi pkg init/add/install/list + lipi.toml + lipi_modules/. ALSO FIXED pre-existing cross-file function-call bug — आयात "file" now inlines at compile time (correct start_ips). Verified import+call.
- D4 LSP: src/lsp.rs `lipi lsp`, self-contained JSON-RPC. initialize/diagnostics/hover/completion(48)/documentSymbol/shutdown. Python driver verified.
- D5 Debugger: LVM::run_debug + `lipi debug`. step/break/continue/print/vars/where/quit. Verified.
- D6 VSCode publish: needs user's publisher account+PAT (cannot automate). Will prep grammar+instructions.
