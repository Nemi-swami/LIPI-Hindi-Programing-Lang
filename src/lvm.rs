//! LIPI Virtual Machine — executes LVM bytecode.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::cell::RefCell;
use crate::interpreter::Value;

// Pre-loaded stdin buffer — used by WASM/web mode so पढ़ो() works without real stdin.
thread_local! {
    static STDIN_BUF: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
}

/// Populate the stdin buffer. Call this before run() in WASM mode.
pub fn set_stdin_buffer(lines: Vec<String>) {
    STDIN_BUF.with(|b| {
        let mut buf = b.borrow_mut();
        buf.clear();
        for line in lines { buf.push_back(line); }
    });
}
use crate::karaka::KarakaEnv;
use crate::opcode::{CompiledProgram, FuncDef, LvmValue, Opcode};

// ── Memory guards (Phase 17C) ───────────────────────────────────────────────────
// Generous ceilings so a runaway program halts with a catchable Hindi error
// instead of OOM-killing the process. Normal programs stay far below these.

/// Maximum number of elements any single list may hold.
const MAX_LIST_LEN: usize = 50_000_000;
/// Maximum operand-stack depth — guards against runaway push loops / deep nesting.
const MAX_STACK_DEPTH: usize = 10_000_000;

/// Error if a list would exceed `MAX_LIST_LEN` elements.
fn check_list_len(len: usize) -> Result<(), String> {
    if len > MAX_LIST_LEN {
        Err(format!(
            "स्मृति सीमा पार: सूची की लम्बाई {} से अधिक नहीं हो सकती",
            MAX_LIST_LEN
        ))
    } else {
        Ok(())
    }
}

// ── Call frame ────────────────────────────────────────────────────────────────

struct Frame {
    return_addr: usize,
    locals: HashMap<String, Value>,
    global_names: std::collections::HashSet<String>,
    /// Stack depth when this frame was entered — used by TailCall to trim the stack (Phase 15)
    base_stack_depth: usize,
    /// Name the function was called by — shown in stack traces (Phase 17 diagnostics)
    func_name: String,
}

struct TryFrame {
    handler_ip: usize,
    stack_depth: usize,
    frame_depth: usize,
}

struct GenState {
    ip: usize,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    done: bool,
}

enum Resumed {
    Yielded(Value),
    Done(Value),
}

// ── VM ────────────────────────────────────────────────────────────────────────

pub struct LVM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_frames: Vec<Frame>,
    functions: HashMap<String, FuncDef>,
    class_parents: HashMap<String, String>,
    native_fns: HashMap<String, crate::bharat_stdlib::NativeFn>,
    try_stack: Vec<TryFrame>,
    /// Typed error value in flight (Phase 17A) — set by Throw, consumed by the
    /// try unwinder so handlers receive the thrown Instance, not just a message.
    thrown: Option<Value>,
    #[allow(dead_code)]
    karaka_env: KarakaEnv,
    /// Names declared स्थिर — any StoreVar to these raises an error
    constants: std::collections::HashSet<String>,
    /// Persistent caches for स्मरण (memoize) closures, keyed by memo id.
    /// Lives on the VM so cache survives across calls (closure captures only the id).
    memo_caches: HashMap<u64, HashMap<String, Value>>,
    memo_next: u64,
    generators: HashMap<u64, GenState>,
    gen_next: u64,
    yielded: bool,
    /// When `capture` is true, Print writes here instead of stdout.
    pub output: String,
    capture: bool,
}

impl LVM {
    pub fn new() -> Self {
        let mut vm = LVM {
            stack: Vec::new(),
            globals: HashMap::new(),
            call_frames: Vec::new(),
            functions: HashMap::new(),
            class_parents: HashMap::new(),
            native_fns: HashMap::new(),
            try_stack: Vec::new(),
            thrown: None,
            karaka_env: KarakaEnv::new(),
            constants: std::collections::HashSet::new(),
            memo_caches: HashMap::new(),
            memo_next: 0,
            generators: HashMap::new(),
            gen_next: 0,
            yielded: false,
            output: String::new(),
            capture: false,
        };
        // Pre-register built-in functions always available without आयात
        vm.native_fns.insert("लम्बाई".into(), builtin_lambai);
        vm.native_fns.insert("पूर्णांक".into(), builtin_purnankin);
        vm.native_fns.insert("__padho__".into(), builtin_padho);
        vm.native_fns.insert("वाक्य".into(), builtin_vakya);
        vm.native_fns.insert("यादृच्छिक".into(), builtin_yadrchik);
        vm.native_fns.insert("यूआईडी".into(), builtin_uuid);
        vm.native_fns.insert("बीज_सेट".into(), builtin_beej_set);
        vm.native_fns.insert("युग्म".into(), builtin_zip);
        vm.native_fns.insert("गणना".into(), builtin_ganana);
        vm.native_fns.insert("श्रृंखला".into(), builtin_chain);
        vm.native_fns.insert("गिनती_कोश".into(), builtin_counter);
        vm.native_fns.insert("कार्तीय".into(), builtin_product);
        vm.native_fns.insert("सर्व_संयोजन".into(), builtin_combinations);
        // Queue/Deque helpers (Phase 17) — copy-on-write, return NEW lists
        vm.native_fns.insert("अग्र_जोड़ो".into(), builtin_agra_jodo);
        vm.native_fns.insert("अग्र".into(), builtin_agra);
        vm.native_fns.insert("पश्च".into(), builtin_pashcha);
        vm.native_fns.insert("अग्र_हटाओ".into(), builtin_agra_hatao);
        vm.native_fns.insert("पश्च_हटाओ".into(), builtin_pashcha_hatao);
        // OrderedDict (Phase 17) — insertion-ordered map as list-of-pairs
        vm.native_fns.insert("क्रमित_कोश".into(), builtin_kramit_kosh);
        vm.native_fns.insert("क्रमित_रखो".into(), builtin_kramit_rakho);
        vm.native_fns.insert("क्रमित_पाओ".into(), builtin_kramit_pao);
        vm.native_fns.insert("क्रमित_कुंजियाँ".into(), builtin_kramit_kunjiyan);
        vm.native_fns.insert("क्रमित_मान".into(), builtin_kramit_maan);
        vm.native_fns.insert("निर्गम".into(), builtin_nirgam);
        // Math
        vm.native_fns.insert("निरपेक्ष".into(), builtin_nirapeksh);
        vm.native_fns.insert("घात".into(), builtin_ghaat);
        vm.native_fns.insert("वर्गमूल".into(), builtin_vargmool);
        vm.native_fns.insert("गोल".into(), builtin_gol);
        // File I/O
        vm.native_fns.insert("संचिका_सामग्री".into(), builtin_sanchika_samagri);
        vm.native_fns.insert("संचिका_लिखो".into(), builtin_sanchika_likho);
        vm.native_fns.insert("संचिका_है".into(), builtin_sanchika_hai);
        // File-system + OS/environment (Phase 17)
        vm.native_fns.insert("फोल्डर_सूची".into(), builtin_folder_suchi);
        vm.native_fns.insert("फोल्डर_बनाओ".into(), builtin_folder_banao);
        vm.native_fns.insert("फाइल_हटाओ".into(), builtin_file_hatao);
        vm.native_fns.insert("फाइल_कॉपी".into(), builtin_file_copy);
        vm.native_fns.insert("पथ_जोड़ो".into(), builtin_path_jodo);
        vm.native_fns.insert("पर्यावरण".into(), builtin_paryavaran);
        vm.native_fns.insert("वर्तमान_फोल्डर".into(), builtin_vartamaan_folder);
        vm.native_fns.insert("तर्क".into(), builtin_tark);
        // Async sleep marker (Phase 18) — await सोओ(ms) suspends to the scheduler
        vm.native_fns.insert("सोओ".into(), builtin_so);
        // Unicode NFC normalization for Devanagari (Phase 17)
        vm.native_fns.insert("सामान्यीकृत".into(), builtin_samanyikrit);
        // Integer predicate (Phase 17) — LIPI numbers are f64; पूर्ण_है tests
        // whether a value is a whole number (exact for |n| <= 2^53)
        vm.native_fns.insert("पूर्ण_है".into(), builtin_purna_hai);
        // Type inspection
        vm.native_fns.insert("प्रकार".into(), builtin_prakar);
        // String formatting
        vm.native_fns.insert("स्वरूप".into(), builtin_swaroop);
        // Ancient Hindu constants (Brahmagupta, Āryabhaṭa)
        vm.globals.insert("पाई".into(), Value::Number(std::f64::consts::PI));
        vm.globals.insert("अनंत".into(), Value::Number(f64::INFINITY));
        vm.globals.insert("ऋण_अनंत".into(), Value::Number(f64::NEG_INFINITY));
        // शून्य = Nil — LIPI's None; functions like ढूंढो return it on no-match
        vm.globals.insert("शून्य".into(), Value::Nil);
        // Vedic large number names (Vedic Samhitas, Shatapatha Brahmana)
        vm.globals.insert("अरब".into(), Value::Number(1_000_000_000.0));
        vm.globals.insert("खरब".into(), Value::Number(10_000_000_000.0));
        vm.globals.insert("नील".into(), Value::Number(100_000_000_000.0));
        vm.globals.insert("शंख".into(), Value::Number(1_000_000_000_000.0));
        vm.globals.insert("पद्म".into(), Value::Number(100_000_000_000_000.0));
        // Aryabhata's constants (Aryabhatiya 499 CE)
        // "Add 4 to 100, multiply by 8, add 62000 → 62832; this is approx circumference for diameter 20000"
        vm.globals.insert("आर्यभट_पाई".into(), Value::Number(3.1416));
        // Aryabhata divided the circle into 21600 minutes; 360°/96 steps = 3.75° = π/48 radians
        vm.globals.insert("आर्यभट_कोण".into(), Value::Number(std::f64::consts::PI / 48.0));
        // Aryabhata's sine table has 24 entries (one per 3.75° step from 0° to 90°)
        vm.globals.insert("आर्यभट_ज्या_गणना".into(), Value::Number(24.0));
        // Astronomical constants (Vedanga Jyotisha ~1200 BCE, Aryabhatiya 499 CE)
        vm.globals.insert("नक्षत्र_संख्या".into(), Value::Number(27.0));
        vm.globals.insert("तिथि_संख्या".into(), Value::Number(30.0));
        // One Mahayuga = 4,320,000 years (Aryabhatiya, Surya Siddhanta)
        vm.globals.insert("युग_वर्ष".into(), Value::Number(4_320_000.0));
        // Brahmagupta's zero (Brahmasphutasiddhanta 628 CE) — first rigorous definition
        vm.globals.insert("ब्रह्मगुप्त_शून्य".into(), Value::Number(0.0));
        init_rand();
        vm
    }

    /// Returns an LVM that captures all Print output into `self.output` instead of stdout.
    /// Used by the WASM playground.
    pub fn new_capturing() -> Self {
        let mut vm = Self::new();
        vm.capture = true;
        vm
    }

    /// Push a call frame, guarding against runaway recursion (Phase 17C).
    /// Python's default limit is 1000; LIPI frames are cheap (flat loop, no
    /// Rust-stack growth except closure calls), so 10000 is safe and generous.
    fn push_frame(&mut self, frame: Frame) -> Result<(), String> {
        const MAX_CALL_DEPTH: usize = 10_000;
        if self.call_frames.len() >= MAX_CALL_DEPTH {
            return Err(format!(
                "अधिकतम पुनरावर्तन गहराई पार ({MAX_CALL_DEPTH}) — विधि '{}'",
                frame.func_name
            ));
        }
        self.call_frames.push(frame);
        Ok(())
    }

    /// Find a method `Class::method`, walking the inheritance chain (Phase 17).
    fn lookup_method(&self, class: &str, method: &str) -> Option<crate::opcode::FuncDef> {
        let mut search = Some(class.to_string());
        while let Some(cls) = search {
            let key = format!("{}::{}", cls, method);
            if let Some(f) = self.functions.get(&key) { return Some(f.clone()); }
            search = self.class_parents.get(&cls).cloned();
        }
        None
    }

