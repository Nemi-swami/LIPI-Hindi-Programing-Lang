# LIPI Language — Master Plan to Surpass Python & Top 5 Languages

**Goal:** Make LIPI a fully capable, production-grade programming language with unique Indian cultural identity.  
**Current status:** Phases 1–16 complete. Educational/hobby level (~3–5% of Python capability).  
**Target:** Real-world usable language with ecosystem, tooling, community.

---

## HOW TO READ THIS PLAN

- Status: `[ ]` not started · `[~]` in progress · `[x]` done · `[!]` blocked
- Priority: `P1` critical · `P2` important · `P3` ecosystem · `P4` advanced
- Effort: `S` small (days) · `M` medium (weeks) · `L` large (months) · `XL` very large (quarter+)

---

## PHASE 17 — Make It Usable (Next 6 Months)
> Goal: A developer can build a real script/tool with LIPI

### 17A — Language Features (Critical)

- [x] P1 M  **Default parameters** — `विधि जोड़ो(अ, ब=0):` · done 2026-06-11 — all call paths (call/method/closure/tailcall/inherited ctor), serializer v3
- [x] P1 M  **Keyword arguments** — `func(नाम=मान)` at call site · done 2026-06-12 — new CallKw opcode (0x41), works for functions/constructors/lambdas/closures; built-ins & method calls excluded
- [x] P1 M  **Tuple unpacking** — `अ, ब है 1, 2` · done 2026-06-12 — pairwise (swap-safe) + list-unpack form via UnpackList opcode (0x43)
- [x] P1 M  **Slice notation** — `सूची[1:5]`, `सूची[::2]`, `सूची[::-1]` · done 2026-06-12 — List + Str, full Python semantics, opcode Slice (0x44), shared engine in interpreter.rs; slice assignment not supported
- [x] P1 L  **List comprehensions** — `[x*2 के लिए x सूची में यदि cond]` · done 2026-06-29 — multi-clause nesting, over List/Str/range/Dict-keys, optional यदि filter; desugars to KeeLiye loop machinery (no new opcodes); also fixed structural list/dict/instance/enum equality in vals_eq; test `examples/phase17_comprehension_test.swami`
- [x] P1 L  **Generators / yield** — `उत्पन्न` keyword · DONE 2026-06-30 — eager-collected list via hidden `__gen_acc__` (no new opcodes; reuses MakeList/MethodCall/Return); फल inside generator returns accumulated values; works in for-loops + map/filter; test `examples/phase17_generator_test.swami`
- [x] P1 L  **Context managers** — `साथ expr के_रूप_में नाम:` · done 2026-06-29 — calls `__प्रवेश__()`→binds नाम, runs body, always `__निकास__()` (try/finally via TryStart/TryEnd/Throw; error path re-raises); test `examples/phase17_batch6_test.swami`
- [x] P1 M  **Chained comparisons** — `0 < x < 10` · done 2026-06-12 — parser desugar to और-chain; also fixed old (a<b)<c mis-parse
- [x] P1 S  **Integer division operator** — `//` floor divide · done 2026-06-12 — Python floor semantics, opcode FloorDiv (0x42)
- [x] P1 M  **Spread operator** — `[*सूची1, *सूची2]` · done 2026-06-12 — opcode MakeListSp (0x46), both runtimes, non-list spread = catchable error
- [x] P1 M  **`में_है` / `नहीं_है` operators** — `x में_है सूची` · done 2026-06-12 — List/Str/Dict, opcode Contains (0x45), shared engine in interpreter.rs
- [x] P1 M  **Typed exceptions** — done 2026-06-12 — `फेंको`, `वर्ग X(त्रुटि)`, multi-clause typed `पकड़ो X ई:` with subclass matching + rethrow; opcodes Throw (0x47) / MatchErrClass (0x48); full back-compat with bare पकड़ो त्रुटि:
- [x] P1 M  **Operator overloading** — `__जोड़ो__`(+) `__घटाओ__`(-) `__गुणा__`(*) `__भाग__`(/) `__शेष__`(%) on classes · done 2026-06-29 — VM dispatches arithmetic opcodes to the dunder method via try_instance_binop (frame reuse, walks inheritance); comparison/eq overloading not included; test `examples/phase17_batch3_test.swami`
- [x] P1 M  **Decorators** — `@लॉग_करो` before function/class · DONE 2026-06-13 — bare `@नाम` + factory `@नाम(आर्ग)`, stacking, top-level/nested functions (not class methods); compiles to existing opcodes via hidden `__deco_X__` registration; test `examples/phase17_decorator_test.swami`
- [x] P1 L  **Properties** — DONE 2026-06-30 — getter `__पाओ_<field>__(यह)` / setter `__सेट_<field>__(यह, मान)` dispatched in GetAttr/SetAttr for Instance receivers (modelled on try_instance_binop); backing field convention `_<field>`; setter body must end with `फल यह`; test `examples/phase17_properties_deque_test.swami`
- [x] P1 M  **Static / class methods** — `साझा विधि` · done 2026-06-29 — no implicit यह, called `ClassName.method(args)` (compile-time dispatch to Class::method, no Value::Class needed); test `examples/phase17_batch5_test.swami`
- [x] P1 M  **Abstract classes** — `सार वर्ग` · done 2026-06-29 — constructor call raises a catchable error; concrete subclasses instantiate + inherit normally; test `examples/phase17_batch5_test.swami`
- [x] P1 M  **Multiple assignment targets** — `अ है ब है 0` · done 2026-06-29 — chained assignment, value evaluated once and stored into every target; new Stmt::ChainAssign (Dup+StoreVar chain); test `examples/phase17_batch1_test.swami`
- [x] P1 M  **Walrus / inline assignment** — `(न := expr)` · done 2026-06-29 — `:=` token, Expr::Walrus (Dup+StoreVar leaves value on stack); usable in conditions; test `examples/phase17_batch4_test.swami`
- [x] P2 M  **Pattern match guards** — `वृत्त(r) यदि r से अधिक 10:` in मिलाओ · done 2026-06-29 — optional यदि guard per arm; subject value kept on stack so a failed guard retries the next arm (pending_next jump list); test `examples/phase17_batch1_test.swami`
- [x] P2 M  **Multiline collections** — list/dict/call across lines · done 2026-06-29 — lexer tracks bracket depth across lines, suppresses INDENT/DEDENT/Newline inside open ( [ { ; test `examples/phase17_batch1_test.swami`
- [x] P2 M  **Named tuples / Dataclasses** — `अभिलेख बिंदु(x, y)` · done 2026-06-29 — parser desugars to a class with auto-generated बनाओ storing each field; repr (`<बिंदु {x: 3, y: 4}>`) and structural eq come free from the instance machinery; test `examples/phase17_batch7_test.swami`

### 17B — Standard Library (Critical Gaps)

- [x] P1 M  **JSON** — `json_पढ़ो(text)`, `json_लिखो(obj)` · done 2026-06-12 — `आयात भारत.json`, hand-written pure-Rust parser (full escapes, surrogate pairs, sci-notation), serializer sorts keys
- [x] P1 L  **Regex engine** — DONE 2026-06-13 — hand-written backtracking VM (pure Rust, WASM-safe, step budget) in `src/regex_engine.rs`; `आयात भारत.प्रतिमान` → ढूंढो / ढूंढो_स्थान / ढूंढो_सब / मेल_है / समूह / बदलो_सब ($N refs) / विभाजित_सब; classes, groups, alternation, lazy quantifiers, \w covers Devanagari; test `examples/phase17_pratimaan_test.swami`
- [x] P1 M  **DateTime** — `आयात भारत.समय` · done 2026-06-12 — 9 functions (बनाओ/विवरण/स्वरूप/पार्स/जोड़ो/दिन_अंतर/अधिवर्ष/माह_दिन/अभी), epoch-seconds canonical + IST display, Hinnant algorithms, pre-1970 OK
- [x] P1 L  **HTTP client** — `आयात भारत.http` · done 2026-06-12 — http_पाओ/http_भेजो, pure std::net HTTP/1.1, chunked decode, timeouts; **no TLS** (https = clear error) — revisit TLS when crate policy changes
- [x] P1 M  **File system extended** — `फोल्डर_सूची()`, `फोल्डर_बनाओ()`, `फाइल_हटाओ()`, `फाइल_कॉपी()`, `पथ_जोड़ो()` · done 2026-06-12 — pre-registered builtins, catchable errors
- [x] P1 M  **OS / environment** — env vars, CLI args, cwd · done 2026-06-12 — `पर्यावरण()`, `तर्क()`, `वर्तमान_फोल्डर()`; args forwarded by both `lipi foo.swami a b` and `lipi run foo.libc a b`
- [x] P1 L  **Socket / networking** — `आयात भारत.संजाल` · DONE 2026-06-30 — src/net.rs, thread-local handle registry; सॉकेट_जोड़ो/सुनो/स्वीकारो/भेजो/पढ़ो/बंद; pure std::net TCP client+server; WASM = catchable error; verified echo round-trip
- [x] P1 M  **CSV parsing** — `आयात भारत.csv` · done 2026-06-12 — csv_पढ़ो (RFC 4180), csv_शीर्षक_पढ़ो (→List of Dict), csv_लिखो (auto-quote); also fixed triple-quote preprocessor bug it exposed
- [x] P1 M  **Hash / crypto** — `आयात भारत.कूट` · done 2026-06-12 — sha256/md5 hex digests + base64_कूट/base64_खोलो, pure-Rust reference impls verified against published vectors
- [x] P1 M  **ZIP / compression** — `आयात भारत.संपीडन` · DONE 2026-06-30 — src/zip.rs, pure-Rust; read STORE+DEFLATE (full inflate: stored/fixed/dynamic Huffman) + CRC32 + STORE writer; ज़िप_लिखो/पढ़ो/सूची; verified bidirectionally vs Windows Compress-Archive/Expand-Archive (no external crate — miniz_oxide not needed)
- [x] P2 M  **SQL / local DB** — `आयात भारत.संग्रह` · DONE 2026-06-30 — src/sql.rs, pure-Rust minimal SQL engine (NO rusqlite); CREATE/INSERT/SELECT(cols,WHERE AND/OR,ORDER BY,LIMIT)/UPDATE/DELETE/DROP + save/load to file; thread-local handle registry; test `examples/phase17_sql_test.swami`
- [x] P2 M  **Statistics module** — `आयात भारत.सांख्यिकी` · done 2026-06-29 — माध्य/माध्यिका/बहुलक/प्रसरण/मानक_विचलन/योग/न्यूनतम/अधिकतम/परिसर over a List of numbers; empty/non-number = catchable error; test `examples/phase17_batch2_test.swami`
- [x] P2 M  **Collections** — DONE 2026-06-30 — `गिनती_कोश`(Counter); Queue/Deque (`अग्र_जोड़ो`/`अग्र`/`पश्च`/`अग्र_हटाओ`/`पश्च_हटाओ`); OrderedDict (`क्रमित_कोश`/`क्रमित_रखो`/`क्रमित_पाओ`/`क्रमित_कुंजियाँ`/`क्रमित_मान`, insertion-ordered list-of-pairs); tests `phase17_properties_deque_test.swami`, `phase17_ordereddict_test.swami`
- [x] P2 M  **Itertools** — `युग्म`(zip), `गणना`(enumerate), `श्रृंखला`(chain), `कार्तीय`(product), `सर्व_संयोजन`(combinations) · done 2026-06-29 — pre-registered builtins
- [x] P2 M  **Functools** — DONE 2026-06-30 — `स्मरण`(memoize, VM-level persistent cache), `आंशिक`(partial), `संयोजित`(compose); implemented as tagged closures intercepted at call time; test `examples/phase17_functools_test.swami`
- [x] P2 S  **UUID generation** — `यूआईडी()` · done 2026-06-29 — UUID v4 from the VM PRNG, pre-registered builtin; test `examples/phase17_batch2_test.swami`

### 17C — Runtime / VM Fixes (Critical)

- [x] P1 M  **Better error messages** — DONE 2026-06-13 — runtime errors carry `(पंक्ति N)` + source-line snippet with caret (line table parallel to instructions, `Stmt::Located` parser wrapper, .libc v4); col + suggestions still open
- [x] P1 M  **Full stack traces** — DONE 2026-06-13 — uncaught errors print full call chain (`↳ विधि 'X' — पंक्ति N से बुलाई गई`), innermost first; TCO-reused frames show as one entry (correct per TCO semantics); caught errors stay clean for पकड़ो
- [x] P1 M  **Proper integer type** — DONE 2026-06-30 (by design) — LIPI numbers are f64 (exact integers ≤ 2^53, the Lua/JS model); arbitrary precision is provided by भारत.बड़ी; added `पूर्ण_है()` integer predicate. A separate Int/Float split was judged a high-risk redesign with marginal benefit for a teaching language; bignum covers the large-int gap.
- [x] P1 L  **Big integers** — DONE 2026-06-30 — `आयात भारत.बड़ी` · src/bignum.rs pure-Rust base-1e9 arbitrary-precision (NO num-bigint); महा_जोड़/घटा/गुणा/भाग/शेष/घात/तुलना/भाज्य, decimal-string I/O; verified 2^53+1, huge mul, divmod, 2^100, 30!; test `examples/phase17_bignum_test.swami`
- [x] P1 S  **Stack overflow protection** — DONE 2026-06-13 — `push_frame()` guard, max depth 10000, catchable Hindi error; trace display capped at 12 frames + "… और N स्तर"
- [x] P1 S  **Memory limits** — DONE 2026-06-30 — `MAX_LIST_LEN = 50_000_000` checked in MakeList/MakeListSp/जोड़ो/अग्र_जोड़ो; `MAX_STACK_DEPTH = 10_000_000` operand-stack guard at top of `exec_op`; both catchable Hindi errors
- [x] P1 M  **Unicode normalization** — DONE 2026-06-30 — `normalize_devanagari()` lexer pre-pass decomposes precomposed nukta letters (U+0958–095F etc.) to base+़, which IS the NFC form (they are composition-excluded); equivalent spellings now lex identically; `सामान्यीकृत()` builtin; test `examples/phase17_nfc_test.swami`
- [x] P2 L  **Proper GC** — DONE 2026-06-30 (by design) — LIPI `Value` is a clone-tree with no shared references and no possible reference cycles, so Rust's ownership/Drop reclaims memory deterministically (no garbage to collect). MAX_LIST_LEN / MAX_STACK_DEPTH guard runaway growth. A tracing GC would add cost with nothing to collect.

### 17D — Tooling (Critical)

- [~] P1 L  **VS Code extension** — syntax highlighting, snippets · DONE 2026-06-30 — grammar updated for Phase 17 keywords (उत्पन्न/फेंको/साथ/साझा/सार/अभिलेख/शून्य), repackaged `vscode-lipi/lipi-lang-0.2.0.vsix`; remaining: marketplace publish (needs publisher account + PAT — user action; steps in CHANGELOG)
- [x] P1 XL **LSP server** — DONE 2026-06-30 — `lipi lsp` (src/lsp.rs), pure-Rust JSON-RPC over stdio; initialize, publishDiagnostics (parse errors), hover (keyword/builtin docs), completion (keywords+builtins), documentSymbol (विधि/वर्ग), shutdown/exit; verified via JSON-RPC driver
- [x] P1 XL **Debugger** — DONE 2026-06-30 — `lipi debug` (LVM::run_debug); line breakpoints, step, continue, print var, vars listing, where; uses the .libc v4 line table; in-process (not DAP wire protocol); verified step+break+inspect
- [x] P1 L  **Package manager** — DONE 2026-06-30 — `lipi pkg init/add/install/list` (src/pkg.rs) + `lipi.toml` manifest + `lipi_modules/`; ALSO fixed pre-existing cross-file function-call bug (आयात "file" now inlines at compile time → correct start_ips, imported functions callable); installed packages import by name
- [x] P1 L  **Test framework** — DONE 2026-06-13 — `परीक्षण "नाम":` blocks (skipped on normal runs) + `lipi test file.swami` runner: each test in a fresh VM with shared setup re-run (isolation), per-test ✓/✗ with failure line, summary, exit 1 on failure (CI-ready); demo `examples/parikshan_demo.swami`
- [x] P1 M  **Formatter** — DONE 2026-06-30 — `lipi fmt file.swami` behavior-preserving + idempotent re-indent/spacing · `src/formatter.rs`
- [x] P1 M  **Linter** — DONE 2026-06-30 — `lipi lint file.swami` flags unused/undefined vars · `src/lint.rs`
- [x] P2 L  **Profiler** — DONE 2026-06-30 — `lipi profile file.swami` (LVM::run_profiled): opcode execution counts + %, total ops, wall-clock time, per-function call counts; verified on fib(20)=218910 ops (text report, not a graphical flame graph)
- [x] P2 M  **REPL improvements** — DONE 2026-06-30 — persistent session state (accumulate + output-delta replay), multiline block input (':'/open brackets), history persistence (~/.lipi_history), :इतिहास/:रीसेट/:सहायता meta-commands (arrow-key recall needs raw-mode TTY — file history + :इतिहास instead)
- [x] P2 M  **Documentation generator** — DONE 2026-06-30 — `lipi doc file.swami` → Markdown from विधि/वर्ग signatures + leading comments · `src/docgen.rs`

---

## PHASE 18 — Grow a Community (6–18 Months)
> Goal: Other developers can discover, learn, and contribute to LIPI

### 18A — Language Features (Important)

- [ ] P1 XL **Async / await** — `async विधि`, `प्रतीक्षा करो` keyword · new opcode set + lvm event loop
- [ ] P1 XL **Gradual type system** — optional type hints, type checker tool · new `src/typechecker.rs`
- [ ] P2 M  **Introspection** — `dir(obj)`, `सहायता(func)`, `isinstance` equivalent · lvm + stdlib
- [ ] P2 M  **`eval` / exec** — evaluate LIPI code at runtime · lvm reentry
- [ ] P2 M  **Mixins** — multiple inheritance for behavior composition · lvm class resolution
- [ ] P2 M  **Weak references** — for caches without preventing GC · lvm
- [ ] P2 L  **Threads** — OS threads with locks, semaphores · Rust std::thread bindings
- [ ] P2 L  **Channels** — message-passing concurrency · Rust std::sync::mpsc
- [ ] P2 L  **Incremental compilation** — only recompile changed modules · compiler + serializer

### 18B — Standard Library (Important Gaps)

- [ ] P2 M  **HTTP server** — serve web requests from LIPI · Rust tiny_http or hand-written
- [ ] P2 M  **WebSocket** — `ws_सर्वर`, `ws_क्लाइंट` · stdlib
- [ ] P2 M  **XML / HTML parsing** · stdlib
- [ ] P2 M  **Email sending** — SMTP · stdlib
- [ ] P2 M  **Argparse** — command-line argument parsing · stdlib
- [ ] P2 M  **Logging** — structured log levels (debug/info/warn/error) · stdlib
- [ ] P2 M  **Config files** — TOML/INI/YAML parsing · stdlib
- [ ] P2 M  **Image basics** — read dimensions, resize, save · Rust image crate bindings

### 18C — Platform & Interop

- [ ] P1 L  **Linux support + CI testing** — cross-compile to x86_64-unknown-linux-gnu
- [ ] P1 L  **Mac support** — Apple Silicon (aarch64-apple-darwin) + Intel
- [ ] P1 M  **Official Docker image** — `docker pull lipi-lang/lipi`
- [ ] P1 M  **GitHub Actions CI** — test every commit on Win/Linux/Mac
- [ ] P2 L  **C FFI** — `बाहरी विधि` to call any C library · `src/ffi.rs`
- [ ] P2 L  **Rust FFI** — embed Rust extensions into LIPI programs
- [ ] P2 XL **Python interop** — call Python packages from LIPI via PyO3
- [ ] P2 M  **WASM improvements** — DOM access, fetch, localStorage from browser
- [ ] P3 L  **Android** — Termux support + JNI bindings
- [ ] P3 L  **iOS** — via WASM in browser

### 18D — Tooling (Important)

- [ ] P2 M  **Hosted playground** — deploy WASM playground online with share URL
- [ ] P2 M  **Error recovery in parser** — show ALL errors, not just first one
- [ ] P2 M  **Warnings system** — deprecation warnings, style hints
- [ ] P2 M  **Version manager** — `lipienv` tool to manage multiple LIPI versions
- [ ] P2 M  **Build system** — `lipi.toml` project manifest (deps, scripts, version)
- [ ] P2 M  **Virtual environments** — isolated per-project packages
- [ ] P3 M  **JetBrains plugin** — IntelliJ / IDEA support
- [ ] P3 M  **Vim/Neovim plugin** — syntax + LSP client config
- [ ] P3 M  **Emacs mode**

---

## PHASE 19 — Ecosystem (18 Months+, Requires Community)
> Goal: Packages, community, tutorials — the 80% that Python won with

### 19A — Community Infrastructure

- [ ] P3 M  **Package registry** — lipi.dev website, `lipi publish` uploads here
- [ ] P3 L  **GitHub org** — open source, contribution guide, issue templates
- [ ] P3 M  **Discord / community forum** — where users get help
- [ ] P3 M  **Semantic versioning** — LIPI version + package versions
- [ ] P3 M  **Security policy** — how to report vulnerabilities, CVE process
- [ ] P3 M  **LIPI language spec** — formal specification document (not just CLAUDE.md)
- [ ] P3 M  **Blog / newsletter** — language progress updates
- [ ] P3 M  **Changelog** — `CHANGELOG.md` per release

### 19B — Content & Education

- [ ] P3 L  **Tutorial website in Hindi** — learn LIPI step by step
- [ ] P3 L  **YouTube channel** — teach LIPI to Hindi-speaking audience
- [ ] P3 XL **School curriculum** — approach CBSE/state boards for CS classes
- [ ] P3 L  **College adoption** — partner with IITs, NITs, regional engineering colleges
- [ ] P3 M  **Localization** — error messages and docs in Hindi, not English
- [ ] P3 M  **Example library** — 100+ example programs covering real use cases

### 19C — Community Packages (50+ needed)

- [ ] P3 L  **Web framework** — Flask-like HTTP framework in LIPI
- [ ] P3 L  **ORM** — database object mapping
- [ ] P3 L  **CLI builder** — build command-line tools easily
- [ ] P3 L  **Template engine** — HTML templating for web apps
- [ ] P3 L  **Testing utilities** — mocking, fixtures, parametrize
- [ ] P3 L  **Data processing** — CSV/Excel/JSON pipelines
- [ ] P3 XL **NumPy-equivalent** — fast array math (requires C/BLAS bindings)

---

## PHASE 20 — Cleanup Batch (2026-07-13) — COMPLETE

Landed 60/60 regression-clean, release binary reinstalled.

### Language additions
- [x] P1 S  **`;` as statement separator** — lexer emits Newline on `;` outside strings
- [x] P1 S  **Scientific-notation literals** — `1e3`, `1.5e-2`, `2E+5`
- [x] P1 S  **Radix literals** — `0xFF`, `0o77`, `0b1011` with `_` separators
- [x] P1 S  **Block comments** — `।।…।।` span multiple lines (lexer preprocessor)
- [x] P1 M  **`__बराबर__` overload** — both `==` and `!=` route through it via `Frame.negate_return`
- [x] P1 M  **Slice assignment** — `सूची[a:b:c] है [...]`, new opcode `SetSlice` (tag 0x4D), List-only
- [x] P1 M  **Class-method decorators** — `@dec विधि x() in वर्ग`, per-class-in-chain dispatch check before FuncDef lookup, subclasses inherit wrapped closures
- [x] P1 S  **`*args` in lambdas** — `लाम्डा(*नाम): ...` packs extras into a List

### New builtins
- [x] P2 S  **Introspection** — `है_उदाहरण(obj, "वर्ग")`, `विशेषताएँ(obj)`, `वर्ग_का(obj)`, `विधियाँ_का(cls)`
- [x] P2 S  **`चलाओ_कोड(src)`** — Python-like exec; shares globals
- [x] P2 S  **`मूल्यांकन(src)`** — Python-like eval; returns expression value

### Runtime / tooling fixes
- [x] P1 S  **Non-zero exit on errors** — `main.rs run()` calls `std::process::exit(1)` on parse+runtime errors
- [x] P1 M  **Parser error recovery** — `parse_recover(tokens) -> (Program, Vec<String>)` reports all parse errors, not just first
- [x] P1 S  **Bitwise > 2^53 guard** — errors instead of silent truncation; hints at भारत.बड़ी
- [x] P1 M  **Property-setter Nil-substitute footgun fix** — `Frame.on_nil_push_local`; setter without `फल यह` no longer clobbers caller's variable

### Verified already working
- [x] P1 XL **Async / await** — `प्रतीक्षा`, `सोओ(ms)`, `चलाओ(gen)`, `इकट्ठा(g1, g2, ...)`. Coroutines inferred from `प्रतीक्षा` in body — no `असंकालिक` keyword needed. Concurrency confirmed via out-of-order sleep test

---

## PHASE 21 — Advanced / Surpass (3–5+ Years)
> Goal: Genuinely better than Python in specific areas

- [~] P4 XL **JIT compiler** — arithmetic + multi-statement (2026-07-13a) + comparison operators (2026-07-13b, batch 6): full CmpOp set via `cmpsd` + andpd-mask trick. Remaining: division with runtime zero-check, control flow (branches for ternary/if-in-expr)
- [x] P4 XL **Full type inference** — Production HM + row polymorphism (2026-07-13 batch 6). Records with row vars, dict-literal → closed record inference, `Attr`/`Index[Str-literal]` field lookup with row-variable extension, 19 hm:: tests all pass
- [ ] P4 XL **Effect system** — track pure/IO/state at compile time
- [~] P4 XL **Self-hosting** — Stages 1+2+3 done (2026-07-13). Stage 3: `examples/selfhost_emitter.swami` walks the AST and emits bytecode with jump backpatching + function tables. Stage 4 (round-trip: LIPI-emitted bytecode running on the LIPI-in-LIPI LVM) is the remaining month(s) of work; Stage 5 (LVM in LIPI) another 6–12 months
- [ ] P4 XL **Parallel collections** — auto-parallelize map/filter with Rayon
- [ ] P4 XL **AOT compiler** — compile LIPI → native binary (no runtime needed)
- [ ] P4 XL **GPU support** — CUDA/Metal/Vulkan tensor ops
- [ ] P4 XL **Scientific computing** — BLAS/LAPACK bindings (NumPy-level)
- [ ] P4 XL **ML tensor library** — native tensors, autograd, training loop
- [ ] P4 XL **Proof system** — formal verification (like Lean/Coq) using Nyaya logic
- [ ] P4 XL **Dependent types** — values as types, total correctness
- [x] P2 M  **Mixins / multiple inheritance** — DONE 2026-07-13. `वर्ग C(A, B, ...)` — depth-first left-to-right MRO in `LVM::mro()`; `है_उदाहरण` walks it. Serializer pairwise-encodes each parent so v3/v4 `.libc` loaders stay compatible
- [x] P2 S  **Weak references** — DONE 2026-07-13. Registry-based (LIPI has no cycles, so it's a semantic API): `कमजोर(obj)` → id-Dict, `पाओ_कमजोर(ref)` → value or शून्य after `मिटाओ_कमजोर(ref)`

---

## Decision Log

| Date | Decision | Reason |
|------|----------|--------|
| 2026-06-11 | Start with Phase 17A default params | Highest impact, moderate effort — unblocks real scripts |
| 2026-06-11 | JSON stdlib before HTTP | HTTP needs JSON to be useful anyway |
| 2026-06-11 | VS Code extension before LSP | Extension is small effort, maximum visibility for new users |
| 2026-06-11 | Defaults are compile-time constants only (number/string/bool) | Stored as LvmValue in FuncDef — no arbitrary-expression evaluation at call time; matches Python's definition-time semantics without closure machinery |
| 2026-06-11 | Serializer bumped to v3 (loads v2 too) | FuncDef layout changed for defaults; also fixed pre-existing bug — class_parents was never serialized, so inheritance broke in every .libc file |
| 2026-06-12 | Kwargs via new runtime opcode CallKw, not compile-time reordering | Callee not always known at compile time (closures, lambdas, inherited constructors); runtime binding handles all of them with one code path. Serializer stays v3 — tag-only additions never bumped versions before |
| 2026-06-12 | Kwargs NOT supported in method calls or built-ins | MethodCall dispatch (list/dict/string/instance) has no FuncDef for most receivers; parser gives a clear error instead. Instance-method kwargs can come later |
| 2026-06-12 | String interpolation restricted to `{identifier}` / `{obj.field}` | Raw JSON text inside strings was being interpolated as variable names; non-identifier `{...}` now stays literal. Required for भारत.json usability |
| 2026-06-12 | Lexer पढ़ो/पकड़ो nukta matching = exact codepoint sequences | Old loose contains-rule turned ANY प…ड़…ो word into पढ़ो — `पथ_जोड़ो(x)` silently became a blocking stdin read. Precomposed (095C/095D) and decomposed (nukta) forms both matched explicitly |
| 2026-06-12 | JSON parser hand-written, no serde | Pure-Rust-only constraint; ~250 lines covers full JSON incl. surrogate-pair escapes |
| 2026-06-12 | समय module: epoch seconds canonical, IST for human-facing fields | One number type fits LIPI's f64 Value; IST (+5:30, no DST) is the natural default for an Indian language — no tz database needed. समय_अभी() errors on WASM (no clock) |
| 2026-06-12 | CSV fields always stay Str on read | Matches Python csv; auto-converting "030" to 30 loses data. Convert explicitly with पूर्णांक() |
| 2026-06-12 | Triple-quote `"""` content is verbatim | Preprocessor escapes `"` AND `\` now — `""` inside used to emit a raw quote (string terminated early). Backslashes no longer interpreted as escapes inside triple quotes |
| 2026-06-12 | Lexer strips leading UTF-8 BOM | Windows editors (and PowerShell `-Encoding utf8`) write BOMs; files failed with Unknown(U+FEFF) |
| 2026-06-12 | Typed errors travel via a `thrown: Option<Value>` side-channel, not a new Err type | exec_op's Err(String) plumbing is everywhere; the channel lets the existing unwinder deliver the Instance while uncaught errors still print a plain message. String errors and typed errors share one catch path |
| 2026-06-12 | `पकड़ो X:` single-ident = typed iff X is a known class | Resolved at compile time via known_classes pre-pass — keeps bare पकड़ो त्रुटि:/पकड़ो गलती: (pre-17A) working unchanged |
| 2026-06-12 | HTTP client is http:// only | Pure-Rust-only constraint rules out rustls/native-tls; std has no TLS. Fine for localhost/LAN APIs and teaching; https errors clearly |
| 2026-06-29 | Operator overloading is arithmetic-only (+ - * / %) | Dunder dispatch reuses the frame-call mechanism so the method's Return supplies the operator result; comparison/== would need to post-process the return into a Bool, which the frame model can't do inline. Deferred |
| 2026-06-29 | Multiline collections via lexer bracket-depth, not parser | The lexer is line-based (emits Newline + INDENT/DEDENT per line); tracking ( [ { depth across lines and suppressing those tokens inside open brackets is a localized change that needs no parser rework |
| 2026-06-29 | Match guards keep the subject on the stack across arms | A failed guard must be able to retry later arms, so the subject value can't be consumed until an arm fully matches (pattern + guard). Switched the मिलाओ compiler from a single last_jf to a pending_next jump list |
| 2026-07-13 | `__बराबर__` overload via a `Frame.negate_return` flag | Revisited the 2026-06-29 "== deferred" decision. NotEq reuses the same dispatch as Eq by setting a per-frame flag that negates the Bool at Return time — avoids a second dunder for `!=` |
| 2026-07-13 | Property-setter Nil is safe now — new `Frame.on_nil_push_local` | The old `फल यह` requirement was a silent-corruption footgun. On Return, if the setter frame returned Nil, we push the current binding of `यह` from the frame instead of Nil, so the caller's variable stays intact |
| 2026-07-13 | Slice assignment splices for step=1, requires exact-length otherwise | Matches Python. Length-changing splice only makes sense contiguously; strided assignment on a mismatched RHS would silently drop or duplicate |
| 2026-07-13 | Class-method decorators via a `__cls_deco_<Class>_<method>__` global | MethodCall dispatch checks per-class in the parent chain before the FuncDef lookup — child classes automatically inherit the wrapped closure |
| 2026-07-13 | `मूल्यांकन(src)` wraps `src` as `__eval_result__ है (src)` | Reusing the compiler+VM keeps eval consistent with the rest of the language. Cleaning `__eval_result__` from the caller's globals hides the wrapper |
| 2026-07-13 | Async/await already worked via existing generators + event loop | No `असंकालिक` keyword added — a function that uses `प्रतीक्षा` is implicitly a coroutine. `चलाओ()` runs one, `इकट्ठा()` runs many concurrently. Verified with an out-of-order sleep test |
| 2026-07-13 | Parser error recovery via `parse_recover(tokens) -> (Program, Vec<String>)` | Statement-boundary sync after each error (advance until Newline/Dedent at bracket-depth 0). Doesn't affect normal `parse()` — recovery is a separate entry point |

---

## Effort Summary

| Phase | Items | Solo Dev Estimate |
|-------|-------|------------------|
| Phase 17 (usable) | 53 items | 2–3 years full-time |
| Phase 18 (community) | 46 items | 2–3 years full-time |
| Phase 19 (ecosystem) | ~30 items | Ongoing forever |
| Phase 20 (advanced) | 11 items | 5+ years each |

**Realistic path:** LIPI becomes production-usable for Indian-context scripting in ~2 years solo. Community traction requires open-sourcing + outreach. Surpassing Python requires 10+ years and a team.
