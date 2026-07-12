//! LIPI bytecode serializer — save/load .libc files.
//!
//! File format
//!   [0..4]  b"LIPI"          — magic
//!   [4]     u8 = 3           — format version (v3 adds param defaults; v2 still loads)
//!   [5..9]  u32 LE           — function count
//!   for each function:
//!     u32 name_len + name bytes (UTF-8)
//!     u8  param_count
//!     for each param:
//!       u32 len + bytes
//!     for each param (v3 only):
//!       u8 default flag (0 = required, 1 = default follows as PUSH_*-tagged value)
//!     u8  vararg flag (+ name if 1)
//!     u64 start_ip (LE)
//!   u32 LE (v3 only)         — class parent count, then child/parent str pairs
//!   u32 LE                   — instruction count
//!   for each instruction:
//!     u8 opcode tag + payload (see TAG_* constants)

use std::collections::HashMap;
use crate::opcode::{CompiledProgram, FuncDef, LvmValue, Opcode};

// ── Opcode tags ───────────────────────────────────────────────────────────────
const TAG_PUSH_NIL:       u8 = 0x00;
const TAG_PUSH_FALSE:     u8 = 0x01;
const TAG_PUSH_TRUE:      u8 = 0x02;
const TAG_PUSH_NUM:       u8 = 0x03; // + 8 bytes f64 LE
const TAG_PUSH_STR:       u8 = 0x04; // + u32 len + bytes
const TAG_POP:            u8 = 0x05;
const TAG_DUP:            u8 = 0x06;
const TAG_LOAD_VAR:       u8 = 0x07; // + u32 len + bytes
const TAG_STORE_VAR:      u8 = 0x08; // + u32 len + bytes
const TAG_ADD:            u8 = 0x09;
const TAG_SUB:            u8 = 0x0A;
const TAG_MUL:            u8 = 0x0B;
const TAG_DIV:            u8 = 0x0C;
const TAG_MOD:            u8 = 0x0D;
const TAG_EQ:             u8 = 0x0E;
const TAG_NEQ:            u8 = 0x0F;
const TAG_GT:             u8 = 0x10;
const TAG_LT:             u8 = 0x11;
const TAG_GTEQ:           u8 = 0x12;
const TAG_LTEQ:           u8 = 0x13;
const TAG_AND:            u8 = 0x14;
const TAG_OR:             u8 = 0x15;
const TAG_NOT:            u8 = 0x16;
const TAG_JUMP:           u8 = 0x17; // + u32 addr
const TAG_JUMP_IF_FALSE:  u8 = 0x18; // + u32 addr
const TAG_JUMP_IF_TRUE:   u8 = 0x19; // + u32 addr
const TAG_CALL:           u8 = 0x1A; // + u32 name_len + name + u8 argc
const TAG_CALL_NATIVE:    u8 = 0x1B; // + u32 name_len + name + u8 argc
const TAG_RETURN:         u8 = 0x1C;
const TAG_PRINT:          u8 = 0x1D;
const TAG_KARAKA_CHECK:   u8 = 0x1E; // + u32 len + str + u32 len + str
const TAG_IMPORT:         u8 = 0x1F; // + u32 len + bytes
const TAG_AADHAAR_VERIFY: u8 = 0x20;
const TAG_UPI_SEND:       u8 = 0x21;
const TAG_GST_ADD:        u8 = 0x22;
const TAG_LAKH_PARSE:     u8 = 0x23;
const TAG_RUPEE_FORMAT:   u8 = 0x24;
const TAG_METHOD_CALL:    u8 = 0x25; // + u32 name_len + name + u8 argc
const TAG_MAKE_LIST:      u8 = 0x26; // + u32 count
const TAG_MAKE_DICT:      u8 = 0x27; // + u32 count
const TAG_GET_INDEX:      u8 = 0x28;
const TAG_SET_INDEX:      u8 = 0x29;
const TAG_MAKE_INSTANCE:  u8 = 0x2A; // + u32 class_name_len + bytes
const TAG_GET_ATTR:       u8 = 0x2B; // + u32 field_len + bytes
const TAG_SET_ATTR:       u8 = 0x2C; // + u32 field_len + bytes
const TAG_PRINT_INLINE:   u8 = 0x2D; // print without newline (लिखो)
const TAG_TRY_START:      u8 = 0x2E; // + u32 handler_ip
const TAG_TRY_END:        u8 = 0x2F;
const TAG_IMPORT_FILE:    u8 = 0x30; // + u32 path_len + bytes
const TAG_MAKE_CLOSURE:   u8 = 0x31; // + u32 name_len + bytes  (Phase 10)
const TAG_GET_ITER_LEN:   u8 = 0x32; // no payload               (Phase 11)
const TAG_GET_ITER_ITEM:  u8 = 0x33; // no payload               (Phase 11)
const TAG_BIT_AND:        u8 = 0x34; // no payload               (Phase 12)
const TAG_BIT_OR:         u8 = 0x35;
const TAG_BIT_XOR:        u8 = 0x36;
const TAG_BIT_NOT:        u8 = 0x37;
const TAG_LSHIFT:         u8 = 0x38;
const TAG_RSHIFT:         u8 = 0x39;
const TAG_DECLARE_GLOBAL: u8 = 0x3A; // + u32 name_len + bytes    (Phase 13)
const TAG_DEFINE_ENUM:    u8 = 0x3B; // + str name + u32 count + (str variant, u32 arity)*  (Phase 15)
const TAG_MATCH_VARIANT:  u8 = 0x3C; // + str variant_name       (Phase 15)
const TAG_ENUM_UNPACK:    u8 = 0x3D; // + u32 count + str* names  (Phase 15)
const TAG_TAIL_CALL:      u8 = 0x3E; // + str name + u8 argc      (Phase 15)
const TAG_ASSERT:         u8 = 0x3F; // + u8 has_msg + [str msg]  (Phase 16)
const TAG_DECLARE_CONST:  u8 = 0x40; // + str name                (Phase 16)
const TAG_CALL_KW:        u8 = 0x41; // + str name + u8 pos_argc + u8 kw_count + str* kwnames (Phase 17)
const TAG_FLOOR_DIV:      u8 = 0x42; // no payload — `//` floor division (Phase 17)
const TAG_UNPACK_LIST:    u8 = 0x43; // + u8 count — tuple unpacking (Phase 17)
const TAG_SLICE:          u8 = 0x44; // no payload — obj[start:end:step] (Phase 17)
const TAG_CONTAINS:       u8 = 0x45; // no payload — में_है membership (Phase 17)
const TAG_MAKE_LIST_SP:   u8 = 0x46; // + u8 count + count× u8 flag (0/1) — spread list literal (Phase 17)
const TAG_THROW:          u8 = 0x47; // no payload — फेंको / rethrow (Phase 17A typed exceptions)
const TAG_MATCH_ERR_CLASS: u8 = 0x48; // + str class_name — typed पकड़ो dispatch (Phase 17A)
const TAG_ITER_NEXT:      u8 = 0x49; // + str container_var + str index_var — in-place loop step (Phase 17 perf)
const TAG_YIELD:          u8 = 0x4A; // no payload — उत्पन्न (Phase 18 generators)
const TAG_ITER_STEP:      u8 = 0x4B; // + str loop_var + str container_var + str idx_var (Phase 18)
const TAG_METHOD_CALL_KW: u8 = 0x4C; // + str method + u8 pos_argc + u8 kw_count + str* kwnames (Phase 18)
const TAG_SET_SLICE:      u8 = 0x4D;

