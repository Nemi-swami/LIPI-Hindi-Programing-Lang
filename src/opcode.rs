//! LVM Opcode definitions — the instruction set for the LIPI Virtual Machine.

use std::collections::HashMap;

/// A user-defined function: parameter names and entry-point instruction index.
#[derive(Debug, Clone)]
pub struct FuncDef {
    pub params: Vec<String>,
    pub start_ip: usize,
    /// Vararg parameter name — collects extra positional args into a List (Phase 12)
    pub vararg: Option<String>,
    /// Per-param constant default values, parallel to `params` (Phase 17).
    /// `None` = required parameter; missing trailing args are filled at call time.
    pub defaults: Vec<Option<LvmValue>>,
    /// True if the body contains `उत्पन्न` — calling it returns a lazy generator
    /// object instead of running the body (Phase 18 — true coroutines).
    pub is_generator: bool,
}

/// Values that can be encoded directly in Push instructions.
#[derive(Debug, Clone)]
pub enum LvmValue {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for LvmValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LvmValue::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 { write!(f, "{}", *n as i64) }
                else { write!(f, "{n}") }
            }
            LvmValue::Str(s)  => write!(f, "{s}"),
            LvmValue::Bool(b) => write!(f, "{}", if *b { "सत्य" } else { "असत्य" }),
            LvmValue::Nil     => write!(f, "शून्य"),
        }
    }
}

/// Output of the compiler: flat bytecode array + function table.
#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub instructions: Vec<Opcode>,
    /// Source line per instruction, parallel to `instructions` — 0 = unknown
    /// (e.g. loaded from a pre-v4 .libc). Phase 17 runtime diagnostics.
    pub lines: Vec<u32>,
    pub functions: HashMap<String, FuncDef>,
    /// Parent class lookup for inheritance: child → parent class name
    pub class_parents: HashMap<String, Vec<String>>,
}

/// LIPI VM instruction set.
#[derive(Debug, Clone)]
pub enum Opcode {
    // ── Stack ─────────────────────────────────────────────────────────────
    Push(LvmValue),     // push constant onto stack
    Pop,                // discard top of stack
    Dup,                // duplicate top of stack

    // ── Variables ─────────────────────────────────────────────────────────
    LoadVar(String),    // push variable value onto stack
    StoreVar(String),   // pop stack → store in variable

    // ── Arithmetic ────────────────────────────────────────────────────────
    Add, Sub, Mul, Div, Mod,

    // ── Comparison (each pushes a Bool result) ────────────────────────────
    Eq, NotEq, Gt, Lt, GtEq, LtEq,

    // ── Logic ─────────────────────────────────────────────────────────────
    And, Or, Not,

    // ── Control flow ──────────────────────────────────────────────────────
    Jump(usize),            // unconditional jump to instruction index
    JumpIfFalse(usize),     // pop; jump if falsy
    JumpIfTrue(usize),      // pop; jump if truthy

    // ── Calls ─────────────────────────────────────────────────────────────
    Call(String, usize),        // function name, arg count — resolved at runtime
    CallNative(String, usize),  // native function name, arg count
    Return,                     // return top-of-stack to caller

    // ── Output ────────────────────────────────────────────────────────────
    Print,
    PrintInline,  // print without newline (लिखो) — Phase 7

    // ── Karaka check (soft semantic warning, never a hard error) ──────────
    KarakaCheck(String, String),    // param_name, expected_karaka_name

    // ── Module import ─────────────────────────────────────────────────────
    Import(String),

    // ── Method calls ──────────────────────────────────────────────────────
    MethodCall(String, usize),      // method_name, n_args (object below args)

    // ═══ First-class Indian opcodes — unique to LVM ══════════════════════
    AadhaarVerify,  // pop string  → push bool  (Verhoeff / UIDAI spec)
    UpiSend,        // pop (from, to, amount, note) → push result string
    GstAdd,         // pop (amount, rate) → push amount with GST
    LakhParse,      // pop f64 → push "X लाख" string
    RupeeFormat,    // pop f64 → push "₹X,XX,XXX" string

    // ── Phase 5: सूची + कोश ──────────────────────────────────────────────
    MakeList(usize), // pop N values (top=last) → push List
    MakeDict(usize), // pop 2*N values (val,key pairs) → push Dict
    GetIndex,        // pop idx, pop obj → push obj[idx]
    SetIndex,        // pop val, pop idx, pop obj → push updated obj

    // ── Phase 6: वर्ग (Classes) ───────────────────────────────────────────
    MakeInstance(String), // push empty Instance { class, fields:{} }
    GetAttr(String),      // pop instance → push instance.fields[name]
    SetAttr(String),      // pop val, pop instance → push updated instance

