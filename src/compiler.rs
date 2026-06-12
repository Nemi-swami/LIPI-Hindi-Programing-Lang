//! LIPI AST → LVM Bytecode Compiler

use std::collections::{HashMap, HashSet};
use crate::ast::{unwrap_located, BinOp, CmpOp, Expr, Stmt};
use crate::opcode::{CompiledProgram, FuncDef, LvmValue, Opcode};

pub struct Compiler {
    instructions: Vec<Opcode>,
    /// Source line per instruction, parallel to `instructions` (Phase 17 diagnostics)
    lines: Vec<u32>,
    /// Line of the statement currently being compiled
    cur_line: u32,
    functions: HashMap<String, FuncDef>,
    imported_natives: HashSet<String>,
    indian_fns: HashMap<String, IndianOp>,
    uid: usize,
    known_classes: HashSet<String>,
    class_parents: HashMap<String, String>,
    break_sites: Vec<Vec<usize>>,
    continue_stack: Vec<(Option<usize>, Vec<usize>)>,
    /// Nesting depth inside विधि bodies — used for tail-call detection (Phase 15)
    in_function: usize,
}

#[derive(Clone, Copy)]
enum IndianOp {
    AadhaarVerify,
    UpiSend,
    GstAdd,
    LakhParse,
    RupeeFormat,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            instructions: Vec::new(),
            lines: Vec::new(),
            cur_line: 0,
            functions: HashMap::new(),
            imported_natives: HashSet::new(),
            indian_fns: HashMap::new(),
            uid: 0,
            known_classes: HashSet::new(),
            class_parents: HashMap::new(),
            break_sites: Vec::new(),
            continue_stack: Vec::new(),
            in_function: 0,
        }
    }

    pub fn compile_program(stmts: &[Stmt]) -> CompiledProgram {
        let mut c = Compiler::new();
        // Pre-pass: collect class names and parent relationships
        for stmt in stmts {
            if let Stmt::Varg { name, parent, .. } = unwrap_located(stmt) {
                c.known_classes.insert(name.clone());
                if let Some(p) = parent {
                    c.class_parents.insert(name.clone(), p.clone());
                }
            }
        }
        for stmt in stmts {
            c.compile_stmt(stmt);
        }
        CompiledProgram {
            instructions: c.instructions,
            lines: c.lines,
            functions: c.functions,
            class_parents: c.class_parents,
        }
    }

    // ── Emit helpers ─────────────────────────────────────────────────────

    fn emit(&mut self, op: Opcode) -> usize {
        let idx = self.instructions.len();
        self.instructions.push(op);
        self.lines.push(self.cur_line);
        idx
    }

    fn here(&self) -> usize {
        self.instructions.len()
    }

    fn fresh_id(&mut self) -> usize {
        let id = self.uid;
        self.uid += 1;
        id
    }

    fn patch(&mut self, idx: usize, op: Opcode) {
        self.instructions[idx] = op;
    }

    // ── Statement compilation ─────────────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            // Source-position wrapper — record the line, compile the real stmt
            Stmt::Located { line, inner } => {
                self.cur_line = *line as u32;
                self.compile_stmt(inner);
            }

            // परीक्षण blocks emit nothing on a normal compile —
            // `lipi test` extracts and runs them separately (main.rs)
            Stmt::Parikshan { .. } => {}

            // नाम है expr
            Stmt::Assign { name, value, .. } => {
                self.compile_expr(value);
                self.emit(Opcode::StoreVar(name.clone()));
            }

            // बताओ expr
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(Opcode::Print);
            }

            // यदि cond: then [अन्यथा: else]
            Stmt::Yadi { condition, then, otherwise } => {
                self.compile_expr(condition);
                let jif = self.emit(Opcode::JumpIfFalse(0)); // placeholder

                for s in then { self.compile_stmt(s); }

                if let Some(els) = otherwise {
                    let jmp = self.emit(Opcode::Jump(0));
                    let else_start = self.here();
                    self.patch(jif, Opcode::JumpIfFalse(else_start));
                    for s in els { self.compile_stmt(s); }
                    let end = self.here();
                    self.patch(jmp, Opcode::Jump(end));
                } else {
                    self.patch(jif, Opcode::JumpIfFalse(self.here()));
                }
            }

            // N बार करो: body  — countdown loop
            Stmt::BarKaro { count, body } => {
                let id = self.fresh_id();
                let cnt_var = format!("__lc{id}__");

                self.compile_expr(count);
                self.emit(Opcode::StoreVar(cnt_var.clone()));

                let loop_start = self.here();
                self.emit(Opcode::LoadVar(cnt_var.clone()));
                self.emit(Opcode::Push(LvmValue::Number(0.0)));
                self.emit(Opcode::Gt);
                let jif = self.emit(Opcode::JumpIfFalse(0));

                self.break_sites.push(vec![]);
                self.continue_stack.push((None, vec![])); // decrement addr TBD

                for s in body { self.compile_stmt(s); }

                // Decrement — अगला (continue) jumps here
                let decrement_addr = self.here();
                let (_, cont_sites) = self.continue_stack.pop().unwrap();
                for site in cont_sites { self.patch(site, Opcode::Jump(decrement_addr)); }

                self.emit(Opcode::LoadVar(cnt_var.clone()));
                self.emit(Opcode::Push(LvmValue::Number(1.0)));
                self.emit(Opcode::Sub);
                self.emit(Opcode::StoreVar(cnt_var));
                self.emit(Opcode::Jump(loop_start));

                let end = self.here();
                self.patch(jif, Opcode::JumpIfFalse(end));
                for site in self.break_sites.pop().unwrap() {
                    self.patch(site, Opcode::Jump(end));
                }
            }

            // var के लिए iter में: body — iterable for-loop (Phase 11)
            // Works with: Number (0..N range), List (elements), Str (characters)
            Stmt::KeeLiye { var, iter, body } => {
                let id = self.fresh_id();
                let val_var = format!("__kl{id}_val__"); // the iterable
                let lim_var = format!("__kl{id}_lim__"); // its length
                let idx_var = format!("__kl{id}_idx__"); // current index

                // Store iterable, then get its length
                self.compile_expr(iter);
                self.emit(Opcode::StoreVar(val_var.clone()));

                self.emit(Opcode::LoadVar(val_var.clone()));
                self.emit(Opcode::GetIterLen);
                self.emit(Opcode::StoreVar(lim_var.clone()));

                self.emit(Opcode::Push(LvmValue::Number(0.0)));
                self.emit(Opcode::StoreVar(idx_var.clone()));

                let loop_start = self.here();
                self.emit(Opcode::LoadVar(idx_var.clone()));
                self.emit(Opcode::LoadVar(lim_var.clone()));
                self.emit(Opcode::Lt);
                let jif = self.emit(Opcode::JumpIfFalse(0));

                // Get the current element in place — no container clone per
                // iteration (was LoadVar+LoadVar+GetIterItem before Phase 17 perf)
                self.emit(Opcode::IterNext(val_var.clone(), idx_var.clone()));
                self.emit(Opcode::StoreVar(var.clone()));

                self.break_sites.push(vec![]);
                self.continue_stack.push((None, vec![])); // increment addr TBD

                for s in body { self.compile_stmt(s); }

                // Increment — अगला (continue) jumps here
                let increment_addr = self.here();
                let (_, cont_sites) = self.continue_stack.pop().unwrap();
                for site in cont_sites { self.patch(site, Opcode::Jump(increment_addr)); }

                self.emit(Opcode::LoadVar(idx_var.clone()));
                self.emit(Opcode::Push(LvmValue::Number(1.0)));
                self.emit(Opcode::Add);
                self.emit(Opcode::StoreVar(idx_var));
                self.emit(Opcode::Jump(loop_start));

                let end = self.here();
                self.patch(jif, Opcode::JumpIfFalse(end));
                for site in self.break_sites.pop().unwrap() {
                    self.patch(site, Opcode::Jump(end));
                }
            }

            // जब तक cond: body  — while loop (Phase 7)
            Stmt::JabTak { condition, body } => {
                let loop_start = self.here();
                self.break_sites.push(vec![]);
                self.continue_stack.push((Some(loop_start), vec![]));

                self.compile_expr(condition);
                let jif = self.emit(Opcode::JumpIfFalse(0));

                for s in body { self.compile_stmt(s); }

                // Patch continue sites (all point to loop_start)
                let (_, cont_sites) = self.continue_stack.pop().unwrap();
                for site in cont_sites { self.patch(site, Opcode::Jump(loop_start)); }

                self.emit(Opcode::Jump(loop_start));
                let end = self.here();
                self.patch(jif, Opcode::JumpIfFalse(end));
                for site in self.break_sites.pop().unwrap() {
                    self.patch(site, Opcode::Jump(end));
                }
            }

            // बंद करो  — break out of innermost loop
            Stmt::BandKaro => {
                let site = self.emit(Opcode::Jump(0));
                if let Some(sites) = self.break_sites.last_mut() {
                    sites.push(site);
                }
            }

            // अगला  — continue to next iteration of innermost loop
            Stmt::Agla => {
                // Copy the addr out (releases the borrow) before calling emit
                let cont_addr = self.continue_stack.last().map(|(a, _)| *a);
                match cont_addr {
                    Some(Some(addr)) => { self.emit(Opcode::Jump(addr)); }
                    Some(None) => {
                        let site = self.emit(Opcode::Jump(0));
                        if let Some((_, sites)) = self.continue_stack.last_mut() {
                            sites.push(site);
                        }
                    }
                    None => {} // अगला outside any loop — no-op
                }
            }

            // लिखो expr  — print without newline
            Stmt::Likho(expr) => {
                self.compile_expr(expr);
                self.emit(Opcode::PrintInline);
            }

            // विधि name(params[, *vararg]): body
            Stmt::Vidhi { name, params, body, vararg, decorators, .. } => {
                // Decorated functions register under a hidden name; the visible
                // name becomes a variable holding the decorated closure, so calls
                // resolve through the existing closure-variable path (Phase 17).
                let reg_name = if decorators.is_empty() {
                    name.clone()
                } else {
                    format!("__deco_{name}__")
                };

                // Jump over the function body so top-level execution skips it
                let jmp = self.emit(Opcode::Jump(0));
                let start_ip = self.here();

                self.functions.insert(
                    reg_name.clone(),
                    FuncDef {
                        params: params.iter().map(|p| p.name.clone()).collect(),
                        start_ip,
                        vararg: vararg.clone(),
                        defaults: params.iter().map(|p| p.default.as_ref().map(default_to_lvm)).collect(),
                    },
                );

                self.in_function += 1;
                for s in body { self.compile_stmt(s); }
                self.in_function -= 1;
                // Implicit Nil return for functions that fall through
                self.emit(Opcode::Push(LvmValue::Nil));
                self.emit(Opcode::Return);

                self.patch(jmp, Opcode::Jump(self.here()));

                // Apply decorators innermost-first: @अ @ब विधि f → f है अ(ब(f))
                if !decorators.is_empty() {
                    // LoadVar on a function name pushes a Value::Closure reference
                    self.emit(Opcode::LoadVar(reg_name));
                    for deco in decorators.iter().rev() {
                        match deco {
                            Expr::Ident(dname) => {
                                self.emit(Opcode::Call(dname.clone(), 1));
                            }
                            // Factory form @कारखाना(आर्ग): evaluate the factory to
                            // a closure, park it in a temp var, then call the temp
                            // with the function value already on the stack.
                            _ => {
                                let tmp = format!("__deco_tmp{}__", self.fresh_id());
                                self.compile_expr(deco);
                                self.emit(Opcode::StoreVar(tmp.clone()));
                                self.emit(Opcode::Call(tmp, 1));
                            }
                        }
                    }
                    self.emit(Opcode::StoreVar(name.clone()));
                }
            }

            // फल expr — explicit return; use TailCall when possible (Phase 15 TCO)
            Stmt::Fal(expr) => {
                // Tail-call optimization: only for calls to user-defined functions
                // (not native functions, not class constructors, not closures).
                if self.in_function > 0 {
                    if let Expr::Call { name, args } = expr {
                        if self.functions.contains_key(name) {
                            for a in args { self.compile_expr(a); }
                            self.emit(Opcode::TailCall(name.clone(), args.len()));
                            return;
                        }
                    }
                }
                self.compile_expr(expr);
                self.emit(Opcode::Return);
            }

            // bare expression (function call as statement — discard result)
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr);
                self.emit(Opcode::Pop);
            }

            // अ, ब है 1, 2  — tuple unpacking (Phase 17)
            Stmt::MultiAssign { names, values } => {
                if values.len() == names.len() {
                    // Pairwise: evaluate ALL values first, then store in reverse —
                    // makes swap (अ, ब है ब, अ) work
                    for v in values { self.compile_expr(v); }
                    for n in names.iter().rev() { self.emit(Opcode::StoreVar(n.clone())); }
                } else {
                    // Single RHS: must be a List at runtime, unpacked by position
                    self.compile_expr(&values[0]);
                    self.emit(Opcode::UnpackList(names.len()));
                    for n in names { self.emit(Opcode::StoreVar(n.clone())); }
                }
            }

            // name[idx] है val  — index assignment
            Stmt::IndexAssign { obj, idx, val } => {
                self.emit(Opcode::LoadVar(obj.clone()));
                self.compile_expr(idx);
                self.compile_expr(val);
                self.emit(Opcode::SetIndex);
                self.emit(Opcode::StoreVar(obj.clone()));
            }

            // कोशिश: body + पकड़ो clauses — typed exceptions (Phase 17A).
            // The handler address is a dispatch chain: each typed clause does
            // Dup + MatchErrClass + JumpIfFalse(next); a catch-all clause binds
            // unconditionally. If no clause matched, the error is rethrown.
            Stmt::TryCatch { body, clauses } => {
                let try_start = self.emit(Opcode::TryStart(0)); // patch later
                for s in body { self.compile_stmt(s); }
                self.emit(Opcode::TryEnd);
                let jmp = self.emit(Opcode::Jump(0)); // skip handler
                let handler_addr = self.here();
                self.patch(try_start, Opcode::TryStart(handler_addr));

                // Error value is on the stack at handler_addr.
                let mut end_jumps = vec![jmp];
                let mut has_catch_all = false;
                for clause in clauses {
                    // Resolve single-ident ambiguity (पकड़ो X:): if X is a known
                    // class it's a typed clause binding त्रुटि; otherwise it's a
                    // catch-all binding X (pre-17A behavior).
                    let (class, var): (Option<&str>, &str) = match &clause.class {
                        Some(c) => (Some(c.as_str()), clause.var.as_str()),
                        None if clause.var != "त्रुटि"
                            && self.known_classes.contains(&clause.var) =>
                            (Some(clause.var.as_str()), "त्रुटि"),
                        None => (None, clause.var.as_str()),
                    };
                    match class {
                        None => {
                            // Catch-all: bind error value and run handler
                            self.emit(Opcode::StoreVar(var.to_string()));
                            for s in &clause.body { self.compile_stmt(s); }
                            end_jumps.push(self.emit(Opcode::Jump(0)));
                            has_catch_all = true;
                            break; // any later clauses are unreachable
                        }
                        Some(cls) => {
                            self.emit(Opcode::Dup);
                            self.emit(Opcode::MatchErrClass(cls.to_string()));
                            let jf = self.emit(Opcode::JumpIfFalse(0));
                            self.emit(Opcode::StoreVar(var.to_string()));
                            for s in &clause.body { self.compile_stmt(s); }
                            end_jumps.push(self.emit(Opcode::Jump(0)));
                            self.patch(jf, Opcode::JumpIfFalse(self.here()));
                        }
                    }
                }
                if !has_catch_all {
                    // No clause matched — rethrow the error value outward
                    self.emit(Opcode::Throw);
                }
                let end = self.here();
                for j in end_jumps { self.patch(j, Opcode::Jump(end)); }
            }

            // फेंको expr — throw an error value (Phase 17A)
            Stmt::Phenko(expr) => {
                self.compile_expr(expr);
                self.emit(Opcode::Throw);
            }

            // आयात "file.swami"  — multi-file import
            Stmt::AayatFile(path) => {
                self.emit(Opcode::ImportFile(path.clone()));
            }

            // वैश्विक नाम  — declare global variables (Phase 13)
            Stmt::Global(names) => {
                for name in names {
                    self.emit(Opcode::DeclareGlobal(name.clone()));
                }
            }

            // जाँचो expr [, msg]  — assert (Phase 16, Nyaya Pratijna)
            Stmt::Jancho { expr, message } => {
                self.compile_expr(expr);
                let msg = message.as_ref().and_then(|m| {
                    if let crate::ast::Expr::Str(s) = m { Some(s.clone()) } else { None }
                });
                self.emit(Opcode::Assert(msg));
            }

            // स्थिर name है expr  — immutable constant (Phase 16, Samkhya)
            Stmt::SthirDecl { name, value } => {
                self.compile_expr(value);
                self.emit(Opcode::DeclareConst(name.clone()));
            }

            // वर्ग name[(parent)]: [methods]  — class definition
            Stmt::Varg { name: class_name, methods, .. } => {
                for method_stmt in methods {
                    if let Stmt::Vidhi { name: method_name, params, body, vararg, .. } = unwrap_located(method_stmt) {
                        let jmp = self.emit(Opcode::Jump(0));
                        let start_ip = self.here();

                        // Params: यह is always first (self), then user params
                        let mut full_params = vec!["यह".to_string()];
                        full_params.extend(params.iter().map(|p| p.name.clone()));
                        // यह never has a default; user params may (Phase 17)
                        let mut defaults: Vec<Option<LvmValue>> = vec![None];
                        defaults.extend(params.iter().map(|p| p.default.as_ref().map(default_to_lvm)));

                        self.functions.insert(
                            format!("{}::{}", class_name, method_name),
                            FuncDef { params: full_params, start_ip, vararg: vararg.clone(), defaults },
                        );

                        self.in_function += 1;
                        for s in body { self.compile_stmt(s); }
                        self.in_function -= 1;

                        // बनाओ (constructor) returns यह implicitly
                        if method_name == "बनाओ" {
                            self.emit(Opcode::LoadVar("यह".into()));
                        } else {
                            self.emit(Opcode::Push(LvmValue::Nil));
                        }
                        self.emit(Opcode::Return);
                        self.patch(jmp, Opcode::Jump(self.here()));
                    }
                }
            }

            // obj.field है val  — attribute assignment
            Stmt::AttrAssign { obj, field, val } => {
                self.emit(Opcode::LoadVar(obj.clone()));
                self.compile_expr(val);
                self.emit(Opcode::SetAttr(field.clone()));
                self.emit(Opcode::StoreVar(obj.clone()));
            }

            // विकल्प Name: variants  — enum definition (Phase 15)
            Stmt::ViKalp { name, variants } => {
                let variant_defs: Vec<(String, usize)> = variants.iter()
                    .map(|v| (v.name.clone(), v.fields.len()))
                    .collect();
                self.emit(Opcode::DefineEnum(name.clone(), variant_defs));
            }

            // मिलाओ expr: arms  — pattern match (Phase 15)
            Stmt::Milao { subject, arms } => {
                self.compile_expr(subject); // enum value on stack

                let mut jump_ends: Vec<usize> = Vec::new();
                let mut last_jf: Option<usize> = None;

                for arm in arms {
                    // Patch previous JumpIfFalse to here
                    if let Some(jf) = last_jf.take() {
                        self.patch(jf, Opcode::JumpIfFalse(self.here()));
                    }
                    match &arm.pattern {
                        crate::ast::MilaoPattern::Wildcard => {
                            // No variant test — just pop enum and run body
                            self.emit(Opcode::Pop);
                            for s in &arm.body { self.compile_stmt(s); }
                            let je = self.emit(Opcode::Jump(0));
                            jump_ends.push(je);
                        }
                        crate::ast::MilaoPattern::Variant(vname, binds) => {
                            // Dup enum, test variant, conditional jump
                            self.emit(Opcode::Dup);
                            self.emit(Opcode::MatchVariant(vname.clone()));
                            let jf = self.emit(Opcode::JumpIfFalse(0));
                            last_jf = Some(jf);

                            if binds.is_empty() {
                                self.emit(Opcode::Pop); // discard enum copy
                            } else {
                                self.emit(Opcode::EnumUnpack(binds.clone()));
                            }
                            for s in &arm.body { self.compile_stmt(s); }
                            let je = self.emit(Opcode::Jump(0));
                            jump_ends.push(je);
                        }
                    }
                }

                // If last arm had no wildcard the JumpIfFalse still points to 0 — patch to Pop
                if let Some(jf) = last_jf.take() {
                    self.patch(jf, Opcode::JumpIfFalse(self.here()));
                }
                // Discard enum when no arm matched (no wildcard case)
                self.emit(Opcode::Pop);

                let end = self.here();
                for je in jump_ends { self.patch(je, Opcode::Jump(end)); }
            }

            // आयात भारत.मॉड्यूल
            Stmt::Aayat(module) => {
                match module.as_str() {
                    "भारत.पहचान" => {
                        self.indian_fns.insert("आधार_जाँचो".into(), IndianOp::AadhaarVerify);
                        self.imported_natives.insert("pan_जाँचो".into());
                        self.imported_natives.insert("ifsc_जाँचो".into());
                    }
                    "भारत.संख्या" => {
                        self.indian_fns.insert("लाख_में".into(), IndianOp::LakhParse);
                        self.indian_fns.insert("रुपये_में".into(), IndianOp::RupeeFormat);
                        self.indian_fns.insert("gst_जोड़ो".into(), IndianOp::GstAdd);
                        self.imported_natives.insert("करोड़_में".into());
                        self.imported_natives.insert("emi_निकालो".into());
                    }
                    "भारत.भुगतान" => {
                        self.indian_fns.insert("upi_भेजो".into(), IndianOp::UpiSend);
                        self.imported_natives.insert("upi_वैध_है".into());
                    }
                    "भारत.भाषा" => {
                        self.imported_natives.insert("devanagari_है".into());
                        self.imported_natives.insert("roman_में".into());
                        self.imported_natives.insert("शब्द_गिनो".into());
                    }
                    "भारत.json" => {
                        self.imported_natives.insert("json_पढ़ो".into());
                        self.imported_natives.insert("json_लिखो".into());
                    }
                    "भारत.समय" => {
                        for f in ["समय_अभी", "समय_बनाओ", "समय_विवरण", "समय_स्वरूप",
                                  "दिनांक_पार्स", "समय_जोड़ो", "दिन_अंतर", "अधिवर्ष", "माह_दिन"] {
                            self.imported_natives.insert(f.into());
                        }
                    }
                    "भारत.csv" => {
                        for f in ["csv_पढ़ो", "csv_शीर्षक_पढ़ो", "csv_लिखो"] {
                            self.imported_natives.insert(f.into());
                        }
                    }
                    "भारत.कूट" => {
                        for f in ["sha256", "md5", "base64_कूट", "base64_खोलो"] {
                            self.imported_natives.insert(f.into());
                        }
                    }
                    "भारत.http" => {
                        for f in ["http_पाओ", "http_भेजो"] {
                            self.imported_natives.insert(f.into());
                        }
                    }
                    "भारत.प्रतिमान" => {
                        for f in ["ढूंढो", "ढूंढो_स्थान", "ढूंढो_सब", "मेल_है",
                                  "समूह", "बदलो_सब", "विभाजित_सब"] {
                            self.imported_natives.insert(f.into());
                        }
                    }
                    _ => {}
                }
                self.emit(Opcode::Import(module.clone()));
            }
        }
    }

    // ── Expression compilation ────────────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => { self.emit(Opcode::Push(LvmValue::Number(*n))); }
            Expr::Str(s)    => { self.emit(Opcode::Push(LvmValue::Str(s.clone()))); }
            Expr::Bool(b)   => { self.emit(Opcode::Push(LvmValue::Bool(*b))); }
            Expr::Ident(n)  => { self.emit(Opcode::LoadVar(n.clone())); }

            Expr::Binary { left, op, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                self.emit(match op {
                    BinOp::Add    => Opcode::Add,
                    BinOp::Sub    => Opcode::Sub,
                    BinOp::Mul    => Opcode::Mul,
                    BinOp::Div      => Opcode::Div,
                    BinOp::FloorDiv => Opcode::FloorDiv,
                    BinOp::Mod      => Opcode::Mod,
                    BinOp::BitAnd => Opcode::BitAnd,
                    BinOp::BitOr  => Opcode::BitOr,
                    BinOp::BitXor => Opcode::BitXor,
                    BinOp::LShift => Opcode::LShift,
                    BinOp::RShift => Opcode::RShift,
                    BinOp::And    => Opcode::And,
                    BinOp::Or     => Opcode::Or,
                });
            }

            // ~expr  — bitwise NOT (Phase 12)
            Expr::BitNot(operand) => {
                self.compile_expr(operand);
                self.emit(Opcode::BitNot);
            }

            // नहीं expr  — logical NOT (Phase 13)
            Expr::Not(operand) => {
                self.compile_expr(operand);
                self.emit(Opcode::Not);
            }

            // यदि cond तो a अन्यथा b  — ternary (Phase 12)
            Expr::Ternary { condition, then_val, else_val } => {
                self.compile_expr(condition);
                let jif = self.emit(Opcode::JumpIfFalse(0));
                self.compile_expr(then_val);
                let jmp = self.emit(Opcode::Jump(0));
                let else_start = self.here();
                self.patch(jif, Opcode::JumpIfFalse(else_start));
                self.compile_expr(else_val);
                self.patch(jmp, Opcode::Jump(self.here()));
            }

            Expr::Compare { left, op, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                self.emit(match op {
                    CmpOp::Eq                   => Opcode::Eq,
                    CmpOp::NotEq                => Opcode::NotEq,
                    CmpOp::SeAdhik | CmpOp::Gt => Opcode::Gt,
                    CmpOp::SeKam   | CmpOp::Lt => Opcode::Lt,
                    CmpOp::GtEq                => Opcode::GtEq,
                    CmpOp::LtEq                => Opcode::LtEq,
                });
            }

            Expr::Call { name, args } => {
                if self.known_classes.contains(name) {
                    // Constructor call: push empty instance, then args, then call बनाओ
                    self.emit(Opcode::MakeInstance(name.clone()));
                    for arg in args { self.compile_expr(arg); }
                    // argc+1 because यह (the instance) is the first param
                    self.emit(Opcode::Call(format!("{}::बनाओ", name), args.len() + 1));
                } else {
                    for arg in args { self.compile_expr(arg); }
                    // Specialize to a first-class Indian opcode where possible
                    if let Some(&iop) = self.indian_fns.get(name) {
                        self.emit(match iop {
                            IndianOp::AadhaarVerify => Opcode::AadhaarVerify,
                            IndianOp::UpiSend       => Opcode::UpiSend,
                            IndianOp::GstAdd        => Opcode::GstAdd,
                            IndianOp::LakhParse     => Opcode::LakhParse,
                            IndianOp::RupeeFormat   => Opcode::RupeeFormat,
                        });
                    } else if self.imported_natives.contains(name) {
                        self.emit(Opcode::CallNative(name.clone(), args.len()));
                    } else {
                        // General call — VM resolves at runtime (user-defined or built-in)
                        self.emit(Opcode::Call(name.clone(), args.len()));
                    }
                }
            }

            // func(अ, नाम=मान) — call with keyword arguments (Phase 17)
            Expr::CallKw { name, args, kwargs } => {
                let kwnames: Vec<String> = kwargs.iter().map(|(n, _)| n.clone()).collect();
                if self.known_classes.contains(name) {
                    // Constructor: instance is the first positional arg (यह)
                    self.emit(Opcode::MakeInstance(name.clone()));
                    for arg in args { self.compile_expr(arg); }
                    for (_, v) in kwargs { self.compile_expr(v); }
                    self.emit(Opcode::CallKw(format!("{}::बनाओ", name), args.len() + 1, kwnames));
                } else {
                    for arg in args { self.compile_expr(arg); }
                    for (_, v) in kwargs { self.compile_expr(v); }
                    self.emit(Opcode::CallKw(name.clone(), args.len(), kwnames));
                }
            }

            // item में_है container — membership test (Phase 17)
            Expr::Membership { item, container, negated } => {
                self.compile_expr(item);
                self.compile_expr(container);
                self.emit(Opcode::Contains);
                if *negated { self.emit(Opcode::Not); }
            }

            Expr::MethodCall { object, method, args } => {
                self.compile_expr(object);
                for arg in args { self.compile_expr(arg); }
                self.emit(Opcode::MethodCall(method.clone(), args.len()));
            }

            // [e1, e2, ...]  — सूची literal
            Expr::List(elems) => {
                for e in elems { self.compile_expr(e); }
                self.emit(Opcode::MakeList(elems.len()));
            }

            // [*अ, 99, *ब]  — सूची literal with spread elements (Phase 17)
            Expr::ListWithSpread(elems) => {
                for (_, e) in elems { self.compile_expr(e); }
                let flags: Vec<bool> = elems.iter().map(|(s, _)| *s).collect();
                self.emit(Opcode::MakeListSp(flags));
            }

            // {"k": v, ...}  — कोश literal
            Expr::Dict(pairs) => {
                for (k, v) in pairs {
                    self.compile_expr(k);
                    self.compile_expr(v);
                }
                self.emit(Opcode::MakeDict(pairs.len()));
            }

            // obj[idx]  — index access
            Expr::Index { obj, idx } => {
                self.compile_expr(obj);
                self.compile_expr(idx);
                self.emit(Opcode::GetIndex);
            }

            // obj[start:end:step]  — slice (Phase 17); omitted parts push Nil
            Expr::Slice { obj, start, end, step } => {
                self.compile_expr(obj);
                for part in [start, end, step] {
                    match part {
                        Some(e) => self.compile_expr(e),
                        None    => { self.emit(Opcode::Push(LvmValue::Nil)); }
                    }
                }
                self.emit(Opcode::Slice);
            }

            // obj.field  — attribute access
            Expr::Attr { obj, field } => {
                self.compile_expr(obj);
                self.emit(Opcode::GetAttr(field.clone()));
            }

            // लाम्डा(params): body  — anonymous function (Phase 10)
            Expr::Lambda { params, body } => {
                let id = self.fresh_id();
                let lam_name = format!("__lam_{id}__");

                // Compile the lambda body as an inline function (skipped by Jump)
                let jmp = self.emit(Opcode::Jump(0));
                let start_ip = self.here();
                self.functions.insert(lam_name.clone(), FuncDef { params: params.clone(), start_ip, vararg: None, defaults: vec![None; params.len()] });
                for s in body { self.compile_stmt(s); }
                self.emit(Opcode::Push(LvmValue::Nil));
                self.emit(Opcode::Return);
                self.patch(jmp, Opcode::Jump(self.here()));

                // Push a closure reference onto the stack
                self.emit(Opcode::MakeClosure(lam_name));
            }
        }
    }
}

/// Convert a parser-validated constant default expression to an LvmValue (Phase 17).
/// The parser only allows Number / Str / Bool (with optional leading minus already folded).
fn default_to_lvm(e: &Expr) -> LvmValue {
    match e {
        Expr::Number(n) => LvmValue::Number(*n),
        Expr::Str(s)    => LvmValue::Str(s.clone()),
        Expr::Bool(b)   => LvmValue::Bool(*b),
        _               => LvmValue::Nil,
    }
}
