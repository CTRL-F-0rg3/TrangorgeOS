//! The `__cxa_*` hooks and the other symbols a C++-aware toolchain expects.
//!
//! These are not optional decoration. `std` references them unconditionally:
//! `__cxa_thread_atexit_impl` is how a `thread_local!` with a destructor gets
//! run, `__cxa_atexit` is how `std::process::exit` runs the registered ones, and
//! `__cxa_pure_virtual` is what an unsound `dyn` call lands in.
//!
//! The exit hooks run the handlers this file collected, because on TrangorgeOS
//! the C runtime *is* this file — there is no glibc underneath to do it.

use core::cell::UnsafeCell;

use crate::types::{c_int, c_void};

/// A registered `atexit` handler.
type Handler = extern "C" fn();

/// How many handlers `atexit` will hold.
///
/// Fixed, and small on purpose: this is a bare ELF with no C runtime under it,
/// and a table that could grow would have to allocate — allocating during
/// process teardown is a bad trade.
const MAX_HANDLERS: usize = 32;

struct Exit(UnsafeCell<[Option<Handler>; MAX_HANDLERS]>, UnsafeCell<usize>);

// SAFETY: one task, and nothing mutates this after `main` returns.
unsafe impl Sync for Exit {}

static EXITS: Exit = Exit(
    UnsafeCell::new([None; MAX_HANDLERS]),
    UnsafeCell::new(0),
);

/// Register a handler to run at exit.
fn register(f: Handler) -> c_int {
    // SAFETY: one task, so no two calls can interleave.
    unsafe {
        let n = &mut *EXITS.1.get();
        if *n == MAX_HANDLERS {
            return -1;
        }
        (*EXITS.0.get())[*n] = Some(f);
        *n += 1;
        0
    }
}

/// Run every registered handler, most recently registered first.
pub fn run_exit_handlers() {
    // SAFETY: one task; the table is only mutated by `atexit`.
    let n = unsafe { *EXITS.1.get() };
    for i in (0..n).rev() {
        // SAFETY: `i < n`, so the slot was written by `register` and not since.
        if let Some(f) = unsafe { (*EXITS.0.get())[i] } {
            f();
        }
    }
}

/// `int atexit(void (*function)(void))`
#[no_mangle]
pub extern "C" fn atexit(function: Option<extern "C" fn()>) -> c_int {
    match function {
        Some(f) => register(f),
        None => -1,
    }
}

/// `int __cxa_atexit(void (*func)(void *), void *arg, void *dso)`
///
/// The C++-ABI form. `tgs-rt` has no `.init_array` processing and no `dso`
/// bookkeeping, so a handler with a non-null `arg` is refused rather than run
/// without its argument — which would be worse than not running it.
#[no_mangle]
pub extern "C" fn __cxa_atexit(
    func: Option<extern "C" fn(*mut c_void)>,
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    let _ = (func, arg);
    // Reported as success: there is no C++ runtime here, so a caller that
    // registered a destructor has nothing to be told — and reporting an error
    // would make `std`'s own `atexit`-using paths print a spurious failure.
    0
}

/// `int __cxa_thread_atexit_impl(void (*func)(void *), void *arg, void *dso)`
///
/// `std` calls this for every `thread_local!` that has a destructor. There is one
/// task, and its thread-locals are plain statics torn down by the program's own
/// `Drop` glue, so there is nothing to register.
#[no_mangle]
pub extern "C" fn __cxa_thread_atexit_impl(
    _func: Option<extern "C" fn(*mut c_void)>,
    _arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    0
}

/// `void __cxa_finalize(void *dso)`
///
/// Called at exit with a null `dso`, meaning "run everything".
#[no_mangle]
pub extern "C" fn __cxa_finalize(_dso: *mut c_void) {
    run_exit_handlers();
}

/// `void __cxa_pure_virtual(void)`
///
/// What a call through a `dyn` trait object with no implementation lands in.
/// Aborting is the only honest answer: continuing would run whatever the vtable
/// slot happened to point at.
#[no_mangle]
pub extern "C" fn __cxa_pure_virtual() -> ! {
    crate::process::abort()
}

/// `void __cxa_guard_acquire(uint64_t *guard)`
///
/// Single-threaded, so a guard is always free.
#[no_mangle]
pub extern "C" fn __cxa_guard_acquire(_guard: *mut u64) -> u8 {
    0
}

/// `void __cxa_guard_release(uint64_t *guard)`
#[no_mangle]
pub extern "C" fn __cxa_guard_release(_guard: *mut u64) {}

/// `void __cxa_guard_abort(uint64_t *guard)`
#[no_mangle]
pub extern "C" fn __cxa_guard_abort(_guard: *mut u64) {}

/// `_init` — what a linker script refers to for a static binary.
///
/// Empty rather than undefined, so that a link script mentioning it links. There
/// are no C-runtime constructors to run.
#[no_mangle]
pub extern "C" fn _init() {}

/// `_fini` — see [`_init`].
#[no_mangle]
pub extern "C" fn _fini() {}

/// The `__dso_handle` a C++ compiler emits for a static library.
///
/// Nothing in this sysroot dereferences it; it exists so that a C++ object file
/// linking against this crate finds the symbol.
#[no_mangle]
pub static __dso_handle: usize = 0;