// ── Public API ────────────────────────────────────────────────────────────────

/// Serialize a compiled program to a .libc file.
pub fn save(program: &CompiledProgram, path: &str) -> Result<(), String> {
    let mut buf: Vec<u8> = Vec::new();

    // Magic + version (v5: per-function is_generator flag — Phase 18)
    buf.extend_from_slice(b"LIPI");
    write_u8(&mut buf, 5);

    // Function table (sorted by name for determinism)
    let mut funcs: Vec<(&String, &FuncDef)> = program.functions.iter().collect();
    funcs.sort_by_key(|(n, _)| n.as_str());

    write_u32(&mut buf, funcs.len() as u32);
    for (name, def) in &funcs {
        write_str(&mut buf, name);
        write_u8(&mut buf, def.params.len() as u8);
        for p in &def.params { write_str(&mut buf, p); }
        // v3: per-param defaults — 0 = required, 1 = has default (followed by value)
        for i in 0..def.params.len() {
            match def.defaults.get(i).and_then(|d| d.as_ref()) {
                None    => write_u8(&mut buf, 0),
                Some(v) => { write_u8(&mut buf, 1); write_lvm_value(&mut buf, v); }
            }
        }
        // vararg: 0 = none, 1 = has vararg (followed by name)
        match &def.vararg {
            None    => write_u8(&mut buf, 0),
            Some(v) => { write_u8(&mut buf, 1); write_str(&mut buf, v); }
        }
        write_u64(&mut buf, def.start_ip as u64);
        write_u8(&mut buf, if def.is_generator { 1 } else { 0 });
    }

    // v3: class inheritance table (child → parent), sorted for determinism
    let mut parents: Vec<(&String, &String)> = program.class_parents.iter().collect();
    parents.sort_by_key(|(c, _)| c.as_str());
    write_u32(&mut buf, parents.len() as u32);
    for (child, parent) in parents {
        write_str(&mut buf, child);
        write_str(&mut buf, parent);
    }

    // Instructions
    write_u32(&mut buf, program.instructions.len() as u32);
    for op in &program.instructions {
        encode_op(&mut buf, op)?;
    }

    // v4: source line per instruction (0 = unknown), parallel to instructions
    for i in 0..program.instructions.len() {
        write_u32(&mut buf, program.lines.get(i).copied().unwrap_or(0));
    }

    std::fs::write(path, &buf)
        .map_err(|e| format!("लिखने में त्रुटि '{}': {}", path, e))
}

