# Making LIPI fast — the JIT roadmap

LIPI runs on a bytecode VM. That's fine for scripting but leaves 10–100× on the
table for hot numeric loops. This is the plan to close that gap, ordered so each
step ships value on its own.

## Done — Step 0: compile-time constant folding
Literal arithmetic subtrees fold to a single `Push` at compile time
(`Compiler::fold_const`, Add/Sub/Mul + bitwise; Div/Mod left to runtime to preserve
error semantics). No jump addresses move, so it's safe. This is the first real
speed win and the groundwork the later steps build on.

## Step 1 — a bytecode optimizer pass (safe, pure-Rust)
A post-compile pass over `CompiledProgram.instructions` that must rewrite every jump
target (`Jump`/`JumpIfFalse`/`JumpIfTrue`/`TailCall`, `FuncDef.start_ip`) and the
line table when it removes instructions. Candidates:
- dead `Push … Pop` elimination,
- redundant `LoadVar x; StoreVar x`,
- jump-to-jump collapsing,
- `LoadVar` of a known-constant local → `Push`.
Build it behind a `--opt` flag first; validate against the full example suite before
making it default.

## Step 2 — dispatch speedup
Move the VM's `match` dispatch toward a threaded/computed-goto style (or a jump
table), and specialize the hottest opcodes (`Add`, `LoadVar`, `IterNext`). Pure
Rust, measurable, no ABI risk.

## Step 3 — the native JIT (the big one)
A true JIT emits machine code for hot functions. Two paths:
- **Cranelift backend** — the pragmatic industry choice, but Cranelift is an external
  crate, which conflicts with LIPI's pure-Rust/no-crates rule. Adopting it means
  relaxing that rule (a real, defensible decision) — the fastest route to a working
  JIT.
- **Hand-written x86-64/AArch64 emitter** — keeps the no-crates rule but is a
  multi-month project: a register allocator, a code buffer with `mmap`+exec
  permissions, per-opcode codegen, and deopt back to the VM for unsupported ops.

Recommended: profile first (`lipi profile --flame`), JIT only functions that
dominate, keep the VM as the fallback tier, and gate the whole thing behind a
runtime flag so correctness always has the interpreter to fall back to.

## Where the wins are
Tight numeric loops and recursive math (the `भारत.रेखीय`/`गणित` style workloads)
benefit most. I/O-bound and string-heavy code sees little from a JIT — for those,
the stdlib and FFI already do the heavy lifting in native Rust/C.