    /// Operator overloading (Phase 17): if `a` is an Instance whose class
    /// defines the dunder `method` (e.g. `__जोड़ो__` for +), set up a call
    /// frame `method(यह=a, अन्य=b)` and return Ok(true) — the method's Return
    /// pushes the result where the operator's result would go. Returns Ok(false)
    /// if `a` is not an instance or has no such method (caller falls back).
    fn try_instance_binop(&mut self, a: Value, b: Value, method: &str, ip: &mut usize) -> Result<bool, String> {
        if let Value::Instance { ref class, .. } = a {
            if let Some(func) = self.lookup_method(class, method) {
                let class_name = class.clone();
                let mut locals = HashMap::new();
                for (param, val) in func.params.iter().zip([a.clone(), b]) {
                    locals.insert(param.clone(), val);
                }
                fill_defaults(&mut locals, &func, func.params.len().min(2));
                self.push_frame(Frame {
                    return_addr: *ip,
                    locals,
                    global_names: std::collections::HashSet::new(),
                    base_stack_depth: self.stack.len(),
                    func_name: format!("{}.{}", class_name, method),
                })?;
                *ip = func.start_ip;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Look up a variable by reference — frame locals first, then globals.
    /// Mirrors LoadVar resolution without cloning the value.
    fn var_ref(&self, name: &str) -> Option<&Value> {
        self.call_frames.last()
            .and_then(|f| f.locals.get(name))
            .or_else(|| self.globals.get(name))
    }

    fn set_var(&mut self, name: &str, val: Value) -> Result<(), String> {
        if self.constants.contains(name) {
            return Err(format!("स्थिर '{}' को बदला नहीं जा सकता (Samkhya: नित्यम्)", name));
        }
        if self.call_frames.is_empty() {
            self.globals.insert(name.to_string(), val);
            return Ok(());
        }
        let is_global = self.call_frames.last().map_or(false, |f| f.global_names.contains(name));
        if is_global {
            self.globals.insert(name.to_string(), val);
            return Ok(());
        }
        let in_locals = self.call_frames.last().map_or(false, |f| f.locals.contains_key(name));
        if in_locals || !self.globals.contains_key(name) {
            if let Some(f) = self.call_frames.last_mut() { f.locals.insert(name.to_string(), val); }
        } else {
            self.globals.insert(name.to_string(), val);
        }
        Ok(())
    }

    fn make_generator(&mut self, start_ip: usize, locals: HashMap<String, Value>, name: String) -> Value {
        let id = self.gen_next;
        self.gen_next += 1;
        let frame = Frame {
            return_addr: usize::MAX,
            locals,
            global_names: std::collections::HashSet::new(),
            base_stack_depth: 0,
            func_name: name,
        };
        self.generators.insert(id, GenState { ip: start_ip, stack: Vec::new(), frames: vec![frame], done: false });
        Value::Generator(id)
    }

    fn resume_generator(&mut self, id: u64, instructions: &[Opcode]) -> Result<Option<Value>, String> {
        match self.drive(id, None, instructions)? {
            Resumed::Yielded(v) => Ok(Some(v)),
            Resumed::Done(_) => Ok(None),
        }
    }

    fn drive(&mut self, id: u64, send: Option<Value>, instructions: &[Opcode]) -> Result<Resumed, String> {
        let mut gs = match self.generators.remove(&id) {
            Some(g) => g,
            None => return Err("जनित्र अमान्य".into()),
        };
        if gs.done {
            self.generators.insert(id, gs);
            return Ok(Resumed::Done(Value::Nil));
        }
        let base_stack = self.stack.len();
        let base_frames = self.call_frames.len();
        self.stack.append(&mut gs.stack);
        self.call_frames.append(&mut gs.frames);
        if let Some(v) = send { self.stack.push(v); }

        let mut ip = gs.ip;
        let outcome;
        loop {
            if self.call_frames.len() <= base_frames || ip >= instructions.len() {
                gs.done = true;
                let ret = if self.stack.len() > base_stack { self.stack.pop().unwrap_or(Value::Nil) } else { Value::Nil };
                outcome = Resumed::Done(ret);
                break;
            }
            let op = &instructions[ip];
            ip += 1;
            match self.exec_op(op, &mut ip, instructions) {
                Ok(_) => {}
                Err(e) => {
                    self.stack.truncate(base_stack);
                    self.call_frames.truncate(base_frames);
                    gs.done = true;
                    self.generators.insert(id, gs);
                    return Err(e);
                }
            }
            if self.yielded {
                self.yielded = false;
                outcome = Resumed::Yielded(pop(&mut self.stack)?);
                break;
            }
        }
        if gs.done {
            self.stack.truncate(base_stack);
            self.call_frames.truncate(base_frames);
        } else {
            gs.ip = ip;
            gs.stack = self.stack.split_off(base_stack);
            gs.frames = self.call_frames.split_off(base_frames);
        }
        self.generators.insert(id, gs);
        Ok(outcome)
    }

    fn run_event_loop(&mut self, roots: Vec<u64>, instructions: &[Opcode]) -> Result<Vec<Value>, String> {
        use std::collections::VecDeque;
        let mut ready: VecDeque<(u64, Option<Value>)> = VecDeque::new();
        let mut waiters: HashMap<u64, u64> = HashMap::new();
        let mut results: HashMap<u64, Value> = HashMap::new();
        for r in &roots { ready.push_back((*r, None)); }

        #[cfg(not(target_arch = "wasm32"))]
        let mut sleeping: Vec<(std::time::Instant, u64)> = Vec::new();
        #[cfg(target_arch = "wasm32")]
        let mut sleeping: VecDeque<u64> = VecDeque::new();

        loop {
            if ready.is_empty() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if sleeping.is_empty() { break; }
                    sleeping.sort_by_key(|(t, _)| *t);
                    let (wake, tid) = sleeping.remove(0);
                    let now = std::time::Instant::now();
                    if wake > now { std::thread::sleep(wake - now); }
                    ready.push_back((tid, Some(Value::Nil)));
                    continue;
                }
                #[cfg(target_arch = "wasm32")]
                {
                    match sleeping.pop_front() {
                        Some(tid) => { ready.push_back((tid, Some(Value::Nil))); continue; }
                        None => break,
                    }
                }
            }
            let (tid, send) = ready.pop_front().unwrap();
            match self.drive(tid, send, instructions)? {
                Resumed::Done(val) => {
                    results.insert(tid, val.clone());
                    if let Some(awaiter) = waiters.remove(&tid) {
                        ready.push_back((awaiter, Some(val)));
                    }
                }
                Resumed::Yielded(y) => {
                    match y {
                        Value::Generator(child) => {
                            waiters.insert(child, tid);
                            ready.push_back((child, None));
                        }
                        Value::Dict(ref m) if m.contains_key("__नींद_मिलि__") => {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let ms = match m.get("__नींद_मिलि__") { Some(Value::Number(n)) => *n as u64, _ => 0 };
                                sleeping.push((std::time::Instant::now() + std::time::Duration::from_millis(ms), tid));
                            }
                            #[cfg(target_arch = "wasm32")]
                            sleeping.push_back(tid);
                        }
                        other => {
                            ready.push_back((tid, Some(other)));
                        }
                    }
                }
            }
        }
        Ok(roots.iter().map(|r| results.remove(r).unwrap_or(Value::Nil)).collect())
    }