/// Load a .libc file into a CompiledProgram.
pub fn load(path: &str) -> Result<CompiledProgram, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("फ़ाइल नहीं खुली '{}': {}", path, e))?;

    let mut pos = 0usize;

    // Magic check
    if data.get(..4) != Some(b"LIPI") {
        return Err(format!("'{}' LIPI bytecode नहीं है", path));
    }
    pos += 4;

    let version = read_u8(&data, &mut pos)?;
    if !(2..=5).contains(&version) {
        return Err(format!("असमर्थित bytecode संस्करण: {}", version));
    }

    // Function table
    let func_count = read_u32(&data, &mut pos)? as usize;
    let mut functions = HashMap::new();
    for _ in 0..func_count {
        let name      = read_str(&data, &mut pos)?;
        let param_cnt = read_u8(&data, &mut pos)? as usize;
        let mut params = Vec::with_capacity(param_cnt);
        for _ in 0..param_cnt { params.push(read_str(&data, &mut pos)?); }
        // v3: per-param defaults; v2 files have none
        let mut defaults: Vec<Option<LvmValue>> = Vec::with_capacity(param_cnt);
        if version >= 3 {
            for _ in 0..param_cnt {
                let flag = read_u8(&data, &mut pos)?;
                defaults.push(if flag == 1 { Some(read_lvm_value(&data, &mut pos)?) } else { None });
            }
        } else {
            defaults.resize(param_cnt, None);
        }
        let vararg_flag = read_u8(&data, &mut pos)?;
        let vararg = if vararg_flag == 1 { Some(read_str(&data, &mut pos)?) } else { None };
        let start_ip = read_u64(&data, &mut pos)? as usize;
        let is_generator = if version >= 5 { read_u8(&data, &mut pos)? == 1 } else { false };
        functions.insert(name, FuncDef { params, start_ip, vararg, defaults, is_generator });
    }

    // v3: class inheritance table — v2 files have none (inheritance broke in old .libc)
    let mut class_parents = HashMap::new();
    if version >= 3 {
        let parent_count = read_u32(&data, &mut pos)? as usize;
        for _ in 0..parent_count {
            let child  = read_str(&data, &mut pos)?;
            let parent = read_str(&data, &mut pos)?;
            class_parents.insert(child, parent);
        }
    }

    // Instructions
    let instr_count = read_u32(&data, &mut pos)? as usize;
    let mut instructions = Vec::with_capacity(instr_count);
    for _ in 0..instr_count {
        instructions.push(decode_op(&data, &mut pos)?);
    }

    // v4: line table — pre-v4 files report line 0 (unknown) everywhere
    let mut lines = Vec::with_capacity(instr_count);
    if version >= 4 {
        for _ in 0..instr_count {
            lines.push(read_u32(&data, &mut pos)?);
        }
    } else {
        lines.resize(instr_count, 0);
    }

    Ok(CompiledProgram { instructions, lines, functions, class_parents })
}

