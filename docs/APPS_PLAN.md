# Building Apps in LIPI — Plan

> **Status: PLAN ONLY.** Nothing here is implemented yet. This document lays out
> how real applications (desktop GUI, web, CLI) would be built in LIPI, what
> already exists to support each path, and the concrete next steps. It exists so
> the direction is agreed before any code is written.

LIPI already has the two things an app platform needs at its foundation: a
**bytecode VM that also runs in the browser (WASM)**, and **`भारत.बाह्य` (FFI)**
to call any native UI/OS library. Every path below builds on one of those.

---

## Path A — Web apps (highest leverage, mostly exists)

The LVM already compiles to `wasm32-unknown-unknown` and runs in the browser
(`web/index.html`, `web/pkg/`). **LIPI Studio** (`web/studio/`) is a full
Monaco-based IDE served by `lipi ide`. So the runtime is done; what's missing is a
**DOM/browser API bridge**.

### Plan

1. **`भारत.दस्तावेज़` (DOM module, WASM-only)** — thin `wasm-bindgen` imports for
   `document.getElementById`, `createElement`, `addEventListener`, `setText`,
   `setAttribute`. Gated behind the `wasm` feature; native builds return a
   catchable error (mirror of how `भारत.संजाल` handles WASM).
2. **Event loop** — reuse the async/await machinery (`प्रतीक्षा`, already present)
   so click handlers are LIPI closures scheduled on the existing task queue.
3. **`भारत.आनयन` (fetch)** — WASM `fetch()` wrapper, returning the same Dict shape
   as the existing `भारत.http` client so code is portable native↔web.
4. **Bundling** — `lipi build --web app.swami` emits an `index.html` + `app.wasm`
   + glue, deployable to any static host.

**Effort:** M. **Blocking nothing** — pure additive WASM modules.
**Deliverable:** to-do app / dashboard written entirely in `.swami`, running in a
browser tab with no server.

---

## Path B — Desktop GUI apps (via FFI, no new core)

With `भारत.बाह्य` we can drive a native windowing/UI library **today**, without
adding anything to the interpreter. Two sub-options:

### B1 — WebView (recommended: reuse the web UI)

Render the UI as HTML/CSS/JS in an OS WebView, LIPI as the backend logic.

- **Windows:** bind `WebView2Loader.dll` (Microsoft Edge WebView2) via FFI, or the
  simpler `Shell_NotifyIcon`/`CreateWindowExW` + embedded control.
- LIPI ships the HTML, handles events over a small message channel.
- **Win:** one UI codebase shared with Path A; native window, native perf.

### B2 — Immediate-mode / native widgets

Bind SDL2 (`SDL2.dll`) or raw Win32 (`user32.dll`, `gdi32.dll`) through FFI for
games, kiosks, instrument panels.

- `user32.dll`: `CreateWindowExW`, `GetMessageW`, `DefWindowProcW` (needs a
  callback-pointer story — see Risks).
- SDL2: `SDL_CreateWindow`, `SDL_CreateRenderer`, event polling loop in LIPI.

### Plan (shared)

1. Author a Tier-2 addon package `lipi-gui` (per `docs/COMMUNITY.md`) wrapping the
   chosen DLL so app authors never see `बाह्य_बुलाओ`.
2. Provide a `विंडो`/`बटन`/`घटना` (window/button/event) LIPI API on top.
3. ~~**Prerequisite core work:** FFI callbacks~~ — **DONE** (Phase 19 F3,
   `src/cbthunk.rs`). `बाह्य_कॉलबैक(closure, "ll:i")` returns a C function pointer
   backed by a LIPI closure (int/ptr args, ≤4), proven with C `qsort`. Window procs
   and event callbacks can now be LIPI closures.

**Effort:** B1 = M, B2 = L. **No longer blocked** — the callback primitive exists.

---

## Path C — CLI apps & services (essentially ready)

LIPI is already a competent scripting language for terminal tools and network
services: `तर्क()` for argv, full FS builtins, `भारत.सर्वर` (HTTP server),
`भारत.संजाल` (sockets), `भारत.सूत्र` (threads), `भारत.संग्रह` (SQL).

### Plan

1. **`भारत.तर्कपार्स` (argparse)** — declarative flags/subcommands → Dict. Pure
   Rust, portable. (Listed in the roadmap as "Argparse".)
2. **`lipi build` single-file bundle** — bundle a script + its `lipi_modules/`
   into one `.libc`, run with `lipi run app.libc`. Extends the existing serializer.
3. **Distribution** — document shipping `lipi.exe` + `app.libc` as a folder, or a
   future self-extracting stub (roadmap "one-file installer").

**Effort:** S–M. **Blocking nothing.** **Deliverable:** a real CLI tool (e.g. a
git-style multi-command app) distributed as `lipi run app.libc`.

---

## Cross-cutting: packaging

| Target | Artifact | Mechanism |
|--------|----------|-----------|
| Web | `index.html` + `.wasm` | `lipi build --web` (Path A step 4) |
| Desktop | `lipi.exe` + `app.libc` + bundled DLLs | folder or installer |
| CLI | `app.libc` | `lipi run`, or self-extracting stub |

A `lipi.toml` `[app]` section would declare entry point, target, icon, and bundled
native deps so `lipi package` produces the right artifact per target.

---

## Risks / open questions

1. **FFI callbacks** (Path B) — passing a LIPI closure as a C function pointer is
   the main new primitive. Needs a marshalling trampoline; single-threaded VM
   keeps it tractable but re-entrancy must be handled.
2. **Cross-platform FFI** — `भारत.बाह्य` is Windows-first (LoadLibrary). Linux/mac
   (`dlopen`) is a straightforward addition but currently unbuilt.
3. **Bundled-DLL licensing** — apps redistributing native libs must track licences
   (documented in the addon convention).
4. **Security** — FFI is unrestricted by design ("no limits"). An app-distribution
   story should let end users see which native libraries an app loads.

---

## Recommended sequencing

1. **Path C step 1–2** (argparse + single-file bundle) — small, unblocks real CLI
   apps immediately, no new risk.
2. **Path A** (DOM + fetch WASM modules) — highest reach, purely additive, gives
   browser apps.
3. **FFI callbacks**, then **Path B1** (WebView desktop) — reuses Path A's UI.
4. **Cross-platform FFI** (`dlopen`) to lift the Windows-only constraint.

Nothing here changes existing behavior; every item is additive. The foundation
(WASM runtime + FFI) already exists — these are bridges on top of it.
