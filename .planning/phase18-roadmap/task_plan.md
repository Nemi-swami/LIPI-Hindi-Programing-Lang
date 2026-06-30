# Phase 18 — LIPI Studio + Async + Mission-Critical Computing

**Status: PLAN ONLY (not started).** Decisions locked with the user 2026-06-30:
- Editor = a **full-fledged GUI IDE that looks like VSCode/Sublime** (→ Monaco-based, since Monaco IS VSCode's editor engine). NOT a VSCode extension.
- Mission-critical built-ins = **ALL six modules**.
- Generators + async = **both**, with async kept as an independent opt-in layer.

Pure Rust core, no external crates (frontend JS assets like Monaco are allowed, same as the existing web/ playground). Each item: design → implement → build → test with real output → check off.

---

## Phase E — Language & runtime
- [x] E1 **Lazy generators (true coroutines)** — DONE 2026-06-30 — `Value::Generator(id)` handle + VM
      generator registry (GenState{ip,stack,frames,done}); `resume_generator` runs a sub-loop until
      `Yield` (tag 0x4A) or frame return. New unified `IterStep` opcode (tag 0x4B) drives ALL for-loops
      (list/str/dict/range + generators); FuncDef.is_generator (serializer v5). Builtins `आगे(gen)` (manual
      advance) + `सूची_में(gen)` (materialize). Infinite generators + break verified. 7/7 groups + 54/54
      regression pass (examples/phase17_generator_test.swami).
- [x] E2 **Async / await** — DONE 2026-06-30 — `प्रतीक्षा expr` (await, Expr::Await→Yield); functions
      containing await/उत्पन्न are coroutines (body_is_coroutine). LVM::drive (resume + send-value) +
      run_event_loop cooperative scheduler. Builtins: `सोओ(ms)` (sleep marker), `चलाओ(task)` (run one),
      `इकट्ठा(tasks…)` (gather/concurrent). Concurrency PROVEN: 3×200ms gather → 232ms not 600ms. 57/57 regression.
- [x] E3 **Module namespaces** — DONE 2026-06-30 — `आयात "x" के_रूप_में नाम` (Stmt::AayatFileAs); compile-time
      alias→funcset, `नाम.func()` compiles to direct Call. v1 inlines under original names. Test passes.
- [x] E4 **Method-call keyword args** — DONE 2026-06-30 — `obj.m(क=व)` via Expr::MethodCallKw + MethodCallKw
      opcode (0x4C); binds through bind_args_kw. 56/56 regression. Test passes.

## Phase F — Profiling
- [x] F1 **Flame-graph profiler** — DONE 2026-06-30 — `lipi profile --flame file.swami > out.svg`;
      LVM::run_flame samples the call-stack path per instruction (folded stacks); src/flame.rs builds a
      tree + emits a self-contained pure-Rust SVG flame graph (+ folded text to stderr). Verified on
      recursive fib (4652 samples, correct recursion tree).

## Phase G — LIPI Studio (full-fledged IDE, VSCode/Sublime look)
- [x] G1 **WASM API surface** — DONE 2026-06-30 — lib.rs wasm_bindgen exports: run_source, lipi_diagnostics,
      lipi_symbols, lipi_completions, lipi_hover; built with wasm-pack (web/pkg), exports verified.
- [x] G2 **Monaco language integration** — DONE 2026-06-30 — Monarch tokenizer for LIPI (keywords/strings/
      comments/numbers incl Devanagari digits), lipi-dark theme, completion + hover + live diagnostics
      providers wired to the WASM API. Monaco = authentic VSCode editor.
- [x] G3 **IDE shell** — DONE 2026-06-30 — web/studio/index.html: activity bar, file explorer, multi-file
      tabs (localStorage), Monaco editor + minimap, output + Problems panels (click-to-line), status bar
      (Ln/Col, problem count), run/format/new-file buttons, command palette + find/replace (Monaco built-in).
- [x] G4 **`lipi ide` launcher** — DONE 2026-06-30 — src/ide.rs pure-Rust std::net static server (port 8790),
      correct MIME types incl application/wasm, auto-opens browser; verified serving all assets.
- [x] G5 **Desktop feel** — DONE 2026-06-30 — launcher opens Chrome/Edge in `--app` mode (standalone-window
      look) when found, else default browser. No GUI crate.
- [~] G6 **In-IDE debugger** — PARTIAL — Problems panel with click-to-line navigation + live error markers
      shipped; full in-browser step-debugger (breakpoints/variables) deferred (CLI `lipi debug` covers it).
      NOTE: browser rendering itself not auto-tested (no headless browser available); server/MIME/WASM
      exports/JS-syntax/HTML-structure all verified.

## Phase H — Mission-Critical Computing suite (ALL six, pure Rust)
- [ ] H1 **`भारत.मात्रक` (units / dimensional analysis)** — values carry SI units; arithmetic checks/propagates
      dimensions; mismatches = catchable error. *The Mars Climate Orbiter ($327M, lbf-vs-N) bug catcher.*
      Conversions (m/ft, kg/lb, N/lbf, K/°C, …). FLAGSHIP feature.
- [ ] H2 **`भारत.रेखीय` (linear algebra)** — vectors (dot/cross/norm), matrices (mul/transpose/det/inverse,
      solve), **quaternions** (mul/normalize/slerp/to-from-euler) for attitude without gimbal lock.
- [ ] H3 **`भारत.नियंत्रण` (control)** — **PID controller** (with anti-windup), **Kalman filter** (predict/
      update) for GPS+IMU sensor fusion.
- [ ] H4 **`भारत.दिशा` (navigation / geodesy)** — haversine/great-circle distance, initial bearing,
      destination point, ECEF↔lat-lon transforms.
- [ ] H5 **`भारत.सुरक्षा` (fault tolerance)** — **Hamming(7,4) ECC** (correct single bit-flips — cosmic-ray
      SEUs), CRC-32 (reuse zip CRC), **triple-modular-redundancy voter**, watchdog/deadline budget.
- [ ] H6 **`भारत.अंतराल` (interval arithmetic) + deterministic mode** — guaranteed numeric bounds
      [lo, hi]; a `--deterministic` run mode forbidding यादृच्छिक/समय_अभी/network for bit-exact reproducible
      simulations (verification & validation).

## Phase I — Shareable addons / package integrations (was lost #10)
- [ ] I1 **Remote package install via git** — let ANY person publish a `.swami` addon (a repo with
      `<name>.swami` or `lib.swami` at root) and let anyone install it by link. PENDING — designed
      with user 2026-06-30, approved.
      - `lipi pkg add <नाम> <स्रोत>` accepts a git URL (starts `http://`/`https://`/`git@`/`ssh://`,
        or ends `.git`) in addition to a local path; optional `#ref` pins a tag/branch. Stored in
        `lipi.toml` deps as-is (value = URL).
      - `lipi pkg install`: for git deps → `git clone --depth 1` into a temp dir, locate entry file
        (`<name>.swami` else `lib.swami`, same rule as existing dir deps), copy to
        `lipi_modules/<name>.swami`, clean up temp. Local-path deps unchanged. `आयात "<नाम>"` works as-is.
      - Pure Rust: shells out to the user's `git` (LIPI's own http client is http-only / no TLS, can't
        reach GitHub https). Decided over curl/http-only.
      - Clear catchable Hindi errors: git not installed / clone failed / entry file missing; install
        still prints "X सफल, Y विफल" and never crashes.
      - Test OFFLINE: create a local throwaway git repo as a package, `pkg add` + `install` it, run a
        `.swami` that imports + calls it (git clones from a local folder, no network needed).
      - Files: `src/pkg.rs` (no compiler/VM/serializer changes — import resolution already handles
        `lipi_modules/`).

