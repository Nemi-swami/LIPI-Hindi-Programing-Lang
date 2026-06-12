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
- [ ] P1 L  **List/dict comprehensions** — `[x*2 x के लिए सूची में]` · parser + compiler
- [ ] P1 L  **Generators / yield** — `उत्पन्न करो` keyword · new opcode + lvm
- [ ] P1 L  **Context managers** — `साथ ... करो:` (with statement) · parser + compiler + lvm
- [x] P1 M  **Chained comparisons** — `0 < x < 10` · done 2026-06-12 — parser desugar to और-chain; also fixed old (a<b)<c mis-parse
- [x] P1 S  **Integer division operator** — `//` floor divide · done 2026-06-12 — Python floor semantics, opcode FloorDiv (0x42)
- [x] P1 M  **Spread operator** — `[*सूची1, *सूची2]` · done 2026-06-12 — opcode MakeListSp (0x46), both runtimes, non-list spread = catchable error
- [x] P1 M  **`में_है` / `नहीं_है` operators** — `x में_है सूची` · done 2026-06-12 — List/Str/Dict, opcode Contains (0x45), shared engine in interpreter.rs
- [x] P1 M  **Typed exceptions** — done 2026-06-12 — `फेंको`, `वर्ग X(त्रुटि)`, multi-clause typed `पकड़ो X ई:` with subclass matching + rethrow; opcodes Throw (0x47) / MatchErrClass (0x48); full back-compat with bare पकड़ो त्रुटि:
- [ ] P1 M  **Operator overloading** — `__जोड़ो__`, `__गुणा__` etc. on classes · lvm
- [x] P1 M  **Decorators** — `@लॉग_करो` before function/class · DONE 2026-06-13 — bare `@नाम` + factory `@नाम(आर्ग)`, stacking, top-level/nested functions (not class methods); compiles to existing opcodes via hidden `__deco_X__` registration; test `examples/phase17_decorator_test.swami`
- [ ] P1 L  **Properties** — getter/setter syntax on class fields · parser + compiler + lvm
- [ ] P1 M  **Static / class methods** — `@वर्ग_विधि`, `@स्थिर_विधि` · parser + lvm
- [ ] P1 M  **Abstract classes / interfaces** — duck-type contracts · parser + lvm
- [ ] P1 M  **Multiple assignment targets** — `a = b = 0` · parser + compiler
- [ ] P1 M  **Walrus / inline assignment** — assign + test in one expression · parser
- [ ] P2 M  **Pattern match guards** — `लाल यदि밝기 > 50:` in मिलाओ · parser + compiler
- [ ] P2 M  **Multiline collections** — list/dict across lines without `\` · lexer INDENT handling
- [ ] P2 M  **Named tuples** — lightweight immutable structs · stdlib + lvm
- [ ] P2 M  **Dataclasses** — auto-generate बनाओ/repr/eq · parser modifier

### 17B — Standard Library (Critical Gaps)

- [x] P1 M  **JSON** — `json_पढ़ो(text)`, `json_लिखो(obj)` · done 2026-06-12 — `आयात भारत.json`, hand-written pure-Rust parser (full escapes, surrogate pairs, sci-notation), serializer sorts keys
- [x] P1 L  **Regex engine** — DONE 2026-06-13 — hand-written backtracking VM (pure Rust, WASM-safe, step budget) in `src/regex_engine.rs`; `आयात भारत.प्रतिमान` → ढूंढो / ढूंढो_स्थान / ढूंढो_सब / मेल_है / समूह / बदलो_सब ($N refs) / विभाजित_सब; classes, groups, alternation, lazy quantifiers, \w covers Devanagari; test `examples/phase17_pratimaan_test.swami`
- [x] P1 M  **DateTime** — `आयात भारत.समय` · done 2026-06-12 — 9 functions (बनाओ/विवरण/स्वरूप/पार्स/जोड़ो/दिन_अंतर/अधिवर्ष/माह_दिन/अभी), epoch-seconds canonical + IST display, Hinnant algorithms, pre-1970 OK
- [x] P1 L  **HTTP client** — `आयात भारत.http` · done 2026-06-12 — http_पाओ/http_भेजो, pure std::net HTTP/1.1, chunked decode, timeouts; **no TLS** (https = clear error) — revisit TLS when crate policy changes
- [x] P1 M  **File system extended** — `फोल्डर_सूची()`, `फोल्डर_बनाओ()`, `फाइल_हटाओ()`, `फाइल_कॉपी()`, `पथ_जोड़ो()` · done 2026-06-12 — pre-registered builtins, catchable errors
- [x] P1 M  **OS / environment** — env vars, CLI args, cwd · done 2026-06-12 — `पर्यावरण()`, `तर्क()`, `वर्तमान_फोल्डर()`; args forwarded by both `lipi foo.swami a b` and `lipi run foo.libc a b`
- [ ] P1 L  **Socket / networking** — TCP client/server · Rust std::net bindings
- [x] P1 M  **CSV parsing** — `आयात भारत.csv` · done 2026-06-12 — csv_पढ़ो (RFC 4180), csv_शीर्षक_पढ़ो (→List of Dict), csv_लिखो (auto-quote); also fixed triple-quote preprocessor bug it exposed
- [x] P1 M  **Hash / crypto** — `आयात भारत.कूट` · done 2026-06-12 — sha256/md5 hex digests + base64_कूट/base64_खोलो, pure-Rust reference impls verified against published vectors
- [ ] P1 M  **ZIP / compression** — read/write zip files · Rust miniz_oxide
- [ ] P2 M  **SQL / SQLite** — query local db · rusqlite bindings
- [ ] P2 M  **Statistics module** — mean, median, std-dev, variance, distributions
- [ ] P2 M  **Collections** — Queue, Deque, OrderedDict, Counter · stdlib
- [ ] P2 M  **Itertools** — `zip`, `enumerate`, `chain`, `product`, `combinations` · stdlib HOFs
- [ ] P2 M  **Functools** — memoize, partial application, compose · stdlib
- [ ] P2 S  **UUID generation** · Rust uuid crate or simple random

### 17C — Runtime / VM Fixes (Critical)

- [x] P1 M  **Better error messages** — DONE 2026-06-13 — runtime errors carry `(पंक्ति N)` + source-line snippet with caret (line table parallel to instructions, `Stmt::Located` parser wrapper, .libc v4); col + suggestions still open
- [x] P1 M  **Full stack traces** — DONE 2026-06-13 — uncaught errors print full call chain (`↳ विधि 'X' — पंक्ति N से बुलाई गई`), innermost first; TCO-reused frames show as one entry (correct per TCO semantics); caught errors stay clean for पकड़ो
- [ ] P1 M  **Proper integer type** — separate Int from Float (f64 fails for large ints) · opcode.rs + lvm
- [ ] P1 L  **Big integers** — arbitrary precision · Rust num-bigint or pure implementation
- [x] P1 S  **Stack overflow protection** — DONE 2026-06-13 — `push_frame()` guard, max depth 10000, catchable Hindi error; trace display capped at 12 frames + "… और N स्तर"
- [ ] P1 S  **Memory limits** — configurable max memory, halt gracefully · lvm
- [ ] P1 M  **Unicode normalization** — NFC/NFD for Devanagari · lexer preprocessing
- [ ] P2 L  **Proper GC** — mark-and-sweep or reference counting for large data · lvm refactor

### 17D — Tooling (Critical)

- [~] P1 L  **VS Code extension** — syntax highlighting, snippets · built + packaged 2026-06-12 — `vscode-lipi/lipi-lang-0.1.0.vsix`, installed + verified locally (`code --install-extension`); remaining: marketplace publish (needs publisher account + PAT — user action)
- [ ] P1 XL **LSP server** — autocomplete, go-to-def, hover docs, in any editor · new binary `lipi-lsp`
- [ ] P1 XL **Debugger** — breakpoints, step-through, inspect variables · DAP protocol
- [ ] P1 L  **Package manager** — `lipi install`, `lipi publish`, `lipi.toml` · new CLI subcommand
- [x] P1 L  **Test framework** — DONE 2026-06-13 — `परीक्षण "नाम":` blocks (skipped on normal runs) + `lipi test file.swami` runner: each test in a fresh VM with shared setup re-run (isolation), per-test ✓/✗ with failure line, summary, exit 1 on failure (CI-ready); demo `examples/parikshan_demo.swami`
- [ ] P1 M  **Formatter** — `lipi fmt file.swami` auto-formats code · new `src/formatter.rs`
- [ ] P1 M  **Linter** — `lipi lint` catches unused vars, type mismatches · new `src/lint.rs`
- [ ] P2 L  **Profiler** — `lipi profile file.swami` + flame graph output · lvm instrumentation
- [ ] P2 M  **REPL improvements** — history persistence, tab completion, multiline input · `src/repl.rs`
- [ ] P2 M  **Documentation generator** — `lipi doc` → HTML from source comments · new tool

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

## PHASE 20 — Advanced / Surpass (3–5+ Years)
> Goal: Genuinely better than Python in specific areas

- [ ] P4 XL **JIT compiler** — 10–100x speed for hot loops · LLVM or Cranelift backend
- [ ] P4 XL **Full type inference** — Hindley-Milner, no annotations needed
- [ ] P4 XL **Effect system** — track pure/IO/state at compile time
- [ ] P4 XL **Self-hosting** — LIPI compiler written in LIPI
- [ ] P4 XL **Parallel collections** — auto-parallelize map/filter with Rayon
- [ ] P4 XL **AOT compiler** — compile LIPI → native binary (no runtime needed)
- [ ] P4 XL **GPU support** — CUDA/Metal/Vulkan tensor ops
- [ ] P4 XL **Scientific computing** — BLAS/LAPACK bindings (NumPy-level)
- [ ] P4 XL **ML tensor library** — native tensors, autograd, training loop
- [ ] P4 XL **Proof system** — formal verification (like Lean/Coq) using Nyaya logic
- [ ] P4 XL **Dependent types** — values as types, total correctness

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

---

## Effort Summary

| Phase | Items | Solo Dev Estimate |
|-------|-------|------------------|
| Phase 17 (usable) | 53 items | 2–3 years full-time |
| Phase 18 (community) | 46 items | 2–3 years full-time |
| Phase 19 (ecosystem) | ~30 items | Ongoing forever |
| Phase 20 (advanced) | 11 items | 5+ years each |

**Realistic path:** LIPI becomes production-usable for Indian-context scripting in ~2 years solo. Community traction requires open-sourcing + outreach. Surpassing Python requires 10+ years and a team.
