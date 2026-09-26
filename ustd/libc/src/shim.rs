//! The `Result`-to-C bridge, in one place.
//!
//! Every shim in this crate has the same shape: call something in `tgs-hal`, and
//! on failure set `errno` and return the C failure value. Writing that once means
//! the conventions cannot drift between, say, `read` and `lseek` — and getting
//! the conventions wrong is how a C library ends up reporting success for a
//! failed syscall.

use tgs_hal::errno::Errno;
use tgs_hal::Result;

use crate::errno;
use crate::types::{c_int, c_long, ssize_t};

/// A C return type that a syscall result can be narrowed into.
///
/// `read` returns `ssize_t`, `close` returns `int`, and `syscall` returns `long`.
/// They are three distinct Rust types, and writing three near-identical bridges
/// is how one of them ends up returning the wrong width.
pub(crate) trait CRet: Sized {
    /// Narrow a decoded syscall result, or produce the C failure value.
    fn from_syscall(v: c_long) -> Self;
}

impl CRet for c_int {
    fn from_syscall(v: c_long) -> Self {
        v as c_int
    }
}

impl CRet for c_long {
    fn from_syscall(v: c_long) -> Self {
        v
    }
}

impl CRet for ssize_t {
    fn from_syscall(v: c_long) -> Self {
        v as ssize_t
    }
}

/// The value a C function returns when the underlying call failed.
pub(crate) const FAIL: c_long = -1;

/// A call whose success value is a count, a descriptor or a length: the error
/// value is `-1`.
pub(crate) fn count<T, R: CRet>(r: Result<T>, ok: impl FnOnce(T) -> R) -> R {
    match r {
        Ok(v) => {
            errno::clear();
            ok(v)
        }
        Err(e) => {
            errno::set_errno(e);
            R::from_syscall(FAIL)
        }
    }
}

/// A call that returns the error number itself, 0 on success.
pub(crate) fn code<T>(r: Result<T>, ok: impl FnOnce(T) -> c_int) -> c_int {
    match r {
        Ok(v) => {
            errno::clear();
            ok(v)
        }
        Err(e) => {
            errno::set_errno(e);
            e.0
        }
    }
}

/// Report a failure from a shim that produced no `Result` of its own, in the
/// common `int`-returning case.
pub(crate) fn fail(e: Errno) -> c_int {
    errno::set_errno(e);
    FAIL as c_int
}

/// [`fail`] for a `long`-returning shim.
pub(crate) fn fail_long(e: Errno) -> c_long {
    errno::set_errno(e);
    FAIL
}

/// [`fail`] for an `ssize_t`-returning shim.
pub(crate) fn fail_ssize(e: Errno) -> ssize_t {
    errno::set_errno(e);
    FAIL as ssize_t
}