    // ── Phase 9: Error handling & multi-file ──────────────────────────────
    TryStart(usize),      // push try frame with handler_ip
    TryEnd,               // pop try frame (no error in body)
    ImportFile(String),   // load, compile, run another .swami file

    // ── Phase 10: First-class functions ───────────────────────────────────
    MakeClosure(String),  // push Value::Closure{func_name, captured} onto stack

    // ── Phase 11: Iterable for-loop ───────────────────────────────────────
    GetIterLen,  // pop iterable → push length (Number→n, List→len, Str→char count)
    GetIterItem, // pop (iterable, idx) → push element (Number→idx, List→item, Str→char)
    /// (container_var, index_var) — read both variables in place, push current
    /// element. Avoids cloning the whole container per iteration (Phase 17 perf).
    IterNext(String, String),

    // ── Phase 12: Bitwise operations ──────────────────────────────────────
    BitAnd,   // pop (a, b) → push a & b  (integer)
    BitOr,    // pop (a, b) → push a | b
    BitXor,   // pop (a, b) → push a ^ b
    BitNot,   // pop a      → push ~a
    LShift,   // pop (a, b) → push a << b
    RShift,   // pop (a, b) → push a >> b

    // ── Phase 13: Global variable declaration ─────────────────────────────
    DeclareGlobal(String), // marks name as global in current function frame

    // ── Phase 15: Enums ───────────────────────────────────────────────────
    /// Store enum type in globals: name → Value::EnumDef { name, variants }
    DefineEnum(String, Vec<(String, usize)>),
    /// Peek top of stack (enum), push Bool: is it this variant?
    MatchVariant(String),
    /// Pop enum from stack, store its field values as locals with given names
    EnumUnpack(Vec<String>),

    // ── Phase 15: Tail-call optimization ──────────────────────────────────
    /// Like Call but reuses the current frame — only valid as last call before Return
    TailCall(String, usize),

    // ── Phase 16: Nyaya assert + Samkhya const ──────────────────────────────
    /// Pop top of stack; if falsy, halt with error message (Nyaya Pratijna verification)
    Assert(Option<String>),
    /// Store name in globals AND mark it immutable (Samkhya nityam — the permanent)
    DeclareConst(String),

    // ── Phase 17: Keyword arguments ───────────────────────────────────────
    /// Call with keyword args: (name, positional argc, keyword names).
    /// Stack layout: positional args first, then keyword values in kwname order.
    CallKw(String, usize, Vec<String>),

    /// `//` — floor (integer) division: pop (a, b) → push floor(a / b)
    FloorDiv,

    /// Tuple unpacking: pop a List of exactly N elements, push them in reverse
    /// order so N following StoreVar ops bind left-to-right (Phase 17)
    UnpackList(usize),

    /// Slice: pop (obj, start, end, step) — Nil = omitted part — push sliced
    /// List/Str with Python semantics (Phase 17)
    Slice,

    /// में_है membership: pop (item, container) → push Bool.
    /// List = element, Str = substring, Dict = key (Phase 17)
    Contains,

    /// List literal with spread: pop flags.len() values (top = last element);
    /// flag=true values must be Lists and are spliced, flag=false values are
    /// appended as-is — push the combined List (Phase 17)
    MakeListSp(Vec<bool>),

    // ── Phase 17A: Typed exceptions ───────────────────────────────────────
    /// फेंको — pop value and throw it. Str throws a plain error (message);
    /// an Instance whose class chain reaches त्रुटि throws a typed error.
    /// Also used to rethrow when no पकड़ो clause of a कोशिश matched.
    Throw,
    /// Peek top of stack (the in-flight error value), push Bool:
    /// does it match this error class (or one of its subclasses)?
    /// Str errors match only the base class त्रुटि.
    MatchErrClass(String),

    // ── Phase 18: lazy generators (true coroutines) ──────────────────────────
    /// उत्पन्न — suspend the running generator, leaving the yielded value on the
    /// stack for the resumer (set inside `resume_generator`'s loop). Tag 0x4A.
    Yield,
    /// Unified for-loop step: advance an iterable held in `container_var` (List/
    /// Str/Dict/Number range OR a lazy Generator), store the next element into
    /// `loop_var`, and push Bool — true if a value was produced, false if the
    /// iterable is exhausted. `idx_var` tracks position for indexable types
    /// (ignored for generators). Tag 0x4B. (Phase 18)
    IterStep { loop_var: String, container_var: String, idx_var: String },
    /// obj.method(pos…, kw=…) — instance method call with keyword args. Stack:
    /// object, positional args, then keyword values in `kwnames` order. Tag 0x4C.
    MethodCallKw { method: String, pos_argc: usize, kwnames: Vec<String> },

    SetSlice,
}