// ── Encode ────────────────────────────────────────────────────────────────────

fn encode_op(buf: &mut Vec<u8>, op: &Opcode) -> Result<(), String> {
    match op {
        Opcode::Push(v) => match v {
            LvmValue::Nil       => write_u8(buf, TAG_PUSH_NIL),
            LvmValue::Bool(false) => write_u8(buf, TAG_PUSH_FALSE),
            LvmValue::Bool(true)  => write_u8(buf, TAG_PUSH_TRUE),
            LvmValue::Number(n)   => { write_u8(buf, TAG_PUSH_NUM); write_f64(buf, *n); }
            LvmValue::Str(s)      => { write_u8(buf, TAG_PUSH_STR); write_str(buf, s); }
        },
        Opcode::Pop    => write_u8(buf, TAG_POP),
        Opcode::Dup    => write_u8(buf, TAG_DUP),

        Opcode::LoadVar(s)  => { write_u8(buf, TAG_LOAD_VAR);  write_str(buf, s); }
        Opcode::StoreVar(s) => { write_u8(buf, TAG_STORE_VAR); write_str(buf, s); }

        Opcode::Add  => write_u8(buf, TAG_ADD),
        Opcode::Sub  => write_u8(buf, TAG_SUB),
        Opcode::Mul  => write_u8(buf, TAG_MUL),
        Opcode::Div  => write_u8(buf, TAG_DIV),
        Opcode::Mod  => write_u8(buf, TAG_MOD),

        Opcode::Eq    => write_u8(buf, TAG_EQ),
        Opcode::NotEq => write_u8(buf, TAG_NEQ),
        Opcode::Gt    => write_u8(buf, TAG_GT),
        Opcode::Lt    => write_u8(buf, TAG_LT),
        Opcode::GtEq  => write_u8(buf, TAG_GTEQ),
        Opcode::LtEq  => write_u8(buf, TAG_LTEQ),

        Opcode::And => write_u8(buf, TAG_AND),
        Opcode::Or  => write_u8(buf, TAG_OR),
        Opcode::Not => write_u8(buf, TAG_NOT),

        Opcode::Jump(a)         => { write_u8(buf, TAG_JUMP);           write_u32(buf, addr32(*a)?); }
        Opcode::JumpIfFalse(a)  => { write_u8(buf, TAG_JUMP_IF_FALSE);  write_u32(buf, addr32(*a)?); }
        Opcode::JumpIfTrue(a)   => { write_u8(buf, TAG_JUMP_IF_TRUE);   write_u32(buf, addr32(*a)?); }

        Opcode::Call(n, c) => {
            write_u8(buf, TAG_CALL); write_str(buf, n); write_u8(buf, *c as u8);
        }
        Opcode::CallNative(n, c) => {
            write_u8(buf, TAG_CALL_NATIVE); write_str(buf, n); write_u8(buf, *c as u8);
        }

        Opcode::Return => write_u8(buf, TAG_RETURN),
        Opcode::Print  => write_u8(buf, TAG_PRINT),

        Opcode::KarakaCheck(a, b) => {
            write_u8(buf, TAG_KARAKA_CHECK); write_str(buf, a); write_str(buf, b);
        }
        Opcode::Import(s) => { write_u8(buf, TAG_IMPORT); write_str(buf, s); }

        Opcode::MethodCall(n, c) => {
            write_u8(buf, TAG_METHOD_CALL); write_str(buf, n); write_u8(buf, *c as u8);
        }

        Opcode::AadhaarVerify => write_u8(buf, TAG_AADHAAR_VERIFY),
        Opcode::UpiSend       => write_u8(buf, TAG_UPI_SEND),
        Opcode::GstAdd        => write_u8(buf, TAG_GST_ADD),
        Opcode::LakhParse     => write_u8(buf, TAG_LAKH_PARSE),
        Opcode::RupeeFormat   => write_u8(buf, TAG_RUPEE_FORMAT),

        Opcode::MakeList(n) => { write_u8(buf, TAG_MAKE_LIST); write_u32(buf, *n as u32); }
        Opcode::MakeDict(n) => { write_u8(buf, TAG_MAKE_DICT); write_u32(buf, *n as u32); }
        Opcode::GetIndex    => write_u8(buf, TAG_GET_INDEX),
        Opcode::SetIndex    => write_u8(buf, TAG_SET_INDEX),

        Opcode::MakeInstance(s) => { write_u8(buf, TAG_MAKE_INSTANCE); write_str(buf, s); }
        Opcode::GetAttr(s)      => { write_u8(buf, TAG_GET_ATTR);      write_str(buf, s); }
        Opcode::SetAttr(s)      => { write_u8(buf, TAG_SET_ATTR);      write_str(buf, s); }
        Opcode::PrintInline     => write_u8(buf, TAG_PRINT_INLINE),
        Opcode::TryStart(addr)  => { write_u8(buf, TAG_TRY_START); write_u32(buf, *addr as u32); }
        Opcode::TryEnd          => write_u8(buf, TAG_TRY_END),
        Opcode::ImportFile(p)   => { write_u8(buf, TAG_IMPORT_FILE); write_str(buf, p); }
        Opcode::MakeClosure(n)  => { write_u8(buf, TAG_MAKE_CLOSURE); write_str(buf, n); }
        Opcode::GetIterLen      => write_u8(buf, TAG_GET_ITER_LEN),
        Opcode::GetIterItem     => write_u8(buf, TAG_GET_ITER_ITEM),
        Opcode::BitAnd          => write_u8(buf, TAG_BIT_AND),
        Opcode::BitOr           => write_u8(buf, TAG_BIT_OR),
        Opcode::BitXor          => write_u8(buf, TAG_BIT_XOR),
        Opcode::BitNot          => write_u8(buf, TAG_BIT_NOT),
        Opcode::LShift          => write_u8(buf, TAG_LSHIFT),
        Opcode::RShift          => write_u8(buf, TAG_RSHIFT),
        Opcode::DeclareGlobal(n) => { write_u8(buf, TAG_DECLARE_GLOBAL); write_str(buf, n); }

        Opcode::DefineEnum(name, variants) => {
            write_u8(buf, TAG_DEFINE_ENUM);
            write_str(buf, name);
            write_u32(buf, variants.len() as u32);
            for (vname, arity) in variants {
                write_str(buf, vname);
                write_u32(buf, *arity as u32);
            }
        }
        Opcode::MatchVariant(vname) => { write_u8(buf, TAG_MATCH_VARIANT); write_str(buf, vname); }
        Opcode::EnumUnpack(names) => {
            write_u8(buf, TAG_ENUM_UNPACK);
            write_u32(buf, names.len() as u32);
            for n in names { write_str(buf, n); }
        }
        Opcode::TailCall(name, argc) => {
            write_u8(buf, TAG_TAIL_CALL); write_str(buf, name); write_u8(buf, *argc as u8);
        }
        Opcode::Assert(msg) => {
            write_u8(buf, TAG_ASSERT);
            match msg {
                Some(s) => { write_u8(buf, 1); write_str(buf, s); }
                None    => { write_u8(buf, 0); }
            }
        }
        Opcode::DeclareConst(name) => {
            write_u8(buf, TAG_DECLARE_CONST); write_str(buf, name);
        }
        Opcode::CallKw(name, pos_argc, kwnames) => {
            write_u8(buf, TAG_CALL_KW);
            write_str(buf, name);
            write_u8(buf, *pos_argc as u8);
            write_u8(buf, kwnames.len() as u8);
            for n in kwnames { write_str(buf, n); }
        }
        Opcode::FloorDiv => write_u8(buf, TAG_FLOOR_DIV),
        Opcode::UnpackList(n) => { write_u8(buf, TAG_UNPACK_LIST); write_u8(buf, *n as u8); }
        Opcode::Slice => write_u8(buf, TAG_SLICE),
        Opcode::Contains => write_u8(buf, TAG_CONTAINS),
        Opcode::MakeListSp(flags) => {
            write_u8(buf, TAG_MAKE_LIST_SP);
            write_u8(buf, flags.len() as u8);
            for f in flags { write_u8(buf, if *f { 1 } else { 0 }); }
        }
        Opcode::Throw => write_u8(buf, TAG_THROW),
        Opcode::MatchErrClass(name) => { write_u8(buf, TAG_MATCH_ERR_CLASS); write_str(buf, name); }
        Opcode::IterNext(cvar, ivar) => {
            write_u8(buf, TAG_ITER_NEXT);
            write_str(buf, cvar);
            write_str(buf, ivar);
        }
        Opcode::Yield => write_u8(buf, TAG_YIELD),
        Opcode::IterStep { loop_var, container_var, idx_var } => {
            write_u8(buf, TAG_ITER_STEP);
            write_str(buf, loop_var);
            write_str(buf, container_var);
            write_str(buf, idx_var);
        }
        Opcode::MethodCallKw { method, pos_argc, kwnames } => {
            write_u8(buf, TAG_METHOD_CALL_KW);
            write_str(buf, method);
            write_u8(buf, *pos_argc as u8);
            write_u8(buf, kwnames.len() as u8);
            for n in kwnames { write_str(buf, n); }
        }
        Opcode::SetSlice => write_u8(buf, TAG_SET_SLICE),
    }
    Ok(())
}

