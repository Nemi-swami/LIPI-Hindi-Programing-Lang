/// LIPI 2.0 AST — nodes named after Sanskrit grammatical terms

pub use crate::karaka::Karaka;
pub use crate::types::TypeHint;

pub type Program = Vec<Stmt>;

#[derive(Debug, Clone)]
pub enum Stmt {
    /// नाम [करक] [: प्रकार] है expr  — assignment with optional Karaka role
    /// and optional type annotation (Phase 18 #7, gradual — parse-only metadata).
    Assign { name: String, karaka: Option<Karaka>, type_hint: Option<TypeHint>, value: Expr },

    /// बताओ expr
    Print(Expr),

    /// यदि cond: block [अन्यथा: block]
    Yadi { condition: Expr, then: Vec<Stmt>, otherwise: Option<Vec<Stmt>> },

    /// N बार करो: block
    BarKaro { count: Expr, body: Vec<Stmt> },

    /// var के लिए iter में: block
    KeeLiye { var: String, iter: Expr, body: Vec<Stmt> },

    /// विधि name(params[, *vararg]): body    शुद्ध = no global side effects (Phase 16)
    /// decorators (Phase 17): `@सजावट` lines above the definition, outermost first.
    /// Each is Expr::Ident (bare @नाम) or Expr::Call/CallKw (@कारखाना(आर्ग) factory).
    /// is_static (Phase 17): a class method declared `साझा विधि` — no implicit
    /// यह, callable as `ClassName.method(args)`.
    /// ret_type (Phase 18 #7): optional `-> प्रकार` return annotation — gradual,
    /// parse-only metadata read only by the static checker.
    Vidhi { name: String, params: Vec<Param>, body: Vec<Stmt>, vararg: Option<String>, pure: bool, decorators: Vec<Expr>, is_static: bool, ret_type: Option<TypeHint> },

    /// फल expr
    Fal(Expr),

    /// उत्पन्न expr — yield a value from a generator function (Phase 17)
    Yield(Expr),

    /// bare expression (function call as statement)
    ExprStmt(Expr),

    /// आयात भारत.पहचान  — import stdlib module
    Aayat(String),

    /// name[idx] है val  — index assignment (Phase 5)
    IndexAssign { obj: String, idx: Expr, val: Expr },

    /// वर्ग name[(parent)]: [methods]  — class definition (Phase 6/9).
    /// is_abstract (Phase 17): `सार वर्ग` — cannot be instantiated directly.
    Varg { name: String, parent: Option<String>, methods: Vec<Stmt>, is_abstract: bool },

    /// obj.field है val  — attribute assignment (Phase 6)
    AttrAssign { obj: String, field: String, val: Expr },

    /// जब तक cond: body  — while loop (Phase 7)
    JabTak { condition: Expr, body: Vec<Stmt> },

    /// बंद करो  — break out of loop (Phase 7)
    BandKaro,

    /// अगला  — continue to next iteration (Phase 7)
    Agla,

    /// लिखो expr  — print without newline (Phase 7)
    Likho(Expr),

    /// कोशिश: body + one or more पकड़ो clauses — try/catch (Phase 9, typed Phase 17A).
    /// Clauses are checked in order; `class: None` = catch-all (bare पकड़ो त्रुटि:).
    TryCatch { body: Vec<Stmt>, clauses: Vec<CatchClause> },

    /// फेंको expr  — throw a typed error instance or a plain string (Phase 17A)
    Phenko(Expr),

    /// आयात "file.swami"  — import another .swami file (Phase 9)
    AayatFile(String),

    /// आयात "file.swami" के_रूप_में नाम  — namespaced import (Phase 18)
    AayatFileAs { path: String, alias: String },

    /// वैश्विक नाम1, नाम2  — declare names as global in a function (Phase 13)
    Global(Vec<String>),

    /// विकल्प Name: INDENT variants DEDENT  — enum definition (Phase 15)
    ViKalp { name: String, variants: Vec<ViKalpVariant> },

    /// मिलाओ expr: INDENT arms DEDENT  — pattern match (Phase 15)
    Milao { subject: Expr, arms: Vec<MilaoArm> },

    /// जाँचो expr [, "message"]  — assert (Phase 16, Nyaya Pratijna)
    Jancho { expr: Expr, message: Option<Expr> },

    /// स्थिर name है expr  — immutable constant declaration (Phase 16, Samkhya)
    SthirDecl { name: String, value: Expr },

    /// अ, ब है 1, 2  — tuple unpacking (Phase 17).
    /// values.len() == names.len() → pairwise (all RHS evaluated before stores, swap-safe);
    /// values.len() == 1 → RHS must evaluate to a List of exactly names.len() elements.
    MultiAssign { names: Vec<String>, values: Vec<Expr> },

    /// अ है ब है 0  — chained assignment (Phase 17). The single RHS is
    /// evaluated once and stored into every target (rightmost binds last).
    ChainAssign { names: Vec<String>, value: Expr },

    /// साथ expr के_रूप_में नाम: body  — context manager (Phase 17).
    /// Calls expr.__प्रवेश__() → binds नाम, runs body, then always calls
    /// expr.__निकास__() (on normal completion AND on a thrown error, which is
    /// re-raised afterwards).
    Saath { expr: Expr, var: String, body: Vec<Stmt> },

    /// परीक्षण "नाम": block — test definition (Phase 17 test framework).
    /// Skipped entirely on normal runs; executed by `lipi test file.swami`.
    Parikshan { name: String, body: Vec<Stmt> },

