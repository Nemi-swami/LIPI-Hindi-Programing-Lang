# LIPI — Session Progress Log

Track what was done each session. Most recent at top.

---

## Session: 2026-06-13 — Decorators + runtime error diagnostics (waves 2+3) + install

**Focus:** Finish the queued waves from the perf session: decorators (17A), then
line numbers + stack traces (17C), plus the skipped WASM check and binary install.

**Completed — decorators (17A):**
- `@सजावट` and factory `@कारखाना(आर्ग)` before `विधि`/`शुद्ध विधि`; stacking bottom-up
- Lexer `@` → `TokenKind::At`; `Stmt::Vidhi.decorators: Vec<Expr>`
- Compiler registers decorated fn under `__deco_<name>__`, emits apply-chain +
  `StoreVar(name)` — visible name becomes a closure variable; ZERO new opcodes
- Works: stacking, factories, HOFs, assignment, nested fns; NOT class methods;
  tree-walk interpreter rejects with clear error
- Test: `phase17_decorator_test.swami` — 7 groups, all pass + .libc roundtrip

**Completed — runtime error diagnostics (17C):**
- `Stmt::Located { line, inner }` wrapper from parser; `ast::unwrap_located()` helper
  (used by compiler pre-pass + Varg method extraction)
- Compiler line table `lines: Vec<u32>` parallel to instructions; in CompiledProgram
- `.libc` v4: line table appended after instructions; v2/v3 still load (lines = 0)
- `Frame.func_name` (all 6 construction sites + TailCall rename on reuse)
- Uncaught errors: `msg (पंक्ति N)` + `↳ विधि 'X' — पंक्ति N से बुलाई गई` per frame;
  caught errors stay clean (पकड़ो handlers see plain message — exceptions test green)
- main.rs shows source line + caret for runtime errors (extended show_error_line
  to match `(पंक्ति N)`)
- Demo: `examples/trace_demo.swami` — 3-frame trace verified; TCO frames collapse
  into one entry (correct)

**Verification (real output):**
- Regression 22/22 examples pass; decorator + phase15 .libc v4 roundtrips identical
- wasm32 `cargo check --features wasm` OK
- Release benchmarks: loop 1,638 ms / fib 79 ms / list 166 ms (no regression)
- Release binary installed to `D:\Rust\cargo\bin\lipi.exe` (was a May 24 build)

**Also this session — stack-overflow guard + regex (continued):**
- `LVM::push_frame()` — max call depth 10000, catchable error; trace capped at
  12 frames + "… और N स्तर"; demo `examples/overflow_demo.swami`
- **भारत.प्रतिमान regex** — `src/regex_engine.rs`, hand-written backtracking VM
  (pure Rust, WASM-safe, 2M-step budget): ढूंढो/ढूंढो_स्थान/ढूंढो_सब/मेल_है/समूह/
  बदलो_सब($N refs)/विभाजित_सब; classes/groups/alternation/lazy quantifiers;
  \w covers Devanagari block. Test: phase17_pratimaan_test.swami (10 groups)
- Lexer: unknown escapes keep backslash (Python behavior — "\d+" just works);
  strip_comment now string-aware ("#" in strings no longer truncates the line — was
  a real pre-existing bug)
- New global शून्य = Nil; vals_eq handles Nil == Nil
- Verified: 23/23 regression, .libc roundtrip, wasm32 check, release binary
  re-installed to D:\Rust\cargo\bin\lipi.exe

**Also — test framework (17D):**
- `परीक्षण "नाम":` blocks + `lipi test file.swami` — fresh VM per test, setup
  re-run (isolation verified), ✓/✗ + failure line, exit 1 on fail (CI-ready)
- Demo: `examples/parikshan_demo.swami` (4 pass + 1 deliberate fail, verified)

**Next session:** 17C proper integer type (f64 breaks past 2^53), 17A
comprehensions, or 17D package manager. Trust tier (errors/traces/overflow
guard/regex/test framework) is now complete.