    fn iter_step(&mut self, loop_var: &str, container_var: &str, idx_var: &str, instructions: &[Opcode]) -> Result<bool, String> {
        if let Some(Value::Generator(id)) = self.var_ref(container_var).cloned() {
            return match self.resume_generator(id, instructions)? {
                Some(v) => { self.set_var(loop_var, v)?; Ok(true) }
                None => Ok(false),
            };
        }
        let idx = match self.var_ref(idx_var) {
            Some(Value::Number(n)) => *n as usize,
            _ => return Err("IterStep: अनुक्रमणिका अमान्य".into()),
        };
        let item = match self.var_ref(container_var) {
            Some(Value::Number(n)) => { if idx >= *n as usize { return Ok(false); } Value::Number(idx as f64) }
            Some(Value::List(v)) => { if idx >= v.len() { return Ok(false); } v[idx].clone() }
            Some(Value::Str(s)) => {
                match s.chars().nth(idx) { Some(c) => Value::Str(c.to_string()), None => return Ok(false) }
            }
            Some(Value::Dict(m)) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                match keys.get(idx) { Some(k) => Value::Str((*k).clone()), None => return Ok(false) }
            }
            Some(other) => return Err(format!("'{}' पर 'के लिए' नहीं चल सकता", other)),
            None => return Err(format!("'{}' परिभाषित नहीं है", container_var)),
        };
        self.set_var(loop_var, item)?;
        self.set_var(idx_var, Value::Number((idx + 1) as f64))?;
        Ok(true)
    }

    /// Walk the inheritance chain from `class` upward (inclusive) looking for
    /// `target`. वर्ग X(त्रुटि): records त्रुटि as a parent like any other class.
    fn err_chain_contains(&self, class: &str, target: &str) -> bool {
        let mut cur = class;
        loop {
            if cur == target { return true; }
            match self.class_parents.get(cur) {
                Some(parent) => cur = parent.as_str(),
                None => return false,
            }
        }
    }

    pub fn run(&mut self, program: &CompiledProgram) -> Result<(), String> {
        self.functions = program.functions.clone();
        self.class_parents = program.class_parents.clone();
        let instructions = program.instructions.clone();
        let mut ip = 0usize;

        'vm: while ip < instructions.len() {
            let op = &instructions[ip];
            ip += 1; // pre-advance; jumps/calls overwrite ip directly

            match self.exec_op(op, &mut ip, &instructions) {
                Ok(true)  => break 'vm,
                Ok(false) => {}
                Err(e) => {
                    if let Some(tf) = self.try_stack.pop() {
                        self.stack.truncate(tf.stack_depth);
                        self.call_frames.truncate(tf.frame_depth);
                        // Typed throw (Phase 17A) delivers the Instance itself;
                        // plain Err(String) keeps delivering the message Str.
                        let errval = self.thrown.take().unwrap_or(Value::Str(e));
                        self.stack.push(errval);
                        ip = tf.handler_ip;
                    } else {
                        self.thrown = None;
                        // Uncaught: attach source line + call-stack trace
                        // (Phase 17 diagnostics). Caught errors above stay
                        // clean so पकड़ो handlers see the plain message.
                        return Err(self.format_uncaught(e, ip, &program.lines));
                    }
                }
            }
        }
        Ok(())
    }

    /// Run with flame-graph sampling: each executed instruction is attributed to
    /// the current call-stack path (folded-stack format). Returns path → samples.
    pub fn run_flame(&mut self, program: &CompiledProgram) -> Result<HashMap<String, u64>, String> {
        self.functions = program.functions.clone();
        self.class_parents = program.class_parents.clone();
        let instructions = program.instructions.clone();
        let mut ip = 0usize;
        let mut folded: HashMap<String, u64> = HashMap::new();

        'vm: while ip < instructions.len() {
            let mut path = String::from("मुख्य");
            for f in &self.call_frames {
                path.push(';');
                path.push_str(&f.func_name);
            }
            *folded.entry(path).or_insert(0) += 1;

            let op = &instructions[ip];
            ip += 1;
            match self.exec_op(op, &mut ip, &instructions) {
                Ok(true)  => break 'vm,
                Ok(false) => {}
                Err(e) => {
                    if let Some(tf) = self.try_stack.pop() {
                        self.stack.truncate(tf.stack_depth);
                        self.call_frames.truncate(tf.frame_depth);
                        let errval = self.thrown.take().unwrap_or(Value::Str(e));
                        self.stack.push(errval);
                        ip = tf.handler_ip;
                    } else {
                        self.thrown = None;
                        return Err(self.format_uncaught(e, ip, &program.lines));
                    }
                }
            }
        }
        Ok(folded)
    }

    /// Like `run`, but instruments execution: counts opcode executions and
    /// function calls, times the run, and prints a profile report to stderr
    /// (Phase 17D — `lipi profile`). Behaviour is otherwise identical to `run`.
    pub fn run_profiled(&mut self, program: &CompiledProgram) -> Result<(), String> {
        self.functions = program.functions.clone();
        self.class_parents = program.class_parents.clone();
        let instructions = program.instructions.clone();
        let mut ip = 0usize;

        let mut op_counts: HashMap<String, u64> = HashMap::new();
        let mut fn_calls: HashMap<String, u64> = HashMap::new();
        let mut total_ops: u64 = 0;
        let start = std::time::Instant::now();

        'vm: while ip < instructions.len() {
            let op = &instructions[ip];
            ip += 1;

            total_ops += 1;
            let name = format!("{op:?}");
            let short = name.split(['(', ' ']).next().unwrap_or("?").to_string();
            *op_counts.entry(short).or_insert(0) += 1;
            match op {
                Opcode::Call(n, _) | Opcode::TailCall(n, _) => { *fn_calls.entry(n.clone()).or_insert(0) += 1; }
                Opcode::CallKw(n, _, _) => { *fn_calls.entry(n.clone()).or_insert(0) += 1; }
                _ => {}
            }

            match self.exec_op(op, &mut ip, &instructions) {
                Ok(true)  => break 'vm,
                Ok(false) => {}
                Err(e) => {
                    if let Some(tf) = self.try_stack.pop() {
                        self.stack.truncate(tf.stack_depth);
                        self.call_frames.truncate(tf.frame_depth);
                        let errval = self.thrown.take().unwrap_or(Value::Str(e));
                        self.stack.push(errval);
                        ip = tf.handler_ip;
                    } else {
                        self.thrown = None;
                        return Err(self.format_uncaught(e, ip, &program.lines));
                    }
                }
            }
        }
        let elapsed = start.elapsed();

        // ── Report ──
        eprintln!("\n──────── प्रोफ़ाइल (lipi profile) ────────");
        eprintln!("कुल निर्देश (opcodes executed): {total_ops}");
        eprintln!("समय (time): {:.3} ms", elapsed.as_secs_f64() * 1000.0);
        let mut ops: Vec<(&String, &u64)> = op_counts.iter().collect();
        ops.sort_by(|a, b| b.1.cmp(a.1));
        eprintln!("\nशीर्ष opcodes:");
        eprintln!("  {:<22} {:>12}  {:>6}", "opcode", "गिनती", "%");
        for (name, count) in ops.iter().take(15) {
            let pct = if total_ops > 0 { **count as f64 / total_ops as f64 * 100.0 } else { 0.0 };
            eprintln!("  {:<22} {:>12}  {:>5.1}%", name, count, pct);
        }
        if !fn_calls.is_empty() {
            let mut fns: Vec<(&String, &u64)> = fn_calls.iter().collect();
            fns.sort_by(|a, b| b.1.cmp(a.1));
            eprintln!("\nफलन कॉल (function calls):");
            eprintln!("  {:<28} {:>10}", "function", "कॉल");
            for (name, count) in fns.iter().take(15) {
                eprintln!("  {:<28} {:>10}", name, count);
            }
        }
        eprintln!("──────────────────────────────────────────");
        Ok(())
    }

    /// Interactive line debugger (Phase 17D — `lipi debug`). Steps the program by
    /// source line, honouring breakpoints, and lets the user inspect variables.
    /// `source` is the program text (for showing the current line).
    pub fn run_debug(&mut self, program: &CompiledProgram, source: &str) -> Result<(), String> {
        use std::io::Write;
        self.functions = program.functions.clone();
        self.class_parents = program.class_parents.clone();
        let instructions = program.instructions.clone();
        let src_lines: Vec<&str> = source.lines().collect();
        let mut ip = 0usize;

        let mut breakpoints: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut stepping = true;          // start paused at the first line
        let mut last_line: u32 = 0;

        println!("LIPI डिबगर — 'help' मदद के लिए, 'c' चलाने के लिए, 'q' बाहर\n");

        'vm: while ip < instructions.len() {
            let line = program.lines.get(ip).copied().unwrap_or(0);
            // Decide whether to pause before executing this instruction.
            let hit_bp = line != 0 && breakpoints.contains(&line) && line != last_line;
            let step_pause = stepping && line != 0 && line != last_line;
            if hit_bp || step_pause {
                last_line = line;
                if let Some(text) = src_lines.get(line.saturating_sub(1) as usize) {
                    println!("\x1b[33m►\x1b[0m पंक्ति {line}: {}", text.trim_end());
                }
                // command loop
                loop {
                    print!("(डिबग) ");
                    std::io::stdout().flush().ok();
                    let mut cmd = String::new();
                    if std::io::stdin().read_line(&mut cmd).unwrap_or(0) == 0 { break 'vm; }
                    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
                    match parts.as_slice() {
                        [] => continue,
                        ["s"] | ["step"] => { stepping = true; break; }
                        ["c"] | ["cont"] | ["continue"] => { stepping = false; break; }
                        ["b", n] | ["break", n] => {
                            if let Ok(l) = n.parse::<u32>() { breakpoints.insert(l); println!("ब्रेकपॉइंट सेट: पंक्ति {l}"); }
                        }
                        ["d", n] | ["delete", n] => {
                            if let Ok(l) = n.parse::<u32>() { breakpoints.remove(&l); println!("ब्रेकपॉइंट हटाया: पंक्ति {l}"); }
                        }
                        ["p", name] | ["print", name] => {
                            let v = self.call_frames.last().and_then(|f| f.locals.get(*name))
                                .or_else(|| self.globals.get(*name));
                            match v { Some(val) => println!("  {name} = {val}"), None => println!("  '{name}' परिभाषित नहीं है") }
                        }
                        ["vars"] => {
                            if let Some(f) = self.call_frames.last() {
                                println!("  स्थानीय (locals):");
                                let mut keys: Vec<&String> = f.locals.keys().filter(|k| !k.starts_with("__")).collect();
                                keys.sort();
                                for k in keys { println!("    {k} = {}", f.locals[k]); }
                            }
                            let mut gkeys: Vec<&String> = self.globals.keys()
                                .filter(|k| !k.starts_with("__") && !is_builtin_global(k)).collect();
                            gkeys.sort();
                            if !gkeys.is_empty() {
                                println!("  वैश्विक (globals):");
                                for k in gkeys { println!("    {k} = {}", self.globals[k]); }
                            }
                        }
                        ["where"] | ["bt"] => {
                            println!("  पंक्ति {line}, कॉल गहराई {}", self.call_frames.len());
                        }
                        ["q"] | ["quit"] => { println!("डिबग समाप्त"); return Ok(()); }
                        ["h"] | ["help"] => {
                            println!("  s/step       — एक पंक्ति आगे");
                            println!("  c/continue   — अगले ब्रेकपॉइंट तक चलाएँ");
                            println!("  b/break N    — पंक्ति N पर ब्रेकपॉइंट");
                            println!("  d/delete N   — ब्रेकपॉइंट हटाएँ");
                            println!("  p/print नाम  — चर का मान दिखाएँ");
                            println!("  vars         — सभी चर दिखाएँ");
                            println!("  where/bt     — वर्तमान पंक्ति व गहराई");
                            println!("  q/quit       — डिबगर बंद करें");
                        }
                        _ => println!("अज्ञात कमांड — 'help' देखें"),
                    }
                }
            }

            let op = &instructions[ip];
            ip += 1;
            match self.exec_op(op, &mut ip, &instructions) {
                Ok(true)  => break 'vm,
                Ok(false) => {}
                Err(e) => {
                    if let Some(tf) = self.try_stack.pop() {
                        self.stack.truncate(tf.stack_depth);
                        self.call_frames.truncate(tf.frame_depth);
                        let errval = self.thrown.take().unwrap_or(Value::Str(e));
                        self.stack.push(errval);
                        ip = tf.handler_ip;
                    } else {
                        self.thrown = None;
                        return Err(self.format_uncaught(e, ip, &program.lines));
                    }
                }
            }
        }
        println!("\nकार्यक्रम समाप्त।");
        Ok(())
    }

    /// Build the user-facing report for an uncaught runtime error:
    /// `message (पंक्ति N)` plus one trace line per active call frame,
    /// innermost first. Line 0 / sentinel addresses are silently skipped.
    fn format_uncaught(&self, msg: String, ip: usize, lines: &[u32]) -> String {
        let line_at = |addr: usize| -> u32 {
            if addr == 0 || addr == usize::MAX { return 0; }
            lines.get(addr - 1).copied().unwrap_or(0)
        };
        let mut out = msg;
        let cur = line_at(ip);
        if cur > 0 {
            out.push_str(&format!(" (पंक्ति {cur})"));
        }
        // Cap the trace — a 10k-deep recursion overflow shouldn't print
        // 10k lines. Show the innermost frames; summarise the rest.
        const MAX_TRACE_FRAMES: usize = 12;
        let total = self.call_frames.len();
        for frame in self.call_frames.iter().rev().take(MAX_TRACE_FRAMES) {
            let call_line = line_at(frame.return_addr);
            if call_line > 0 {
                out.push_str(&format!("\n  ↳ विधि '{}' — पंक्ति {} से बुलाई गई", frame.func_name, call_line));
            } else {
                out.push_str(&format!("\n  ↳ विधि '{}'", frame.func_name));
            }
        }
        if total > MAX_TRACE_FRAMES {
            out.push_str(&format!("\n  … और {} स्तर", total - MAX_TRACE_FRAMES));
        }
        out
    }

    // Returns Ok(true) = halt, Ok(false) = continue, Err = runtime error
    // Takes &Opcode — cloning every instruction (Strings included) on each
    // step dominated the interpreter loop before Phase 17 perf work.
    #[allow(clippy::too_many_lines)]
    fn exec_op(&mut self, op: &Opcode, ip: &mut usize, instructions: &[Opcode]) -> Result<bool, String> {
        // Operand-stack guard (Phase 17C): each opcode grows the stack by a
        // bounded amount, so checking once per instruction catches runaway
        // growth before it OOMs. Returned as a catchable error.
        if self.stack.len() > MAX_STACK_DEPTH {
            return Err(format!(
                "स्मृति सीमा पार: ढेर की गहराई {} से अधिक हो गई", MAX_STACK_DEPTH
            ));
        }
        match op {
                // ── Stack ──────────────────────────────────────────────────
                Opcode::Push(v) => {
                    self.stack.push(lvm_to_val(v.clone()));
                }

                Opcode::Pop => {
                    pop(&mut self.stack)?;
                }

                Opcode::Dup => {
                    let v = self.stack.last()
                        .ok_or("LVM: Dup — stack empty")?
                        .clone();
                    self.stack.push(v);
                }

                // ── Variables ──────────────────────────────────────────────
                Opcode::LoadVar(name) => {
                    // Check current frame locals first, then globals
                    let local = self.call_frames.last()
                        .and_then(|f| f.locals.get(name).cloned());
                    let val = if local.is_some() {
                        local
                    } else {
                        self.globals.get(name).cloned()
                    };
                    match val {
                        Some(v) => self.stack.push(v),
                        None => {
                            // Fall back: check if it's a named function → push as closure ref
                            if self.functions.contains_key(name) {
                                self.stack.push(Value::Closure {
                                    func_name: name.clone(),
                                    captured: HashMap::new(),
                                });
                            } else {
                                return Err(format!("'{}' परिभाषित नहीं है", name));
                            }
                        }
                    }
                }

                Opcode::StoreVar(name) => {
                    let val = pop(&mut self.stack)?;
                    self.set_var(name, val)?;
                }

                // ── Arithmetic ─────────────────────────────────────────────
                Opcode::Add => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if matches!(a, Value::Instance { .. })
                        && self.try_instance_binop(a.clone(), b.clone(), "__जोड़ो__", ip)? {
                        return Ok(false);
                    }
                    self.stack.push(vm_add(a, b)?);
                }
                Opcode::Sub => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if matches!(a, Value::Instance { .. })
                        && self.try_instance_binop(a.clone(), b.clone(), "__घटाओ__", ip)? {
                        return Ok(false);
                    }
                    self.stack.push(vm_num2(a, b, |x, y| x - y, "-")?);
                }
                Opcode::Mul => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if matches!(a, Value::Instance { .. })
                        && self.try_instance_binop(a.clone(), b.clone(), "__गुणा__", ip)? {
                        return Ok(false);
                    }
                    self.stack.push(vm_num2(a, b, |x, y| x * y, "*")?);
                }
                Opcode::Div => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if matches!(a, Value::Instance { .. })
                        && self.try_instance_binop(a.clone(), b.clone(), "__भाग__", ip)? {
                        return Ok(false);
                    }
                    if let Value::Number(dv) = &b {
                        if *dv == 0.0 { return Err("शून्य से भाग नहीं होता".into()); }
                    }
                    self.stack.push(vm_num2(a, b, |x, y| x / y, "/")?);
                }
                Opcode::FloorDiv => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if let Value::Number(dv) = &b {
                        if *dv == 0.0 { return Err("शून्य से भाग नहीं होता".into()); }
                    }
                    self.stack.push(vm_num2(a, b, |x, y| (x / y).floor(), "//")?);
                }
                Opcode::Mod => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    if matches!(a, Value::Instance { .. })
                        && self.try_instance_binop(a.clone(), b.clone(), "__शेष__", ip)? {
                        return Ok(false);
                    }
                    self.stack.push(vm_num2(a, b, |x, y| x % y, "%")?);
                }

                // ── Comparison ─────────────────────────────────────────────
                Opcode::Eq => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(vals_eq(&a, &b)));
                }
                Opcode::NotEq => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(!vals_eq(&a, &b)));
                }
                Opcode::Gt => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(num_cmp(a, b, |x, y| x > y, ">")?));
                }
                Opcode::Lt => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(num_cmp(a, b, |x, y| x < y, "<")?));
                }
                Opcode::GtEq => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(num_cmp(a, b, |x, y| x >= y, ">=")?));
                }
                Opcode::LtEq => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(num_cmp(a, b, |x, y| x <= y, "<=")?));
                }

                // ── Logic ──────────────────────────────────────────────────
                Opcode::And => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(truthy(&a) && truthy(&b)));
                }
                Opcode::Or => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(truthy(&a) || truthy(&b)));
                }
                Opcode::Not => {
                    let a = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(!truthy(&a)));
                }

                // ── Control flow ───────────────────────────────────────────
                Opcode::Jump(addr) => {
                    *ip = *addr;
                    return Ok(false);
                }

                Opcode::JumpIfFalse(addr) => {
                    let v = pop(&mut self.stack)?;
                    if !truthy(&v) { *ip = *addr; return Ok(false); }
                }

                Opcode::JumpIfTrue(addr) => {
                    let v = pop(&mut self.stack)?;
                    if truthy(&v) { *ip = *addr; return Ok(false); }
                }

                // ── Function calls ─────────────────────────────────────────
                Opcode::Call(name, argc) => {
                    let argc = *argc;
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc { args.push(pop(&mut self.stack)?); }
                    args.reverse();

                    if let Some(func) = self.functions.get(name).cloned() {
                        let mut locals = HashMap::new();
                        let regular = func.params.len();
                        for (param, arg) in func.params.iter().zip(args.iter()) {
                            locals.insert(param.clone(), arg.clone());
                        }
                        fill_defaults(&mut locals, &func, args.len());
                        if let Some(ref vname) = func.vararg {
                            let rest: Vec<Value> = args.into_iter().skip(regular).collect();
                            locals.insert(vname.clone(), Value::List(rest));
                        }
                        if func.is_generator {
                            let g = self.make_generator(func.start_ip, locals, name.clone());
                            self.stack.push(g);
                            return Ok(false);
                        }
                        self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: name.clone() })?;
                        *ip = func.start_ip;
                        return Ok(false);
                    }

                    // Constructor: walk parent chain for inherited बनाओ
                    if name.ends_with("::बनाओ") {
                        let class = name.trim_end_matches("::बनाओ").to_string();
                        let inherited = {
                            let mut search = self.class_parents.get(&class).cloned();
                            let mut found = None;
                            while let Some(parent) = search {
                                let key = format!("{}::बनाओ", parent);
                                if let Some(f) = self.functions.get(&key).cloned() {
                                    found = Some(f);
                                    break;
                                }
                                search = self.class_parents.get(&parent).cloned();
                            }
                            found
                        };
                        if let Some(func) = inherited {
                            let mut locals = HashMap::new();
                            let provided = args.len();
                            for (param, arg) in func.params.iter().zip(args) {
                                locals.insert(param.clone(), arg);
                            }
                            fill_defaults(&mut locals, &func, provided);
                            self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: name.clone() })?;
                            *ip = func.start_ip;
                            return Ok(false);
                        }
                        // No बनाओ anywhere — return instance unchanged
                        let instance = args.into_iter().next().unwrap_or(Value::Nil);
                        self.stack.push(instance);
                    }
                    // Native function (built-in or imported)
                    else if let Some(native) = self.native_fns.get(name).copied() {
                        let result = native(args)?;
                        self.stack.push(result);
                    }
                    else if name == "आगे" && argc == 1 {
                        let g = args[0].clone();
                        match g {
                            Value::Generator(id) => {
                                let v = self.resume_generator(id, instructions)?.unwrap_or(Value::Nil);
                                self.stack.push(v);
                            }
                            other => return Err(format!("आगे(): जनित्र अपेक्षित, मिला: {}", other)),
                        }
                    }
                    else if name == "चलाओ" && argc == 1 {
                        match args[0].clone() {
                            Value::Generator(id) => {
                                let res = self.run_event_loop(vec![id], instructions)?;
                                self.stack.push(res.into_iter().next().unwrap_or(Value::Nil));
                            }
                            other => return Err(format!("चलाओ(): कार्य (async) अपेक्षित, मिला: {}", other)),
                        }
                    }
                    else if name == "इकट्ठा" && argc >= 1 {
                        let mut ids = Vec::with_capacity(argc);
                        for a in &args {
                            match a {
                                Value::Generator(id) => ids.push(*id),
                                other => return Err(format!("इकट्ठा(): कार्य अपेक्षित, मिला: {}", other)),
                            }
                        }
                        let res = self.run_event_loop(ids, instructions)?;
                        self.stack.push(Value::List(res));
                    }
                    else if name == "सूची_में" && argc == 1 {
                        let out = match args[0].clone() {
                            Value::Generator(id) => {
                                let mut v = Vec::new();
                                while let Some(item) = self.resume_generator(id, instructions)? {
                                    check_list_len(v.len() + 1)?;
                                    v.push(item);
                                }
                                Value::List(v)
                            }
                            Value::List(v) => Value::List(v),
                            Value::Str(s) => Value::List(s.chars().map(|c| Value::Str(c.to_string())).collect()),
                            other => return Err(format!("सूची_में(): जनित्र/सूची/वाक्य अपेक्षित, मिला: {}", other)),
                        };
                        self.stack.push(out);
                    }
                    // HOF meta-functions (मानचित्र, छानो, मोड़ो) — need VM access to call closures
                    else if name == "मानचित्र" && argc == 2 {
                        let func_ref = args[1].clone();
                        let list = args[0].clone();
                        if let Value::List(items) = list {
                            let mut result = Vec::new();
                            for item in items {
                                let val = self.call_closure_value(&func_ref, vec![item], instructions)?;
                                result.push(val);
                            }
                            self.stack.push(Value::List(result));
                        } else {
                            return Err("मानचित्र: पहला तर्क सूची होनी चाहिए".into());
                        }
                    }
                    else if name == "छानो" && argc == 2 {
                        let func_ref = args[1].clone();
                        let list = args[0].clone();
                        if let Value::List(items) = list {
                            let mut result = Vec::new();
                            for item in items {
                                let keep = self.call_closure_value(&func_ref, vec![item.clone()], instructions)?;
                                if truthy(&keep) { result.push(item); }
                            }
                            self.stack.push(Value::List(result));
                        } else {
                            return Err("छानो: पहला तर्क सूची होनी चाहिए".into());
                        }
                    }
                    else if name == "मोड़ो" && argc == 3 {
                        let func_ref = args[2].clone();
                        let mut acc = args[1].clone();
                        let list = args[0].clone();
                        if let Value::List(items) = list {
                            for item in items {
                                acc = self.call_closure_value(&func_ref, vec![acc, item], instructions)?;
                            }
                            self.stack.push(acc);
                        } else {
                            return Err("मोड़ो: पहला तर्क सूची होनी चाहिए".into());
                        }
                    }
                    // Functools (Phase 17): build a tagged closure that is
                    // intercepted at call time by `call_functools`.
                    else if name == "संयोजित" && argc == 2 {
                        // संयोजित(f, g) → लाम्डा(x): f(g(x))
                        let mut cap = HashMap::new();
                        cap.insert("__f".to_string(), args[0].clone());
                        cap.insert("__g".to_string(), args[1].clone());
                        self.stack.push(Value::Closure { func_name: "__functools_compose__".into(), captured: cap });
                    }
                    else if name == "आंशिक" && argc >= 1 {
                        // आंशिक(f, a, b, ...) → partial application binding leading args
                        let mut cap = HashMap::new();
                        cap.insert("__f".to_string(), args[0].clone());
                        cap.insert("__bound".to_string(), Value::List(args[1..].to_vec()));
                        self.stack.push(Value::Closure { func_name: "__functools_partial__".into(), captured: cap });
                    }
                    else if name == "स्मरण" && argc == 1 {
                        // स्मरण(f) → memoizing wrapper; cache lives on the VM by id
                        let id = self.memo_next;
                        self.memo_next += 1;
                        self.memo_caches.insert(id, HashMap::new());
                        let mut cap = HashMap::new();
                        cap.insert("__f".to_string(), args[0].clone());
                        cap.insert("__id".to_string(), Value::Number(id as f64));
                        self.stack.push(Value::Closure { func_name: "__functools_memo__".into(), captured: cap });
                    }
                    // Closure variable: look up `name` as a variable holding a Value::Closure
                    else {
                        let maybe_closure = self.call_frames.last()
                            .and_then(|f| f.locals.get(name).cloned())
                            .or_else(|| self.globals.get(name).cloned());
                        if let Some(Value::Closure { func_name, captured }) = maybe_closure {
                            if func_name.starts_with("__functools_") {
                                let res = self.call_functools(&func_name, &captured, args, instructions)?;
                                self.stack.push(res);
                            } else if let Some(func) = self.functions.get(&func_name).cloned() {
                                let mut locals = captured; // start with captured scope
                                let provided = args.len();
                                for (param, arg) in func.params.iter().zip(args) {
                                    locals.insert(param.clone(), arg);
                                }
                                fill_defaults(&mut locals, &func, provided);
                                self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: name.clone() })?;
                                *ip = func.start_ip;
                            } else {
                                return Err(format!("विधि '{}' परिभाषित नहीं है", func_name));
                            }
                        } else {
                            return Err(format!("विधि '{}' परिभाषित नहीं है", name));
                        }
                    }
                }

                // func(अ, नाम=मान) — call with keyword arguments (Phase 17)
                Opcode::CallKw(name, pos_argc, kwnames) => {
                    let mut kwvals = Vec::with_capacity(kwnames.len());
                    for _ in 0..kwnames.len() { kwvals.push(pop(&mut self.stack)?); }
                    kwvals.reverse();
                    let mut args = Vec::with_capacity(*pos_argc);
                    for _ in 0..*pos_argc { args.push(pop(&mut self.stack)?); }
                    args.reverse();

                    // Resolve: user function, inherited constructor, or closure variable
                    let resolved: Option<(FuncDef, HashMap<String, Value>)> =
                        if let Some(f) = self.functions.get(name).cloned() {
                            Some((f, HashMap::new()))
                        } else if name.ends_with("::बनाओ") {
                            let class = name.trim_end_matches("::बनाओ").to_string();
                            let mut search = self.class_parents.get(&class).cloned();
                            let mut found = None;
                            while let Some(parent) = search {
                                let key = format!("{}::बनाओ", parent);
                                if let Some(f) = self.functions.get(&key).cloned() {
                                    found = Some((f, HashMap::new()));
                                    break;
                                }
                                search = self.class_parents.get(&parent).cloned();
                            }
                            found
                        } else {
                            let maybe_closure = self.call_frames.last()
                                .and_then(|f| f.locals.get(name).cloned())
                                .or_else(|| self.globals.get(name).cloned());
                            if let Some(Value::Closure { func_name, captured }) = maybe_closure {
                                self.functions.get(&func_name).cloned().map(|f| (f, captured))
                            } else {
                                None
                            }
                        };

                    match resolved {
                        Some((func, base_locals)) => {
                            let mut locals = base_locals;
                            bind_args_kw(&mut locals, &func, args, kwnames, kwvals, name)?;
                            self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: name.clone() })?;
                            *ip = func.start_ip;
                            return Ok(false);
                        }
                        None => {
                            if self.native_fns.contains_key(name) {
                                return Err(format!("'{}': कीवर्ड तर्क अंतर्निहित विधियों में समर्थित नहीं", name));
                            }
                            return Err(format!("विधि '{}' परिभाषित नहीं है", name));
                        }
                    }
                }

                Opcode::CallNative(name, argc) => {
                    let mut args = Vec::with_capacity(*argc);
                    for _ in 0..*argc { args.push(pop(&mut self.stack)?); }
                    args.reverse();

                    if let Some(native) = self.native_fns.get(name).copied() {
                        let result = native(args)?;
                        self.stack.push(result);
                    } else {
                        return Err(format!("स्वदेशी विधि '{}' नहीं मिली (आयात किया?)", name));
                    }
                }

                Opcode::Return => {
                    let val = pop(&mut self.stack)?;
                    if let Some(frame) = self.call_frames.pop() {
                        self.stack.push(val);
                        *ip = frame.return_addr;
                        return Ok(false);
                    } else {
                        return Ok(true); // Top-level फल — halt
                    }
                }

                // ── Output ─────────────────────────────────────────────────
                Opcode::Print => {
                    let v = pop(&mut self.stack)?;
                    if self.capture {
                        if !self.output.is_empty() { self.output.push('\n'); }
                        self.output.push_str(&format!("{v}"));
                    } else {
                        println!("{v}");
                    }
                }

                Opcode::PrintInline => {
                    use std::io::Write;
                    let v = pop(&mut self.stack)?;
                    if self.capture {
                        self.output.push_str(&format!("{v}"));
                    } else {
                        print!("{v}");
                        std::io::stdout().flush().ok();
                    }
                }

                // ── Import ─────────────────────────────────────────────────
                Opcode::Import(module) => {
                    let registry = match module.as_str() {
                        "भारत.पहचान"    => crate::bharat_stdlib::pehchaan_registry(),
                        "भारत.संख्या"   => crate::bharat_stdlib::sankhya_registry(),
                        "भारत.भुगतान"   => crate::bharat_stdlib::bhugtaan_registry(),
                        "भारत.भाषा"     => crate::bharat_stdlib::bhasha_registry(),
                        "भारत.गणित"     => crate::bharat_stdlib::ganit_registry(),
                        "भारत.छन्दस्"   => crate::bharat_stdlib::chhandas_registry(),
                        "भारत.न्याय"    => crate::bharat_stdlib::nyaya_registry(),
                        "भारत.शुल्ब"    => crate::bharat_stdlib::shulba_registry(),
                        "भारत.ज्योतिष"  => crate::bharat_stdlib::jyotish_registry(),
                        "भारत.नाट्य"    => crate::bharat_stdlib::natya_registry(),
                        "भारत.तंत्रिका" => crate::bharat_stdlib::tantrika_registry(),
                        "भारत.अनुकूलन"  => crate::bharat_stdlib::anukooland_registry(),
                        "भारत.प्रज्ञा"    => crate::bharat_stdlib::pragya_registry(),
                        "भारत.तुरिंग"     => crate::bharat_stdlib::turing_registry(),
                        "भारत.यंत्र"      => crate::bharat_stdlib::yantra_registry(),
                        "भारत.व्याकरण"   => crate::bharat_stdlib::vyakaran_registry(),
                        "भारत.विज्ञान"    => crate::bharat_stdlib::vigyan_registry(),
                        "भारत.json"       => crate::bharat_stdlib::json_registry(),
                        "भारत.समय"        => crate::bharat_stdlib::samay_registry(),
                        "भारत.csv"        => crate::bharat_stdlib::csv_registry(),
                        "भारत.कूट"        => crate::bharat_stdlib::koot_registry(),
                        "भारत.http"       => crate::bharat_stdlib::http_registry(),
                        "भारत.प्रतिमान"   => crate::regex_engine::pratimaan_registry(),
                        "भारत.सांख्यिकी"  => crate::bharat_stdlib::sankhyiki_registry(),
                        "भारत.बड़ी"       => crate::bignum::badi_registry(),
                        "भारत.संजाल"      => crate::net::sanjaal_registry(),
                        "भारत.संपीडन"     => crate::zip::sampidan_registry(),
                        "भारत.संग्रह"     => crate::sql::sangraha_registry(),
                        "भारत.सर्वर"      => crate::server::sarvar_registry(),
                        "भारत.सूत्र"       => crate::threads::sutra_registry(),
                        "भारत.मात्रक"     => crate::matrak::matrak_registry(),
                        "भारत.रेखीय"      => crate::rekhiy::rekhiy_registry(),
                        "भारत.नियंत्रण"   => crate::niyantran::niyantran_registry(),
                        "भारत.दिशा"       => crate::disha::disha_registry(),
                        "भारत.सुरक्षा"    => crate::suraksha::suraksha_registry(),
                        "भारत.अंतराल"     => crate::antaral::antaral_registry(),
                        other => return Err(format!("अज्ञात मॉड्यूल: {}", other)),
                    };
                    for (fname, func) in registry {
                        self.native_fns.insert(fname.to_string(), func);
                    }
                }

                // ── Method calls ───────────────────────────────────────────
                Opcode::MethodCallKw { method, pos_argc, kwnames } => {
                    let mut kwvals = Vec::with_capacity(kwnames.len());
                    for _ in 0..kwnames.len() { kwvals.push(pop(&mut self.stack)?); }
                    kwvals.reverse();
                    let mut args = Vec::with_capacity(*pos_argc);
                    for _ in 0..*pos_argc { args.push(pop(&mut self.stack)?); }
                    args.reverse();
                    let obj = pop(&mut self.stack)?;

                    let class_name = match &obj {
                        Value::Instance { class, .. } => class.clone(),
                        other => return Err(format!("कीवर्ड तर्क विधि कॉल '{}' पर असमर्थित: {}", method, other)),
                    };
                    let func = {
                        let mut search = Some(class_name.clone());
                        let mut found = None;
                        while let Some(cls) = search {
                            let key = format!("{}::{}", cls, method);
                            if let Some(f) = self.functions.get(&key).cloned() { found = Some(f); break; }
                            search = self.class_parents.get(&cls).cloned();
                        }
                        found
                    };
                    let func = func.ok_or_else(|| format!("'{}' में विधि '{}' नहीं है", class_name, method))?;
                    let mut locals = HashMap::new();
                    let all: Vec<Value> = std::iter::once(obj).chain(args).collect();
                    bind_args_kw(&mut locals, &func, all, kwnames, kwvals, method)?;
                    self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: format!("{}.{}", class_name, method) })?;
                    *ip = func.start_ip;
                    return Ok(false);
                }

                Opcode::MethodCall(method, n_args) => {
                    let mut args = Vec::with_capacity(*n_args);
                    for _ in 0..*n_args { args.push(pop(&mut self.stack)?); }
                    args.reverse();
                    let obj = pop(&mut self.stack)?;

                    // Enum constructor — EnumName.Variant(args) (Phase 15)
                    if let Value::EnumDef { name: enum_name, variants } = &obj {
                        let arity = variants.get(method).copied()
                            .ok_or_else(|| format!("विकल्प '{}' में '{}' नहीं है", enum_name, method))?;
                        if args.len() != arity {
                            return Err(format!("'{}::{}' को {} तर्क चाहिए, मिले {}", enum_name, method, arity, args.len()));
                        }
                        self.stack.push(Value::Enum {
                            enum_name: enum_name.clone(),
                            variant: method.clone(),
                            values: args,
                        });
                        return Ok(false);
                    }

                    // User-defined instance method → push frame and jump
                    if let Value::Instance { ref class, .. } = obj {
                        let class_name = class.clone();
                        // Look up method in own class, then walk parent chain
                        let func = {
                            let mut search = Some(class_name.clone());
                            let mut found = None;
                            while let Some(cls) = search {
                                let key = format!("{}::{}", cls, method);
                                if let Some(f) = self.functions.get(&key).cloned() {
                                    found = Some(f);
                                    break;
                                }
                                search = self.class_parents.get(&cls).cloned();
                            }
                            found
                        };
                        if let Some(func) = func {
                            let mut locals = HashMap::new();
                            let all: Vec<Value> = std::iter::once(obj).chain(args).collect();
                            let provided = all.len();
                            for (param, val) in func.params.iter().zip(all) {
                                locals.insert(param.clone(), val);
                            }
                            fill_defaults(&mut locals, &func, provided);
                            self.push_frame(Frame { return_addr: *ip, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name: format!("{}.{}", class_name, method) })?;
                            *ip = func.start_ip;
                            return Ok(false);
                        }
                        return Err(format!("'{}' में विधि '{}' नहीं है", class_name, method));
                    }

                    let result = match (obj, method.as_str()) {
                        // String methods
                        (Value::Str(s), "लम्बाई")     => Value::Number(s.chars().count() as f64),
                        (Value::Str(s), "रोमन_में")   => Value::Str(crate::bharat_stdlib::romanize(&s)),
                        (Value::Str(s), "बड़े_अक्षर") => Value::Str(s.to_uppercase()),
                        (Value::Str(s), "छोटे_अक्षर") => Value::Str(s.to_lowercase()),
                        (Value::Str(s), "उलटा")       => Value::Str(s.chars().rev().collect()),
                        (Value::Str(s), "ट्रिम")      => Value::Str(s.trim().to_string()),
                        (Value::Str(s), "शुरू_में")   => {
                            let prefix = args.into_iter().next()
                                .ok_or_else(|| "शुरू_में(): एक तर्क आवश्यक".to_string())?;
                            Value::Bool(s.starts_with(&format!("{prefix}")))
                        }
                        (Value::Str(s), "अंत_में")    => {
                            let suffix = args.into_iter().next()
                                .ok_or_else(|| "अंत_में(): एक तर्क आवश्यक".to_string())?;
                            Value::Bool(s.ends_with(&format!("{suffix}")))
                        }
                        (Value::Str(s), "खोजो")       => {
                            let query = args.into_iter().next()
                                .ok_or_else(|| "खोजो(): एक तर्क आवश्यक".to_string())?;
                            let q = format!("{query}");
                            match s.find(&q) {
                                Some(byte_idx) => Value::Number(s[..byte_idx].chars().count() as f64),
                                None           => Value::Number(-1.0),
                            }
                        }
                        (Value::Str(s), "विभाजित")    => {
                            let sep = match args.into_iter().next() {
                                Some(v) => format!("{v}"),
                                None    => " ".to_string(),
                            };
                            let parts: Vec<Value> = s.split(&sep)
                                .map(|p| Value::Str(p.to_string()))
                                .collect();
                            Value::List(parts)
                        }
                        (Value::Str(s), "बदलो")       => {
                            let mut iter = args.into_iter();
                            let old = iter.next().ok_or_else(|| "बदलो(): दो तर्क आवश्यक".to_string())?;
                            let new = iter.next().ok_or_else(|| "बदलो(): दो तर्क आवश्यक".to_string())?;
                            Value::Str(s.replace(&format!("{old}"), &format!("{new}")))
                        }
                        // Number methods
                        (Value::Number(n), "पूर्णांक") => Value::Number(n.floor()),
                        // List methods
                        (Value::List(v), "लम्बाई") => Value::Number(v.len() as f64),
                        (Value::List(mut v), "जोड़ो") => {
                            let val = args.into_iter().next()
                                .ok_or_else(|| "जोड़ो(): एक तर्क आवश्यक".to_string())?;
                            check_list_len(v.len() + 1)?;
                            v.push(val);
                            Value::List(v)
                        }
                        (Value::List(mut v), "हटाओ") => {
                            let i = match args.first() {
                                Some(Value::Number(n)) => *n as usize,
                                _ => return Err("हटाओ(): संख्या-अनुक्रमांक आवश्यक".into()),
                            };
                            if i < v.len() { v.remove(i); Value::List(v) }
                            else { return Err(format!("हटाओ(): सूची सीमा से बाहर {i}")); }
                        }
                        (Value::List(v), "उलटा") => {
                            let mut rv = v;
                            rv.reverse();
                            Value::List(rv)
                        }
                        (Value::List(v), "क्रमबद्ध") => {
                            let mut sv = v;
                            sv.sort_by(|a, b| format!("{a}").cmp(&format!("{b}")));
                            Value::List(sv)
                        }
                        (Value::List(v), "मिलाओ") => {
                            let sep = match args.into_iter().next() {
                                Some(val) => format!("{val}"),
                                None      => String::new(),
                            };
                            let joined = v.iter().map(|x| format!("{x}")).collect::<Vec<_>>().join(&sep);
                            Value::Str(joined)
                        }
                        // Dict methods
                        (Value::Dict(m), "लम्बाई") => Value::Number(m.len() as f64),
                        (Value::Dict(m), "कुंजियाँ") => {
                            let mut keys: Vec<Value> = m.keys().map(|k| Value::Str(k.clone())).collect();
                            keys.sort_by(|a, b| format!("{a}").cmp(&format!("{b}")));
                            Value::List(keys)
                        }
                        (Value::Dict(m), "मान") => {
                            Value::List(m.values().cloned().collect())
                        }
                        (obj, meth) => return Err(format!("'{}' विधि '{}' पर उपलब्ध नहीं है", meth, obj)),
                    };
                    self.stack.push(result);
                }

                // ── Karaka check (soft warning — not enforced in LVM yet) ──
                Opcode::KarakaCheck(_, _) => {}

                // ── Phase 5: सूची + कोश ────────────────────────────────────

                Opcode::MakeList(n) => {
                    check_list_len(*n)?;
                    let mut elems: Vec<Value> = (0..*n).map(|_| pop(&mut self.stack)).collect::<Result<_,_>>()?;
                    elems.reverse();
                    self.stack.push(Value::List(elems));
                }

                Opcode::MakeDict(n) => {
                    // Pairs pushed as key, val, key, val …  so stack top is last val.
                    let mut pairs: Vec<(Value, Value)> = (0..*n).map(|_| {
                        let v = pop(&mut self.stack)?;
                        let k = pop(&mut self.stack)?;
                        Ok((k, v))
                    }).collect::<Result<_,String>>()?;
                    pairs.reverse();
                    let mut map = std::collections::HashMap::new();
                    for (k, v) in pairs {
                        map.insert(format!("{k}"), v);
                    }
                    self.stack.push(Value::Dict(map));
                }

                Opcode::GetIndex => {
                    let idx = pop(&mut self.stack)?;
                    let obj = pop(&mut self.stack)?;
                    let result = match (obj, &idx) {
                        (Value::List(v), Value::Number(n)) => {
                            let i = *n as usize;
                            v.get(i).cloned().ok_or_else(|| format!("सूची सीमा से बाहर: {i}"))?
                        }
                        (Value::Dict(m), _) => {
                            let key = format!("{idx}");
                            m.get(&key).cloned().ok_or_else(|| format!("कोश में कुंजी नहीं मिली: '{key}'"))?
                        }
                        (Value::Str(s), Value::Number(n)) => {
                            let i = *n as usize;
                            s.chars().nth(i)
                                .map(|c| Value::Str(c.to_string()))
                                .ok_or_else(|| format!("वाक्य सीमा से बाहर: {i}"))?
                        }
                        (obj, _) => return Err(format!("'{}' पर अनुक्रमण नहीं होता", obj)),
                    };
                    self.stack.push(result);
                }

                Opcode::SetIndex => {
                    let val = pop(&mut self.stack)?;
                    let idx = pop(&mut self.stack)?;
                    let obj = pop(&mut self.stack)?;
                    let updated = match (obj, idx) {
                        (Value::List(mut v), Value::Number(n)) => {
                            let i = n as usize;
                            if i < v.len() { v[i] = val; Value::List(v) }
                            else { return Err(format!("सूची सीमा से बाहर: {i}")); }
                        }
                        (Value::Dict(mut m), k) => {
                            m.insert(format!("{k}"), val);
                            Value::Dict(m)
                        }
                        (obj, _) => return Err(format!("'{}' पर SetIndex नहीं होता", obj)),
                    };
                    self.stack.push(updated);
                }

                // ── Phase 6: वर्ग (Classes) ────────────────────────────────

                Opcode::MakeInstance(class_name) => {
                    self.stack.push(Value::Instance {
                        class: class_name.clone(),
                        fields: HashMap::new(),
                    });
                }

                Opcode::GetAttr(field) => {
                    match pop(&mut self.stack)? {
                        Value::Instance { class, fields } => {
                            // Property getter dispatch (Phase 17): if the class (or a
                            // parent) defines `__पाओ_<field>__`, call it and use its
                            // return value instead of reading the raw field. The
                            // getter reads a DIFFERENT backing field (convention:
                            // `__पाओ_मान__` reads `यह._मान`); the backing field has
                            // no `__पाओ__मान__` method, so it reads normally — no loop.
                            let getter = format!("__पाओ_{}__", field);
                            if let Some(func) = self.lookup_method(&class, &getter) {
                                let class_name = class.clone();
                                let mut locals = HashMap::new();
                                let inst = Value::Instance { class, fields };
                                if let Some(p) = func.params.first() {
                                    locals.insert(p.clone(), inst);
                                }
                                fill_defaults(&mut locals, &func, 1);
                                self.push_frame(Frame {
                                    return_addr: *ip,
                                    locals,
                                    global_names: std::collections::HashSet::new(),
                                    base_stack_depth: self.stack.len(),
                                    func_name: format!("{}.{}", class_name, getter),
                                })?;
                                *ip = func.start_ip;
                                return Ok(false);
                            }
                            let val = fields.get(field).cloned()
                                .ok_or_else(|| format!("'{}' का क्षेत्र '{}' परिभाषित नहीं", class, field))?;
                            self.stack.push(val);
                        }
                        // Enum type — accessing a zero-field variant creates the instance
                        Value::EnumDef { name: enum_name, variants } => {
                            let arity = variants.get(field)
                                .copied()
                                .ok_or_else(|| format!("विकल्प '{}' में '{}' नहीं है", enum_name, field))?;
                            if arity == 0 {
                                self.stack.push(Value::Enum {
                                    enum_name,
                                    variant: field.clone(),
                                    values: Vec::new(),
                                });
                            } else {
                                // Return a closure-like value? No — push a constructor marker
                                // We encode as a special Enum with empty values and let MethodCall create it
                                // Actually push the EnumDef back and a string marker — simpler:
                                // We push back EnumDef so the subsequent Call can be handled.
                                // But GetAttr is used for `EnumName.Variant(args)` via MethodCall.
                                // For non-zero arity, return a Closure wrapping the construction.
                                self.stack.push(Value::EnumDef { name: enum_name, variants });
                                return Err(format!(
                                    "विकल्प '{}' को तर्कों के साथ बनाएं: EnumName.Variant(args)",
                                    field
                                ));
                            }
                        }
                        other => return Err(format!("GetAttr '{}': वस्तु अपेक्षित, मिला: {}", field, other)),
                    }
                }

                Opcode::SetAttr(field) => {
                    let val = pop(&mut self.stack)?;
                    match pop(&mut self.stack)? {
                        Value::Instance { class, mut fields } => {
                            // Property setter dispatch (Phase 17): if the class (or a
                            // parent) defines `__सेट_<field>__(यह, मान)`, call it
                            // instead of writing the raw field. The setter is expected
                            // to store into a backing field and `फल यह` so the mutated
                            // instance flows back to the StoreVar that follows SetAttr.
                            // The backing field (`_मान`) has no `__सेट__मान__` method,
                            // so writing it takes the raw path — no recursion.
                            let setter = format!("__सेट_{}__", field);
                            if let Some(func) = self.lookup_method(&class, &setter) {
                                let class_name = class.clone();
                                let mut locals = HashMap::new();
                                let inst = Value::Instance { class, fields };
                                for (param, v) in func.params.iter().zip([inst, val]) {
                                    locals.insert(param.clone(), v);
                                }
                                fill_defaults(&mut locals, &func, func.params.len().min(2));
                                self.push_frame(Frame {
                                    return_addr: *ip,
                                    locals,
                                    global_names: std::collections::HashSet::new(),
                                    base_stack_depth: self.stack.len(),
                                    func_name: format!("{}.{}", class_name, setter),
                                })?;
                                *ip = func.start_ip;
                                return Ok(false);
                            }
                            fields.insert(field.clone(), val);
                            self.stack.push(Value::Instance { class, fields });
                        }
                        other => return Err(format!("SetAttr '{}': वस्तु अपेक्षित, मिला: {}", field, other)),
                    }
                }

                // ═══ Indian opcodes ═══════════════════════════════════════

                // आधार_जाँचो(s) — full Verhoeff/UIDAI validation
                Opcode::AadhaarVerify => {
                    let s = pop_str(&mut self.stack)?;
                    let ok = crate::bharat_stdlib::aadhaar_valid(&s);
                    self.stack.push(Value::Bool(ok));
                }

                // upi_भेजो(from, to, amount, note)
                Opcode::UpiSend => {
                    let note   = pop_str(&mut self.stack)?;
                    let amount = pop_num(&mut self.stack)?;
                    let to     = pop_str(&mut self.stack)?;
                    let from   = pop_str(&mut self.stack)?;
                    let result = crate::bharat_stdlib::upi_send(&from, &to, amount, &note)?;
                    self.stack.push(Value::Str(result));
                }

                // gst_जोड़ो(amount, rate)
                Opcode::GstAdd => {
                    let rate   = pop_num(&mut self.stack)?;
                    let amount = pop_num(&mut self.stack)?;
                    let result = crate::bharat_stdlib::gst_add(amount, rate);
                    self.stack.push(Value::Number(result));
                }

                // लाख_में(n) — format number as lakh string
                Opcode::LakhParse => {
                    let n = pop_num(&mut self.stack)?;
                    self.stack.push(Value::Str(crate::bharat_stdlib::format_lakh(n)));
                }

                // रुपये_में(n) — format with ₹ and Indian commas
                Opcode::RupeeFormat => {
                    let n = pop_num(&mut self.stack)?;
                    self.stack.push(Value::Str(crate::bharat_stdlib::format_rupees(n)));
                }

                // ── Phase 17A: Typed exceptions ────────────────────────────
                // Pops the error value, stores it in the `thrown` channel and
                // raises — the unwinder pushes it at the handler address.
                // Also used to RETHROW when no catch clause matched.
                Opcode::Throw => {
                    let v = pop(&mut self.stack)?;
                    let msg = match &v {
                        Value::Str(s) => s.clone(),
                        Value::Instance { class, fields } => {
                            if !self.err_chain_contains(class, "त्रुटि") {
                                return Err(format!(
                                    "फेंको: '{}' त्रुटि वर्ग नहीं है — वर्ग {}(त्रुटि): से बनाएं",
                                    class, class
                                ));
                            }
                            match fields.get("संदेश") {
                                Some(Value::Str(s)) => format!("{}: {}", class, s),
                                Some(other) => format!("{}: {}", class, other),
                                None => class.clone(),
                            }
                        }
                        other => return Err(format!(
                            "फेंको: त्रुटि instance या वाक्य चाहिए, मिला {}", other
                        )),
                    };
                    self.thrown = Some(v);
                    return Err(msg);
                }

                // Pops the error value, pushes Bool: does it match the clause
                // class? Catch dispatch compiles as Dup + MatchErrClass + JumpIfFalse.
                Opcode::MatchErrClass(target) => {
                    let v = pop(&mut self.stack)?;
                    let matched = match &v {
                        // Any thrown instance is an error instance (validated at
                        // throw time) — so the त्रुटि base class matches them all.
                        Value::Instance { class, .. } =>
                            target == "त्रुटि" || self.err_chain_contains(class, &target),
                        // Plain string errors only match the त्रुटि catch-all.
                        Value::Str(_) => target == "त्रुटि",
                        _ => false,
                    };
                    self.stack.push(Value::Bool(matched));
                }

                // ── Phase 9: Error handling ────────────────────────────────
                Opcode::TryStart(handler_ip) => {
                    self.try_stack.push(TryFrame {
                        handler_ip: *handler_ip,
                        stack_depth: self.stack.len(),
                        frame_depth: self.call_frames.len(),
                    });
                }

                Opcode::TryEnd => {
                    self.try_stack.pop();
                }

                // ── Phase 9: Multi-file import ─────────────────────────────
                Opcode::ImportFile(path) => {
                    // Resolve: literal path first, then an installed package in
                    // lipi_modules/ (Phase 17D package manager): `आयात "नाम"` finds
                    // lipi_modules/नाम.swami or lipi_modules/नाम/नाम.swami.
                    let resolved = if std::path::Path::new(path).exists() {
                        path.clone()
                    } else {
                        let m1 = format!("lipi_modules/{path}");
                        let m2 = format!("lipi_modules/{path}.swami");
                        let m3 = format!("lipi_modules/{path}/{path}.swami");
                        [m1, m2, m3].into_iter().find(|p| std::path::Path::new(p).exists())
                            .unwrap_or_else(|| path.clone())
                    };
                    let src = std::fs::read_to_string(&resolved)
                        .map_err(|e| format!("फ़ाइल नहीं खुली '{}': {}", path, e))?;
                    let tokens = crate::lexer::tokenize(&src);
                    let stmts = crate::parser::parse(tokens)
                        .map_err(|e| format!("'{}' में व्याकरण त्रुटि: {}", path, e))?;
                    let prog = crate::compiler::Compiler::compile_program(&stmts);
                    // Merge imported functions and class parents
                    for (k, v) in prog.functions { self.functions.insert(k, v); }
                    for (k, v) in prog.class_parents { self.class_parents.insert(k, v); }
                    // Run top-level statements from the imported file
                    let sub_instrs = prog.instructions;
                    let mut sub_ip = 0usize;
                    while sub_ip < sub_instrs.len() {
                        let cur = sub_ip;
                        sub_ip += 1;
                        match self.exec_op(&sub_instrs[cur], &mut sub_ip, &sub_instrs) {
                            Ok(true) | Ok(false) => {}
                            Err(e) => return Err(e),
                        }
                    }
                }

                // ── Phase 12: Bitwise operations ────────────────────────────
                Opcode::BitAnd => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(vm_bitwise2(a, b, |x, y| x & y, "&")?);
                }
                Opcode::BitOr => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(vm_bitwise2(a, b, |x, y| x | y, "|")?);
                }
                Opcode::BitXor => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(vm_bitwise2(a, b, |x, y| x ^ y, "^")?);
                }
                Opcode::BitNot => {
                    let a = pop(&mut self.stack)?;
                    match a {
                        Value::Number(n) => self.stack.push(Value::Number(!(n as i64) as f64)),
                        other => return Err(format!("'~' के लिए संख्या चाहिए, मिला: {}", other)),
                    }
                }
                Opcode::LShift => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(vm_bitwise2(a, b, |x, y| x << (y as u32), "<<")?);
                }
                Opcode::RShift => {
                    let b = pop(&mut self.stack)?;
                    let a = pop(&mut self.stack)?;
                    self.stack.push(vm_bitwise2(a, b, |x, y| x >> (y as u32), ">>")?);
                }

                // ── Phase 13: Global variable declaration ──────────────────
                Opcode::DeclareGlobal(name) => {
                    if let Some(frame) = self.call_frames.last_mut() {
                        frame.global_names.insert(name.clone());
                    }
                }

                // ── Phase 15: Enum definition and matching ─────────────────

                Opcode::DefineEnum(name, variant_defs) => {
                    let mut variants = HashMap::new();
                    for (vname, arity) in variant_defs {
                        variants.insert(vname.clone(), *arity);
                    }
                    self.globals.insert(name.clone(), Value::EnumDef { name: name.clone(), variants });
                }

                Opcode::MatchVariant(vname) => {
                    // Peek top of stack (the enum value), push Bool result
                    let top = self.stack.last()
                        .ok_or_else(|| "MatchVariant: stack empty".to_string())?
                        .clone();
                    let matches = match &top {
                        Value::Enum { variant, .. } => variant == vname,
                        _ => false,
                    };
                    self.stack.push(Value::Bool(matches));
                }

                // ── Phase 17: Membership (में_है) ──────────────────────────
                Opcode::Contains => {
                    let container = pop(&mut self.stack)?;
                    let item = pop(&mut self.stack)?;
                    self.stack.push(Value::Bool(crate::interpreter::contains_value(&item, &container)?));
                }

                // ── Phase 17: Slice ────────────────────────────────────────
                Opcode::Slice => {
                    let step  = pop(&mut self.stack)?;
                    let end   = pop(&mut self.stack)?;
                    let start = pop(&mut self.stack)?;
                    let obj   = pop(&mut self.stack)?;
                    self.stack.push(crate::interpreter::slice_value(obj, start, end, step)?);
                }

                // ── Phase 17: Spread in list literals ──────────────────────
                Opcode::MakeListSp(flags) => {
                    let mut vals: Vec<Value> = (0..flags.len())
                        .map(|_| pop(&mut self.stack))
                        .collect::<Result<_, _>>()?;
                    vals.reverse(); // now in source order, aligned with flags
                    let mut out: Vec<Value> = Vec::new();
                    for (is_spread, v) in flags.iter().zip(vals.into_iter()) {
                        if *is_spread {
                            match v {
                                Value::List(items) => out.extend(items),
                                other => return Err(format!(
                                    "फैलाव (*) केवल सूची पर हो सकता है, मिला: {other}"
                                )),
                            }
                        } else {
                            out.push(v);
                        }
                        check_list_len(out.len())?;
                    }
                    self.stack.push(Value::List(out));
                }

                // ── Phase 17: Tuple unpacking ──────────────────────────────
                Opcode::UnpackList(n) => {
                    let val = pop(&mut self.stack)?;
                    match val {
                        Value::List(items) => {
                            if items.len() != *n {
                                return Err(format!(
                                    "खोलना विफल: {} नाम हैं पर सूची में {} मान", n, items.len()
                                ));
                            }
                            // Reverse push → following StoreVar ops bind left-to-right
                            for v in items.into_iter().rev() { self.stack.push(v); }
                        }
                        other => return Err(format!("खोलने के लिए सूची अपेक्षित, मिला: {}", other)),
                    }
                }

                Opcode::EnumUnpack(names) => {
                    // Pop enum from stack, store field values as locals
                    let val = pop(&mut self.stack)?;
                    match val {
                        Value::Enum { values, .. } => {
                            for (name, val) in names.iter().zip(values.into_iter()) {
                                if let Some(frame) = self.call_frames.last_mut() {
                                    frame.locals.insert(name.clone(), val);
                                } else {
                                    self.globals.insert(name.clone(), val);
                                }
                            }
                        }
                        other => return Err(format!("EnumUnpack: विकल्प मान अपेक्षित, मिला: {}", other)),
                    }
                }

                // ── Phase 15: Tail-call optimization ──────────────────────

                Opcode::TailCall(name, argc) => {
                    let mut args: Vec<Value> = Vec::with_capacity(*argc);
                    for _ in 0..*argc { args.push(pop(&mut self.stack)?); }
                    args.reverse();

                    // Look up the function (plain named function only)
                    let func = self.functions.get(name).cloned();
                    if let Some(func) = func {
                        // Reuse the current call frame: clear locals, inject new args
                        if let Some(frame) = self.call_frames.last_mut() {
                            let base = frame.base_stack_depth;
                            frame.locals.clear();
                            frame.global_names.clear();
                            frame.func_name = name.clone();

                            // Handle varargs
                            let n_plain = func.params.len();
                            for (param, arg) in func.params.iter().zip(args.iter()) {
                                frame.locals.insert(param.clone(), arg.clone());
                            }
                            fill_defaults(&mut frame.locals, &func, args.len());
                            if let Some(ref vararg_name) = func.vararg {
                                let extra: Vec<Value> = args.into_iter().skip(n_plain).collect();
                                frame.locals.insert(vararg_name.clone(), Value::List(extra));
                            }
                            // Trim stack back to base (discard any temporaries)
                            self.stack.truncate(base);
                        }
                        *ip = func.start_ip;
                    } else {
                        // Fall back to regular call + return for unknown/closure targets
                        for a in args { self.stack.push(a); }
                        return Err(format!("TailCall: विधि '{}' नहीं मिली", name));
                    }
                }

                // ── Phase 16: Nyaya assert + Samkhya const ─────────────────

                Opcode::Assert(msg) => {
                    let val = pop(&mut self.stack)?;
                    let ok = match &val {
                        Value::Bool(b)   => *b,
                        Value::Nil       => false,
                        Value::Number(n) => *n != 0.0,
                        _                => true,
                    };
                    if !ok {
                        let errmsg = msg.as_deref().unwrap_or("मान्यता असत्य निकली");
                        return Err(format!("जाँचो विफल: {} (Nyaya: प्रतिज्ञा असिद्ध)", errmsg));
                    }
                }

                Opcode::DeclareConst(name) => {
                    let val = pop(&mut self.stack)?;
                    self.constants.insert(name.clone());
                    self.globals.insert(name.clone(), val);
                }

                // ── Phase 10/11: First-class functions + true closures ─────
                Opcode::MakeClosure(func_name) => {
                    // Snapshot current frame locals so the lambda captures outer scope
                    let captured = self.call_frames.last()
                        .map(|f| f.locals.clone())
                        .unwrap_or_default();
                    self.stack.push(Value::Closure { func_name: func_name.clone(), captured });
                }

                // ── Phase 11: Iterable for-loop opcodes ────────────────────
                Opcode::GetIterLen => {
                    let val = pop(&mut self.stack)?;
                    let len = match &val {
                        Value::Number(n)  => *n,
                        Value::List(v)    => v.len() as f64,
                        Value::Str(s)     => s.chars().count() as f64,
                        Value::Dict(m)    => m.len() as f64,
                        other => return Err(format!("'{}' पर 'के लिए' नहीं चल सकता", other)),
                    };
                    self.stack.push(Value::Number(len));
                }

                // In-place for-loop step (Phase 17 perf) — no container clone
                Opcode::IterNext(val_var, idx_var) => {
                    let idx = match self.var_ref(idx_var) {
                        Some(Value::Number(n)) => *n as usize,
                        _ => return Err("IterNext: अनुक्रमणिका अमान्य".into()),
                    };
                    let item = match self.var_ref(val_var) {
                        Some(Value::Number(_)) => Value::Number(idx as f64),
                        Some(Value::List(v)) => v.get(idx).cloned()
                            .ok_or_else(|| format!("सूची सीमा से बाहर: {}", idx))?,
                        Some(Value::Str(s)) => {
                            let ch = s.chars().nth(idx)
                                .ok_or_else(|| format!("वाक्य सीमा से बाहर: {}", idx))?;
                            Value::Str(ch.to_string())
                        }
                        Some(Value::Dict(m)) => {
                            let mut keys: Vec<&String> = m.keys().collect();
                            keys.sort();
                            keys.get(idx)
                                .map(|k| Value::Str((*k).clone()))
                                .ok_or_else(|| format!("कोश सीमा से बाहर: {}", idx))?
                        }
                        Some(other) => return Err(format!("'{}' पर 'के लिए' नहीं चल सकता", other)),
                        None => return Err(format!("'{}' परिभाषित नहीं है", val_var)),
                    };
                    self.stack.push(item);
                }

                Opcode::GetIterItem => {
                    let idx_val = pop(&mut self.stack)?;
                    let iter_val = pop(&mut self.stack)?;
                    let idx = match &idx_val {
                        Value::Number(n) => *n as usize,
                        other => return Err(format!("अनुक्रमणिका संख्या होनी चाहिए, मिला: {}", other)),
                    };
                    let item = match iter_val {
                        Value::Number(_)  => idx_val,
                        Value::List(v)    => v.get(idx).cloned().ok_or_else(|| format!("सूची सीमा से बाहर: {}", idx))?,
                        Value::Str(s)     => {
                            let ch = s.chars().nth(idx).ok_or_else(|| format!("वाक्य सीमा से बाहर: {}", idx))?;
                            Value::Str(ch.to_string())
                        }
                        Value::Dict(m) => {
                            let mut keys: Vec<&String> = m.keys().collect();
                            keys.sort();
                            keys.get(idx)
                                .map(|k| Value::Str((*k).clone()))
                                .ok_or_else(|| format!("कोश सीमा से बाहर: {}", idx))?
                        }
                        other => return Err(format!("'{}' पर 'के लिए' नहीं चल सकता", other)),
                    };
                    self.stack.push(item);
                }

                Opcode::Yield => {
                    self.yielded = true;
                }

                Opcode::IterStep { loop_var, container_var, idx_var } => {
                    let produced = self.iter_step(loop_var, container_var, idx_var, instructions)?;
                    self.stack.push(Value::Bool(produced));
                }
            }
        Ok(false)
    }

    /// Call a closure value (Value::Closure) with given args.
    /// Runs the function body inline using a mini-loop (the sentinel return_addr trick).
    /// Returns the function's return value.
    fn call_closure_value(&mut self, func_ref: &Value, args: Vec<Value>, instructions: &[Opcode]) -> Result<Value, String> {
        let (func_name, captured) = match func_ref {
            Value::Closure { func_name, captured } => (func_name.clone(), captured.clone()),
            other => return Err(format!("'{other}' विधि नहीं है")),
        };

        // Functools wrappers (Phase 17) are not real functions — dispatch specially.
        if func_name.starts_with("__functools_") {
            return self.call_functools(&func_name, &captured, args, instructions);
        }

        let func = self.functions.get(&func_name).cloned()
            .ok_or_else(|| format!("विधि '{}' नहीं मिली", func_name))?;

        // Start with captured scope, then overlay actual arguments
        let mut locals = captured;
        let provided = args.len();
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            locals.insert(param.clone(), arg);
        }
        fill_defaults(&mut locals, &func, provided);

        // Sentinel return_addr: usize::MAX causes the mini-loop to exit when Return fires
        self.push_frame(Frame { return_addr: usize::MAX, locals, global_names: std::collections::HashSet::new(), base_stack_depth: self.stack.len(), func_name })?;
        let mut sub_ip = func.start_ip;

        loop {
            if sub_ip >= instructions.len() { break; }
            let cur = sub_ip;
            sub_ip += 1;
            match self.exec_op(&instructions[cur], &mut sub_ip, instructions) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            // After Return, sub_ip == usize::MAX (sentinel) → loop exits
            if sub_ip >= instructions.len() { break; }
        }

        Ok(self.stack.pop().unwrap_or(Value::Nil))
    }

    /// Dispatch a functools wrapper closure (संयोजित / आंशिक / स्मरण) — Phase 17.
    fn call_functools(&mut self, func_name: &str, captured: &HashMap<String, Value>,
                      args: Vec<Value>, instructions: &[Opcode]) -> Result<Value, String> {
        let f = captured.get("__f").cloned()
            .ok_or_else(|| "functools: आंतरिक त्रुटि (कोई फलन नहीं)".to_string())?;
        match func_name {
            "__functools_compose__" => {
                // f(g(args...))
                let g = captured.get("__g").cloned().unwrap_or(Value::Nil);
                let mid = self.call_closure_value(&g, args, instructions)?;
                self.call_closure_value(&f, vec![mid], instructions)
            }
            "__functools_partial__" => {
                // f(bound..., args...)
                let mut full = match captured.get("__bound") {
                    Some(Value::List(b)) => b.clone(),
                    _ => Vec::new(),
                };
                full.extend(args);
                self.call_closure_value(&f, full, instructions)
            }
            "__functools_memo__" => {
                let id = match captured.get("__id") { Some(Value::Number(n)) => *n as u64, _ => 0 };
                // Build a cache key from the argument values (structural repr)
                let key = args.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>().join("\u{1}");
                if let Some(cache) = self.memo_caches.get(&id) {
                    if let Some(hit) = cache.get(&key) { return Ok(hit.clone()); }
                }
                let val = self.call_closure_value(&f, args, instructions)?;
                if let Some(cache) = self.memo_caches.get_mut(&id) {
                    cache.insert(key, val.clone());
                }
                Ok(val)
            }
            other => Err(format!("अज्ञात functools wrapper: {other}")),
        }
    }
}