fn addr32(a: usize) -> Result<u32, String> {
    u32::try_from(a).map_err(|_| format!("jump address {} > u32::MAX", a))
}

// ── Decode ────────────────────────────────────────────────────────────────────

fn decode_op(data: &[u8], pos: &mut usize) -> Result<Opcode, String> {
    let tag = read_u8(data, pos)?;
    Ok(match tag {
        TAG_PUSH_NIL   => Opcode::Push(LvmValue::Nil),
        TAG_PUSH_FALSE => Opcode::Push(LvmValue::Bool(false)),
        TAG_PUSH_TRUE  => Opcode::Push(LvmValue::Bool(true)),
        TAG_PUSH_NUM   => Opcode::Push(LvmValue::Number(read_f64(data, pos)?)),
        TAG_PUSH_STR   => Opcode::Push(LvmValue::Str(read_str(data, pos)?)),

        TAG_POP => Opcode::Pop,
        TAG_DUP => Opcode::Dup,

        TAG_LOAD_VAR  => Opcode::LoadVar(read_str(data, pos)?),
        TAG_STORE_VAR => Opcode::StoreVar(read_str(data, pos)?),

        TAG_ADD => Opcode::Add,
        TAG_SUB => Opcode::Sub,
        TAG_MUL => Opcode::Mul,
        TAG_DIV => Opcode::Div,
        TAG_MOD => Opcode::Mod,

        TAG_EQ   => Opcode::Eq,
        TAG_NEQ  => Opcode::NotEq,
        TAG_GT   => Opcode::Gt,
        TAG_LT   => Opcode::Lt,
        TAG_GTEQ => Opcode::GtEq,
        TAG_LTEQ => Opcode::LtEq,

        TAG_AND => Opcode::And,
        TAG_OR  => Opcode::Or,
        TAG_NOT => Opcode::Not,

        TAG_JUMP          => Opcode::Jump(read_u32(data, pos)? as usize),
        TAG_JUMP_IF_FALSE => Opcode::JumpIfFalse(read_u32(data, pos)? as usize),
        TAG_JUMP_IF_TRUE  => Opcode::JumpIfTrue(read_u32(data, pos)? as usize),

        TAG_CALL => {
            let name = read_str(data, pos)?;
            let argc = read_u8(data, pos)? as usize;
            Opcode::Call(name, argc)
        }
        TAG_CALL_NATIVE => {
            let name = read_str(data, pos)?;
            let argc = read_u8(data, pos)? as usize;
            Opcode::CallNative(name, argc)
        }

        TAG_RETURN => Opcode::Return,
        TAG_PRINT  => Opcode::Print,

        TAG_KARAKA_CHECK => {
            let a = read_str(data, pos)?;
            let b = read_str(data, pos)?;
            Opcode::KarakaCheck(a, b)
        }
        TAG_IMPORT => Opcode::Import(read_str(data, pos)?),

        TAG_METHOD_CALL => {
            let name = read_str(data, pos)?;
            let argc = read_u8(data, pos)? as usize;
            Opcode::MethodCall(name, argc)
        }

        TAG_AADHAAR_VERIFY => Opcode::AadhaarVerify,
        TAG_UPI_SEND       => Opcode::UpiSend,
        TAG_GST_ADD        => Opcode::GstAdd,
        TAG_LAKH_PARSE     => Opcode::LakhParse,
        TAG_RUPEE_FORMAT   => Opcode::RupeeFormat,

        TAG_MAKE_LIST => Opcode::MakeList(read_u32(data, pos)? as usize),
        TAG_MAKE_DICT => Opcode::MakeDict(read_u32(data, pos)? as usize),
        TAG_GET_INDEX => Opcode::GetIndex,
        TAG_SET_INDEX => Opcode::SetIndex,

        TAG_MAKE_INSTANCE => Opcode::MakeInstance(read_str(data, pos)?),
        TAG_GET_ATTR      => Opcode::GetAttr(read_str(data, pos)?),
        TAG_SET_ATTR      => Opcode::SetAttr(read_str(data, pos)?),
        TAG_PRINT_INLINE  => Opcode::PrintInline,
        TAG_TRY_START     => Opcode::TryStart(read_u32(data, pos)? as usize),
        TAG_TRY_END       => Opcode::TryEnd,
        TAG_IMPORT_FILE   => Opcode::ImportFile(read_str(data, pos)?),
        TAG_MAKE_CLOSURE  => Opcode::MakeClosure(read_str(data, pos)?),
        TAG_GET_ITER_LEN  => Opcode::GetIterLen,
        TAG_GET_ITER_ITEM => Opcode::GetIterItem,
        TAG_BIT_AND       => Opcode::BitAnd,
        TAG_BIT_OR        => Opcode::BitOr,
        TAG_BIT_XOR       => Opcode::BitXor,
        TAG_BIT_NOT       => Opcode::BitNot,
        TAG_LSHIFT        => Opcode::LShift,
        TAG_RSHIFT        => Opcode::RShift,
        TAG_DECLARE_GLOBAL => Opcode::DeclareGlobal(read_str(data, pos)?),

        TAG_DEFINE_ENUM => {
            let name = read_str(data, pos)?;
            let count = read_u32(data, pos)? as usize;
            let mut variants = Vec::with_capacity(count);
            for _ in 0..count {
                let vname = read_str(data, pos)?;
                let arity = read_u32(data, pos)? as usize;
                variants.push((vname, arity));
            }
            Opcode::DefineEnum(name, variants)
        }
        TAG_MATCH_VARIANT => Opcode::MatchVariant(read_str(data, pos)?),
        TAG_ENUM_UNPACK => {
            let count = read_u32(data, pos)? as usize;
            let mut names = Vec::with_capacity(count);
            for _ in 0..count { names.push(read_str(data, pos)?); }
            Opcode::EnumUnpack(names)
        }
        TAG_TAIL_CALL => {
            let name = read_str(data, pos)?;
            let argc = read_u8(data, pos)? as usize;
            Opcode::TailCall(name, argc)
        }
        TAG_ASSERT => {
            let has_msg = read_u8(data, pos)?;
            let msg = if has_msg == 1 { Some(read_str(data, pos)?) } else { None };
            Opcode::Assert(msg)
        }
        TAG_DECLARE_CONST => {
            let name = read_str(data, pos)?;
            Opcode::DeclareConst(name)
        }
        TAG_CALL_KW => {
            let name = read_str(data, pos)?;
            let pos_argc = read_u8(data, pos)? as usize;
            let kw_count = read_u8(data, pos)? as usize;
            let mut kwnames = Vec::with_capacity(kw_count);
            for _ in 0..kw_count { kwnames.push(read_str(data, pos)?); }
            Opcode::CallKw(name, pos_argc, kwnames)
        }
        TAG_FLOOR_DIV => Opcode::FloorDiv,
        TAG_UNPACK_LIST => Opcode::UnpackList(read_u8(data, pos)? as usize),
        TAG_SLICE => Opcode::Slice,
        TAG_CONTAINS => Opcode::Contains,
        TAG_MAKE_LIST_SP => {
            let count = read_u8(data, pos)? as usize;
            let mut flags = Vec::with_capacity(count);
            for _ in 0..count { flags.push(read_u8(data, pos)? != 0); }
            Opcode::MakeListSp(flags)
        }
        TAG_THROW => Opcode::Throw,
        TAG_MATCH_ERR_CLASS => Opcode::MatchErrClass(read_str(data, pos)?),
        TAG_ITER_NEXT => {
            let cvar = read_str(data, pos)?;
            let ivar = read_str(data, pos)?;
            Opcode::IterNext(cvar, ivar)
        }
        TAG_YIELD => Opcode::Yield,
        TAG_ITER_STEP => {
            let loop_var = read_str(data, pos)?;
            let container_var = read_str(data, pos)?;
            let idx_var = read_str(data, pos)?;
            Opcode::IterStep { loop_var, container_var, idx_var }
        }
        TAG_METHOD_CALL_KW => {
            let method = read_str(data, pos)?;
            let pos_argc = read_u8(data, pos)? as usize;
            let kw_count = read_u8(data, pos)? as usize;
            let mut kwnames = Vec::with_capacity(kw_count);
            for _ in 0..kw_count { kwnames.push(read_str(data, pos)?); }
            Opcode::MethodCallKw { method, pos_argc, kwnames }
        }
        TAG_SET_SLICE => Opcode::SetSlice,

        other => return Err(format!("अज्ञात opcode tag: 0x{:02X}", other)),
    })
}