## Final
- [ ] Full regression green + new test files per item
- [ ] Update CLAUDE.md (modules/builtins/CLI), task_plan.md, memory

---

## Dependencies / sequencing notes
- E2 (async) requires E1 (coroutines) first.
- G2/G6 (Monaco language features + in-IDE debug) require G1 (WASM API).
- H5 CRC reuses existing src/zip.rs crc32.
- H1 (units) is the highest-distinctiveness item — recommend building it first in Phase H.
- Suggested build order: E1→E3→E4→F1 (quick wins) · then H1→H2→H3→H4→H5→H6 · then G1→G2→G3→G4→G6→G5 (IDE is the largest) · E2 (async) any time after E1.

## Open item (not a code task)
- No git remote configured — to publish `master` to GitHub/GitLab, add a remote first (user action).

## Phase H — Mission-Critical Computing suite (ALL six, pure Rust)
- [x] H1 **भारत.मात्रक (units / dimensional analysis)** — DONE 2026-06-30 — src/matrak.rs; quantities = SI
      value + 7-vector dimension exponents; मात्रा/जोड़_मात्रा/घटा/गुणा/भाग/मान_में/मात्रा_वाक्य/विमा_बराबर;
      arithmetic dimension-checked (force+length → विमा बेमेल; 500N+100lbf auto-converts). 6/6 pass.
- [x] H2 **भारत.रेखीय** — DONE 2026-06-30 — vectors/matrices(det,inverse)/quaternions; 4/4 pass.
- [x] H3 **भारत.नियंत्रण** — DONE 2026-06-30 — PID (anti-windup) + 1D Kalman; converge/smooth verified.
- [x] H4 **भारत.दिशा** — DONE 2026-06-30 — haversine/bearing/destination/ECEF; Delhi→Mumbai=1149km.
- [x] H5 **भारत.सुरक्षा** — DONE 2026-06-30 — Hamming(7,4) corrects bit-flips, CRC32, TMR voter, deadline.
- [x] H6 **भारत.अंतराल** — DONE 2026-06-30 — interval arithmetic + बीज_सेट() deterministic seed.