/// Pre-loaded constant globals (पाई, अनंत, …) — hidden from the debugger `vars`
/// listing so only user variables show.
fn is_builtin_global(name: &str) -> bool {
    matches!(name,
        "पाई" | "अनंत" | "ऋण_अनंत" | "शून्य" | "अरब" | "खरब" | "नील" | "शंख" | "पद्म" |
        "आर्यभट_पाई" | "आर्यभट_कोण" | "आर्यभट_ज्या_गणना" | "नक्षत्र_संख्या" |
        "तिथि_संख्या" | "युग_वर्ष" | "ब्रह्मगुप्त_शून्य")
}

// ── Built-in functions (always available, no import needed) ──────────────────

fn builtin_lambai(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(s))  => Ok(Value::Number(s.chars().count() as f64)),
        Some(Value::List(v)) => Ok(Value::Number(v.len() as f64)),
        Some(Value::Dict(m)) => Ok(Value::Number(m.len() as f64)),
        Some(other) => Err(format!("लम्बाई(): वाक्य/सूची/कोश अपेक्षित, मिला: {}", other)),
        None => Err("लम्बाई(): एक तर्क आवश्यक है".into()),
    }
}

fn builtin_padho(args: Vec<Value>) -> Result<Value, String> {
    // Check pre-loaded buffer first (used by WASM/web mode)
    let buffered = STDIN_BUF.with(|b| b.borrow_mut().pop_front());
    if let Some(line) = buffered {
        return Ok(Value::Str(line.trim_matches(['\n', '\r']).to_string()));
    }
    // WASM has no real stdin — return empty string instead of panicking
    #[cfg(target_arch = "wasm32")]
    return Ok(Value::Str(String::new()));
    // Native CLI: print optional prompt then read stdin
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        if let Some(prompt) = args.first() {
            print!("{}", prompt);
            std::io::stdout().flush().ok();
        }
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)
            .map_err(|e| format!("पढ़ो(): इनपुट त्रुटि — {e}"))?;
        let s = line.trim_matches(['\n', '\r', ' ', '\t', '\u{feff}']);
        Ok(Value::Str(s.to_string()))
    }
}