---

## Session: 2026-06-12 (i) — Typed exceptions + भारत.http (2 parallel agents + integration)

**Focus:** Wave 1 of the 4-feature multiagent run: typed exceptions (17A) + HTTP client (17B).
Agents hit the session limit mid-run; exceptions agent left lexer/ast/parser/compiler/
opcode/serializer done but LVM handlers missing (2 compile errors). Finished by hand.

**Completed — typed exceptions (17A):**
- `फेंको expr` throws; `वर्ग X(त्रुटि):` defines error classes; `पकड़ो X ई:` typed clauses
  with subclass matching, checked in order; unmatched → rethrown to outer कोशिश
- Opcodes Throw (0x47) + MatchErrClass (0x48); serializer v3 unchanged
- LVM: `thrown: Option<Value>` channel — unwinder delivers Instance (or message Str)
- Back-compat verified: bare `पकड़ो त्रुटि:` catches strings AND instances; `पकड़ो गलती:`
  (old custom-name form) still works; compiler resolves single-ident via known_classes
- फेंको validates chain reaches त्रुटि; Str throws OK; 42/non-error instance = catchable error
- Uncaught throw halts: `LVM त्रुटि: <class>: <संदेश>`

**Completed — भारत.http (17B):**
- `http_पाओ(url[, headers])` / `http_भेजो(url, body[, headers])` → Dict
  {स्थिति, शीर्षक (lowercase keys), सामग्री}
- Pure std::net HTTP/1.1: Connection: close, 10s timeouts, chunked decoding,
  IPv4+hostnames; https/ftp/bare URLs = clear catchable errors; WASM = error
- Test infra: examples/http_test_server.js (node, 127.0.0.1:8731)

**Tests run (real output, all verified):**
- `phase17_exceptions_test.swami` — 9 groups: typed catch, subclass via parent clause,
  clause order, fall-through to outer, string back-compat ×4, function propagation
  (incl. TCO path), throw-in-loop, 2 invalid-throw errors — all correct
- Uncaught throw halts with class+message (separate check)
- `phase17_http_test.swami` — 8 groups: GET, POST echo (json round trip), chunked,
  404-as-value, https/ftp/no-scheme/conn-refused all caught, custom headers — all correct
- `.libc` round trips: exceptions + http (server up) — identical
- Regression 27/27 + fs test; wasm32 check OK

**Next session should start with:**
- Wave 2 of the multiagent plan: decorators (17A), then wave 3: better error
  messages + stack traces (17C — most cross-cutting, goes last)

---

## Session: 2026-06-12 (h) — भारत.csv + भारत.कूट (CSV, hash, base64)

**Focus:** Two more 17B stdlib gaps in one session

**Completed — भारत.csv:**
- `csv_पढ़ो(text)` → List of List of Str — RFC 4180: quoted fields, `""` escapes,
  embedded commas/newlines, CRLF, blank lines skipped; unterminated quote = catchable
- `csv_शीर्षक_पढ़ो(text)` → List of Dict (header row = keys); length mismatch = catchable
- `csv_लिखो(rows)` → text — auto-quotes `,` `"` newline; `""` escaping; trailing newline
- Fields always stay Str on read (convert with पूर्णांक()) — matches Python csv

**Completed — भारत.कूट:**
- `sha256` / `md5` → hex digests (pure-Rust reference implementations; MD5 K-table
  computed from sin() per RFC 1321)
- `base64_कूट` / `base64_खोलो` — standard alphabet + padding; decode validates
  placement of `=`, rejects bad chars, requires UTF-8 result
- Verified against published vectors ("", "abc", quick-brown-fox for both hashes;
  TWFu/aGVsbG8=) AND independently against .NET SHA256/ToBase64String for Devanagari

