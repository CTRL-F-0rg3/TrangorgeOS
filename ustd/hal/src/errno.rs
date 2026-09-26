//! Error numbers as `std` expects to find them.
//!
//! The values are the Linux ones. `std::io::Error::from_raw_os_error` maps them
//! by number, so reusing the numbering is what lets upstream `std` produce
//! sensible `ErrorKind`s for a TrangorgeOS program instead of treating every
//! failure as "other".

/// A `no_std` stand-in for `std::io::Error`.
///
/// Deliberately tiny: it carries the number (so it can round-trip through
/// `from_raw_os_error`) and nothing else. Formatting is left to the caller so
/// that this crate stays allocation-free and usable from an allocator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Errno(pub i32);

impl Errno {
    /// A short, stable, allocation-free description.
    pub const fn text(self) -> &'static str {
        match self {
            EPERM => "operation not permitted",
            ENOENT => "no such file or directory",
            ESRCH => "no such process",
            EINTR => "interrupted system call",
            EIO => "input/output error",
            ENXIO => "no such device or address",
            E2BIG => "argument list too long",
            EBADF => "bad file descriptor",
            ECHILD => "no child processes",
            EAGAIN => "resource temporarily unavailable",
            ENOMEM => "cannot allocate memory",
            EACCES => "permission denied",
            EFAULT => "bad address",
            ENOTBLK => "block device required",
            EBUSY => "device or resource busy",
            EEXIST => "file exists",
            EXDEV => "invalid cross-device link",
            ENODEV => "no such device",
            ENOTDIR => "not a directory",
            EISDIR => "is a directory",
            EINVAL => "invalid argument",
            ENFILE => "too many open files in system",
            EMFILE => "too many open files",
            ENOSPC => "no space left on device",
            ESPIPE => "illegal seek",
            ENOTTY => "inappropriate ioctl for device",
            EROFS => "read-only file system",
            EPIPE => "broken pipe",
            ENOSYS => "function not implemented",
            ENOTEMPTY => "directory not empty",
            ELOOP => "too many levels of symbolic links",
            ENAMETOOLONG => "file name too long",
            EADDRINUSE => "address already in use",
            EAFNOSUPPORT => "address family not supported by protocol",
            ENOTCONN => "transport endpoint is not connected",
            ETIMEDOUT => "connection timed out",
            ECONNREFUSED => "connection refused",
            EHOSTUNREACH => "no route to host",
            _ => "unknown error",
        }
    }
}

macro_rules! errno {
    ($($(#[$m:meta])* $name:ident = $val:expr,)*) => {$(
        $(#[$m])*
        pub const $name: Errno = Errno($val);
    )*};
}

errno! {
    EPERM = 1, ENOENT = 2, ESRCH = 3, EINTR = 4, EIO = 5, ENXIO = 6,
    E2BIG = 7, EBADF = 9, ECHILD = 10, EAGAIN = 11, ENOMEM = 12, EACCES = 13,
    EFAULT = 14, ENOTBLK = 15, EBUSY = 16, EEXIST = 17, EXDEV = 18,
    ERANGE = 34,
    ENODEV = 19, ENOTDIR = 20, EISDIR = 21, EINVAL = 22, ENFILE = 23,
    EMFILE = 24, ENOSPC = 28, ESPIPE = 29, EROFS = 30, EPIPE = 32,
    ENOSYS = 38, ENOTTY = 25, ENOTEMPTY = 39, ELOOP = 40, ENAMETOOLONG = 36, ENOTCONN = 107,
    EADDRINUSE = 98, EAFNOSUPPORT = 97, ETIMEDOUT = 110, ECONNREFUSED = 111,
    EHOSTUNREACH = 113, ECANCELED = 125, EOWNERDEAD = 130, ENOTRECOVERABLE = 131,
}

/// `EWOULDBLOCK` is an alias of `EAGAIN` on Linux; `std` reports it as
/// `ErrorKind::WouldBlock`, which it derives from the *number*, so the alias
/// has to stay identical.
pub const EWOULDBLOCK: Errno = EAGAIN;

/// The result type used throughout `tgs-hal`.
pub type Result<T> = core::result::Result<T, Errno>;

impl Errno {
    /// Decode the kernel's return value.
    ///
    /// A non-negative `raw` is a success and is passed through unchanged, so
    /// this is only interesting for the negative case: `-errno`, the Linux
    /// convention, which is what `std::io::Error::from_raw_os_error` expects.
    ///
    /// # On the legacy `u64::MAX`
    ///
    /// The TrangorgeOS kernel's dispatcher ends in `_ => u64::MAX`, so a
    /// syscall it does not implement reads as `-1`. Guessing that `-1` means
    /// "unknown" was tried and rejected: `-1` is `EPERM`, a real error that
    /// `open`/`access` on a capability-denied path return, and silently turning
    /// those into "not implemented" would hide exactly the failures the
    /// capability model exists to surface. The contract is therefore the plain
    /// one — a missing syscall must answer `-ENOSYS`, which is a one-line change
    /// to `kernel/src/userspace/process/syscall.rs` and is written up in
    /// `SYSCALLS.md`.
    #[inline]
    pub fn from_raw(raw: i64) -> Result<usize> {
        if raw >= 0 {
            return Ok(raw as usize);
        }
        Err(Errno(raw.saturating_neg() as i32))
    }

    /// The value the kernel must put in `rax` to report this error.
    #[inline]
    pub const fn to_raw(self) -> i64 {
        -(self.0 as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_passes_through() {
        assert_eq!(Errno::from_raw(0), Ok(0));
        assert_eq!(Errno::from_raw(4096), Ok(4096));
    }

    #[test]
    fn negative_rax_is_the_errno() {
        assert_eq!(Errno::from_raw(-2), Err(ENOENT));
        assert_eq!(Errno::from_raw(-12), Err(ENOMEM));
    }

    /// `-1` is `EPERM`, a real error. The kernel's legacy `u64::MAX` for an
    /// unknown syscall is ambiguous with it, and resolving the ambiguity in
    /// favour of "permission denied" is the only safe direction.
    #[test]
    fn minus_one_is_eperm_not_not_implemented() {
        assert_eq!(Errno::from_raw(-1), Err(EPERM));
    }

    #[test]
    fn to_raw_round_trips() {
        for e in [EPERM, ENOENT, ENOMEM, EINVAL, ENOSYS, EBADF, EMFILE] {
            assert_eq!(Errno::from_raw(e.to_raw()), Err(e));
        }
    }

    #[test]
    fn text_is_never_empty() {
        assert_eq!(ENOSYS.text(), "function not implemented");
        assert_eq!(Errno(9999).text(), "unknown error");
    }
}