/// पूर्ण_है(x) — true if x is a whole-number value (no fractional part).
fn builtin_purna_hai(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(Value::Bool(n.fract() == 0.0 && n.is_finite())),
        Some(_) => Ok(Value::Bool(false)),
        None => Err("पूर्ण_है(): एक तर्क आवश्यक".to_string()),
    }
}

fn builtin_so(args: Vec<Value>) -> Result<Value, String> {
    let ms = match args.first() {
        Some(Value::Number(n)) => *n,
        _ => return Err("सोओ(): मिलिसेकंड (संख्या) अपेक्षित".to_string()),
    };
    let mut m = HashMap::new();
    m.insert("__नींद_मिलि__".to_string(), Value::Number(ms));
    Ok(Value::Dict(m))
}

/// सामान्यीकृत(text) — normalize Devanagari to NFC (decompose precomposed nukta
/// letters to base + ़). Makes equivalent spellings compare/lex identically.
fn builtin_samanyikrit(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::Str(s)) => Ok(Value::Str(crate::lexer::normalize_devanagari(&s))),
        Some(other) => Err(format!("सामान्यीकृत(): वाक्य अपेक्षित, मिला: {}", other)),
        None => Err("सामान्यीकृत(): एक तर्क आवश्यक".to_string()),
    }
}

