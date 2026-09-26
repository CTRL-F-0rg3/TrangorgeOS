//! The environment block.
//!
//! `std::env::vars` reads `environ`, which `tgs-rt::_start` points at the block
//! the kernel left on the stack. `getenv`/`setenv` are here for the C programs
//! layered on top; `setenv` cannot work the way glibc's does, because the block
//! lives in the stack image and growing it would overwrite the auxv. It is
//! therefore a documented no-op that returns success — see the note on the
//! function.

use core::ptr;

use tgs_hal::errno;

use crate::auxv::{self, environ};
use crate::types::{c_char, c_int};

/// `char *getenv(const char *name)`
#[no_mangle]
pub extern "C" fn getenv(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return ptr::null();
    }
    // SAFETY: the caller passed a non-null NUL-terminated name, per the contract
    // of `getenv`.
    let want = unsafe { cstr_bytes(name) };
    // SAFETY: `environ` is written once by `_start`, before `main`, and only
    // read after that.
    let block = unsafe { environ };
    if block.is_null() {
        return ptr::null();
    }

    let mut i = 0isize;
    loop {
        // SAFETY: the block is NUL-terminated, so reading entry `i` is in bounds
        // until an entry is null.
        let entry = unsafe { *block.offset(i) };
        if entry.is_null() {
            return ptr::null();
        }
        // SAFETY: every non-null entry in the block is a NUL-terminated string.
        let bytes = unsafe { cstr_bytes(entry) };
        // `NAME=VALUE`: a prefix match on `NAME` plus the `=`, so that
        // `getenv("HOME")` does not match `HOMEBREW_PATH`.
        if bytes.len() > want.len()
            && bytes[want.len()] == b'='
            && &bytes[..want.len()] == want
        {
            // SAFETY: the `=` is inside the string, so `want.len() + 1` is a
            // valid offset and the result is still NUL-terminated.
            return unsafe { entry.add(want.len() + 1) };
        }
        i += 1;
    }
}

/// `int setenv(const char *name, const char *value, int overwrite)`
///
/// A no-op that reports success, and this is a real limitation rather than a
/// stub: the environment block is part of the stack image the kernel built, so
/// appending to it would overwrite the auxv that follows it. `std::env::set_var`
/// is documented as unsafe for the same reason — "you must not rely on the
/// environment being unchanged" — but it does actually write, and a program that
/// calls it will not see the value.
///
/// The honest fix is a private, growable copy of the block owned by this
/// process, which `getenv` and `environ` would then both point at. That is a
/// small change and it is the next thing to do; it is called out in
/// `../README.md` under "known gaps".
#[no_mangle]
pub extern "C" fn setenv(
    _name: *const c_char,
    _value: *const c_char,
    _overwrite: c_int,
) -> c_int {
    0
}

/// `int unsetenv(const char *name)` — see [`setenv`].
#[no_mangle]
pub extern "C" fn unsetenv(_name: *const c_char) -> c_int {
    0
}

/// `int putenv(char *string)` — see [`setenv`].
#[no_mangle]
pub extern "C" fn putenv(_string: *mut c_char) -> c_int {
    0
}

/// `char *secure_getenv(const char *name)`
///
/// glibc's hardened spelling. TrangorgeOS has no set-uid execve yet, so there is
/// nothing to be hardened against and it is exactly `getenv`.
#[no_mangle]
pub extern "C" fn secure_getenv(name: *const c_char) -> *const c_char {
    getenv(name)
}

/// `int clearenv(void)` — see [`setenv`]. Refused, because pretending to have
/// emptied the environment while `environ` still points at it would be worse.
#[no_mangle]
pub extern "C" fn clearenv() -> c_int {
    errno::ENOSYS.0
}

/// The bytes of a NUL-terminated string, without the NUL.
///
/// # Safety
///
/// `s` must be non-null and point at a NUL-terminated string.
unsafe fn cstr_bytes(s: *const c_char) -> &'static [u8] {
    // SAFETY: forwarded from this function's contract; the kernel built the block
    // and it outlives the program.
    unsafe { core::slice::from_raw_parts(s.cast::<u8>(), cstrlen(s)) }
}

/// The length of a NUL-terminated string.
///
/// # Safety
///
/// `s` must be non-null and NUL-terminated.
unsafe fn cstrlen(s: *const c_char) -> usize {
    let mut n = 0usize;
    // SAFETY: the caller guarantees a NUL terminates the string, so this stops.
    while unsafe { *s.add(n) } != 0 {
        n += 1;
    }
    n
}

/// Make `environ` point at the block `tgs-rt` found on the stack.
///
/// # Safety
///
/// Must be called once from `_start`, before `main`.
pub unsafe fn publish(environ_block: *mut *const c_char, aux: &auxv::Auxv) {
    // SAFETY: called once from `_start`; both pointers come from the stack image
    // the kernel built, which outlives the program.
    unsafe {
        ptr::write(&raw mut environ, environ_block);
        auxv::set(*aux);
    }
}