// ── Binary I/O helpers ────────────────────────────────────────────────────────

fn write_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_f64(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

fn read_u8(data: &[u8], pos: &mut usize) -> Result<u8, String> {
    data.get(*pos).copied()
        .map(|v| { *pos += 1; v })
        .ok_or_else(|| "bytecode अपूर्ण (u8 read)".to_string())
}

fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32, String> {
    if *pos + 4 > data.len() { return Err("bytecode अपूर्ण (u32 read)".into()); }
    let v = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]);
    *pos += 4;
    Ok(v)
}

fn read_u64(data: &[u8], pos: &mut usize) -> Result<u64, String> {
    if *pos + 8 > data.len() { return Err("bytecode अपूर्ण (u64 read)".into()); }
    let bytes: [u8; 8] = data[*pos..*pos+8].try_into().unwrap();
    *pos += 8;
    Ok(u64::from_le_bytes(bytes))
}

fn read_f64(data: &[u8], pos: &mut usize) -> Result<f64, String> {
    if *pos + 8 > data.len() { return Err("bytecode अपूर्ण (f64 read)".into()); }
    let bytes: [u8; 8] = data[*pos..*pos+8].try_into().unwrap();
    *pos += 8;
    Ok(f64::from_le_bytes(bytes))
}

fn read_str(data: &[u8], pos: &mut usize) -> Result<String, String> {
    let len = read_u32(data, pos)? as usize;
    if *pos + len > data.len() { return Err("bytecode अपूर्ण (str read)".into()); }
    let s = std::str::from_utf8(&data[*pos..*pos+len])
        .map_err(|e| format!("invalid UTF-8 in bytecode: {}", e))?
        .to_string();
    *pos += len;
    Ok(s)
}