fn builtin_purnankin(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(Value::Number(n.floor())),
        Some(Value::Str(s)) => s.trim().parse::<f64>()
            .map(|n| Value::Number(n.floor()))
            .map_err(|_| format!("पूर्णांक(): '{}' को संख्या में नहीं बदल सका", s)),
        Some(other) => Err(format!("पूर्णांक(): संख्या अपेक्षित, मिला: {}", other)),
        None => Err("पूर्णांक(): एक तर्क आवश्यक है".into()),
    }
}

// ── Stack helpers ─────────────────────────────────────────────────────────────

fn pop(stack: &mut Vec<Value>) -> Result<Value, String> {
    stack.pop().ok_or_else(|| "LVM: stack underflow".to_string())
}

fn pop_str(stack: &mut Vec<Value>) -> Result<String, String> {
    match pop(stack)? {
        Value::Str(s)  => Ok(s),
        Value::Number(n) => Ok(fmt_num(n)),
        Value::Bool(b) => Ok(if b { "सत्य".into() } else { "असत्य".into() }),
        other => Err(format!("वाक्य अपेक्षित, मिला: {}", other)),
    }
}

fn pop_num(stack: &mut Vec<Value>) -> Result<f64, String> {
    match pop(stack)? {
        Value::Number(n) => Ok(n),
        Value::Str(s) => s.trim().parse::<f64>()
            .map_err(|_| format!("'{}' संख्या नहीं है", s)),
        other => Err(format!("संख्या अपेक्षित, मिला: {}", other)),
    }
}

