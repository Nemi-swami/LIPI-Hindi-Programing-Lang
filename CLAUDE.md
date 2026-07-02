# LIPI — CLAUDE.md

LIPI is an Indian programming language written in pure Rust. It uses Devanagari script and
Hindi/Sanskrit keywords. The runtime is a bytecode VM (LVM) — the AST is compiled to opcodes,
then executed. A tree-walk interpreter exists in `interpreter.rs` but is no longer the default.

---

## Build

```powershell
$env:RUSTUP_HOME = "D:\Rust\rustup"
$env:CARGO_HOME  = "D:\Rust\cargo"
$env:PATH = "D:\Rust\cargo\bin;D:\msys64\mingw64\bin;$env:PATH"
Set-Location "D:\Projects\lipi-lang"
cargo build --target x86_64-pc-windows-gnu
```

Binary: `target\x86_64-pc-windows-gnu\debug\lipi.exe`

## CLI

```
lipi foo.swami            # compile + run (default)
lipi build foo.swami      # compile → foo.libc
lipi run foo.libc         # execute precompiled bytecode
lipi test foo.swami       # run परीक्षण blocks (Phase 17 test framework)
lipi fmt foo.swami        # auto-format (behavior-preserving + idempotent) — Phase 17D
lipi lint foo.swami       # flag unused/undefined variables — Phase 17D
lipi check foo.swami      # static gradual type checker (optional hints) — Phase 18 #7
lipi doc foo.swami        # emit Markdown docs from विधि/वर्ग + leading comments — Phase 17D
lipi profile foo.swami    # run + print opcode/function profile report — Phase 17D
lipi debug foo.swami      # interactive line debugger (step/break/print/vars) — Phase 17D
lipi trace foo.swami      # JSON step trace (line+vars+depth) for the in-IDE debugger — Phase I
lipi lsp                  # Language Server Protocol over stdio — Phase 17D
lipi pkg init|add|install|list   # local package manager (lipi.toml + lipi_modules/) — Phase 17D
lipi edit foo.swami       # open terminal editor with line numbers
lipi                      # REPL (persistent state, multiline blocks, ~/.lipi_history)

# Roman QWERTY → Devanagari (keyword-only replacement)
lipi roman foo.roman      # translate keywords, then run
lipi roman-show foo.roman # print translated source

# Phonetic input → Devanagari (keywords + identifiers)
lipi phonetic foo.vani    # full phonetic transliteration, then run
lipi phonetic-show foo.vani  # print translated source
lipi foo.vani             # auto-detected by extension
```

---

## Source layout