**Bugs found & fixed (exposed by CSV test data):**
1. **Triple-quote preprocessor emitted unescaped quote for `""`** — `"""अ ""ब"" स"""`
   broke the lexer (string terminated early, stray `\` leaked). Now emits `\"`.
   Also: literal `\` inside triple quotes now escaped to `\\` — content is verbatim.
2. **UTF-8 BOM crashed the lexer** — files saved with BOM (Windows editors,
   PowerShell `-Encoding utf8`) failed with Unknown(U+FEFF). Lexer strips it now.

**Tests run (real output, all verified):**
- `phase17_csv_test.swami` — 6 groups: parse, quoted/multiline fields, header dicts,
  round trip, number rows, 2 caught errors — all correct
- `phase17_koot_test.swami` — 7 groups: all published vectors exact, Devanagari
  round trip, 3 caught errors — all correct
- `.libc` round trips both identical; regression 26/26 + fs test; wasm32 check OK

**Next session should start with:**
- 17A: typed exceptions / decorators / multiline collections, or 17C: better error
  messages (file:line:col) / stack traces — tooling-grade polish
- Or 17B: HTTP client (JSON done) — needs design decision re: pure-Rust socket impl

---

## Session: 2026-06-12 (g) — VS Code packaging + भारत.समय DateTime module

**Focus:** Finish the VS Code 17D item, then the next 17B stdlib gap (DateTime)

**Completed — VS Code extension packaged:**
- `npx @vscode/vsce package` → `vscode-lipi/lipi-lang-0.1.0.vsix` (8 files, 6.88 KB)
- Installed + verified: `code --install-extension` → `lipi-lang.lipi-lang` listed
- Remaining (user action): marketplace publish — needs publisher account + PAT
- npm cache redirected to `D:\Projects\lipi-lang\.npm-cache` (C-drive rule); gitignored
- `.gitignore` extended: `.npm-cache/`, `*.vsix`, `*.libc`

**Completed — भारत.समय (DateTime, 17B P1):**
- 9 functions: `समय_अभी` `समय_बनाओ` `समय_विवरण` `समय_स्वरूप` `दिनांक_पार्स` `समय_जोड़ो` `दिन_अंतर` `अधिवर्ष` `माह_दिन`
- Design: UTC epoch seconds (Number) = canonical; IST (+5:30, no DST) for wall-time
  in/out — no tz database. Hinnant civil-date algorithms, pure Rust, pre-1970 works
- Validation errors catchable (bad month/day/time, bad parse format)
- `समय_अभी()` = catchable error on WASM (no system clock); rest is WASM-safe
- Wired: lvm.rs Import arm + compiler.rs imported_natives

**Tests run (real output, all verified against known facts):**
- `phase17_samay_test.swami` — epoch 0 = 1970-01-01 05:30 IST; 2000-01-01 = 946665000;
  2026-06-12 = शुक्रवार; 1947-08-15 = शुक्रवार (negative epoch); 162-day span;
  +10h day rollover; leap years 2000/1900/2024/2026; 3 caught errors — all correct
- `.libc` round trip identical; regression 24/24; wasm32 check OK

**Next session should start with:**
- 17B: CSV parsing or hash/crypto (SHA256/base64), or 17A: typed exceptions / decorators
- Marketplace publish of the .vsix when user has a publisher account

---

## Session: 2026-06-12 (f) — Integration of 4 parallel-agent features

**Focus:** Previous session (e) ran 4 agents in parallel (spread operator, भारत.json,
FS/OS builtins, VS Code extension) but hit the session limit right as they finished —
nothing was integration-tested. This session verified, fixed, and documented all 4.

**Verified working as delivered:**
- Spread operator `[*अ, 99, *ब]` — opcode `MakeListSp` (tag 0x46), test passes
- VS Code extension `vscode-lipi/` — all 4 JSON files valid, grammar/snippets/config present

**Bugs found & fixed during integration:**
1. **Lexer पढ़ो rule swallowed `पथ_जोड़ो`** (CRITICAL) — the loose nukta-normalization
   rule (`starts प + contains ़ + ड/ढ + ends ो`) matched `पथ_जोड़ो`, turning every call
   into a blocking stdin read (पढ़ो). Both पढ़ो and पकड़ो rules now exact-match the
   decomposed AND precomposed (U+095C/U+095D) codepoint sequences. (`lexer.rs`)
2. **String interpolation mangled JSON-in-string** — `{...}` was interpolated as a
   variable unless empty/`:`-prefixed, so `"""{"नाम": ...}"""` errored. Now only
   `{identifier}` / `{obj.field}` interpolate; anything else stays literal. (`parser.rs`)
3. **`lipi run foo.libc` dropped CLI args** — added arg forwarding to `तर्क()` for both
   `lipi run x.libc a b` and `lipi x.libc a b`. (`main.rs`)
4. **FS test bugs** — `"..." + सूची` and `"..." + शून्य` concatenations (Str+List/Nil
   unsupported) — wrapped in `वाक्य()`.

**Tests run (real output, all verified):**
- `phase17_spread_test.swami` — 7 groups incl. caught non-list spread error — all correct
- `phase17_json_test.swami` — 5 groups: nested parse, serialize (sorted keys), round trip,
  full escape suite (`\uXXXX` + surrogate-pair emoji), 2 caught invalid-JSON errors — all correct
- `phase17_fs_test.swami <tmp>` — 10 groups: तर्क/पथ_जोड़ो/folder create/write/copy/list/env/cwd/delete/caught error — all correct
- `.libc` round trips: spread, json, fs (with args) — all identical
- Full regression: 23/23 pass (phase5–17 all, demo_full, lvm_test, bharat_stdlib_test, new_modules_test, karaka_test)
- `cargo check --target wasm32-unknown-unknown --features wasm` — OK

**Next session should start with:**
- Package the VS Code extension (`vsce package`) and test install — finishes the 17D item
- Or next 17A: typed exceptions / decorators / multiline collections
- Or next 17B: DateTime module, or CSV (JSON now done)

---

## Session: 2026-06-12 — Phase 17A: Keyword Arguments

**Focus:** `func(नाम=मान)` at call sites — second Phase 17 item

**Completed:**
- Parser: `arg_list_kw()` — `Ident =` lookahead; keywords after positionals only; duplicate keyword rejected; method calls reject keywords with clear error
- AST: new `Expr::CallKw { name, args, kwargs }` (additive — `Expr::Call` untouched)
- New opcode `CallKw(name, pos_argc, kwnames)` — serializer tag 0x41, version stays 3
- LVM `bind_args_kw()`: positionals → params (extras → vararg), keywords by name, defaults fill the rest; errors: unknown keyword / double-binding / missing required param
- Resolution covers: user functions, constructors (incl. inherited `बनाओ` via parent walk), lambdas, closure variables, first-class function refs
- Built-ins/natives + HOFs → runtime error; tree-walk interpreter → error (legacy)
- No TCO for kwarg calls (`फल f(x, k=v)` compiles as normal call + return)

**Tests run (real output, all verified):**
- `examples/phase17_kwargs_test.swami` — 6 groups: reorder, mixed, skip-middle-default, constructor kwargs, inherited ctor kwargs, lambda/first-class, 3 caught runtime errors — all correct
- 3 parse-error cases — clean Hindi messages with line numbers
- `.libc` round trip — identical output
- Full regression: 13/13 pass (phase5–17 + demo_full)
- `cargo check --target wasm32-unknown-unknown --features wasm` — OK

**Next session should start with:**
- `//` integer division (P1 S — quick win), or tuple unpacking `अ, ब है 1, 2` (P1 M)
- Or start 17B: JSON stdlib (decided before HTTP)

---

## Session: 2026-06-12 (b) — Phase 17A: Floor Division + Tuple Unpacking

**Focus:** Two more 17A items in one session

**Completed — `//` floor division:**
- Lexer `SlashSlash`, `BinOp::FloorDiv`, opcode `FloorDiv` (tag 0x42)
- Python semantics: `(a/b).floor()` — `-7 // 2` → -4; div-by-zero error (catchable)
- Same precedence as `*` `/` `%`, left-associative; also added to tree-walk interpreter

**Completed — tuple unpacking `अ, ब है 1, 2`:**
- `Stmt::MultiAssign`; parser arm in `stmt_ident_lead` on `Ident Comma` lead
- Pairwise form: all RHS compiled before reverse-order StoreVar → swap works
- Single-RHS form: `क, ख, ग है सूची` / from function returning list — opcode `UnpackList(n)` (tag 0x43) validates length, pushes reversed, normal StoreVar ops follow (keeps स्थिर/वैश्विक semantics)
- Parse error on explicit count mismatch; runtime errors catchable
- Tree-walk interpreter implements it too

**Tests run (real output):**
- `examples/phase17_floordiv_test.swami` — 6 groups incl. negatives, decimals, precedence, div-by-zero — all correct
- `examples/phase17_unpack_test.swami` — 7 groups incl. swap, list unpack, function multi-return, in-function, 2 caught errors — all correct
- Both `.libc` round trips OK; regression 15/15; wasm32 check OK

**Next session should start with:**
- Slice notation `सूची[1:5]` (P1 M) or `में_है` membership operator (P1 M)
- Or start 17B: JSON stdlib

---

## Session: 2026-06-12 (c) — Phase 17A: Slice Notation

**Focus:** `सूची[start:end:step]` — Python-style slicing

**Completed:**
- `Expr::Slice { obj, start, end, step }` — all parts optional
- Parser: postfix `[` now routes through `bracket_suffix()` — plain `[i]` stays `Expr::Index`, any `:` makes a slice
- New opcode `Slice` (tag 0x44): pops (obj, start, end, step), Nil = omitted part
- Engine `slice_value()` + `slice_indices()` in interpreter.rs — shared by LVM AND tree-walk; verified against Python clamping rules (negative indices, out-of-range → empty, negative step defaults)
- Works on List and Str (char-level); Dict slice + step 0 = catchable errors
- Chained slices work (`सूची[1:5][1:3]`); expression bounds work (`सूची[न:न+2]`)
- NOT supported: slice assignment (`सूची[1:3] है ...`)

**Tests run (real output):**
- `examples/phase17_slice_test.swami` — 7 groups, 20 outputs, all match Python behavior
- `.libc` round trip — byte-identical output
- Regression 16/16; wasm32 check OK

**Next session should start with:**
- `में_है` / `नहीं_है` membership operators (P1 M)
- Chained comparisons `0 < x < 10` (P1 M)
- Or 17B: JSON stdlib

---

## Session: 2026-06-12 (d) — Phase 17A: Membership + Chained Comparisons

**Focus:** `में_है`/`नहीं_है` operators + `0 < x < 10` chains

**Completed — membership:**
- Lexer keywords `में_है` (MeinHai), `नहीं_है` (NahinHai) — single-word match works because `_` is a word char
- `Expr::Membership { item, container, negated }`, parsed at comparison level
- Opcode `Contains` (tag 0x45); `नहीं_है` compiles to `Contains` + `Not`
- Engine `contains_value()` in interpreter.rs (shared both runtimes): List element equality, Str substring (Str items only), Dict key existence; other containers = catchable error

**Completed — chained comparisons:**
- `comparison()` rewritten: `a < b < c` desugars to `(a<b) और (b<c)`, any length, mixed ops
- FIXED pre-existing bug: old loop parsed `a < b < c` as `(a<b) < c` (Bool vs Number)
- Caveat documented: middle expr compiled per pair (side-effecting middles run twice)

**Tests run (real output):**
- `examples/phase17_membership_test.swami` — 7 groups: list/str/dict membership, negation, यदि usage, chained comparisons (incl. `1<5<3` → असत्य, 4-element chain), ternary membership, caught error — all correct
- `.libc` round trip IDENTICAL; regression 17/17; wasm32 OK

**Next session should start with:**
- Spread operator `[*सूची1, *सूची2]` (P1 M) — last quick 17A syntax item
- Or 17B: JSON stdlib (`json_पढ़ो`/`json_लिखो`, pure Rust, decided before HTTP)

---

## Session: 2026-06-11 (b) — Phase 17A: Default Parameters

**Focus:** First Phase 17 implementation item — default parameter values

**Completed:**
- `विधि नमस्ते(नाम="दुनिया"):` — constant defaults (number, string, bool, negative number)
- Lexer: bare `=` now `TokenKind::Assign` (was `Unknown('=')`)
- AST: `Param.default: Option<Expr>`; parser validates constants-only + no required-after-default
- `FuncDef.defaults: Vec<Option<LvmValue>>` — filled at compile time
- LVM `fill_defaults()` applied at ALL 5 bind sites: Call, inherited constructor, closure-var call, MethodCall (instance), TailCall, call_closure_value
- Serializer **v3** (still loads v2): per-param default flag+value reusing PUSH_* tags

**Bugs fixed:**
- Pre-existing: `class_parents` was never serialized — inheritance broke in every `.libc` file (confirmed with phase9_test.libc before fix). v3 now stores the child→parent table.

**Tests run:**
- `examples/phase17_default_params.swami` — 8 groups, all pass (incl. defaults+vararg, class/inherited ctor, first-class fn, TCO)
- All `phase*_test.swami` + `demo_full.swami` — 0 failures
- `.libc` round trip for phase17 + phase9 — pass
- Parse errors verified: non-constant default, required-param-after-default

**Next session should start with:**
- Next Phase 17A item: keyword arguments (`func(नाम=मान)` at call site) — natural follow-on, parser `arg_list` + new call opcode or arg-reorder at compile time
- Or: `//` integer division (P1 S — quick win)

---

## Session: 2026-06-11

**Focus:** Planning session — competitive analysis + master roadmap

**Completed:**
- Full competitive analysis: LIPI vs Python, JS, Java, C++, Rust
- Identified 124 specific items needed to surpass top 5 languages
- Organized into 4 tiers (Critical / Important / Ecosystem / Advanced)
- Created master plan (`task_plan.md`) with all items tracked
- Created findings log (`findings.md`) with research and decisions
- Created this progress log (`progress.md`)

**Key decisions:**
- Phase 17 next: default params, JSON stdlib, VS Code extension, better errors, test framework
- Focus on Indian-context niche rather than trying to out-Python Python generally
- Open source + community building is the real path to growth

**Current state:** Phases 1–16 complete. 17 stdlib modules. TUI editor v3. WASM playground.  
**Opcodes used:** 0x00–0x40. Next free: 0x41.  
**Serializer version:** 2. Bump to 3 when next opcode tags added.

**Next session should start with:**
- Pick one Phase 17 item and implement it
- Recommended first: default parameters (17A) — high impact, moderate effort

---

## Session: 2026-06-09 (Phase 16 + Phonetic + Editor v3)

**Completed:**
- Phase 16: `जाँचो` assert, `स्थिर` immutable, `शुद्ध` pure function syntax
- TUI Editor v3: syntax highlighting, undo/redo, find/replace, go-to-line, comment toggle
- Phonetic input `.vani` — full two-pass character-level transliteration
- 17 stdlib modules including भारत.तंत्रिका, भारत.अनुकूलन, भारत.प्रज्ञा, भारत.तुरिंग, भारत.यंत्र
- New opcodes: Assert (0x3F), DeclareConst (0x40)
- All phase tests passing: `examples/phase16_test.swami`

---

## Template for Future Sessions

```
## Session: YYYY-MM-DD

**Focus:** [what was worked on]

**Completed:**
- 

**Bugs fixed:**
- 

**Decisions made:**
- 

**Tests run:** [which .swami files, what output]

**Next session should start with:**
- 
```