// ── Value helpers ─────────────────────────────────────────────────────────────

/// Fill missing trailing parameters with their default values (Phase 17).
/// `provided` = number of positional args actually passed (incl. यह for methods).
/// Bind positional + keyword args into call locals (Phase 17 keyword arguments).
/// Errors on: unknown keyword name, keyword for an already-positionally-bound
/// param, or a required param left without any value.
fn bind_args_kw(
    locals: &mut HashMap<String, Value>,
    func: &FuncDef,
    args: Vec<Value>,
    kwnames: &[String],
    kwvals: Vec<Value>,
    fname: &str,
) -> Result<(), String> {
    let regular = func.params.len();
    let pos_bound = args.len().min(regular);
    for (param, arg) in func.params.iter().zip(args.iter()) {
        locals.insert(param.clone(), arg.clone());
    }
    if let Some(ref vname) = func.vararg {
        locals.insert(vname.clone(), Value::List(args.into_iter().skip(regular).collect()));
    }
    for (kname, kval) in kwnames.iter().zip(kwvals) {
        match func.params.iter().position(|p| p == kname) {
            None => return Err(format!("'{}': अज्ञात कीवर्ड तर्क '{}'", fname, kname)),
            Some(idx) if idx < pos_bound => return Err(format!(
                "'{}': '{}' को स्थान और कीवर्ड दोनों से मान मिला", fname, kname)),
            Some(_) => { locals.insert(kname.clone(), kval); }
        }
    }
    for i in pos_bound..regular {
        let p = &func.params[i];
        if !locals.contains_key(p) {
            if let Some(Some(d)) = func.defaults.get(i) {
                locals.insert(p.clone(), lvm_to_val(d.clone()));
            } else {
                return Err(format!("'{}': पैरामीटर '{}' का मान नहीं मिला", fname, p));
            }
        }
    }
    Ok(())
}

fn fill_defaults(locals: &mut HashMap<String, Value>, func: &FuncDef, provided: usize) {
    for i in provided..func.params.len() {
        if let Some(Some(d)) = func.defaults.get(i) {
            locals.entry(func.params[i].clone()).or_insert_with(|| lvm_to_val(d.clone()));
        }
    }
}

fn lvm_to_val(v: LvmValue) -> Value {
    match v {
        LvmValue::Number(n) => Value::Number(n),
        LvmValue::Str(s)    => Value::Str(s),
        LvmValue::Bool(b)   => Value::Bool(b),
        LvmValue::Nil       => Value::Nil,
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b)           => *b,
        Value::Number(n)         => *n != 0.0,
        Value::Str(s)            => !s.is_empty(),
        Value::Nil               => false,
        Value::Function { .. }   => true,
        Value::NativeFunction(_) => true,
        Value::List(v)           => !v.is_empty(),
        Value::Dict(m)           => !m.is_empty(),
        Value::Instance { .. }   => true,
        Value::Closure { .. }    => true,
        Value::EnumDef { .. }    => true,
        Value::Enum { .. }       => true,
        Value::Generator(_)      => true,
    }
}

fn vals_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::Str(x), Value::Str(y))       => x == y,
        (Value::Bool(x), Value::Bool(y))     => x == y,
        (Value::Nil, Value::Nil)             => true,
        (Value::List(x), Value::List(y)) =>
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| vals_eq(a, b)),
        (Value::Dict(x), Value::Dict(y)) =>
            x.len() == y.len()
                && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| vals_eq(v, w))),
        (Value::Instance { class: c1, fields: f1 },
         Value::Instance { class: c2, fields: f2 }) =>
            c1 == c2 && f1.len() == f2.len()
                && f1.iter().all(|(k, v)| f2.get(k).is_some_and(|w| vals_eq(v, w))),
        (Value::Enum { enum_name: e1, variant: v1, values: a1 },
         Value::Enum { enum_name: e2, variant: v2, values: a2 }) =>
            e1 == e2 && v1 == v2 && a1.len() == a2.len()
                && a1.iter().zip(a2).all(|(a, b)| vals_eq(a, b)),
        _ => false,
    }
}

fn vm_add(a: Value, b: Value) -> Result<Value, String> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
        (Value::Str(x), Value::Str(y))       => Ok(Value::Str(x + &y)),
        (Value::Str(x), Value::Number(y))    => Ok(Value::Str(format!("{x}{}", fmt_num(y)))),
        (Value::Number(x), Value::Str(y))    => Ok(Value::Str(format!("{}{y}", fmt_num(x)))),
        (Value::Str(x), Value::Bool(y))      => Ok(Value::Str(
            format!("{x}{}", if y { "सत्य" } else { "असत्य" })
        )),
        (Value::List(mut a), Value::List(b)) => { a.extend(b); Ok(Value::List(a)) }
        (a, b) => Err(format!("जोड़ नहीं हो सकता: '{a}' + '{b}'")),
    }
}

fn vm_num2<F: Fn(f64, f64) -> f64>(a: Value, b: Value, op: F, sym: &str) -> Result<Value, String> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(op(x, y))),
        (a, b) => Err(format!("'{sym}' के लिए संख्याएं चाहिए, मिला: '{a}' और '{b}'")),
    }
}

fn vm_bitwise2<F: Fn(i64, i64) -> i64>(a: Value, b: Value, op: F, sym: &str) -> Result<Value, String> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Ok(Value::Number(op(x as i64, y as i64) as f64)),
        (a, b) => Err(format!("'{sym}' के लिए पूर्णांक चाहिए, मिला: '{a}' और '{b}'")),
    }
}

fn num_cmp<F: Fn(f64, f64) -> bool>(a: Value, b: Value, op: F, sym: &str) -> Result<bool, String> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => Ok(op(x, y)),
        (a, b) => Err(format!("'{sym}' के लिए संख्याएं चाहिए, मिला: '{a}' और '{b}'")),
    }
}

fn fmt_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) }
    else { format!("{n}") }
}

// ── Phase 7 builtins ──────────────────────────────────────────────────────────

/// वाक्य(val) — convert any value to a string
fn builtin_vakya(args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::Str(match args.first() {
        Some(v) => format!("{v}"),
        None    => String::new(),
    }))
}

/// निर्गम(code) — exit the program
fn builtin_nirgam(args: Vec<Value>) -> Result<Value, String> {
    let code = match args.first() {
        Some(Value::Number(n)) => *n as i32,
        _ => 0,
    };
    std::process::exit(code);
}

// LCG random number generator (no external crates)
static RAND_STATE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn init_rand() {
    #[cfg(not(target_arch = "wasm32"))]
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42);
    #[cfg(target_arch = "wasm32")]
    let seed = RAND_STATE.load(std::sync::atomic::Ordering::Relaxed).wrapping_add(0x9E3779B97F4A7C15);
    RAND_STATE.store(seed, std::sync::atomic::Ordering::Relaxed);
}

/// बीज_सेट(n) — seed the PRNG to a fixed value for reproducible/deterministic
/// runs (verification & validation). After this, यादृच्छिक is deterministic.
fn builtin_beej_set(args: Vec<Value>) -> Result<Value, String> {
    let seed = match args.first() {
        Some(Value::Number(n)) => *n as u64,
        _ => return Err("बीज_सेट(): संख्या अपेक्षित".to_string()),
    };
    RAND_STATE.store(seed.wrapping_add(1), std::sync::atomic::Ordering::Relaxed);
    Ok(Value::Bool(true))
}

fn next_rand() -> u64 {
    let s = RAND_STATE.load(std::sync::atomic::Ordering::Relaxed);
    let next = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    RAND_STATE.store(next, std::sync::atomic::Ordering::Relaxed);
    next
}

/// यूआईडी() — random UUID v4 string (Phase 17). Uses the same PRNG as यादृच्छिक.
fn builtin_uuid(_args: Vec<Value>) -> Result<Value, String> {
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        let r = next_rand().to_le_bytes();
        for (b, &rb) in chunk.iter_mut().zip(r.iter()) { *b = rb; }
    }
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // variant 10
    let h: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(Value::Str(format!(
        "{}-{}-{}-{}-{}",
        &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32]
    )))
}

/// युग्म(सूची1, सूची2, ...) — zip: pairwise combine lists into a list of lists,
/// truncated to the shortest input (Phase 17).
fn builtin_zip(args: Vec<Value>) -> Result<Value, String> {
    let mut lists: Vec<&Vec<Value>> = Vec::new();
    for a in &args {
        match a {
            Value::List(l) => lists.push(l),
            _ => return Err("युग्म(): सभी तर्क सूची होने चाहिए".into()),
        }
    }
    if lists.is_empty() { return Ok(Value::List(vec![])); }
    let n = lists.iter().map(|l| l.len()).min().unwrap_or(0);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(Value::List(lists.iter().map(|l| l[i].clone()).collect()));
    }
    Ok(Value::List(out))
}

/// श्रृंखला(सूची1, सूची2, ...) — chain: concatenate lists into one (Phase 17).
fn builtin_chain(args: Vec<Value>) -> Result<Value, String> {
    let mut out = Vec::new();
    for a in args {
        match a {
            Value::List(l) => out.extend(l),
            _ => return Err("श्रृंखला(): सभी तर्क सूची होने चाहिए".into()),
        }
    }
    Ok(Value::List(out))
}

// ── Queue/Deque helpers (Phase 17) ──────────────────────────────────────────────
// LIPI lists are copy-on-write, so these return NEW lists (and the popped
// element where relevant). Errors are catchable Hindi strings.

/// अग्र_जोड़ो(सूची, मान) — new list with मान prepended (push to front).
fn builtin_agra_jodo(args: Vec<Value>) -> Result<Value, String> {
    let mut it = args.into_iter();
    let list = it.next().ok_or_else(|| "अग्र_जोड़ो(): दो तर्क आवश्यक (सूची, मान)".to_string())?;
    let val = it.next().ok_or_else(|| "अग्र_जोड़ो(): दो तर्क आवश्यक (सूची, मान)".to_string())?;
    match list {
        Value::List(mut v) => {
            check_list_len(v.len() + 1)?;
            v.insert(0, val);
            Ok(Value::List(v))
        }
        other => Err(format!("अग्र_जोड़ो(): सूची अपेक्षित, मिला: {}", other)),
    }
}

/// अग्र(सूची) — first element (error on empty).
fn builtin_agra(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(v)) => v.into_iter().next()
            .ok_or_else(|| "अग्र(): सूची खाली है".to_string()),
        Some(other) => Err(format!("अग्र(): सूची अपेक्षित, मिला: {}", other)),
        None => Err("अग्र(): एक तर्क आवश्यक".to_string()),
    }
}

/// पश्च(सूची) — last element (error on empty).
fn builtin_pashcha(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(v)) => v.into_iter().next_back()
            .ok_or_else(|| "पश्च(): सूची खाली है".to_string()),
        Some(other) => Err(format!("पश्च(): सूची अपेक्षित, मिला: {}", other)),
        None => Err("पश्च(): एक तर्क आवश्यक".to_string()),
    }
}

/// अग्र_हटाओ(सूची) — new list without the first element (pop from front).
fn builtin_agra_hatao(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(mut v)) => {
            if v.is_empty() { return Err("अग्र_हटाओ(): सूची खाली है".to_string()); }
            v.remove(0);
            Ok(Value::List(v))
        }
        Some(other) => Err(format!("अग्र_हटाओ(): सूची अपेक्षित, मिला: {}", other)),
        None => Err("अग्र_हटाओ(): एक तर्क आवश्यक".to_string()),
    }
}

/// पश्च_हटाओ(सूची) — new list without the last element (pop from back).
fn builtin_pashcha_hatao(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(mut v)) => {
            if v.is_empty() { return Err("पश्च_हटाओ(): सूची खाली है".to_string()); }
            v.pop();
            Ok(Value::List(v))
        }
        Some(other) => Err(format!("पश्च_हटाओ(): सूची अपेक्षित, मिला: {}", other)),
        None => Err("पश्च_हटाओ(): एक तर्क आवश्यक".to_string()),
    }
}

// ── OrderedDict (Phase 17) ───────────────────────────────────────────────────
// Insertion-ordered map represented as a List of [key, value] pairs. Pure builtins,
// copy-on-write (return new value). Keys compared by their Display form, matching
// how Value::Dict stores keys. Unlike कोश, iteration order = insertion order.

/// क्रमित_कोश() — new empty ordered dict (an empty pair-list).
fn builtin_kramit_kosh(_args: Vec<Value>) -> Result<Value, String> {
    Ok(Value::List(Vec::new()))
}

/// क्रमित_रखो(od, कुंजी, मान) — set: update in place if key exists, else append.
fn builtin_kramit_rakho(args: Vec<Value>) -> Result<Value, String> {
    let mut it = args.into_iter();
    let od = it.next().ok_or_else(|| "क्रमित_रखो(): तीन तर्क आवश्यक (क्रमित_कोश, कुंजी, मान)".to_string())?;
    let key = it.next().ok_or_else(|| "क्रमित_रखो(): कुंजी आवश्यक".to_string())?;
    let val = it.next().ok_or_else(|| "क्रमित_रखो(): मान आवश्यक".to_string())?;
    let key_s = format!("{key}");
    match od {
        Value::List(mut pairs) => {
            for p in pairs.iter_mut() {
                if let Value::List(kv) = p {
                    if kv.first().map(|k| format!("{k}")) == Some(key_s.clone()) {
                        if kv.len() >= 2 { kv[1] = val; } else { kv.push(val); }
                        return Ok(Value::List(pairs));
                    }
                }
            }
            check_list_len(pairs.len() + 1)?;
            pairs.push(Value::List(vec![key, val]));
            Ok(Value::List(pairs))
        }
        other => Err(format!("क्रमित_रखो(): क्रमित_कोश अपेक्षित, मिला: {}", other)),
    }
}

/// क्रमित_पाओ(od, कुंजी) — value for key, or शून्य if absent.
fn builtin_kramit_pao(args: Vec<Value>) -> Result<Value, String> {
    let mut it = args.into_iter();
    let od = it.next().ok_or_else(|| "क्रमित_पाओ(): दो तर्क आवश्यक (क्रमित_कोश, कुंजी)".to_string())?;
    let key = it.next().ok_or_else(|| "क्रमित_पाओ(): कुंजी आवश्यक".to_string())?;
    let key_s = format!("{key}");
    match od {
        Value::List(pairs) => {
            for p in &pairs {
                if let Value::List(kv) = p {
                    if kv.first().map(|k| format!("{k}")) == Some(key_s.clone()) {
                        return Ok(kv.get(1).cloned().unwrap_or(Value::Nil));
                    }
                }
            }
            Ok(Value::Nil)
        }
        other => Err(format!("क्रमित_पाओ(): क्रमित_कोश अपेक्षित, मिला: {}", other)),
    }
}

