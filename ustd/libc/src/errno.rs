//! `errno`.
//!
//! One integer, and `__errno_location` is the only way C code gets at it.
//!
//! This is a plain `static`, **not** a `thread_local!`, and that is deliberate.
//! A `#[thread_local]` access compiles to a `%fs`-relative load, and the kernel
//! does not program `fs_base`, so a real thread-local would fault on first touch.
//! A plain `static` is correct precisely because there is only ever one task —
//! and `tgs-rt` refuses to spawn a second one rather than letting that assumption
//! break quietly. When the kernel grows `fs_base`, this becomes a
//! `thread_local!` and nothing else has to change.

use core::cell::UnsafeCell;

use tgs_hal::errno::Errno;

use crate::types::c_int;

struct E(UnsafeCell<c_int>);

// SAFETY: see the module docs — one task, so there is no concurrent access to
// race with. The wrapper exists only to get a `static` past the `Sync` check.
unsafe impl Sync for E {}

/// C's `errno`.
///
/// A `static` rather than a `static mut`, so that no caller can accidentally make
/// two `&mut` at once.
static ERRNO: E = E(UnsafeCell::new(0));

/// Record `e` as the calling thread's `errno`.
pub fn set_errno(e: Errno) {
    // SAFETY: one task, so nothing else can be looking at the cell.
    unsafe { *ERRNO.0.get() = e.0 };
}

/// The calling thread's `errno`.
pub fn errno() -> c_int {
    // SAFETY: as above.
    unsafe { *ERRNO.0.get() }
}

/// `int *__errno_location(void)`
///
/// glibc's name for it, and the one `std` links against through `libc`. Note
/// the return type: a *pointer* to the int. A shim that returned the value by
/// mistake would "work" for a caller that only compares the result and hand every
/// other caller a wild pointer that happens to contain a small integer.
#[no_mangle]
pub extern "C" fn __errno_location() -> *mut c_int {
    ERRNO.0.get()
}

/// `int *__errno(void)` — the older glibc spelling, still emitted by some
/// toolchains.
#[no_mangle]
pub extern "C" fn __errno() -> *mut c_int {
    __errno_location()
}

/// Forget that anything went wrong.
///
/// Every successful shim calls this. A stale `errno` is how a C program ends up
/// reporting "No such file or directory" for an operation that worked, which is
/// exactly the bug class `errno` was invented to end.
pub fn clear() {
    set_errno(Errno(0));
}
