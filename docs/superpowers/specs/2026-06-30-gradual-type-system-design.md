# LIPI Gradual Type System + `lipi check` — Design

**Date:** 2026-06-30
**Status:** Approved
**Phase:** 18 (#7)

## Goal

Add optional Devanagari type annotations and a static checker (`lipi check`) that
flags obvious type mismatches without running the program. Gradual: unannotated
code behaves exactly as today; only annotated code is checked. Zero runtime change
— annotations are parse-only metadata, the compiler/VM ignore them, the `.libc`
format is unchanged.

## 1. Type vocabulary

New module `src/types.rs`:

```rust
pub enum TypeHint { Number, Str, Bool, List, Dict, Nil, Any, Named(String) }
```

`TypeHint::from_name(&str) -> TypeHint` accepts aliases (both vocabularies):

| TypeHint        | Names accepted          |
|-----------------|-------------------------|
| Number          | `संख्या`, `अंक`          |
| Str             | `वाक्य`, `पाठ`           |
| Bool            | `तर्क`, `बूल`            |
| List            | `सूची`                   |
| Dict            | `कोश`                    |
| Nil             | `शून्य`                  |
| Any             | `कुछ_भी`                 |
| Named(class)    | any other identifier    |

`Any` is the gradual escape hatch — compatible with everything, never flagged.
`Named(X)` is a nominal class type, checked permissively (compatible with any
Instance / Any).

## 2. Syntax

Canonical form: Python-style colon for value types, `->` arrow for return type.
(The Devanagari keyword `प्रकार` is *not* used as an annotation keyword because it
collides with the existing `प्रकार()` type-inspection builtin.)

```lipi
विधि जोड़ो(अ: संख्या, ब: संख्या = 0) -> संख्या:
    फल अ + ब

नाम: वाक्य है "राम"
आयु: संख्या है 30
```

All annotations are optional. Annotation order on a parameter:
`name [karaka] [: type] [= default]`.

Lexer: add `TokenKind::Arrow` for `->`. `:` (Colon) already tokenizes. Inside a
parameter list the `:` cannot be confused with the block-terminating `:` because
parsing is bounded by `)`.

## 3. AST changes

- `Param` gains `type_hint: Option<TypeHint>`.
- `Stmt::Vidhi` gains `ret_type: Option<TypeHint>`.
- `Stmt::Assign` gains `type_hint: Option<TypeHint>` (the `क: संख्या है expr` form).

The compiler ignores all three. No new opcodes. `.libc` serializer unchanged.

## 4. The checker — `src/typecheck.rs`

A static pass over the parsed `Vec<Stmt>`, independent of compiler and VM.

State:
- a stack of scopes mapping `var name -> TypeHint`
- a function signature table: `name -> (Vec<param TypeHint>, ret TypeHint)`,
  built in a pre-pass so forward references resolve.

Expression type inference (`infer(expr) -> TypeHint`):
- Number/Str/Bool/List/Dict/Nil literals → the exact type.
- Variable → its annotated type from scope, else `Any`.
- Arithmetic BinOp (`+ - * / // % ** & | ^ << >>`): operands expected Number,
  except `+` also allows Str+Str and List+List. Mismatch → diagnostic. Result type
  inferred from operands (Number for arithmetic, Str/List for `+` of those).
- Comparison / logical / membership → Bool.
- Call → callee's declared return type; each argument checked against the matching
  param type.
- Anything unknown → `Any`.

Statement checks:
- `फल expr` inside a function with a `ret_type` → `infer(expr)` must be compatible
  with `ret_type`.
- `क: T है expr` → `infer(expr)` must be compatible with `T`; record `क: T`.
- Reassignment of an annotated variable → new value must stay compatible.

Compatibility (`compatible(expected, actual) -> bool`), gradual:
- `Any` on either side → compatible.
- `Named(_)` on either side → compatible (permissive nominal).
- otherwise → exact match of the primitive kind.

Only flag when **both** sides are known, concrete, disagreeing primitives.

Output: `Vec<Diagnostic { line: u32, message: String }>` (messages in Hindi).

## 5. CLI

`main.rs`: add a `check` subcommand. Pipeline: read file → `lexer::tokenize` →
`parser::parse` → `typecheck::check(&stmts)`. Print each diagnostic via the
existing `show_error_line` style (source line + caret). Exit 0 if clean, 1 if any
mismatch, 2 on parse/file error. Never executes the program.

## 6. Testing (TDD)

- Rust `#[cfg(test)]` unit tests in `typecheck.rs` for the compatibility matrix and
  a few infer cases.
- `examples/phase18_typecheck_test.swami` — fully annotated, must pass `lipi check`
  cleanly **and** still run correctly under `lipi`.
- A deliberately-bad snippet (in the test or a sibling file) proving mismatches are
  reported with the right line.
- Full existing example suite must stay green (untyped code is unaffected).

## Non-goals (YAGNI)

- No runtime type enforcement.
- No whole-program / Hindley-Milner inference.
- No generics, unions, or optional types (`कुछ_भी` covers gradual cases).
- No user-defined type aliases beyond nominal class names.