/// क्रमित_कुंजियाँ(od) — keys in insertion order.
fn builtin_kramit_kunjiyan(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(pairs)) => Ok(Value::List(
            pairs.into_iter().filter_map(|p| match p {
                Value::List(kv) => kv.into_iter().next(),
                _ => None,
            }).collect()
        )),
        Some(other) => Err(format!("क्रमित_कुंजियाँ(): क्रमित_कोश अपेक्षित, मिला: {}", other)),
        None => Err("क्रमित_कुंजियाँ(): एक तर्क आवश्यक".to_string()),
    }
}

/// क्रमित_मान(od) — values in insertion order.
fn builtin_kramit_maan(args: Vec<Value>) -> Result<Value, String> {
    match args.into_iter().next() {
        Some(Value::List(pairs)) => Ok(Value::List(
            pairs.into_iter().filter_map(|p| match p {
                Value::List(kv) => kv.into_iter().nth(1),
                _ => None,
            }).collect()
        )),
        Some(other) => Err(format!("क्रमित_मान(): क्रमित_कोश अपेक्षित, मिला: {}", other)),
        None => Err("क्रमित_मान(): एक तर्क आवश्यक".to_string()),
    }
}

/// गिनती_कोश(सूची) — Counter: dict mapping each element (as string) to its
/// occurrence count (Phase 17).
fn builtin_counter(args: Vec<Value>) -> Result<Value, String> {
    let items = match args.first() {
        Some(Value::List(l)) => l.clone(),
        Some(Value::Str(s))  => s.chars().map(|c| Value::Str(c.to_string())).collect(),
        _ => return Err("गिनती_कोश(): सूची या वाक्य अपेक्षित".into()),
    };
    let mut counts: HashMap<String, Value> = HashMap::new();
    for v in items {
        let key = format!("{v}");
        let n = match counts.get(&key) {
            Some(Value::Number(c)) => *c + 1.0,
            _ => 1.0,
        };
        counts.insert(key, Value::Number(n));
    }
    Ok(Value::Dict(counts))
}

/// कार्तीय(सूची1, सूची2, ...) — Cartesian product → list of lists (Phase 17).
fn builtin_product(args: Vec<Value>) -> Result<Value, String> {
    let mut lists: Vec<Vec<Value>> = Vec::new();
    for a in args {
        match a {
            Value::List(l) => lists.push(l),
            _ => return Err("कार्तीय(): सभी तर्क सूची होने चाहिए".into()),
        }
    }
    let mut acc: Vec<Vec<Value>> = vec![vec![]];
    for list in &lists {
        let mut next = Vec::new();
        for prefix in &acc {
            for item in list {
                let mut row = prefix.clone();
                row.push(item.clone());
                next.push(row);
            }
        }
        acc = next;
    }
    Ok(Value::List(acc.into_iter().map(Value::List).collect()))
}

/// सर्व_संयोजन(सूची, r) — all r-length combinations → list of lists (Phase 17).
fn builtin_combinations(args: Vec<Value>) -> Result<Value, String> {
    let items = match args.first() {
        Some(Value::List(l)) => l.clone(),
        _ => return Err("सर्व_संयोजन(): पहला तर्क सूची होना चाहिए".into()),
    };
    let r = match args.get(1) {
        Some(Value::Number(n)) if *n >= 0.0 => *n as usize,
        _ => return Err("सर्व_संयोजन(): दूसरा तर्क धनात्मक संख्या होना चाहिए".into()),
    };
    let mut out: Vec<Value> = Vec::new();
    if r <= items.len() {
        let mut idx: Vec<usize> = (0..r).collect();
        loop {
            out.push(Value::List(idx.iter().map(|&i| items[i].clone()).collect()));
            if r == 0 { break; }
            // advance the combination indices
            let mut i = r;
            loop {
                if i == 0 { return Ok(Value::List(out)); }
                i -= 1;
                if idx[i] != i + items.len() - r { break; }
            }
            idx[i] += 1;
            for j in i + 1..r { idx[j] = idx[j - 1] + 1; }
        }
    }
    Ok(Value::List(out))
}

/// गणना(सूची [, शुरू]) — enumerate: list of [index, item] pairs (Phase 17).
fn builtin_ganana(args: Vec<Value>) -> Result<Value, String> {
    let items = match args.first() {
        Some(Value::List(l)) => l.clone(),
        Some(Value::Str(s))  => s.chars().map(|c| Value::Str(c.to_string())).collect(),
        _ => return Err("गणना(): सूची या वाक्य अपेक्षित".into()),
    };
    let start = match args.get(1) {
        Some(Value::Number(n)) => *n as i64,
        None => 0,
        _ => return Err("गणना(): दूसरा तर्क संख्या होना चाहिए".into()),
    };
    let out = items.into_iter().enumerate()
        .map(|(i, v)| Value::List(vec![Value::Number((start + i as i64) as f64), v]))
        .collect();
    Ok(Value::List(out))
}

/// यादृच्छिक(n) — random integer in 0..n-1
fn builtin_yadrchik(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) if *n > 0.0 => {
            Ok(Value::Number((next_rand() % (*n as u64)) as f64))
        }
        _ => Err("यादृच्छिक(): धनात्मक संख्या अपेक्षित".into()),
    }
}

// ── Math builtins ─────────────────────────────────────────────────────────────

fn builtin_nirapeksh(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(Value::Number(n.abs())),
        _ => Err("निरपेक्ष(): संख्या अपेक्षित".into()),
    }
}

fn builtin_ghaat(args: Vec<Value>) -> Result<Value, String> {
    match (args.first(), args.get(1)) {
        (Some(Value::Number(b)), Some(Value::Number(e))) => Ok(Value::Number(b.powf(*e))),
        _ => Err("घात(आधार, घातांक): दो संख्याएं आवश्यक".into()),
    }
}

fn builtin_vargmool(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) if *n >= 0.0 => Ok(Value::Number(n.sqrt())),
        Some(Value::Number(_)) => Err("वर्गमूल(): ऋणात्मक संख्या का वर्गमूल संभव नहीं".into()),
        _ => Err("वर्गमूल(): संख्या अपेक्षित".into()),
    }
}

fn builtin_gol(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Number(n)) => Ok(Value::Number(n.round())),
        _ => Err("गोल(): संख्या अपेक्षित".into()),
    }
}

// ── File I/O builtins ─────────────────────────────────────────────────────────

fn builtin_sanchika_samagri(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(path)) => {
            std::fs::read_to_string(path)
                .map(Value::Str)
                .map_err(|e| format!("संचिका_सामग्री(): '{}' नहीं पढ़ी — {e}", path))
        }
        _ => Err("संचिका_सामग्री(पथ): वाक्य अपेक्षित".into()),
    }
}

fn builtin_sanchika_likho(args: Vec<Value>) -> Result<Value, String> {
    match (args.first(), args.get(1)) {
        (Some(Value::Str(path)), Some(content)) => {
            let text = format!("{content}");
            std::fs::write(path, &text)
                .map_err(|e| format!("संचिका_लिखो(): '{}' में नहीं लिखा — {e}", path))?;
            Ok(Value::Bool(true))
        }
        _ => Err("संचिका_लिखो(पथ, सामग्री): दो तर्क आवश्यक".into()),
    }
}

fn builtin_sanchika_hai(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(path)) => Ok(Value::Bool(std::path::Path::new(path).exists())),
        _ => Err("संचिका_है(पथ): वाक्य अपेक्षित".into()),
    }
}

// ── File-system + OS/environment builtins (Phase 17) ─────────────────────────

/// CLI arguments passed after the script name (`lipi foo.swami a b c` → ["a","b","c"]).
/// Set once by main.rs before the VM runs; तर्क() reads it.
static SCRIPT_ARGS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

pub fn set_script_args(args: Vec<String>) { let _ = SCRIPT_ARGS.set(args); }

fn builtin_folder_suchi(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(path)) => {
            let rd = std::fs::read_dir(path)
                .map_err(|e| format!("फोल्डर_सूची(): '{}' नहीं खुला — {e}", path))?;
            let mut names: Vec<String> = Vec::new();
            for entry in rd {
                let entry = entry
                    .map_err(|e| format!("फोल्डर_सूची(): '{}' पढ़ने में त्रुटि — {e}", path))?;
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
            names.sort();
            Ok(Value::List(names.into_iter().map(Value::Str).collect()))
        }
        _ => Err("फोल्डर_सूची(पथ): वाक्य अपेक्षित".into()),
    }
}

fn builtin_folder_banao(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(path)) => Ok(Value::Bool(std::fs::create_dir_all(path).is_ok())),
        _ => Err("फोल्डर_बनाओ(पथ): वाक्य अपेक्षित".into()),
    }
}

fn builtin_file_hatao(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(path)) => Ok(Value::Bool(std::fs::remove_file(path).is_ok())),
        _ => Err("फाइल_हटाओ(पथ): वाक्य अपेक्षित".into()),
    }
}

fn builtin_file_copy(args: Vec<Value>) -> Result<Value, String> {
    match (args.first(), args.get(1)) {
        (Some(Value::Str(src)), Some(Value::Str(dst))) => {
            Ok(Value::Bool(std::fs::copy(src, dst).is_ok()))
        }
        _ => Err("फाइल_कॉपी(स्रोत, गंतव्य): दो वाक्य आवश्यक".into()),
    }
}

fn builtin_path_jodo(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("पथ_जोड़ो(अ, ब, ...): कम से कम दो वाक्य आवश्यक".into());
    }
    let mut buf = std::path::PathBuf::new();
    for a in &args {
        match a {
            Value::Str(s) => buf.push(s),
            _ => return Err("पथ_जोड़ो(): सभी तर्क वाक्य होने चाहिए".into()),
        }
    }
    Ok(Value::Str(buf.to_string_lossy().into_owned()))
}

fn builtin_paryavaran(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Str(name)) => match std::env::var(name) {
            Ok(v) => Ok(Value::Str(v)),
            Err(_) => Ok(Value::Nil),
        },
        _ => Err("पर्यावरण(नाम): वाक्य अपेक्षित".into()),
    }
}

fn builtin_vartamaan_folder(_args: Vec<Value>) -> Result<Value, String> {
    std::env::current_dir()
        .map(|p| Value::Str(p.to_string_lossy().into_owned()))
        .map_err(|e| format!("वर्तमान_फोल्डर(): नहीं मिला — {e}"))
}

fn builtin_tark(_args: Vec<Value>) -> Result<Value, String> {
    let list = SCRIPT_ARGS
        .get()
        .map(|v| v.iter().map(|s| Value::Str(s.clone())).collect())
        .unwrap_or_default();
    Ok(Value::List(list))
}

// ── String formatting ─────────────────────────────────────────────────────────
//
// स्वरूप(fmt, val1, val2, ...)
//
// Format specifiers inside `{}`:
//   {}        — value as-is
//   {:.N}     — N decimal places  (e.g. {:.2} → "3.14")
//   {:N}      — minimum width, right-aligned  (e.g. {:8} → "      42")
//   {:0N}     — zero-padded width  (e.g. {:05} → "00042")
//   {:%}      — percentage (multiply by 100, append %)

fn builtin_swaroop(args: Vec<Value>) -> Result<Value, String> {
    let fmt = match args.first() {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err("स्वरूप(): पहला तर्क प्रारूप वाक्य होना चाहिए".into()),
    };

    let mut result = String::new();
    let mut arg_idx = 1usize;
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() {
            // Find closing }
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '}' { j += 1; }
            let spec: String = chars[i+1..j].iter().collect();
            i = j + 1;

            let val = args.get(arg_idx).cloned().unwrap_or(Value::Nil);
            arg_idx += 1;

            let formatted = format_spec(&val, &spec)?;
            result.push_str(&formatted);
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Ok(Value::Str(result))
}

fn format_spec(val: &Value, spec: &str) -> Result<String, String> {
    if spec.is_empty() {
        return Ok(format!("{val}"));
    }
    // {:.N} — decimal places
    if let Some(rest) = spec.strip_prefix(":.") {
        if let Ok(places) = rest.parse::<usize>() {
            if let Value::Number(n) = val {
                return Ok(format!("{:.prec$}", n, prec = places));
            }
            return Ok(format!("{val}"));
        }
    }
    // {:%} — percentage
    if spec == "%" || spec == ":%" {
        if let Value::Number(n) = val {
            return Ok(format!("{:.2}%", n * 100.0));
        }
        return Ok(format!("{val}%"));
    }
    // {:,} — Indian number formatting (12,34,567)
    if spec == "," || spec == ":," {
        if let Value::Number(n) = val {
            return Ok(format_indian_number(*n, false));
        }
        return Ok(format!("{val}"));
    }
    // {:₹} — Indian Rupee formatting (₹12,34,567)
    if spec == "₹" || spec == ":₹" {
        if let Value::Number(n) = val {
            return Ok(format_indian_number(*n, true));
        }
        return Ok(format!("{val}"));
    }
    // {:0N} — zero-padded width
    if let Some(rest) = spec.strip_prefix(':') {
        let zero_pad = rest.starts_with('0');
        let width_str = rest.trim_start_matches('0');
        if let Ok(width) = width_str.parse::<usize>() {
            let s = format!("{val}");
            if zero_pad {
                if let Value::Number(n) = val {
                    return Ok(format!("{:0>width$}", fmt_num(*n), width = width));
                }
                return Ok(format!("{:0>width$}", s, width = width));
            }
            return Ok(format!("{:>width$}", s, width = width));
        }
    }
    // Fallback
    Ok(format!("{val}"))
}

/// Format a number using Indian comma grouping: last 3 digits, then groups of 2.
/// E.g. 1234567 → "12,34,567"  or  ₹12,34,567
fn format_indian_number(n: f64, rupee_prefix: bool) -> String {
    let negative = n < 0.0;
    let abs_n = n.abs();
    let integer_part = abs_n.trunc() as u64;
    let frac = abs_n.fract();

    // Build the integer string with Indian commas
    let digits = integer_part.to_string();
    let grouped = if digits.len() <= 3 {
        digits.clone()
    } else {
        let mut result = String::new();
        let len = digits.len();
        // Last 3 digits
        result.insert_str(0, &digits[len - 3..]);
        let mut remaining = len - 3;
        while remaining > 0 {
            let take = if remaining >= 2 { 2 } else { 1 };
            let start = remaining - take;
            result.insert(0, ',');
            result.insert_str(0, &digits[start..remaining]);
            remaining = start;
        }
        result
    };

    let frac_str = if frac > 0.0 {
        // Round to 2 decimal places for display
        format!("{:.2}", frac).trim_start_matches('0').to_string()
    } else {
        String::new()
    };

    let prefix = if rupee_prefix { "₹" } else { "" };
    let sign = if negative { "-" } else { "" };
    format!("{}{}{}{}", sign, prefix, grouped, frac_str)
}

// ── Type inspection ───────────────────────────────────────────────────────────

fn builtin_prakar(args: Vec<Value>) -> Result<Value, String> {
    let name = match args.first() {
        Some(Value::Number(_))         => "संख्या",
        Some(Value::Str(_))            => "वाक्य",
        Some(Value::Bool(_))           => "सत्य_असत्य",
        Some(Value::List(_))           => "सूची",
        Some(Value::Dict(_))           => "कोश",
        Some(Value::Instance { .. })   => "वस्तु",
        Some(Value::Nil) | None        => "शून्य",
        Some(Value::Function { .. })   => "विधि",
        Some(Value::NativeFunction(_)) => "विधि",
        Some(Value::Closure { .. })    => "विधि",
        Some(Value::EnumDef { .. })    => "विकल्प_प्रकार",
        Some(Value::Enum { .. })       => "विकल्प",
        Some(Value::Generator(_))      => "जनित्र",
    };
    Ok(Value::Str(name.to_string()))
}
