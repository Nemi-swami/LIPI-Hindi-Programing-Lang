//! FFI callbacks — pass a LIPI closure as a real C function pointer (Phase 19 F3).
//!
//! A native library often calls YOU back: `qsort` needs a comparator, a windowing
//! toolkit needs an event handler, a driver needs an interrupt/completion callback.
//! To support that, `बाह्य_कॉलबैक(क्लोजर, "ढांचा")` (registered in the LVM Call
//! handler, where `&mut self` is available) hands back the address of a fixed
//! `extern "C"` **trampoline**. When C calls the trampoline, it marshals the
//! integer/pointer arguments into LIPI Numbers, re-enters the VM to run the
//! closure, and returns the closure's numeric result to C.
//!
//! No dynamic code generation (pure Rust) — instead there is a fixed pool of
//! monomorphic trampolines (`tramp0..tramp15`), each hard-coding its slot index.
//! Registration fills the next free slot with the closure plus raw pointers back to
//! the live VM and program.
//!
//! Signature `ढांचा` is the same `"<args>:<ret>"` used by `बाह्य_बुलाओ`, but only
//! integer/pointer args (`i`/`l`) and an `i`/`l`/`v` return are supported, up to 4
//! args — enough for every common C callback (comparators take 2, window procs 4).
//! Read struct/pointer arguments inside the closure with `भारत.तंत्र`.
//!
//! SAFETY: the trampoline reconstructs `&mut LVM` from a raw pointer while the
//! outer `बाह्य_बुलाओ` still holds `&mut self` on the call stack. That outer borrow
//! is suspended for the duration of the synchronous C call, so the single-threaded
//! re-entry is sound in practice; it is the standard FFI-callback pattern. A
//! callback that itself errors returns 0 to C (the error is not propagated across
//! the C frame).

use crate::interpreter::Value;
use crate::lvm::LVM;
use crate::opcode::Opcode;
use std::cell::RefCell;

const MAX_CB_ARGS: usize = 4;
const NUM_SLOTS: usize = 16;

struct Slot {
    closure: Value,
    argc: usize,
    vm: *mut LVM,
    instr_ptr: *const Opcode,
    instr_len: usize,
}

thread_local! {
    static SLOTS: RefCell<[Option<Slot>; NUM_SLOTS]> = RefCell::new(Default::default());
}

/// Register a closure as a callback. Returns the C-callable trampoline address.
/// Called from the LVM Call handler so `vm`/`instructions` are the live ones.
pub(crate) fn register(
    closure: Value,
    argc: usize,
    vm: *mut LVM,
    instructions: &[Opcode],
) -> Result<usize, String> {
    if argc > MAX_CB_ARGS {
        return Err(format!("बाह्य_कॉलबैक(): अधिकतम {MAX_CB_ARGS} तर्क (मिले {argc})"));
    }
    if !matches!(closure, Value::Closure { .. }) {
        return Err("बाह्य_कॉलबैक(): पहला तर्क एक विधि/लाम्डा होना चाहिए".to_string());
    }
    let instr_ptr = instructions.as_ptr();
    let instr_len = instructions.len();
    SLOTS.with(|s| {
        let mut arr = s.borrow_mut();
        for (i, cell) in arr.iter_mut().enumerate() {
            if cell.is_none() {
                *cell = Some(Slot { closure, argc, vm, instr_ptr, instr_len });
                return Ok(TRAMPS[i] as usize);
            }
        }
        Err(format!("बाह्य_कॉलबैक(): सभी {NUM_SLOTS} कॉलबैक स्लॉट भरे हैं (बाह्य_कॉलबैक_मुक्त से खाली करें)"))
    })
}

/// Free a callback slot by its trampoline address (call once C no longer holds it).
pub(crate) fn unregister(addr: usize) -> bool {
    SLOTS.with(|s| {
        let mut arr = s.borrow_mut();
        for (i, cell) in arr.iter_mut().enumerate() {
            if TRAMPS[i] as usize == addr && cell.is_some() {
                *cell = None;
                return true;
            }
        }
        false
    })
}

/// The shared body: recover the slot, marshal args, run the closure, return a usize.
fn dispatch(idx: usize, raw: [usize; MAX_CB_ARGS]) -> usize {
    // Copy everything we need out of the slot, then drop the borrow before we
    // re-enter the VM (the closure could register another callback).
    let snapshot = SLOTS.with(|s| {
        s.borrow()[idx].as_ref().map(|slot| {
            (slot.closure.clone(), slot.argc, slot.vm, slot.instr_ptr, slot.instr_len)
        })
    });
    let (closure, argc, vm_ptr, instr_ptr, instr_len) = match snapshot {
        Some(t) => t,
        None => return 0, // freed slot called by C — nothing to do
    };
    if vm_ptr.is_null() { return 0; }

    let args: Vec<Value> = (0..argc)
        .map(|i| Value::Number(raw[i] as i64 as f64))
        .collect();

    // SAFETY: see module header — synchronous single-threaded re-entry.
    let vm: &mut LVM = unsafe { &mut *vm_ptr };
    let instructions: &[Opcode] = unsafe { std::slice::from_raw_parts(instr_ptr, instr_len) };

    match vm.call_closure_value(&closure, args, instructions) {
        Ok(Value::Number(n)) => n as i64 as usize,
        Ok(Value::Bool(b)) => b as usize,
        Ok(_) | Err(_) => 0,
    }
}

// ── the fixed trampoline pool ────────────────────────────────────────────────

macro_rules! trampoline {
    ($name:ident, $idx:expr) => {
        extern "C" fn $name(a0: usize, a1: usize, a2: usize, a3: usize) -> usize {
            dispatch($idx, [a0, a1, a2, a3])
        }
    };
}

trampoline!(tramp0, 0);   trampoline!(tramp1, 1);   trampoline!(tramp2, 2);   trampoline!(tramp3, 3);
trampoline!(tramp4, 4);   trampoline!(tramp5, 5);   trampoline!(tramp6, 6);   trampoline!(tramp7, 7);
trampoline!(tramp8, 8);   trampoline!(tramp9, 9);   trampoline!(tramp10, 10); trampoline!(tramp11, 11);
trampoline!(tramp12, 12); trampoline!(tramp13, 13); trampoline!(tramp14, 14); trampoline!(tramp15, 15);

type Tramp = extern "C" fn(usize, usize, usize, usize) -> usize;
static TRAMPS: [Tramp; NUM_SLOTS] = [
    tramp0, tramp1, tramp2, tramp3, tramp4, tramp5, tramp6, tramp7,
    tramp8, tramp9, tramp10, tramp11, tramp12, tramp13, tramp14, tramp15,
];