// ── LvmValue encoding (v3 param defaults) — same tags as PUSH_* ──────────────

fn write_lvm_value(buf: &mut Vec<u8>, v: &LvmValue) {
    match v {
        LvmValue::Nil         => write_u8(buf, TAG_PUSH_NIL),
        LvmValue::Bool(false) => write_u8(buf, TAG_PUSH_FALSE),
        LvmValue::Bool(true)  => write_u8(buf, TAG_PUSH_TRUE),
        LvmValue::Number(n)   => { write_u8(buf, TAG_PUSH_NUM); write_f64(buf, *n); }
        LvmValue::Str(s)      => { write_u8(buf, TAG_PUSH_STR); write_str(buf, s); }
    }
}

fn read_lvm_value(data: &[u8], pos: &mut usize) -> Result<LvmValue, String> {
    let tag = read_u8(data, pos)?;
    Ok(match tag {
        TAG_PUSH_NIL   => LvmValue::Nil,
        TAG_PUSH_FALSE => LvmValue::Bool(false),
        TAG_PUSH_TRUE  => LvmValue::Bool(true),
        TAG_PUSH_NUM   => LvmValue::Number(read_f64(data, pos)?),
        TAG_PUSH_STR   => LvmValue::Str(read_str(data, pos)?),
        other => return Err(format!("अमान्य डिफ़ॉल्ट मान tag: 0x{:02X}", other)),
    })
}