| File | Role |
|------|------|
| `src/lexer.rs` | Devanagari tokenizer, Python-style INDENT/DEDENT, lakh/crore number suffixes |
| `src/ast.rs` | AST nodes — `Stmt`, `Expr`, `BinOp`, `CmpOp`, `Param` |
| `src/parser.rs` | SOV grammar parser, produces `Vec<Stmt>` |
| `src/karaka.rs` | 6-Karaka semantic-role type system (soft warnings only) |
| `src/interpreter.rs` | Tree-walk interpreter — kept but no longer default; defines `Value` enum |
| `src/bharat_stdlib.rs` | भारत standard library — 4 modules, pure Rust |
| `src/opcode.rs` | `LvmValue`, `FuncDef`, `CompiledProgram`, `Opcode` enum |
| `src/compiler.rs` | AST → LVM bytecode (`Compiler::compile_program`) |
| `src/lvm.rs` | Stack VM (`LVM::run`) |
| `src/serializer.rs` | Binary `.libc` save/load |
| `src/editor.rs` | Terminal line editor (`lipi edit`) |
| `src/formatter.rs` | `lipi fmt` — behavior-preserving, idempotent source formatter (Phase 17D) |
| `src/lint.rs` | `lipi lint` — unused/undefined variable checker (Phase 17D) |
| `src/types.rs` | `TypeHint` enum + alias map for gradual type annotations (Phase 18 #7) |
| `src/typecheck.rs` | `lipi check` — static gradual type checker over the AST (Phase 18 #7) |
| `src/docgen.rs` | `lipi doc` — Markdown doc generator from विधि/वर्ग + comments (Phase 17D) |
| `src/bignum.rs` | `भारत.बड़ी` — pure-Rust arbitrary-precision integers (Phase 17B) |
| `src/net.rs` | `भारत.संजाल` — TCP sockets via std::net + thread-local handle registry (Phase 17C) |
| `src/zip.rs` | `भारत.संपीडन` — ZIP read (STORE+DEFLATE inflate) / write (STORE) + CRC32 (Phase 17C) |
| `src/sql.rs` | `भारत.संग्रह` — minimal in-memory SQL engine + file persistence (Phase 17C) |
| `src/pkg.rs` | `lipi pkg` — local package manager (lipi.toml + lipi_modules/) (Phase 17D) |
| `src/lsp.rs` | `lipi lsp` — Language Server Protocol over stdio, self-contained JSON (Phase 17D) |
| `src/flame.rs` | `lipi profile --flame` — SVG flame-graph generator (Phase 18 F1) |
| `src/ide.rs` | `lipi ide` — pure-Rust static server launching LIPI Studio (Phase 18 G) |
| `src/matrak.rs` | `भारत.मात्रक` — units / dimensional analysis (Phase 18 H1) |
| `src/rekhiy.rs` | `भारत.रेखीय` — linear algebra: vectors/matrices/quaternions (Phase 18 H2) |
| `src/niyantran.rs` | `भारत.नियंत्रण` — PID + 1-D/N-D Kalman control (Phase 18 H3) |
| `src/server.rs` | `भारत.सर्वर` — minimal HTTP server via std::net (Phase 18 #8) |
| `src/threads.rs` | `भारत.सूत्र` — OS threads / parallel source execution (Phase 18 #9) |
| `src/disha.rs` | `भारत.दिशा` — navigation / geodesy (Phase 18 H4) |
| `src/suraksha.rs` | `भारत.सुरक्षा` — Hamming ECC / CRC / TMR fault tolerance (Phase 18 H5) |
| `src/antaral.rs` | `भारत.अंतराल` — interval arithmetic (Phase 18 H6) |
| `web/studio/index.html` | LIPI Studio — Monaco-based browser IDE (Phase 18 G), served by `lipi ide` |
| `src/main.rs` | Entry point — routes source through compiler → LVM |
| `src/lib.rs` | WASM library root — exposes `run_source` via `wasm_bindgen` |

---

## Execution pipeline

```
source.swami
  → lexer::tokenize()                         Vec<Token>
  → parser::parse()                           Vec<Stmt>
  → compiler::Compiler::compile_program()     CompiledProgram { instructions, functions, class_parents }
  → lvm::LVM::run()                           stdout  (or vm.output for WASM)
```

For `.libc` files the compiler step is replaced by `serializer::load()`.

---

## LIPI syntax quick reference

```lipi
। comment ।                     # inline comment (danda-delimited, single-line only)
# comment                       # hash comment

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Variables & Print
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
क है 5                          # assignment:  name है expr
अ, ब है 1, 2                    # tuple unpacking (Phase 17) — pairwise
अ, ब है ब, अ                    # swap — RHS fully evaluated before stores
क, ख, ग है [10, 20, 30]         # unpack a list (length must match exactly)
भाग, शेष है भाग_शेष(17, 5)      # unpack a function's returned list
बताओ क + 1                      # print with newline
लिखो "नाम: "                    # print without newline
बताओ 7 // 2                     # floor division (Phase 17) → 3 (-7 // 2 → -4)

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Functions
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
विधि जोड़ो(अ, ब):               # function definition
    फल अ + ब                    # return

विधि जोड़_सब(*संख्याएँ):        # varargs — extra args become a List
    कुल है 0
    i के लिए संख्याएँ में:
        कुल है कुल + i
    फल कुल

विधि नमस्ते(नाम="दुनिया"):      # default parameters (Phase 17) — constants only:
    फल "नमस्ते " + नाम          # number, string, सत्य/असत्य, negative number
                                 # defaulted params must come after required ones

परिणाम है जोड़ो(3, 4)           # function call
परिणाम है जोड़ो(ब=4, अ=3)       # keyword arguments (Phase 17) — any order,
जोड़ो(3, ब=4)                   # mix positional + keyword (keywords last);
                                 # works for functions, constructors, lambdas,
                                 # closures — NOT built-ins or method calls

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Decorators (Phase 17)
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
विधि दुगुना_करो(f):             # a decorator is a function taking a function
    फल लाम्डा(x): f(x) * 2     # and returning a closure

@दुगुना_करो                     # decorate — बढ़ाओ है दुगुना_करो(बढ़ाओ)
विधि बढ़ाओ(x):
    फल x + 1

@जोड़_दस                        # stacking — applied bottom-up:
@दुगुना_करो                     # पहचान है जोड़_दस(दुगुना_करो(पहचान))
विधि पहचान(x):
    फल x

@गुणा_से(5)                     # factory form — गुणा_से(5) must return a decorator
विधि आधार(x):
    फल x + 2
                                 # works on top-level + nested विधि (and शुद्ध विधि);
                                 # NOT on class methods. Decorated name resolves as a
                                 # closure variable, so it works in HOFs/assignment.

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Conditionals
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
यदि क से अधिक 10:              # if:  यदि <left> <cmpop> <right>:
    बताओ "बड़ा"
यदि 0 < क < 10:                 # chained comparisons (Phase 17) → (0<क) और (क<10)
    बताओ "बीच में"              # middle expr evaluated per pair — avoid side effects
यदि 20 में_है अंक:              # membership (Phase 17): list element,
    बताओ "मिला"                 # string substring, dict key
यदि "शहर" नहीं_है व्यक्ति:      # negated membership
    बताओ "नहीं"
अन्यथा यदि क बराबर 10:          # else-if
    बताओ "बराबर"
अन्यथा:                         # else (अन्य also works)
    बताओ "छोटा"

परिणाम है यदि क से अधिक 5 तो "बड़ा" अन्यथा "छोटा"   # ternary

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Loops
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
5 बार करो:                      # repeat N times
    बताओ "लूप"

i के लिए 10 में:                # for i in 0..9 (range)
    बताओ i
आइटम के लिए सूची में:          # for item in list
    बताओ आइटम
अ के लिए "नमस्ते" में:          # for char in string
    बताओ अ

जब तक क से कम 10:              # while loop
    क है क + 1
    यदि क बराबर 5:
        अगला                    # continue (skip to next iteration)
    यदि क बराबर 8:
        बंद करो                 # break (exit loop)

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Error handling
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
कोशिश:
    क है पूर्णांक("abc")
पकड़ो त्रुटि:                   # catch-all — binds message Str (or thrown instance)
    बताओ "त्रुटि: " + त्रुटि

वर्ग जाल_त्रुटि(त्रुटि):        # typed exceptions (Phase 17) — inherit त्रुटि
    विधि बनाओ(संदेश):
        यह.संदेश है संदेश

फेंको जाल_त्रुटि("विफल")        # throw — Instance (chain must reach त्रुटि) or Str

कोशिश:
    फेंको जाल_त्रुटि("x")
पकड़ो जाल_त्रुटि ई:             # typed clause — matches class + subclasses, binds ई
    बताओ ई.संदेश
पकड़ो त्रुटि:                   # clauses checked in order; no match → rethrow outward
    बताओ त्रुटि

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Classes
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
वर्ग व्यक्ति:
    विधि बनाओ(नाम, आयु):
        यह.नाम है नाम
        यह.आयु है आयु
    विधि परिचय():
        बताओ यह.नाम + " की आयु " + यह.आयु

वर्ग छात्र(व्यक्ति):            # inheritance
    विधि बनाओ(नाम, आयु, कक्षा):
        यह.नाम है नाम
        यह.आयु है आयु
        यह.कक्षा है कक्षा

प है व्यक्ति("राम", 30)         # constructor call
प.परिचय()                       # method call as statement
बताओ प.नाम                      # field access
बताओ प                          # <व्यक्ति {आयु: 30, नाम: "राम"}>

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। First-class functions & Lambdas
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
दुगुना है लाम्डा(x): x * 2     # lambda expression
बताओ दुगुना(5)                  # 10

f है विधि_नाम                   # named function as value

बताओ मानचित्र(सूची, f)         # map: returns new list
बताओ छानो(सूची, f)             # filter: returns matching elements
बताओ मोड़ो(सूची, 0, f)         # reduce: fold with initial value

# True closures — capture outer scope
विधि गुणक(n):
    फल लाम्डा(x): n * x
दोगुना है गुणक(2)
बताओ दोगुना(7)                  # 14

# Functools (Phase 17) — no import needed
जोड़_दस है आंशिक(जोड़ो, 10)     # partial application
तेज है स्मरण(महंगा)             # memoize (persistent cache)
दफिर है संयोजित(दुगुना, बढ़ाओ)  # compose: दुगुना(बढ़ाओ(x))

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Generators (Phase 17)
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
विधि गिनती(n):                  # any विधि with उत्पन्न is a generator
    i के लिए n में:
        उत्पन्न i               # yield — collected into a returned list
बताओ गिनती(5)                   # [0, 1, 2, 3, 4]
योग के लिए गिनती(5) में:        # generators are iterable
    बताओ योग
# NOTE: eager (collect-to-list) — infinite generators not supported.
# फल inside a generator stops early, returning what's accumulated so far.

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Lists
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
अंक है [10, 20, 30]
बताओ [*अंक, 40, *अन्य]          # spread (Phase 17) — splices list elements in place
बताओ अंक[0]                     # index access → 10
बताओ अंक[1:3]                   # slice (Phase 17) → [20, 30] — Python semantics
बताओ अंक[::-1]                  # reverse; works on strings too: श[2:5], श[::2]
बताओ अंक[-2:]                   # negative indices count from end; out-of-range
                                 # clamps to empty, never errors
अंक[1] है 99                    # index assignment
अंक है अंक.जोड़ो(40)            # append — returns new list, must reassign
अंक है अंक.हटाओ(0)             # remove by index — returns new list, must reassign
बताओ अंक.लम्बाई()               # length
बताओ अंक.उलटा()                # reversed list
बताओ अंक.क्रमबद्ध()             # sorted list (lexicographic)
बताओ अंक.मिलाओ(", ")           # join with separator → String
बताओ क + ख                      # list + list = concatenation

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Dicts
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
व्यक्ति है {"नाम": "राम", "आयु": 25}
बताओ व्यक्ति["नाम"]             # key access
व्यक्ति["शहर"] है "दिल्ली"     # key assignment
बताओ व्यक्ति.लम्बाई()           # key count
बताओ व्यक्ति.कुंजियाँ()         # sorted list of keys
बताओ व्यक्ति.मान()              # list of values (unordered)

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Strings
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
बताओ "नमस्ते {नाम}!"           # string interpolation
बताओ वाक्य[2]                   # nth Unicode character
बताओ वाक्य.लम्बाई()             # character count
बताओ वाक्य.ट्रिम()              # trim whitespace
बताओ वाक्य.शुरू_में("न")        # startswith → Bool
बताओ वाक्य.अंत_में("ते")        # endswith → Bool
बताओ वाक्य.खोजो("स्ते")         # find → char-index or -1
बताओ वाक्य.विभाजित(",")         # split → List
बताओ वाक्य.बदलो("पुरानी", "नई") # replace all → String

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Bitwise operators
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
बताओ 12 & 10                    # AND → 8
बताओ 12 | 10                    # OR  → 14
बताओ 12 ^ 10                    # XOR → 6
बताओ ~5                         # NOT → -6
बताओ 1 << 4                     # left shift → 16
बताओ 32 >> 2                    # right shift → 8

।।।।।।।।।।।।।।।।।।।।।।।।।।।।
। Imports
।।।।।।।।।।।।।।।।।।।।।।।।।।।।
आयात भारत.संख्या               # import stdlib module
आयात "other.swami"              # import another source file
```

**Comparison operators:** `से अधिक` (>), `से कम` (<), `बराबर` (==), `!=`, `<`, `>`, `<=`, `>=` — chainable: `0 < x < 10`
**Membership:** `x में_है सूची` (in), `x नहीं_है सूची` (not in) — List element / Str substring / Dict key

**Karaka annotations** (optional, soft-warning type roles):
`नाम कर्ता है "राम"` — annotates variable with a Sanskrit grammatical role.

**Known constraints / current limitations:**
- `फल` is the return keyword — cannot be used as a variable name
- `है` means assignment — cannot be used as an identifier
- `।...।` comments are single-line only — multi-line block comments not supported
- List/Dict use **copy-on-write** semantics — mutating methods return a new value; you must reassign: `सूची है सूची.जोड़ो(x)`
- **Dict iteration** — `key के लिए कोश में:` iterates sorted keys (Phase 13)
- **`और`/`या`/`नहीं`** boolean keywords available (Phase 13)
- **`वैश्विक`** keyword allows functions to write to global variables (Phase 13)
- **No hex literals** — `0xFF` is a syntax error; use decimal `255`
- Lambdas capture the enclosing frame's locals at definition time; they do not rebind on mutation after capture

---

## Value type (`interpreter::Value`)

Defined in `interpreter.rs`. Used on the LVM stack everywhere.

```rust
pub enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    List(Vec<Value>),                              // Phase 5
    Dict(HashMap<String, Value>),                  // Phase 5 — keys always String
    Instance { class: String, fields: HashMap<String, Value> },  // Phase 6
    Closure { func_name: String, captured: HashMap<String, Value> }, // Phase 10/11
    // (Function, NativeFunction kept for tree-walk interpreter only — not used by LVM)
}
```

**Display rules:**
- Standalone `बताओ "hello"` → prints `hello` (no quotes)
- Inside a List or Dict, strings are quoted: `["राम", "प्रिया"]`, `{"नाम": "राम"}`
- Instance: `<व्यक्ति {आयु: 30, नाम: "राम"}>` (fields sorted alphabetically)
- Closure: `<विधि:func_name>`
- Implemented via `val_repr(v)` helper in `interpreter.rs`

---

## LVM opcode set (`opcode.rs`)

### Core opcodes
`Push(LvmValue)`, `Pop`, `Dup`, `LoadVar(String)`, `StoreVar(String)`,
`Add`, `Sub`, `Mul`, `Div`, `Mod`,
`Eq`, `NotEq`, `Gt`, `Lt`, `GtEq`, `LtEq`,
`And`, `Or`, `Not`,
`Jump(usize)`, `JumpIfFalse(usize)`, `JumpIfTrue(usize)`,
`Call(String, argc)`, `CallNative(String, argc)`, `Return`,
`Print`, `PrintInline`, `Import(String)`, `ImportFile(String)`,
`MethodCall(String, argc)`, `KarakaCheck(String, String)`

### Phase 5 — Lists & Dicts
| Opcode | Tag | Stack effect |
|--------|-----|-------------|
| `MakeList(n)` | 0x26 | pop n values → push `Value::List` |
| `MakeDict(n)` | 0x27 | pop 2n key+value pairs → push `Value::Dict` |
| `GetIndex` | 0x28 | pop (obj, idx) → push `obj[idx]` |
| `SetIndex` | 0x29 | pop (obj, idx, val) → push updated obj |

### Phase 6 — Classes
| Opcode | Tag | Stack effect |
|--------|-----|-------------|
| `MakeInstance(String)` | 0x2A | push empty `Value::Instance` |
| `GetAttr(String)` | 0x2B | pop instance → push field value |
| `SetAttr(String)` | 0x2C | pop (instance, val) → push instance with field set |

### Phase 7 — Interactive
| Opcode | Tag | Note |
|--------|-----|------|
| `PrintInline` | 0x2D | print without newline, flush stdout |

### Phase 9 — Error handling
| Opcode | Tag | Note |
|--------|-----|------|
| `TryStart(handler_ip)` | 0x2E | push try frame with handler address |
| `TryEnd` | 0x2F | pop try frame (normal exit from try block) |

### Phase 10/11 — Closures & Iterables
| Opcode | Tag | Stack effect |
|--------|-----|-------------|
| `MakeClosure(String)` | 0x31 | snapshot frame locals → push `Value::Closure` |
| `GetIterLen` | 0x32 | pop iterable → push length (Number→n, List→len, Str→char_count) |
| `GetIterItem` | 0x33 | pop (iterable, idx) → push element |
| `IterNext(cvar, ivar)` | 0x49 | read container+index vars in place → push element (Phase 17 perf — no container clone) |

### Phase 12 — Bitwise
| Opcode | Tag | Stack effect |
|--------|-----|-------------|
| `BitAnd` | 0x34 | pop (a,b) → push `(a as i64 & b as i64) as f64` |
| `BitOr` | 0x35 | pop (a,b) → push `a \| b` |
| `BitXor` | 0x36 | pop (a,b) → push `a ^ b` |
| `BitNot` | 0x37 | pop a → push `!a as i64` |
| `LShift` | 0x38 | pop (a,b) → push `a << b` |
| `RShift` | 0x39 | pop (a,b) → push `a >> b` |

### First-class Indian opcodes
| Opcode | Stack effect | Calls |
|--------|-------------|-------|
| `AadhaarVerify` | pop str → push bool | `bharat_stdlib::aadhaar_valid` |
| `UpiSend` | pop (from,to,amount,note) → push str | `bharat_stdlib::upi_send` |
| `GstAdd` | pop (amount,rate) → push f64 | `bharat_stdlib::gst_add` |
| `LakhParse` | pop f64 → push "X लाख" str | `bharat_stdlib::format_lakh` |
| `RupeeFormat` | pop f64 → push "₹X,XX,XXX" str | `bharat_stdlib::format_rupees` |

---

## Compiler (`compiler.rs`)

`Compiler::compile_program(stmts) -> CompiledProgram`

Key design points:
- **Loop counters** use temp vars `__lc{N}__` (BarKaro) and `__kl{N}_lim__`/`__kl{N}_idx__`/`__kl{N}_val__` (KeeLiye). Don't name user variables with `__` prefix/suffix.
- **Function bodies** are preceded by `Jump(end)` so top-level execution skips over them. The function's `start_ip` (recorded in `FuncDef`) points to the instruction after the jump.
- **Conditional patching**: `JumpIfFalse(0)` is emitted as a placeholder, then backpatched to the real address after the body is compiled. Same pattern for loops, ternary, and try/catch.
- **Indian opcodes**: compiler tracks `imported_natives` (HashSet) and `indian_fns` (HashMap to `IndianOp` enum) as `आयात` statements are processed. Calls are specialised at compile time.
- **IndexAssign** (`name[idx] है val`) compiles to: `LoadVar` → compile_idx → compile_val → `SetIndex` → `StoreVar`
- **Chained index** (`arr[0][1]`) is handled in parser's `primary()` postfix while-loop wrapping `primary_atom()`
- **Known classes pre-pass**: before compiling, compiler scans top-level stmts for `Varg` nodes to build `known_classes: HashSet` — constructor calls are detected at compile time
- **Varargs**: `FuncDef.vararg: Option<String>` — if set, LVM packs extra args into `Value::List` at call time
- **Break/continue stacks**: `break_sites: Vec<Vec<usize>>` and `continue_stack: Vec<(Option<usize>, Vec<usize>)>` for nested loop patching
- **Parser precedence chain**: `expression() → bitwise_or() → bitwise_xor() → bitwise_and() → comparison() → shift() → additive() → multiplicative() → unary() → primary() → primary_atom()`

---

## LVM (`lvm.rs`)

`LVM::run(&mut self, program: &CompiledProgram) -> Result<(), String>`

- Stack: `Vec<interpreter::Value>`
- Scope: two-tier — `call_frames: Vec<Frame>` (function locals) + `globals: HashMap`
- `LoadVar`: checks `call_frames.last().locals` first, then `globals`, then `self.functions` (returns `Value::Closure`)
- `StoreVar`: if in a frame, updates existing local or global; new variables go in locals
- `Return` with empty call stack = top-level `फल` = stop execution
- Native functions registered by `Import` opcode into `native_fns: HashMap<String, NativeFn>`
- `try_stack: Vec<TryFrame>` — each frame holds `handler_ip` and `stack_depth` for unwinding

**Built-ins pre-registered in `LVM::new()` (no import needed):**

| Function | Description |
|----------|-------------|
| `लम्बाई(v)` | length of list/dict/string |
| `पूर्णांक(v)` | parse string → integer Number |
| `वाक्य(v)` | convert any value to String |
| `पढ़ो(prompt?)` | read line from stdin (optional prompt) |
| `यादृच्छिक(n)` | random int in 0..n-1 (LCG from system time) |
| `निर्गम(code)` | exit with code |
| `निरपेक्ष(n)` | absolute value |
| `घात(base, exp)` | power |
| `वर्गमूल(n)` | square root |
| `गोल(n)` | round |
| `संचिका_सामग्री(path)` | read file → String |
| `संचिका_लिखो(path, s)` | write String to file → Bool |
| `संचिका_है(path)` | file exists → Bool |
| `प्रकार(v)` | type name → String |
| `मानचित्र(list, f)` | HOF map |
| `छानो(list, f)` | HOF filter |
| `मोड़ो(list, init, f)` | HOF reduce |
| `तर्क()` | script CLI args → List of Str (Phase 17) |
| `पथ_जोड़ो(अ, ब, ...)` | join path segments → Str (Phase 17) |
| `फोल्डर_सूची(path)` | sorted directory entries → List (Phase 17) |
| `फोल्डर_बनाओ(path)` | create dir recursively → Bool (Phase 17) |
| `फाइल_हटाओ(path)` | delete file → Bool (Phase 17) |
| `फाइल_कॉपी(src, dst)` | copy file → Bool (Phase 17) |
| `पर्यावरण(name)` | env var → Str or Nil (Phase 17) |
| `वर्तमान_फोल्डर()` | current working dir → Str (Phase 17) |
| `यूआईडी()` | random UUID v4 string (Phase 17) |
| `युग्म(सूची1, सूची2, …)` | zip lists pairwise → List of Lists, truncates to shortest (Phase 17) |
| `गणना(सूची [, शुरू])` | enumerate → List of [index, item] pairs (Phase 17) |
| `श्रृंखला(सूची1, सूची2, …)` | chain → concatenate lists into one (Phase 17) |
| `गिनती_कोश(सूची)` | Counter → Dict of element→count (Phase 17) |
| `कार्तीय(सूची1, सूची2, …)` | Cartesian product → list of lists (Phase 17) |
| `सर्व_संयोजन(सूची, r)` | all r-length combinations → list of lists (Phase 17) |
| `अग्र_जोड़ो(सूची, x)` | deque: prepend x → new list (Phase 17, COW) |
| `अग्र(सूची)` | deque: first element (empty = catchable error) (Phase 17) |
| `पश्च(सूची)` | deque: last element (empty = catchable error) (Phase 17) |
| `अग्र_हटाओ(सूची)` | deque: drop first → new list (Phase 17, COW) |
| `पश्च_हटाओ(सूची)` | deque: drop last → new list (Phase 17, COW) |
| `क्रमित_कोश()` | new empty insertion-ordered dict (list-of-pairs) (Phase 17) |
| `क्रमित_रखो(od, क, म)` | ordered-dict set (update in place or append) → new od (Phase 17) |
| `क्रमित_पाओ(od, क)` | ordered-dict get → value or शून्य (Phase 17) |
| `क्रमित_कुंजियाँ(od)` / `क्रमित_मान(od)` | keys / values in insertion order (Phase 17) |
| `स्मरण(f)` | functools memoize → cached wrapper (persistent cache) (Phase 17) |
| `आंशिक(f, ...)` | functools partial application (Phase 17) |
| `संयोजित(f, g)` | functools compose → `लाम्डा(x): f(g(x))` (Phase 17) |
| `सामान्यीकृत(s)` | normalize Devanagari to NFC (decompose nukta letters) (Phase 17) |
| `पूर्ण_है(x)` | true if x is a whole number (Phase 17) |

**List methods (`MethodCall` in lvm.rs):**

| Method | Behaviour |
|--------|-----------|
| `लम्बाई()` | returns length as Number |
| `जोड़ो(val)` | returns new list with val appended |
| `हटाओ(idx)` | returns new list with element at idx removed |
| `उलटा()` | returns reversed list |
| `क्रमबद्ध()` | returns lexicographically sorted list |
| `मिलाओ(sep)` | joins elements with separator → String |

**Dict methods:**

| Method | Behaviour |
|--------|-----------|
| `लम्बाई()` | returns key count as Number |
| `कुंजियाँ()` | returns sorted list of keys |
| `मान()` | returns list of values (unordered) |

**String methods:**

| Method | Behaviour |
|--------|-----------|
| `लम्बाई()` | character count |
| `ट्रिम()` | trim whitespace |
| `शुरू_में(prefix)` | startswith → Bool |
| `अंत_में(suffix)` | endswith → Bool |
| `खोजो(query)` | find substring → char-index or -1 |
| `विभाजित(sep)` | split by separator → List |
| `बदलो(old, new)` | replace all occurrences → String |

**Dict key storage:** all keys are converted to `String` via `format!("{k}")`.

**String index:** `str[n]` returns the nth Unicode character as a one-char string.

**Closure calls:** `call_closure_value(func_name, captured, args)` — injects captured vars as base frame locals, params overlay on top. Runs with sentinel `return_addr = usize::MAX`.

**Output accumulator (WASM):**
- `LVM::new()` → `capture = false` → Print writes to stdout
- `LVM::new_capturing()` → `capture = true` → Print appends to `vm.output: String`

---

## भारत stdlib (`bharat_stdlib.rs`)

Four registries, each returns `Vec<(&'static str, NativeFn)>`:

| Module | Registry fn | Functions |
|--------|-------------|-----------|
| `भारत.पहचान` | `pehchaan_registry()` | `आधार_जाँचो`, `pan_जाँचो`, `ifsc_जाँचो` |
| `भारत.संख्या` | `sankhya_registry()` | `लाख_में`, `करोड़_में`, `रुपये_में`, `gst_जोड़ो`, `emi_निकालो` |
| `भारत.भुगतान` | `bhugtaan_registry()` | `upi_वैध_है`, `upi_भेजो` |
| `भारत.भाषा` | `bhasha_registry()` | `devanagari_है`, `roman_में`, `शब्द_गिनो` |
| `भारत.json` | `json_registry()` | `json_पढ़ो` (JSON text → Value), `json_लिखो` (Value → JSON text, keys sorted) — hand-written parser, Phase 17 |
| `भारत.समय` | `samay_registry()` | `समय_अभी`, `समय_बनाओ(व,मा,दि,[घं,मि,से])`, `समय_विवरण` (→Dict incl. वार_नाम/माह_नाम), `समय_स्वरूप`, `दिनांक_पार्स`, `समय_जोड़ो`, `दिन_अंतर`, `अधिवर्ष`, `माह_दिन` — epoch seconds (UTC) as canonical value, human-facing fields in IST (+5:30), Hinnant civil-date algorithms, Phase 17 |
| `भारत.csv` | `csv_registry()` | `csv_पढ़ो` (→List of List of Str, RFC 4180), `csv_शीर्षक_पढ़ो` (header row → List of Dict), `csv_लिखो` (List of Lists → text, auto-quoting) — Phase 17 |
| `भारत.कूट` | `koot_registry()` | `sha256`, `md5` (hex digests of UTF-8 bytes), `base64_कूट`, `base64_खोलो` — pure-Rust reference implementations, Phase 17 |
| `भारत.http` | `http_registry()` | `http_पाओ(url[, headers])` GET, `http_भेजो(url, body[, headers])` POST — returns Dict {स्थिति, शीर्षक (lowercased keys), सामग्री}; HTTP/1.1 over std::net, chunked decoding, 10s timeouts; **http:// only, no TLS**; WASM = catchable error — Phase 17 |
| `भारत.सांख्यिकी` | `sankhyiki_registry()` | Statistics (Phase 17): `माध्य` (mean), `माध्यिका` (median), `बहुलक` (mode, ties→smallest), `प्रसरण` (population variance), `मानक_विचलन` (std-dev), `योग` (sum), `न्यूनतम` (min), `अधिकतम` (max), `परिसर` (range) — all take one List of numbers; empty/non-number = catchable error |
| `भारत.प्रतिमान` | `regex_engine::pratimaan_registry()` | Regex (Phase 17, `src/regex_engine.rs`): `ढूंढो(p,t)` first match or शून्य, `ढूंढो_स्थान` char-index or -1, `ढूंढो_सब` → List, `मेल_है` full-match Bool, `समूह` → [पूर्ण, समूह1…] or [], `बदलो_सब(p,बदल,t)` with `$0`–`$9`/`$$`, `विभाजित_सब` — backtracking VM, pure Rust, WASM-safe; supports `. ^ $ \| () (?:) [] [^] \d\D\w\W\s\S * + ? {n,m}` + lazy variants; `\w` includes the full Devanagari block (matras/halant); 2M-step budget → catchable "बहुत जटिल" error; invalid pattern = catchable error |
| `भारत.बड़ी` | `bignum::badi_registry()` | Big integers (Phase 17, `src/bignum.rs`): arbitrary-precision base-1e9, decimal-string I/O — `महा_जोड़`/`महा_घटा`/`महा_गुणा`/`महा_भाग`(trunc)/`महा_शेष`/`महा_घात`(pow)/`महा_तुलना`(→-1/0/1)/`महा_भाज्य`(factorial); pure Rust, no num-bigint |
| `भारत.संजाल` | `net::sanjaal_registry()` | TCP sockets (Phase 17, `src/net.rs`): `सॉकेट_जोड़ो(host,port)` connect, `सॉकेट_सुनो` listen, `सॉकेट_स्वीकारो` accept, `सॉकेट_भेजो`/`सॉकेट_पढ़ो`/`सॉकेट_बंद`; thread-local handle registry (opaque Number ids); pure std::net; WASM = catchable error |
| `भारत.संपीडन` | `zip::sampidan_registry()` | ZIP (Phase 17, `src/zip.rs`): `ज़िप_लिखो(path, कोश)` write (STORE), `ज़िप_पढ़ो(path)` → {name:text} (reads STORE + DEFLATE via full pure-Rust inflate), `ज़िप_सूची(path)` entry names; CRC32; verified bidirectional vs Windows |
| `भारत.संग्रह` | `sql::sangraha_registry()` | Local SQL DB (Phase 17, `src/sql.rs`): `db_नया()` → handle, `db_चलाओ(h, sql)` (SELECT→List of Dict, else affected-count), `db_सहेजो`/`db_खोलो`/`db_बंद`; SQL subset CREATE/INSERT/SELECT(cols,WHERE AND/OR,ORDER BY,LIMIT)/UPDATE/DELETE/DROP; pure Rust, no rusqlite |
| `भारत.मात्रक` | `matrak::matrak_registry()` | **Mission-critical** units / dimensional analysis (Phase 18, `src/matrak.rs`): `मात्रा(मान,इकाई)`, `जोड़_मात्रा`/`घटा_मात्रा`/`गुणा_मात्रा`/`भाग_मात्रा` (dimension-checked → catchable विमा बेमेल), `मान_में`, `मात्रा_वाक्य`, `विमा_बराबर`. Affine temps सेल्सियस/°C, फ़ारेनहाइट/°F (offset-converted, stored as Kelvin); derived dims display as न्यूटन/जूल/वाट/पास्कल. Catches the Mars-Orbiter lbf-vs-N bug class |
| `भारत.रेखीय` | `rekhiy::rekhiy_registry()` | Linear algebra (Phase 18, `src/rekhiy.rs`): vectors (बिंदु_गुणन/कोण_गुणन/परिमाण/सामान्य/सदिश_योग/दूरी), matrices (आव्यूह_गुणन/परिवर्त/सारणिक/प्रतिलोम/तत्समक), quaternions `[w,x,y,z]` (चतुष्क_गुणन/सामान्य/कोण_से_चतुष्क/चतुष्क_से_कोण/घुमाव/प्रक्षेप slerp) |
| `भारत.नियंत्रण` | `niyantran::niyantran_registry()` | Control (Phase 18, `src/niyantran.rs`): पीआईडी_बनाओ/पीआईडी_चरण (PID + anti-windup), कलमैन_बनाओ/कलमैन_चरण (1-D Kalman), कलमैन_एनडी_बनाओ/कलमैन_एनडी_चरण (N-D Kalman — state Dict {x,P,F,H,Q,R}, pure-Rust matrix predict/update); step fns return `[output, new_state]` |
| `भारत.सर्वर` | `server::sarvar_registry()` | HTTP server (Phase 18, `src/server.rs`): `सर्वर_बनाओ(port)` → handle, `सर्वर_मार्ग(h, path, body)` register route, `सर्वर_चलाओ(h[, limit])` blocking serve loop (limit=0 → forever, else stop after N requests) → count served; pure std::net HTTP/1.1, thread-local handle registry; WASM = catchable error |
| `भारत.सूत्र` | `threads::sutra_registry()` | OS threads (Phase 18, `src/threads.rs`): `समानांतर(सूची)` runs each LIPI source string in its own std::thread, collects captured output in input order → List of Str; `सूत्र_गणना()` → available CPU cores; WASM = catchable error |
| `भारत.दिशा` | `disha::disha_registry()` | Navigation/geodesy (Phase 18, `src/disha.rs`): महावृत्त_दूरी (haversine km), दिशा_कोण (bearing°), गंतव्य (destination), ईसीईएफ (WGS-84 → ECEF); lat/lon in degrees |
| `भारत.सुरक्षा` | `suraksha::suraksha_registry()` | Fault tolerance (Phase 18, `src/suraksha.rs`): हैमिंग_कूट/हैमिंग_विकोड (Hamming(7,4) corrects single bit-flips), सीआरसी32, बहुमत (TMR voter), समय_सीमा_जाँच (deadline) |
| `भारत.अंतराल` | `antaral::antaral_registry()` | Interval arithmetic (Phase 18, `src/antaral.rs`): अंतराल/अंतराल_योग/घटा/गुणा/चौड़ाई/मध्य/में — rigorous `[lo,hi]` bounds. Pair with `बीज_सेट(n)` builtin for reproducible runs |

Key implementations:
- **Aadhaar**: full Verhoeff algorithm with `D[10][10]`, `P[8][10]`, `INV[10]` tables. First digit must not be 0 or 1 (UIDAI rule). Test valid: `"234567890124"`.
- **EMI**: `P*r*(1+r)^n / ((1+r)^n - 1)` where `r = rate_annual / 1200`
- **UPI**: local part 1–256 chars alphanumeric+`.-_`, handle in `UPI_HANDLES` const slice
- **Indian commas**: last 3 digits grouped, then groups of 2 (e.g. `₹12,34,567`)

---

## .libc binary format (`serializer.rs`)

```
b"LIPI"          4 bytes magic
u8               version = 4 (loader also accepts 2, 3)
u32 LE           function count
  for each fn:
    u32 + bytes  function name (UTF-8)
    u8           param count
    u32+bytes×N  param names
    u8×N         default flags (v3) — 1 followed by PUSH_*-tagged value
    u8           vararg flag (0 = none, 1 = has vararg)
    [u32+bytes]  vararg name (only if flag = 1)
    u64 LE       start_ip
u32 LE           class parent count (v3) — child/parent str pairs
u32 LE           instruction count
  for each op:
    u8 tag + payload (see TAG_* consts in serializer.rs)
u32 LE × count   source line per instruction (v4) — 0 = unknown
```

v3 (Phase 17) added per-param default values and the class-inheritance table —
v2 files never stored `class_parents`, so inheritance was broken in old `.libc` files.
v4 (Phase 17) added the per-instruction line table for runtime diagnostics —
pre-v4 files load with every line = 0 (errors print without पंक्ति info).

Strings encoded as `u32 len + UTF-8 bytes`. Jump addresses as `u32`. Arg counts as `u8`.

Full tag list: 0x00–0x45 (see `TAG_*` constants in `serializer.rs`). Phase 12 added 0x34–0x39. Phase 13 added 0x3A (`DeclareGlobal`). Phase 15 added 0x3B–0x3E (`DefineEnum`, `MatchVariant`, `EnumUnpack`, `TailCall`). Phase 16 added 0x3F–0x40 (`Assert`, `DeclareConst`). Phase 17 added 0x41–0x49 (`CallKw`, `FloorDiv`, `UnpackList`, `Slice`, `Contains`, `MakeListSp`, `Throw`, `MatchErrClass`, `IterNext`).

---

## Phase 4 — WASM Browser Playground (complete)

The LVM compiles to `wasm32-unknown-unknown` and runs entirely in the browser.

### Extra files

| Path | Role |
|------|------|
| `src/lib.rs` | Library root — exposes `run_source` via `#[wasm_bindgen]` |
| `web/index.html` | Browser playground UI |
| `web/pkg/` | Generated by wasm-pack (`lipi.js`, `lipi_bg.wasm`, `.d.ts`) |
| `play.bat` | One-click launcher — builds WASM if needed, starts HTTP server, opens browser |

### Build WASM

```powershell
$env:RUSTUP_HOME = "D:\Rust\rustup"
$env:CARGO_HOME  = "D:\Rust\cargo"
$env:PATH = "D:\Rust\cargo\bin;D:\msys64\mingw64\bin;$env:PATH"
Set-Location "D:\Projects\lipi-lang"
wasm-pack build --target web --out-dir web/pkg --features wasm
```

Requires: default Rust toolchain = `stable-x86_64-pc-windows-gnu`

`play.bat rebuild` forces a full WASM rebuild before serving.

---

## Phase 5 — सूची + कोश (complete)

Lists and dicts are fully implemented across all layers.

- **Lexer**: `[`, `]`, `{`, `}` tokenized as `LBracket`, `RBracket`, `LBrace`, `RBrace`
- **AST**: `Stmt::IndexAssign`, `Expr::List`, `Expr::Dict`, `Expr::Index`
- **Parser**: `primary_atom()` parses list/dict literals; `primary()` wraps with postfix `[idx]` loop
- **Compiler**: emits `MakeList`, `MakeDict`, `GetIndex`, `SetIndex`
- **LVM**: `MakeList/MakeDict/GetIndex/SetIndex` opcodes; MethodCall extended for list/dict methods
- **Serializer**: TAG_MAKE_LIST/DICT/GET_INDEX/SET_INDEX (0x26–0x29)

Test file: `examples/phase5_test.swami`

---

## Phase 6 — वर्ग (Classes / Objects) — COMPLETE

```lipi
वर्ग व्यक्ति:
    विधि बनाओ(नाम, आयु):
        यह.नाम है नाम
        यह.आयु है आयु
    विधि परिचय():
        बताओ यह.नाम + " की आयु " + यह.आयु

वर्ग छात्र(व्यक्ति):            # inheritance
    विधि बनाओ(नाम, आयु, कक्षा):
        यह.नाम है नाम
        यह.आयु है आयु
        यह.कक्षा है कक्षा
```

**Implementation notes:**
- `वर्ग` is a reserved keyword; `यह` is a regular identifier (self)
- Methods registered as `"ClassName::method"` with `"यह"` prepended as first param
- Constructor call `ClassName(args)` detected by compiler pre-pass (`known_classes`)
- `बनाओ` implicitly returns `यह`; classes without `बनाओ` return empty instance
- `यह.field है val` → `Stmt::AttrAssign` → `LoadVar("यह"), val, SetAttr, StoreVar("यह")`
- Inherited method calls walk `class_parents: HashMap` at runtime

**New opcodes:** `MakeInstance(String)` (0x2A), `GetAttr(String)` (0x2B), `SetAttr(String)` (0x2C)

Test file: `examples/phase6_test.swami`

---

## Phase 7 — Interactive Architecture — COMPLETE

| Feature | Keyword | Example |
|---------|---------|---------|
| While loop | `जब तक` | `जब तक सत्य:` |
| Break | `बंद करो` | exits innermost loop |
| Continue | `अगला` | skips to next iteration |
| Inline print | `लिखो` | `लिखो "नाम: "` (no newline) |
| Else-if chain | `अन्यथा यदि` | `अन्यथा यदि क बराबर 2:` |

`बंद करो` and `अगला` work inside all three loop types (`जब तक`, `बार करो`, `के लिए`).

Test file: `examples/interactive_demo.swami`

---

## Phase 8 — Editor + Extended Builtins — COMPLETE

`lipi edit foo.swami` — vim-inspired terminal line editor (v2).

### Editor display
```
  LIPI  hello.swami [*]   Ln 3  Col 1  ·  Pg 1/2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1  │ बताओ "नमस्ते"
► 2  │ बताओ "भारत"        ← cursor line (highlighted)
  3  │ बताओ "LIPI"
       ~                   ← vim-style tilde for empty rows
       ~
────────────────────────────────────────────────
  3 lines  undo:1  clip:0  ·  ? help
────────────────────────────────────────────────
  Ctrl+O Open  Ctrl+S Save  Ctrl+R Run  Ctrl+F Find  Ctrl+H Replace  Ctrl+Q Quit
────────────────────────────────────────────────
:>
```

### Editor commands (type at `:>` prompt)

| Command | Action |
|---------|--------|
| `s` | Save (Ctrl+S) |
| `r` | Save + Run (Ctrl+R) |
| `q` | Quit (Ctrl+Q) — prompts if unsaved |
| `n` / `p` | Next / Previous page |
| `<n>` | Jump to line n (type number only) |
| `g<n>` | Go to line n |
| `a <text>` | Append line at end |
| `i<n> <text>` | Insert after line n (i0 = before line 1) |
| `e<n> <text>` | Edit (replace) line n |
| `d<n>` | Delete line n |
| `D<n>-<m>` | Delete lines n through m |
| `dup<n>` | Duplicate line n |
| `u` | Undo last change (up to 50 levels) |
| `/<text>` | Find first occurrence (Ctrl+F) |
| `N` | Find next occurrence |
| `h/<old>/<new>` | Replace first match on cursor line (Ctrl+H) |
| `H/<old>/<new>` | Replace ALL occurrences in file |
| `c<n>` | Copy line n to clipboard |
| `x<n>` | Cut line n (copy + delete) |
| `v` / `v<n>` | Paste after cursor / after line n |
| `o <file>` | Open another file (Ctrl+O) |
| `?` | Show help screen |

Math builtins: `निरपेक्ष`, `घात`, `वर्गमूल`, `गोल`
File I/O builtins: `संचिका_सामग्री`, `संचिका_लिखो`, `संचिका_है`
Type inspection: `प्रकार(val)`
New string methods: `.ट्रिम()`, `.शुरू_में()`, `.अंत_में()`, `.खोजो()`, `.विभाजित()`, `.बदलो()`
New list method: `.मिलाओ(sep)`

---

## Phase 9 — त्रुटि + वंशानुक्रम (Error Handling + Inheritance) — COMPLETE

| Feature | Keyword | Example |
|---------|---------|---------|
| Try/catch | `कोशिश` / `पकड़ो` | `कोशिश:` … `पकड़ो त्रुटि:` |
| Inheritance | `वर्ग Child(Parent):` | `वर्ग कुत्ता(प्राणी):` |
| String interpolation | `{var}` in strings | `"नमस्ते {नाम}!"` |
| Multi-file import | `आयात "file.swami"` | runs + merges functions |

- `vm.try_stack: Vec<TryFrame>` catches any `Err` from `exec_op`
- Inherited method calls walk the parent chain at runtime
- String interpolation: `parse_fmt_string()` builds Add chain of literal + var parts

Test file: `examples/phase9_test.swami`

---

## Phase 10 — लाम्डा (First-class Functions + HOFs) — COMPLETE

```lipi
दुगुना है लाम्डा(x): x * 2
बताओ मानचित्र([1,2,3], दुगुना)   # [2, 4, 6]
बताओ छानो([1,2,3,4], लाम्डा(x): x % 2 बराबर 0)   # [2, 4]
बताओ मोड़ो([1,2,3,4,5], 0, लाम्डा(अ, ब): अ + ब)   # 15
```

- Lambda compiles to anonymous `__lam_N__` function + `MakeClosure` opcode
- `Value::Closure { func_name, captured }` — first-class function reference
- `LoadVar` falls back to `self.functions` to enable `f है विधि_नाम` syntax
- HOFs (`मानचित्र`, `छानो`, `मोड़ो`) handled as special cases inside `Call` opcode handler

Test file: `examples/phase10_test.swami`

---

## Phase 11 — पुनरावृत्ति + सच्चे बंद (Iterable For-Loop + True Closures) — COMPLETE

```lipi
# List iteration
नाम है ["राम", "श्याम", "गीता"]
अ के लिए नाम में:
    बताओ "नमस्ते " + अ + "!"

# String iteration
अ के लिए "नमस्ते" में:
    लिखो अ + " "

# True closures — factory function
विधि गुणक(n):
    फल लाम्डा(x): n * x
दोगुना है गुणक(2)
बताओ दोगुना(7)    # 14
```

- `GetIterLen` (0x32) / `GetIterItem` (0x33) — polymorphic runtime dispatch over Number/List/Str
- `MakeClosure` snapshots `call_frames.last().locals` into `captured` at definition time
- `call_closure_value` injects captured vars as base frame locals; params overlay on top

Test file: `examples/phase11_test.swami`

---

## Phase 12 — भाषा पूर्णता (Varargs + Ternary + Bitwise Ops) — COMPLETE

```lipi
# Varargs
विधि जोड़_सब(*संख्याएँ):
    कुल है 0
    i के लिए संख्याएँ में:
        कुल है कुल + i
    फल कुल
बताओ जोड़_सब(1, 2, 3)        # 6

# Ternary
बताओ यदि अ से अधिक 5 तो "बड़ा" अन्यथा "छोटा"

# Bitwise
बताओ 12 & 10    # 8
बताओ 1 << 4     # 16
बताओ ~5         # -6
```

- Extra args packed into `Value::List` in `Call` handler when `func.vararg` is Some
- Ternary: condition + `JumpIfFalse` + then + `Jump` + else + backpatch
- Bitwise uses `vm_bitwise2` helper: converts f64 ↔ i64 for integer operations
- Parser precedence chain extended: `expression → bitwise_or → bitwise_xor → bitwise_and → comparison → shift → additive`

Test file: `examples/phase12_test.swami` — 14 assertions, all verified.

---

## Phase 13 — भाषा विस्तार (Language Extensions) — COMPLETE

### Dict iteration
```lipi
कोश के लिए dict में:          # iterates sorted keys
    बताओ dict[कोश]
```
Fixed `GetIterLen` + `GetIterItem` in `lvm.rs` to handle `Value::Dict` (sorted keys).

### Boolean keywords — `और` / `या` / `नहीं`
```lipi
यदि अ से अधिक 5 और ब से कम 10:    # AND
    बताओ "दोनों सही"

यदि अ से कम 0 या ब से अधिक 100:   # OR
    बताओ "एक सही"

यदि नहीं असत्य:                    # NOT
    बताओ "सत्य"
```
- Lexer: `और` → `Aur`, `या` → `Ya`, `नहीं` → `Nahin`
- Parser chain: `expression → logical_or(या) → logical_and(और) → bitwise_or → ...`
- Compiles to existing `Opcode::And`, `Opcode::Or`, `Opcode::Not`
- `condition_expr()` calls replaced with `expression()` — compound conditions work everywhere

### Global variable keyword — `वैश्विक`
```lipi
गिनती है 0

विधि बढ़ाओ():
    वैश्विक गिनती         # declare before use
    गिनती है गिनती + 1   # writes to global

बढ़ाओ()
बताओ गिनती              # 1

विधि दोनों():
    वैश्विक x, y         # multiple names in one statement
    x है x + 1
    y है y + 1
```
- New opcode `DeclareGlobal(String)` (tag 0x3A) — marks name in `Frame.global_names`
- `StoreVar` checks `global_names` first; if declared global, always writes to `self.globals`

Test file: `examples/phase13_test.swami` — all assertions pass.

---

## Phase 14 — देवनागरी अंक + स्वरूप + भारत.गणित — COMPLETE

### Devanagari digit literals
Already fully implemented in the lexer via `is_dev_digit()` and `dev_to_ascii()`. Works in all contexts:
```lipi
क है ५          # 5
ख है १२         # 12
बताओ ५ लाख     # 500000
i के लिए ५ में:  # loop 0..4
```

### स्वरूप — string formatting
```lipi
स्वरूप("{} का वर्गमूल {} है", 25, 5)  # "25 का वर्गमूल 5 है"
स्वरूप("पाई = {:.4}", पाई)            # "पाई = 3.1416"
स्वरूप("कोड: {:06}", 42)             # "कोड: 000042"
स्वरूप("{} + {} = {}", 1, 2, 3)      # "1 + 2 = 3"
स्वरूप("GST: {:%}", 0.18)            # "GST: 18.00%"
```

Format specifiers inside `{}`:
- `{}` — value as-is
- `{:.N}` — N decimal places
- `{:0N}` — zero-padded to width N
- `{:%}` — percentage (multiply by 100, append %)

**Important:** `{}` and `{:spec}` in strings are kept as literal placeholders (not treated as variable interpolation). Named interpolation `{varname}` still works normally.

### Built-in constants
Pre-loaded in `LVM::new()` — no import needed:
```lipi
बताओ पाई          # 3.141592653589793
बताओ अनंत         # inf
बताओ ऋण_अनंत      # -inf
```

### भारत.गणित module — Ancient Hindu Mathematics
```lipi
आयात भारत.गणित
```

| Function | Hindi name | Source | Description |
|----------|-----------|--------|-------------|
| `ज्या(x)` | jyā | Āryabhaṭa 499 CE | sin(x) — origin of word "sine" |
| `कोज्या(x)` | kojyā | Āryabhaṭa 499 CE | cos(x) |
| `स्पर्शज्या(x)` | sparśajyā | Āryabhaṭa 499 CE | tan(x) |
| `व्युत्क्रम_ज्या(x)` | | | asin(x) |
| `व्युत्क्रम_कोज्या(x)` | | | acos(x) |
| `व्युत्क्रम_स्पर्श(x)` | | | atan(x) |
| `विरहांक(n)` | virahāṅka | Chandahśāstra ~600 CE | nth Fibonacci (600 yrs before Fibonacci) |
| `संयोजन(n,r)` | saṃyojana | Bhāskara II Līlāvatī 1150 CE | C(n,r) combinations |
| `क्रमचय(n,r)` | kramacaya | Bhāskara II 1150 CE | P(n,r) permutations |
| `ब्रह्मगुप्त_क्षेत्र(a,b,c,d)` | | Brahmagupta 628 CE | Cyclic quadrilateral area |
| `हेरॉन_क्षेत्र(a,b,c)` | | Heron's formula | Triangle area from sides |
| `लघुगणक(x)` | laghugaṇaka | | ln(x) natural log |
| `लघुगणक_दस(x)` | | | log₁₀(x) |
| `घातांक(x)` | ghātāṅka | | e^x |

```lipi
बताओ स्वरूप("ज्या(30°) = {:.4}", ज्या(पाई / 6))         # 0.5000
बताओ स्वरूप("C(10,3) = {}", संयोजन(10, 3))              # 120
बताओ स्वरूप("विरहांक(10) = {}", विरहांक(10))             # 55
बताओ स्वरूप("चतुर्भुज = {:.4}", ब्रह्मगुप्त_क्षेत्र(3,4,5,6))  # 18.9737
```

### Parser fix: `{}` in strings
`parse_fmt_string()` now skips interpolation for `{}` and `{:spec}` patterns (empty interior or `:` prefix). Only `{varname}` triggers variable substitution.

Test file: `examples/phase14_test.swami`

---

## Roman QWERTY Input — COMPLETE (`src/roman.rs`)

Write LIPI source code using phonetic Roman spellings — no Devanagari keyboard needed.

```
lipi foo.roman            # auto-detected by extension
lipi roman foo.txt        # explicit subcommand
lipi roman-show foo.roman # print translated Devanagari source (debug)
```

Example `.roman` file:
```
batao "namaste duniya!"
ka hai 10
yadi ka se adhik 5:
    batao "bada hai"
aayat bharat.ganit
batao swaroop("jya(30) = {:.4}", jya(pai / 6))
```

Translates to:
```lipi
बताओ "namaste duniya!"
ka है 10
यदि ka से अधिक 5:
    बताओ "bada hai"
आयात भारत.गणित
बताओ स्वरूप("jya(30) = {:.4}", ज्या(पाई / 6))
```

**Rules:**
- Only replaces whole words (word-boundary matched)
- String literals (`"..."`) are preserved verbatim
- Comments (`#`, `।`) are preserved verbatim
- Longer phrases take priority (`anyatha yadi` before `anyatha`)
- Case-insensitive for keyword matching

**Key mappings (subset):**

| Roman | Devanagari | Roman | Devanagari |
|-------|-----------|-------|-----------|
| `batao` | `बताओ` | `likho` | `लिखो` |
| `hai` | `है` | `yadi` | `यदि` |
| `anyatha` | `अन्यथा` | `toh` | `तो` |
| `vidhi` | `विधि` | `phal` | `फल` |
| `aur` | `और` | `ya` | `या` |
| `nahin` | `नहीं` | `satya` | `सत्य` |
| `jab tak` | `जब तक` | `ke liye` | `के लिए` |
| `band karo` | `बंद करो` | `bar karo` | `बार करो` |
| `se adhik` | `से अधिक` | `se kam` | `से कम` |
| `swaroop` | `स्वरूप` | `pai` | `पाई` |
| `jya` | `ज्या` | `virahank` | `विरहांक` |
| `bharat.ganit` | `भारत.गणित` | `sanyojan` | `संयोजन` |

Variable names and string contents are **not** translated — use any identifiers you like.

Demo file: `examples/demo_roman.roman`

---

## Phonetic Input (.vani) — COMPLETE (`src/phonetic.rs`)

Full character-level transliteration — no Devanagari keyboard needed. Identifiers AND keywords
are both converted. Two passes: (1) roman.rs keyword map, (2) phonetic conversion of remaining ASCII words.

```
lipi phonetic foo.vani       # run
lipi phonetic-show foo.vani  # debug: print Devanagari translation
lipi foo.vani                # auto-detect by extension
```

### Vowel scheme

| Roman | Standalone | Matra (after consonant) |
|-------|-----------|------------------------|
| `a`   | अ | inherent (nothing added) |
| `aa`  | आ | ा |
| `i`   | इ | ि |
| `ee` / `ii` | ई | ी |
| `u`   | उ | ु |
| `oo` / `uu` | ऊ | ू |
| `e`   | ए | े |
| `ai`  | ऐ | ै |
| `o`   | ओ | ो |
| `au` / `ou` / `ow` | औ | ौ |
| `ri`  | ऋ | ृ |

### Consonant scheme

Single consonants (lowercase): `k`→क `kh`→ख `g`→ग `gh`→घ `ng`→ङ `ch`→च `chh`→छ `j`→ज `jh`→झ `ny`→ञ `t`→त `th`→थ `d`→द `dh`→ध `n`→न `p`→प `ph`/`f`→फ `b`→ब `bh`→भ `m`→म `y`→य `r`→र `l`→ल `v`/`w`→व `sh`→श `ssh`→ष `s`→स `h`→ह

Retroflex (double-letter): `tt`→ट `tth`→ठ `dd`→ड `ddh`→ढ `nn`→ण

Conjuncts: `ksh`/`x`→क्ष `gy`/`gny`→ज्ञ

**Key rule:** consecutive consonants get halant (्) between them. To insert inherent 'a', write `a` explicitly:
- `vargphal` → `वर्ग्फल` (halant between g and ph)
- `vargaphal` → `वर्गफल` (explicit 'a' breaks the cluster) ✓

Demo file: `examples/demo_phonetic.vani`

---

## Phase 15 — भाषा पूर्णता II (Language Completeness II) — COMPLETE

### Multi-line strings — triple-quote `"""`
```lipi
संदेश है """नमस्ते!
यह एक
बहु-पंक्ति वाक्य है।"""
बताओ संदेश
```
- `preprocess_triple_quotes()` pre-pass in `lexer.rs` converts `"""..."""` to a single-line escaped string before tokenization
- Newlines become `\n`, internal `"` become `\"`

### Indian number formatting in स्वरूप
```lipi
बताओ स्वरूप("{:,}", 1234567)   # 12,34,567
बताओ स्वरूप("{:₹}", 1234567)   # ₹12,34,567
```
- `{:,}` — Indian comma grouping: last 3 digits, then groups of 2
- `{:₹}` — same with ₹ prefix
- Implemented via `format_indian_number()` in `lvm.rs`

### Enums — विकल्प + pattern matching — मिलाओ
```lipi
विकल्प रंग:
    लाल
    हरा
    नीला

विकल्प आकार:
    वृत्त(त्रिज्या)
    आयत(चौड़ाई, ऊंचाई)

मेरा_रंग है रंग.लाल
आ है आकार.वृत्त(5)

मिलाओ मेरा_रंग:
    लाल:
        बताओ "लाल"
    अन्यथा:
        बताओ "अन्य रंग"

मिलाओ आ:
    वृत्त(r):
        बताओ "वृत्त, त्रिज्या = " + r
    आयत(w, h):
        बताओ w + " x " + h
```

**Implementation:**
- New `Value::EnumDef { name, variants: HashMap<String, usize> }` — the type definition stored in globals
- New `Value::Enum { enum_name, variant, values: Vec<Value> }` — enum instance
- `EnumName.Variant` (zero-arity) → `GetAttr` on `Value::EnumDef` → returns `Value::Enum`
- `EnumName.Variant(args)` → `MethodCall` on `Value::EnumDef` → returns `Value::Enum`
- `मिलाओ` compiles to: `Dup` + `MatchVariant` + `JumpIfFalse` + `EnumUnpack`/`Pop` + body + `Jump(end)` chain
- New opcodes: `DefineEnum(String, Vec<(String, usize)>)`, `MatchVariant(String)`, `EnumUnpack(Vec<String>)`
- New serializer tags: `TAG_DEFINE_ENUM = 0x3B`, `TAG_MATCH_VARIANT = 0x3C`, `TAG_ENUM_UNPACK = 0x3D`

### Tail-call optimization (TCO)
```lipi
विधि संचय(न, कुल):
    यदि न बराबर 0:
        फल कुल
    फल संचय(न - 1, कुल + न)     # tail call — no stack growth

बताओ संचय(1000, 0)              # 500500 — works without overflow
```
- Compiler tracks `in_function: usize` depth counter
- `Stmt::Fal(Expr::Call { name, .. })` inside a function body, where `name` is in `self.functions` → emits `TailCall` instead of `Call + Return`
- `TailCall` in LVM reuses the current frame: clears locals, injects new args, resets `ip` to `func.start_ip`
- `Frame.base_stack_depth` tracks where the stack was on entry, allowing `TailCall` to truncate temporaries
- Only applies to user-defined function calls — native functions (`वर्गमूल` etc.) still use normal `Call + Return`
- New opcode: `TailCall(String, usize)` — tag `TAG_TAIL_CALL = 0x3E`

Test file: `examples/phase15_test.swami` — all assertions pass, 63/63 regression unchanged.

---

## Phase 16 — प्राचीन भारतीय अवधारणाएँ II (Ancient Concepts II) — COMPLETE

### New keywords

| Keyword | Meaning | Source | Example |
|---------|---------|--------|---------|
| `जाँचो` | assert | Nyaya Pratijna (verification) | `जाँचो क से अधिक 0, "धनात्मक होना चाहिए"` |
| `स्थिर` | immutable const | Samkhya Purusha (the unchanging) | `स्थिर गुरुत्व है 9.81` |
| `शुद्ध विधि` | pure function | Gita karma yoga (no side effects) | `शुद्ध विधि वर्गफल(अ): फल अ*अ` |

### New modules

| Module | Source | Functions |
|--------|--------|-----------|
| `भारत.व्याकरण` | Panini Ashtadhyayi ~350 BCE | `संधि_प्रकार`, `समास_प्रकार`, `स्फोट_परीक्षण`, `शिव_सूत्र` |
| `भारत.विज्ञान` | Vijnanabhairava Tantra ~7th CE | `विज्ञानभैरव(n)`, `सभी_धारणाएँ()` |

### Improved गणित module

| Function | Change |
|----------|--------|
| `ब्रह्मगुप्त_अंतर्वेशन` | Renamed from `ब्रह्मगुप्त_अंतर` (correct Sanskrit: interpolation) |
| `आर्यभट_वर्ग_योग` | Renamed from `वर्ग_योग` (proper Aryabhata attribution) |
| `आर्यभट_घन_योग` | Renamed from `घन_योग` (proper Aryabhata attribution) |
| `ब्रह्मगुप्त_गुणन(a,b,c,d,n)` | New: Brahmagupta-Fibonacci identity (628 CE) → `[p, q]` |
| `कटपयादि(text)` | New: Kerala consonant-digit cipher (pre-8th CE) |
| `श्रीधर_सूत्र` | Fixed: now returns `[]` / `[x]` / `[x1,x2]` instead of error for no roots |

### Improved न्याय module

`हेत्वाभास` and `प्रमाण` now accept both number (1-5/1-4) AND string names.

### Improved छन्दस् module

Added `गुरु_लघु_संकेत(n, pos)`, `छन्द_स्थान(meter)`, `मात्रा_भार(meter)` — Pingala's complete binary-meter system.

### Improved नाट्य module

Added `रस_सूत्र(vibhava, anubhava, sanchari)` — Bharata's formula from Natyashastra 6.31.

### Improved ज्योतिष module

Added `ग्रह_परिक्रमा` as the proper Aryabhata name for `ग्रह_क्रम` (orbital period).

### New constants

```lipi
बताओ आर्यभट_पाई         # 3.1416 — Aryabhata's 4-decimal π approximation
बताओ आर्यभट_कोण         # π/48 — Aryabhata's fundamental angle unit (3.75°)
बताओ आर्यभट_ज्या_गणना   # 24.0 — entries in Aryabhata's sine table
बताओ नक्षत्र_संख्या      # 27.0 — 27 lunar mansions
बताओ तिथि_संख्या         # 30.0 — 30 lunar days
बताओ युग_वर्ष            # 4320000.0 — one Mahayuga in years
बताओ ब्रह्मगुप्त_शून्य   # 0.0 — Brahmagupta's zero (628 CE, first rigorous definition)
```

### New opcodes

| Opcode | Tag | Effect |
|--------|-----|--------|
| `Assert(Option<String>)` | `0x3F` | Pop; if falsy, halt with Nyaya error message |
| `DeclareConst(String)` | `0x40` | Pop → globals, add to `constants` set |

**Implementation:**
- `LVM.constants: HashSet<String>` — names that cannot be reassigned
- `StoreVar` checks `constants` before writing; raises `स्थिर 'x' को बदला नहीं जा सकता (Samkhya: नित्यम्)`
- `जाँचो` compiles to: `compile_expr + Assert(msg)`
- `स्थिर name है expr` compiles to: `compile_expr + DeclareConst(name)`
- `शुद्ध विधि` parses as `Vidhi { pure: true, .. }` — syntax works, runtime purity tracking is a future enhancement

Test file: `examples/phase16_test.swami` — all assertions pass.

---

## Phase 17 — IN PROGRESS (see task_plan.md for the full roadmap)

### Done

- **Default parameters** — `विधि नमस्ते(नाम="दुनिया"):` (2026-06-11)
  - Constant defaults only: number / string / bool / negative number — validated by the parser
  - `Param.default: Option<Expr>` (ast.rs) → `FuncDef.defaults: Vec<Option<LvmValue>>` (opcode.rs)
  - Lexer: bare `=` is `TokenKind::Assign`
  - LVM `fill_defaults()` fills missing trailing args at every bind site:
    `Call`, inherited constructor, closure-variable call, instance `MethodCall`, `TailCall`, `call_closure_value`
  - Serializer v3 stores defaults + class_parents (fixes inheritance in `.libc`); v2 still loads
  - Test: `examples/phase17_default_params.swami`

- **Keyword arguments** — `जोड़ो(ब=4, अ=3)` at call sites (2026-06-12)
  - Parser: `arg_list_kw()` detects `Ident =` lookahead; keywords must follow positionals,
    duplicates rejected; method calls reject keywords via shared `arg_list()` wrapper
  - AST: new `Expr::CallKw { name, args, kwargs }` (Expr::Call untouched)
  - New opcode `CallKw(name, pos_argc, kwnames)` — tag `TAG_CALL_KW = 0x41`
  - Stack layout: positional args first, then keyword values in kwname order
  - LVM `bind_args_kw()`: binds positionals (extras → vararg list), then keywords by
    param name, then fills defaults; errors on unknown keyword, double-binding, or
    missing required param. Resolves user functions, inherited `::बनाओ`, closure vars
  - Built-ins/natives + HOFs reject keywords at runtime; no TCO for kwarg calls
  - Tree-walk interpreter (legacy): CallKw returns an error
  - Test: `examples/phase17_kwargs_test.swami`

- **Floor division `//`** — `7 // 2` → 3 (2026-06-12)
  - Lexer: `//` → `TokenKind::SlashSlash`; same precedence level as `*` `/` `%`
  - `BinOp::FloorDiv` → new opcode `FloorDiv` — tag `0x42`
  - Python semantics: `(a / b).floor()` — `-7 // 2` → -4; division by zero errors
  - Test: `examples/phase17_floordiv_test.swami`

- **Tuple unpacking** — `अ, ब है 1, 2` (2026-06-12)
  - `Stmt::MultiAssign { names, values }` — statement-level only
  - Pairwise form: all RHS evaluated before stores → swap `अ, ब है ब, अ` works
  - Single-RHS form: value must be a List of exactly N elements (`क, ख है सूची`,
    `भाग, शेष है विधि_जो_सूची_देती()`) — new opcode `UnpackList(n)` tag `0x43`
    pushes elements in reverse so N normal `StoreVar` ops bind left-to-right
    (keeps स्थिर/वैश्विक StoreVar semantics)
  - Mismatched explicit counts (`अ, ब है 1, 2, 3`) = parse error; runtime
    length/type mismatch = catchable error
  - Test: `examples/phase17_unpack_test.swami`

- **Slice notation** — `सूची[1:4]`, `[:3]`, `[::2]`, `[::-1]` (2026-06-12)
  - Works on List and Str (char-level); full Python semantics: negative indices,
    clamping (out-of-range → empty, never errors), negative step
  - Parser: `bracket_suffix()` after `[` decides Index vs `Expr::Slice` by `:`
  - New opcode `Slice` — tag `0x44` — pops (obj, start, end, step), Nil = omitted
  - Engine is `slice_value()` + `slice_indices()` in `interpreter.rs` — shared by
    LVM and tree-walk interpreter
  - Step 0 and slicing a Dict are catchable runtime errors
  - Slice *assignment* (`सूची[1:3] है ...`) not supported
  - Test: `examples/phase17_slice_test.swami`

- **Membership `में_है` / `नहीं_है`** — `x में_है सूची` (2026-06-12)
  - List → element equality, Str → substring (Str item only), Dict → key existence
  - `Expr::Membership { item, container, negated }`; parsed at comparison level
  - New opcode `Contains` — tag `0x45`; `नहीं_है` = `Contains` + `Not`
  - Engine `contains_value()` in `interpreter.rs` — shared by both runtimes
  - Non-container (Number etc.) = catchable runtime error
  - Test: `examples/phase17_membership_test.swami`

- **Chained comparisons** — `0 < x < 10` (2026-06-12)
  - Parser-only desugar in `comparison()`: `a < b < c` → `(a < b) और (b < c)`
  - Any length (`1 < 2 < 3 < 4`), any mix of comparison operators
  - Also FIXED pre-existing mis-parse: `a < b < c` used to parse `(a<b) < c`
    (Bool compared to Number)
  - Caveat: middle expressions are compiled per pair — a side-effecting middle
    (function call) runs twice; documented, matches simple desugar semantics
  - Test: `examples/phase17_membership_test.swami` (group 5)

- **Spread operator** — `[*सूची1, 99, *सूची2]` (2026-06-12)
  - `Expr::ListWithSpread(Vec<(bool, Expr)>)` — plain lists stay `Expr::List`
  - New opcode `MakeListSp(Vec<bool>)` — tag `TAG_MAKE_LIST_SP = 0x46`
  - Spreading a non-list = catchable runtime error; both runtimes supported
  - Test: `examples/phase17_spread_test.swami`

- **JSON stdlib** — `आयात भारत.json` (2026-06-12)
  - `json_पढ़ो(text)` — full JSON parser (nested, escapes incl. `\uXXXX`
    surrogate pairs, 1e2 floats); JSON null → Nil, true/false → Bool
  - `json_लिखो(value)` — Value → JSON string, dict keys sorted; Inf/NaN,
    Instance/Closure = catchable errors
  - Hand-written recursive-descent parser in `bharat_stdlib.rs` (pure Rust)
  - Test: `examples/phase17_json_test.swami`

- **FS/OS builtins** (2026-06-12) — `तर्क`, `पथ_जोड़ो`, `फोल्डर_सूची`,
  `फोल्डर_बनाओ`, `फाइल_हटाओ`, `फाइल_कॉपी`, `पर्यावरण`, `वर्तमान_फोल्डर`
  - Pre-registered in `LVM::new()` (no import); errors catchable
  - `lipi foo.swami a b c` AND `lipi run foo.libc a b c` forward args to `तर्क()`
  - Test: `examples/phase17_fs_test.swami <temp_dir>` (needs one CLI arg)

- **VS Code extension** — `vscode-lipi/` (2026-06-12)
  - TextMate grammar, snippets, language config for `.swami` / `.roman` / `.vani`
  - Packaged (`lipi-lang-0.1.0.vsix`) + installed/verified locally; marketplace
    publish pending (needs publisher account + PAT)

- **DateTime stdlib** — `आयात भारत.समय` (2026-06-12)
  - Canonical value = UTC epoch seconds (Number); display/breakdown in IST (+5:30, no DST)
  - `समय_बनाओ`/`दिनांक_पार्स` take IST wall time; validate month/day/time → catchable errors
  - `समय_विवरण` → Dict {वर्ष माह दिन घंटा मिनट सेकंड वार वार_नाम माह_नाम}; weekday names रविवार…शनिवार
  - Pure Rust Hinnant `days_from_civil`/`civil_from_days` — works pre-1970 (negative epochs)
  - `समय_अभी()` errors on WASM (no system clock); everything else WASM-safe
  - Test: `examples/phase17_samay_test.swami` (deterministic — fixed epochs, known weekdays)

- **CSV stdlib** — `आयात भारत.csv` (2026-06-12)
  - `csv_पढ़ो` RFC 4180: quoted fields, `""` escapes, embedded commas/newlines, CRLF, blank lines skipped
  - `csv_शीर्षक_पढ़ो` — header row keys → List of Dict; row-length mismatch = catchable error
  - `csv_लिखो` — auto-quotes fields containing `,` `"` newline; fields stay strings on read
  - Test: `examples/phase17_csv_test.swami`

- **Hash/Base64 stdlib** — `आयात भारत.कूट` (2026-06-12)
  - `sha256` / `md5` → hex digest of UTF-8 bytes (pure-Rust reference implementations,
    verified against published vectors + .NET); `base64_कूट` / `base64_खोलो` (standard
    alphabet, padded; decode validates and requires UTF-8 result)
  - Test: `examples/phase17_koot_test.swami`

- **Typed exceptions** — `फेंको` + `वर्ग X(त्रुटि)` + typed `पकड़ो` (2026-06-12)
  - `Stmt::TryCatch { body, clauses: Vec<CatchClause> }` replaced Stmt::Koshish;
    `CatchClause { class: Option<String>, var, body }`; `Stmt::Phenko(Expr)`
  - New opcodes: `Throw` (tag 0x47), `MatchErrClass(String)` (tag 0x48)
  - LVM: `thrown: Option<Value>` channel — Throw pops the value, sets the channel,
    raises; the try unwinder pushes the Instance (or message Str) at handler ip
  - Handler compiles to a dispatch chain (Dup + MatchErrClass + JumpIfFalse);
    no clause matched → bare Throw rethrows to the outer कोशिश
  - `पकड़ो X:` single-ident ambiguity resolved at compile time via known_classes
    (typed if X is a class, else catch-all binding X — full back-compat)
  - Throw validates the instance's class chain reaches त्रुटि; Str throws allowed;
    other values = catchable error. Uncaught → halts with `class: संदेश`
  - Tree-walk interpreter: फेंको = "run in LVM" error (legacy)
  - Test: `examples/phase17_exceptions_test.swami` (9 groups)

- **Test framework** — `परीक्षण` + `lipi test` (2026-06-13)
  - `परीक्षण "नाम":` block → `Stmt::Parikshan` — compiles to NOTHING on normal
    runs (both runtimes skip it); only `lipi test file.swami` executes them
  - Runner (main.rs `run_tests`): partitions the file into tests vs setup
    (everything else); each test runs in a fresh VM with setup re-run first —
    full isolation, global mutations don't leak between tests
  - Per-test ✓/✗ (ANSI colors), failure message + पंक्ति line shown indented,
    summary with timing; exit code 1 if any test failed (CI-usable), 2 on
    parse/file errors
  - Demo: `examples/parikshan_demo.swami` (5 tests, one deliberately failing)

- **Regex stdlib** — `आयात भारत.प्रतिमान` (2026-06-13)
  - `src/regex_engine.rs` — hand-written backtracking VM (Pike/Cox style: pattern
    → Char/Class/Split/Jump/Save instructions, explicit backtrack stack, no Rust
    recursion); 2M-step budget catches catastrophic backtracking as a LIPI error
  - 7 functions: ढूंढो, ढूंढो_स्थान, ढूंढो_सब, मेल_है (full match), समूह,
    बदलो_सब (with $N group refs), विभाजित_सब
  - `\w` includes U+0900–U+097F so Hindi words (matras, halant) match as units
  - Test: `examples/phase17_pratimaan_test.swami` (10 groups)

- **Stack-overflow guard** (2026-06-13) — `LVM::push_frame()` enforces max call
  depth 10000 (catchable Hindi error); uncaught-trace display capped at 12 frames.

- **Lexer fixes** (2026-06-13)
  - Unknown string escapes now KEEP the backslash (`"\d"` stays `\d`, Python
    behavior) — regex patterns don't need doubling; `\r` escape added
  - `strip_comment` is now string-aware — `"#"` or `"।"` inside a string literal
    no longer truncates the line (was a real bug: any string containing # broke)

- **शून्य global** (2026-06-13) — pre-loaded Nil constant (LIPI's None);
  `vals_eq` now treats Nil == Nil as true. `यदि परिणाम बराबर शून्य:` works.

- **Performance pass** (2026-06-12/13) — VM hot loop takes `&Opcode` (no per-step
  clone); new `IterNext(container_var, index_var)` opcode (tag 0x49) reads the loop
  element in place instead of cloning the whole container each iteration (was O(n²)
  on list for-loops); optimized `[profile.release]` (LTO fat, codegen-units 1).
  Measured: loops ~7×, calls ~6×, list iteration ~8× faster. Release binary installed
  to `D:\Rust\cargo\bin\lipi.exe`.

- **Decorators** — `@सजावट` / `@कारखाना(आर्ग)` before `विधि` (2026-06-13)
  - `Stmt::Vidhi.decorators: Vec<Expr>`; lexer `@` → `TokenKind::At`
  - Decorated functions register under hidden `__deco_<name>__`; the visible name
    becomes a variable holding the decorated closure (resolved via the existing
    closure-variable Call path) — no new opcodes, .libc compatible
  - Stacking applies bottom-up; factory form evaluates to a closure parked in a
    `__deco_tmpN__` var; works on nested functions; NOT on class methods;
    tree-walk interpreter rejects decorators (LVM only)
  - Test: `examples/phase17_decorator_test.swami` (7 groups)

- **Runtime error diagnostics** — line numbers + stack traces (2026-06-13)
  - Parser wraps every statement in `Stmt::Located { line, inner }`; code matching
    on statement kind must use `ast::unwrap_located()`
  - Compiler keeps `lines: Vec<u32>` parallel to `instructions` (filled in `emit()`
    from `cur_line`); `CompiledProgram.lines`; serializer v4 persists it
  - `Frame.func_name` records the call name (TailCall updates it on frame reuse)
  - Uncaught errors only: `LVM::format_uncaught()` appends `(पंक्ति N)` + one
    `↳ विधि 'X' — पंक्ति N से बुलाई गई` per live frame (innermost first); errors
    caught by कोशिश stay clean strings so पकड़ो handlers are unaffected
  - `main.rs` `show_error_line` now also matches `(पंक्ति N)` → prints the source
    line with caret for runtime errors; TCO-reused frames appear as one trace entry
  - Demo: `examples/trace_demo.swami`

- **HTTP client stdlib** — `आयात भारत.http` (2026-06-12)
  - Pure Rust std::net TcpStream, HTTP/1.1, Connection: close, 10s timeouts,
    Transfer-Encoding: chunked decoded, response header keys lowercased
  - **No TLS** — https:// = clear catchable error; ftp/bare URLs rejected
  - 404 etc. are normal returns (check स्थिति), only transport/parse failures throw
  - Test: `examples/phase17_http_test.swami` + `examples/http_test_server.js`
    (node server on 127.0.0.1:8731 — start it before running the test)

- **Fixes** (2026-06-12, found while testing CSV)
  - Triple-quote preprocessor: `""` inside `"""..."""` emitted an UNESCAPED quote,
    terminating the string early — now emits `\"`; literal `\` now escaped to `\\`
    (triple-quote content is verbatim)
  - Lexer now strips a leading UTF-8 BOM (U+FEFF) — files saved by Windows editors
    with BOM used to fail with `Unknown('\u{feff}')`

- **Fixes** (2026-06-12, found during integration)
  - String interpolation now only fires for `{identifier}` / `{obj.field}` —
    raw JSON braces inside strings stay literal (`is_interp_target` in parser.rs)
  - Lexer पढ़ो/पकड़ो nukta rules now exact-match codepoint sequences — the old
    loose contains-check swallowed identifiers like `पथ_जोड़ो` (became stdin read!)

- **Named format placeholders** — `{नाम:.2}` in interpolated strings (2026-06-13)
  - `"पाई = {मूल्य:.4}"`, `"{राशि:₹}"`, `"{दर:%}"`, `"{कोड:06}"`, `"{राशि:,}"` —
    variable (or `obj.field`) + any स्वरूप spec, in ANY string literal
  - Parser-only desugar in `parse_fmt_string`: `{target:spec}` → `स्वरूप("{:spec}", target)`
    spliced into the existing concat chain — no new opcodes, .libc untouched
  - Spec must start with digit/`.`/`%`/`,`/`₹` — JSON text and times like `"10:30"`
    stay literal; positional `{:spec}` for स्वरूप() unchanged
  - Test: `examples/phase17_namedspec_test.swami`

- **List comprehensions** — `[x*x के लिए x सूची में यदि cond]` (2026-06-29)
  - `Expr::Comprehension { expr, clauses, cond }`; multi-clause nesting (leftmost
    outermost), optional यदि filter (innermost); iterates List/Str/range/Dict-keys
  - Compiles to KeeLiye loop machinery with the accumulator list on the VM stack —
    each element is `MethodCall("जोड़ो")` on the stack top, no new opcodes
  - Also FIXED `vals_eq` (lvm.rs) to do **structural equality** for List/Dict/
    Instance/Enum (was scalar-only — `[1,2] बराबर [1,2]` returned false)
  - Test: `examples/phase17_comprehension_test.swami`

- **Chained assignment** — `अ है ब है 0` (2026-06-29)
  - `Stmt::ChainAssign { names, value }`; RHS evaluated once, stored into every
    target (Dup+StoreVar chain). Parser uses `Ident है` lookahead in the है branch
  - Test: `examples/phase17_batch1_test.swami`

- **Pattern match guards** — `वृत्त(r) यदि r से अधिक 10:` in मिलाओ (2026-06-29)
  - `MilaoArm.guard: Option<Expr>`; the subject value stays on the stack across all
    arms, so a failed guard (or variant mismatch) retries the next arm. Compiler
    switched to a `pending_next: Vec<usize>` jump list (was a single `last_jf`)
  - Test: `examples/phase17_batch1_test.swami`

- **Multiline collections** — list/dict/call spanning lines (2026-06-29)
  - Lexer tracks ( [ { depth across lines; while depth > 0 it suppresses
    INDENT/DEDENT and the trailing Newline (`bracket_delta` helper in lexer.rs).
    Brackets inside string literals don't count (already folded into Str tokens)
  - Test: `examples/phase17_batch1_test.swami`

- **Operator overloading** — `__जोड़ो__`/`__घटाओ__`/`__गुणा__`/`__भाग__`/`__शेष__` (2026-06-29)
  - VM's Add/Sub/Mul/Div/Mod opcodes: if the left operand is an Instance whose
    class (walking the parent chain) defines the dunder, `try_instance_binop` sets
    up a `method(यह, अन्य)` frame and the method's Return supplies the operator
    result. Arithmetic only — comparison/`बराबर` overloading not included
  - Test: `examples/phase17_batch3_test.swami`

- **Statistics + UUID + zip/enumerate** (2026-06-29)
  - `आयात भारत.सांख्यिकी` → माध्य/माध्यिका/बहुलक/प्रसरण/मानक_विचलन/योग/न्यूनतम/अधिकतम/परिसर
    (`sankhyiki_registry` in bharat_stdlib.rs); `यूआईडी()` (UUID v4), `युग्म()` (zip),
    `गणना()` (enumerate), `श्रृंखला()` (chain), `गिनती_कोश()` (Counter) pre-registered
    builtins in LVM::new()
  - Tests: `examples/phase17_batch2_test.swami`

- **Walrus / static methods / abstract classes / context managers** (2026-06-29)
  - **Walrus** `(न := expr)` — `:=` token (lexer), `Expr::Walrus` compiles to
    `Dup + StoreVar` (value stays on stack); use in conditions. `examples/phase17_batch4_test.swami`
  - **Static methods** `साझा विधि m(...)` in a class — registered as `Class::m`
    WITHOUT यह; `ClassName.m(args)` compiles to a direct `Call("Class::m")` when the
    object identifier is a known class (no Value::Class needed). `phase17_batch5_test`
  - **Abstract classes** `सार वर्ग C:` — `is_abstract` in Stmt::Varg; constructor
    calls (Call/CallKw) on an abstract class emit `Push(msg)+Throw` (catchable);
    concrete subclasses instantiate + inherit normally. `phase17_batch5_test`
  - **Context managers** `साथ expr के_रूप_में नाम:` — `Stmt::Saath`; compiles to
    try/finally: `expr.__प्रवेश__()`→binds नाम, body, then `__निकास__()` on the
    normal path AND the error path (which rethrows). `phase17_batch6_test`
  - **itertools** `कार्तीय`(product), `सर्व_संयोजन`(combinations) builtins. `phase17_batch4_test`
  - New keywords: `साझा` `सार` `साथ` `के_रूप_में` (साझा/सार are keywords only
    before विधि/वर्ग; otherwise plain identifiers)

- **Records / dataclasses** — `अभिलेख बिंदु(x, y)` (2026-06-29)
  - Parser-only sugar (`अभिलेख` keyword) → a class with an auto-generated बनाओ
    that stores each field; repr (`<बिंदु {x: 3, y: 4}>`) and structural equality
    come free from the instance machinery. `examples/phase17_batch7_test.swami`

- **Properties** — getter/setter on class fields (2026-06-30)
  - Define `विधि __पाओ_<field>__(यह):` (getter) and `विधि __सेट_<field>__(यह, मान):`
    (setter) inside a class; GetAttr/SetAttr dispatch to them for Instance receivers
    (modelled on `try_instance_binop`, walks the parent chain). Backing store uses the
    `_<field>` convention (no dunder → no recursion).
  - **CAVEAT:** a setter body MUST end with `फल यह` so the mutated instance flows back to
    the StoreVar after SetAttr (a plain method returns Nil and would clobber the instance) —
    same model as operator overloading. Test: `examples/phase17_properties_deque_test.swami`

- **Queue/Deque builtins** — `अग्र_जोड़ो`/`अग्र`/`पश्च`/`अग्र_हटाओ`/`पश्च_हटाओ` (2026-06-30)
  - Pre-registered, COW (return new list, must reassign); empty/wrong-type = catchable error.
    Test: `examples/phase17_properties_deque_test.swami`

- **Memory limits** (2026-06-30) — `MAX_LIST_LEN = 50_000_000` checked in
  MakeList/MakeListSp/जोड़ो/अग्र_जोड़ो; `MAX_STACK_DEPTH = 10_000_000` operand-stack guard at
  the top of `exec_op`. Both raise catchable Hindi errors.

- **Tooling: fmt / lint / doc** (2026-06-30) — `lipi fmt` (behavior-preserving + idempotent
  formatter, `src/formatter.rs`), `lipi lint` (unused/undefined vars, `src/lint.rs`),
  `lipi doc` (Markdown from विधि/वर्ग signatures + leading comments, `src/docgen.rs`).

### Phase 17 COMPLETION (2026-06-30) — all remaining 17A–17D items finished

- **Generators** — `उत्पन्न expr` (yield). Generator functions (any body containing
  उत्पन्न) collect yielded values into a hidden `__gen_acc__` list and return it;
  `फल` inside a generator stops early and returns what's accumulated. Eager
  (collect-to-list) semantics — infinite generators not supported; works in
  for-loops + मानचित्र/छानो. No new opcodes (reuses MakeList/MethodCall/Return).
  Test: `examples/phase17_generator_test.swami`
- **Functools** — `स्मरण`(memoize, persistent VM-level cache), `आंशिक`(partial),
  `संयोजित`(compose). Built as tagged `Value::Closure` (`__functools_*`) intercepted
  at call time by `LVM::call_functools`. Test: `examples/phase17_functools_test.swami`
- **OrderedDict** — `क्रमित_कोश`/`क्रमित_रखो`/`क्रमित_पाओ`/`क्रमित_कुंजियाँ`/`क्रमित_मान`,
  insertion-ordered list-of-pairs. Test: `examples/phase17_ordereddict_test.swami`
- **Big integers** — `आयात भारत.बड़ी` (`src/bignum.rs`): pure-Rust base-1e9 arbitrary
  precision, decimal-string I/O. Test: `examples/phase17_bignum_test.swami`
- **Unicode NFC** — `normalize_devanagari()` lexer pre-pass + `सामान्यीकृत()` builtin;
  decomposes precomposed nukta letters (NFC form). Test: `examples/phase17_nfc_test.swami`
- **Sockets** — `आयात भारत.संजाल` (`src/net.rs`): TCP client+server, thread-local
  handle registry. Tests: `examples/phase17_socket_{server,client}.swami`
- **ZIP** — `आयात भारत.संपीडन` (`src/zip.rs`): read STORE+DEFLATE (full inflate) /
  write STORE + CRC32. Test: `examples/phase17_zip_test.swami`
- **SQL** — `आयात भारत.संग्रह` (`src/sql.rs`): minimal in-memory SQL engine +
  file persistence. Test: `examples/phase17_sql_test.swami`
- **Profiler** — `lipi profile` (`LVM::run_profiled`): opcode counts/%, time, fn calls.
- **REPL** — persistent session (accumulate + output-delta replay), multiline block
  input, `~/.lipi_history`, `:इतिहास`/`:रीसेट`/`:सहायता`.
- **Package manager** — `lipi pkg init/add/install/list` (`src/pkg.rs`), `lipi.toml` +
  `lipi_modules/`. **ALSO fixed a pre-existing cross-file function-call bug**: `आयात
  "file"` now inlines the imported AST at COMPILE time (shared instruction space →
  correct `start_ip`s), so imported functions are finally callable. `imported_files`
  set guards against import cycles. (The old runtime `ImportFile` opcode + lvm handler
  remain for `.libc` back-compat.)
- **LSP** — `lipi lsp` (`src/lsp.rs`): pure-Rust JSON-RPC over stdio — initialize,
  publishDiagnostics (parse errors), hover, completion, documentSymbol, shutdown.
- **Debugger** — `lipi debug` (`LVM::run_debug`): line breakpoints, step, continue,
  print var, vars, where; uses the v4 line table.
- **Int type / GC** — resolved by design (documented in `task_plan.md`): numbers are
  f64 (exact ≤ 2^53) + भारत.बड़ी for arbitrary precision (`पूर्ण_है()` predicate added);
  Values are clone-trees with no cycles, so Rust ownership = GC.
- **VS Code ext** — grammar updated for new keywords, repackaged `lipi-lang-0.2.0.vsix`;
  marketplace publish still pending the maintainer's publisher account + PAT.

## Phase 17 — fully complete. Possible Phase 18 directions

1. **Module system** — `आयात "file.swami" as नाम` with namespace access
2. **Lazy generators** — true coroutine-style yield (current is eager collect-to-list)
3. **Async / await** — event loop + `प्रतीक्षा करो`
4. **Pure function runtime enforcement** — track global writes inside `शुद्ध` functions
5. **`n बार तक चरण` loop** — Vedic ritual step counter with named iteration variable
6. **Graphical profiler** — flame-graph output (current profiler is a text report)

---

## Phase 18 #7 — Gradual type system + `lipi check` — COMPLETE

Optional Devanagari type hints + a static checker. **Gradual**: unannotated code is
unchanged and never flagged; annotations are **parse-only metadata** — the compiler
and VM ignore them, `.libc` format is untouched. Zero runtime cost.

```lipi
विधि जोड़ो(अ: संख्या, ब: संख्या) -> संख्या:
    फल अ + ब
नाम: वाक्य है "राम"
```

- **Type names** (`types.rs` — `TypeHint::from_name`): `संख्या`/`अंक` (Number),
  `वाक्य`/`पाठ` (Str), `तर्क`/`बूल` (Bool), `सूची` (List), `कोश` (Dict), `शून्य` (Nil),
  `कुछ_भी` (Any — gradual escape hatch, never flagged). Any other identifier →
  `Named(class)`, checked permissively.
- **Syntax**: `:` for value types, `->` for return type (lexer adds `TokenKind::Arrow`).
  `:` inside `(…)` can't clash with the block `:`. Param order: `name [karaka] [: type] [= default]`.
- **AST**: `Param.type_hint`, `Stmt::Vidhi.ret_type`, `Stmt::Assign.type_hint` — all
  `Option<TypeHint>`, ignored by the compiler.
- **Checker** (`typecheck.rs`, `lipi check`): static pass over the AST. Flags only
  *concrete* mismatches — (1) non-number operand to `- * / // % & | ^ << >>`, (2) call
  arg vs declared param type, (3) `फल` vs `-> प्रकार`, (4) annotated var init/reassign.
  `+` is never flagged (LIPI coerces — Str+Number is idiomatic). `Any`/`Named` silence
  checks. Exit 0 clean / 1 mismatch / 2 parse error.
- Tests: `examples/phase18_typecheck_test.swami` (clean — passes check AND runs),
  `examples/phase18_typecheck_bad.swami` (4 deliberate errors); Rust unit tests in
  `types.rs` + `typecheck.rs`. Full regression unaffected (annotations optional).

---

## Environment notes

- **Project path:** `D:\Projects\lipi-lang`
- **All files on D drive** — never write to C drive
- **Rust toolchain:** `stable-x86_64-pc-windows-gnu` (default)
- **Linker config:** `D:\Rust\cargo\config.toml` — points to `D:\msys64\mingw64\bin\gcc.exe`
- **Pure Rust only** — no external crates except `wasm-bindgen` (optional, WASM feature only)
- **Source extension:** `.swami`
- **Examples:** `examples/demo_full.swami` (comprehensive), `examples/phase5_test.swami` through `examples/phase15_test.swami`