    /// Source-position wrapper added by the parser around every statement
    /// (Phase 17 runtime diagnostics). The compiler records `line` into the
    /// instruction line table; everything else must look through it
    /// (see `unwrap_located`).
    Located { line: usize, inner: Box<Stmt> },
}

/// Strip `Stmt::Located` wrappers — for code that matches on statement kind
/// (class-method extraction, compiler pre-passes).
pub fn unwrap_located(stmt: &Stmt) -> &Stmt {
    match stmt {
        Stmt::Located { inner, .. } => unwrap_located(inner),
        other => other,
    }
}

/// One पकड़ो clause of a कोशिश statement (Phase 17A typed exceptions).
/// `पकड़ो त्रुटि:`     → class None,        var "त्रुटि"  (catch-all, back-compat)
/// `पकड़ो त्रुटि ई:`   → class None,        var "ई"       (catch-all, explicit var)
/// `पकड़ो X:`          → class Some("X"),   var "त्रुटि"
/// `पकड़ो X ई:`        → class Some("X"),   var "ई"
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub class: Option<String>,
    pub var: String,
    pub body: Vec<Stmt>,
}

/// One variant inside a विकल्प definition.
/// E.g. `वृत्त(त्रिज्या)` → name="वृत्त", fields=["त्रिज्या"]
#[derive(Debug, Clone)]
pub struct ViKalpVariant {
    pub name: String,
    pub fields: Vec<String>,
}

/// One arm inside a मिलाओ statement.
/// `guard` (Phase 17): optional `यदि <cond>` after the pattern — the arm
/// matches only when the pattern matches AND the guard is truthy.
#[derive(Debug, Clone)]
pub struct MilaoArm {
    pub pattern: MilaoPattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

/// Pattern in a मिलाओ arm.
#[derive(Debug, Clone)]
pub enum MilaoPattern {
    /// VariantName(bound1, bound2, ...)  — matches enum variant, binds fields to vars
    Variant(String, Vec<String>),
    /// अन्यथा  — wildcard / default
    Wildcard,
}

/// Function parameter with optional Karaka role annotation
/// and optional constant default value (Phase 17): `विधि जोड़ो(अ, ब=0):`
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub karaka: Option<Karaka>,
    pub default: Option<Expr>,
    /// Optional `: प्रकार` type annotation (Phase 18 #7, gradual — checker-only).
    pub type_hint: Option<TypeHint>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Compare { left: Box<Expr>, op: CmpOp, right: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    /// func(अ, नाम=मान)  — call with keyword arguments (Phase 17)
    CallKw { name: String, args: Vec<Expr>, kwargs: Vec<(String, Expr)> },
    /// प्रतीक्षा expr — await an async task (Phase 18). Suspends to the scheduler.
    Await(Box<Expr>),
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    /// obj.method(अ, नाम=मान) — method call with keyword arguments (Phase 18)
    MethodCallKw { object: Box<Expr>, method: String, args: Vec<Expr>, kwargs: Vec<(String, Expr)> },
    /// [e1, e2, ...]  — सूची literal (Phase 5)
    List(Vec<Expr>),
    /// [*सूची1, 99, *सूची2] — list literal with spread elements (Phase 17).
    /// bool = is-spread: true elements must evaluate to a List and are spliced in.
    ListWithSpread(Vec<(bool, Expr)>),
    /// {"k": v, ...}  — कोश literal (Phase 5)
    Dict(Vec<(Expr, Expr)>),
    /// expr[idx]       — index access (Phase 5)
    Index { obj: Box<Expr>, idx: Box<Expr> },

    /// expr[start:end:step] — slice on List/Str, Python semantics (Phase 17).
    /// All three parts optional: `[1:]`, `[:3]`, `[::2]`, `[::-1]`
    Slice {
        obj: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },

    /// obj.field       — attribute access (Phase 6)
    Attr { obj: Box<Expr>, field: String },

    /// लाम्डा(x, y): expr  — anonymous function (Phase 10)
    Lambda { params: Vec<String>, body: Vec<Stmt> },

    /// यदि cond तो a अन्यथा b  — ternary expression (Phase 12)
    Ternary { condition: Box<Expr>, then_val: Box<Expr>, else_val: Box<Expr> },

    /// ~x  — bitwise NOT (Phase 12)
    BitNot(Box<Expr>),

    /// नहीं expr  — logical NOT (Phase 13)
    Not(Box<Expr>),

    /// item में_है container — membership test on List/Str/Dict (Phase 17).
    /// negated = नहीं_है
    Membership { item: Box<Expr>, container: Box<Expr>, negated: bool },

    /// नाम := expr — walrus / inline assignment (Phase 17). Stores expr into
    /// नाम and evaluates to that value, so it can be used inside conditions.
    Walrus { name: String, value: Box<Expr> },

    /// [expr के लिए var iter में (के लिए ...)* (यदि cond)?] — list
    /// comprehension (Phase 17). Multiple clauses nest left-to-right
    /// (leftmost is outermost); the यदि filter applies innermost.
    Comprehension { expr: Box<Expr>, clauses: Vec<(String, Expr)>, cond: Option<Box<Expr>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, BitNot }

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp { Add, Sub, Mul, Div, FloorDiv, Mod, BitAnd, BitOr, BitXor, LShift, RShift, And, Or }

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    Eq, NotEq,
    SeAdhik,   // से अधिक — greater than (Hindi-natural)
    SeKam,     // से कम   — less than
    Lt, Gt, LtEq, GtEq,
}
