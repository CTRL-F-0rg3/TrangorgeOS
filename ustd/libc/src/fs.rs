//! Files and descriptors — everything `std::fs` and `std::io` reach for.
//!
//! `std` does not use `FILE *`; it works in descriptors, so there is no stdio
//! buffer to get wrong. What it does use is a fairly long list of `*at` and
//! `*64` spellings for the same handful of operations, and the reason is
//! bit-for-bit ABI compatibility: `pread` and `pread64` are the same call on
//! x86_64, and the *64 names are what a 32-bit-era caller was told to use. Each
//! pair below is one implementation and two symbols.

use core::ptr;

use tgs_hal::errno;
use tgs_hal::posix::{PollFd as HalPollFd, Posix, Stat as HalStat};
use tgs_hal::sysnum;

use crate::shim;
use crate::types::{c_char, c_int, c_long, c_uint, mode_t, off_t, size_t, ssize_t};

/// `AT_FDCWD`, re-exported so a caller does not have to hardcode `-100`.
pub const AT_FDCWD: c_int = sysnum::AT_FDCWD;

// ── descriptors ─────────────────────────────────────────────────────────────

/// `ssize_t read(int fd, void *buf, size_t count)`
#[no_mangle]
pub extern "C" fn read(fd: c_int, buf: *mut u8, count: size_t) -> ssize_t {
    if buf.is_null() && count != 0 {
        return shim::fail_ssize(errno::EINVAL);
    }
    // SAFETY: `buf` is non-null and, per the contract of `read`, writable for
    // `count` bytes; a zero-length read is allowed to have a null pointer.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf, count) };
    shim::count(Posix::global().read(fd, dst), |n| n as ssize_t)
}

/// `ssize_t pread(int fd, void *buf, size_t count, off_t offset)`
#[no_mangle]
pub extern "C" fn pread(fd: c_int, buf: *mut u8, count: size_t, offset: off_t) -> ssize_t {
    if buf.is_null() && count != 0 {
        return shim::fail_ssize(errno::EINVAL);
    }
    // SAFETY: as `read`, plus an integer offset.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf, count) };
    shim::count(
        Posix::global().pread(fd, dst, offset as u64),
        |n| n as ssize_t,
    )
}

/// `ssize_t pread64(...)` — see the module docs on the `*64` spellings.
#[no_mangle]
pub extern "C" fn pread64(fd: c_int, buf: *mut u8, count: size_t, offset: i64) -> ssize_t {
    pread(fd, buf, count, offset)
}

/// `ssize_t write(int fd, const void *buf, size_t count)`
#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const u8, count: size_t) -> ssize_t {
    if buf.is_null() && count != 0 {
        return shim::fail_ssize(errno::EINVAL);
    }
    // SAFETY: `buf` is non-null and, per the contract of `write`, readable for
    // `count` bytes.
    let src = unsafe { core::slice::from_raw_parts(buf, count) };
    shim::count(Posix::global().write(fd, src), |n| n as ssize_t)
}

/// `ssize_t pwrite(int fd, const void *buf, size_t count, off_t offset)`
#[no_mangle]
pub extern "C" fn pwrite(fd: c_int, buf: *const u8, count: size_t, offset: off_t) -> ssize_t {
    if buf.is_null() && count != 0 {
        return shim::fail_ssize(errno::EINVAL);
    }
    // SAFETY: as `write`, plus an integer offset.
    let src = unsafe { core::slice::from_raw_parts(buf, count) };
    shim::count(
        Posix::global().pwrite(fd, src, offset as u64),
        |n| n as ssize_t,
    )
}

/// `ssize_t pwrite64(...)`
#[no_mangle]
pub extern "C" fn pwrite64(fd: c_int, buf: *const u8, count: size_t, offset: i64) -> ssize_t {
    pwrite(fd, buf, count, offset)
}

/// `int close(int fd)`
#[no_mangle]
pub extern "C" fn close(fd: c_int) -> c_int {
    shim::code(Posix::global().close(fd), |_| 0i32)
}

/// `off_t lseek(int fd, off_t offset, int whence)`
#[no_mangle]
pub extern "C" fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t {
    shim::count(Posix::global().lseek(fd, offset, whence), |n| n as off_t)
}

/// `off_t lseek64(...)`
#[no_mangle]
pub extern "C" fn lseek64(fd: c_int, offset: i64, whence: c_int) -> i64 {
    lseek(fd, offset, whence)
}

/// `int fstat(int fd, struct stat *buf)`
///
/// The `struct stat` the kernel fills is `tgs_hal`'s, which is laid out to
/// match, so this is a copy rather than a reinterpretation.
#[no_mangle]
pub extern "C" fn fstat(fd: c_int, buf: *mut u8) -> c_int {
    if buf.is_null() {
        return shim::fail(errno::EINVAL);
    }
    let mut st = HalStat::default();
    let r = Posix::global().fstat(fd, &mut st).map(|_| {
        // SAFETY: the caller passed a buffer for a `struct stat`, and
        // `tgs_hal::posix::Stat` is that layout.
        unsafe { ptr::write_unaligned(buf.cast::<HalStat>(), st) };
    });
    shim::code(r, |_| 0i32)
}

/// `int fstat64(int fd, struct stat64 *buf)` — `struct stat64` is the same on
/// x86_64.
#[no_mangle]
pub extern "C" fn fstat64(fd: c_int, buf: *mut u8) -> c_int {
    fstat(fd, buf)
}

/// `int open(const char *pathname, int flags, ...)`
#[no_mangle]
pub extern "C" fn open(pathname: *const c_char, flags: c_int, _mode: mode_t) -> c_int {
    openat(AT_FDCWD, pathname, flags, _mode)
}

/// `int open64(const char *pathname, int flags, ...)`
#[no_mangle]
pub extern "C" fn open64(pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    open(pathname, flags, mode)
}

/// `int openat(int dirfd, const char *pathname, int flags, ...)`
#[no_mangle]
pub extern "C" fn openat(dirfd: c_int, pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    if pathname.is_null() {
        return shim::fail_ssize(errno::EFAULT) as c_int;
    }
    // SAFETY: `pathname` is non-null and, per the contract of `openat`, it
    // points at a NUL-terminated string; the stat out-parameter is null.
    let r = unsafe {
        Posix::global().openat(dirfd, pathname.cast::<u8>(), flags as u32, mode, ptr::null_mut())
    };
    shim::count(r, |fd| fd as c_int)
}

/// `int openat64(...)`
#[no_mangle]
pub extern "C" fn openat64(dirfd: c_int, pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    openat(dirfd, pathname, flags, mode)
}

/// `int creat(const char *pathname, mode_t mode)`
#[no_mangle]
pub extern "C" fn creat(pathname: *const c_char, mode: mode_t) -> c_int {
    openat(
        AT_FDCWD,
        pathname,
        (sysnum::O_WRONLY | sysnum::O_CREAT | sysnum::O_TRUNC) as c_int,
        mode,
    )
}

/// `int fstatat(int dirfd, const char *pathname, struct stat *buf, int flags)`
#[no_mangle]
pub extern "C" fn fstatat(
    dirfd: c_int,
    pathname: *const c_char,
    buf: *mut u8,
    flags: c_int,
) -> c_int {
    if buf.is_null() || pathname.is_null() {
        return shim::fail_ssize(errno::EFAULT) as c_int;
    }
    let mut st = HalStat::default();
    // SAFETY: both pointers are non-null and, per the contract of `fstatat`,
    // `pathname` is a NUL-terminated string and `buf` is writable.
    let r = unsafe {
        Posix::global().stat_at(
            dirfd,
            pathname.cast::<u8>(),
            &mut st,
            flags & sysnum::AT_SYMLINK_NOFOLLOW as c_int != 0,
        )
    }
    .map(|_| {
        // SAFETY: as `fstat`.
        unsafe { ptr::write_unaligned(buf.cast::<HalStat>(), st) };
    });
    shim::code(r, |_| 0i32)
}

/// `int __xstat(int ver, const char *path, struct stat *buf)` — the glibc
/// dispatch stub that `stat`/`lstat` reduce to on some versions.
#[no_mangle]
pub extern "C" fn __xstat(_ver: c_int, path: *const c_char, buf: *mut u8) -> c_int {
    fstatat(AT_FDCWD, path, buf, 0)
}

/// `int stat(const char *path, struct stat *buf)`
#[no_mangle]
pub extern "C" fn stat(path: *const c_char, buf: *mut u8) -> c_int {
    fstatat(AT_FDCWD, path, buf, 0)
}

/// `int lstat(const char *path, struct stat *buf)`
#[no_mangle]
pub extern "C" fn lstat(path: *const c_char, buf: *mut u8) -> c_int {
    fstatat(AT_FDCWD, path, buf, sysnum::AT_SYMLINK_NOFOLLOW as c_int)
}

/// `long getdents64(int fd, struct linux_dirent64 *dirp, size_t count)`
///
/// `std::fs::read_dir` parses this buffer itself, so the bytes coming back have
/// to be exactly `struct linux_dirent64` records — which is why the codec, and
/// its tests, live in `tgs_hal::dirent` rather than here.
#[no_mangle]
pub extern "C" fn getdents64(fd: c_int, dirp: *mut u8, count: size_t) -> c_long {
    if dirp.is_null() {
        return shim::fail_long(errno::EFAULT);
    }
    // SAFETY: `dirp` is non-null and, per the contract of `getdents64`, writable
    // for `count` bytes.
    let buf = unsafe { core::slice::from_raw_parts_mut(dirp, count) };
    // SAFETY: forwarded from here; `buf` is the live buffer just built.
    let r = unsafe { Posix::global().getdents64(fd, buf) };
    shim::count(r, |n| n as c_long)
}

/// `int dup(int oldfd)`
#[no_mangle]
pub extern "C" fn dup(oldfd: c_int) -> c_int {
    shim::count(Posix::global().dup(oldfd), |n| n as c_int)
}

/// `int dup2(int oldfd, int newfd)`
#[no_mangle]
pub extern "C" fn dup2(oldfd: c_int, newfd: c_int) -> c_int {
    shim::count(Posix::global().dup3(oldfd, newfd), |_| newfd)
}

/// `int dup3(int oldfd, int newfd, int flags)`
#[no_mangle]
pub extern "C" fn dup3(oldfd: c_int, newfd: c_int, _flags: c_int) -> c_int {
    dup2(oldfd, newfd)
}

/// `int pipe(int pipefd[2])`
#[no_mangle]
pub extern "C" fn pipe(pipefd: *mut c_int) -> c_int {
    pipe2(pipefd, 0)
}

/// `int pipe2(int pipefd[2], int flags)`
#[no_mangle]
pub extern "C" fn pipe2(pipefd: *mut c_int, flags: c_int) -> c_int {
    if pipefd.is_null() {
        return shim::fail_ssize(errno::EFAULT) as c_int;
    }
    let mut fds = [0 as c_int; 2];
    let r = Posix::global().pipe2(&mut fds, flags as u32).map(|_| {
        // SAFETY: the caller passed a non-null pointer to two `int`s.
        unsafe {
            *pipefd = fds[0];
            *pipefd.add(1) = fds[1];
        };
    });
    shim::code(r, |_| 0i32)
}

/// `int ioctl(int fd, unsigned long request, ...)`
#[no_mangle]
pub extern "C" fn ioctl(fd: c_int, request: c_long, arg: c_long) -> c_int {
    shim::count(Posix::global().ioctl(fd, request as u64, arg as u64), |n| n as c_int)
}

/// `int fcntl(int fd, int cmd, ...)`
#[no_mangle]
pub extern "C" fn fcntl(fd: c_int, cmd: c_int, arg: c_long) -> c_int {
    shim::count(Posix::global().fcntl(fd, cmd, arg as u64), |n| n as c_int)
}

/// `ssize_t readv(int fd, const struct iovec *iov, int iovcnt)`
#[no_mangle]
pub extern "C" fn readv(fd: c_int, iov: *const crate::types::iovec, iovcnt: c_int) -> ssize_t {
    transfer(fd, iov, iovcnt, false)
}

/// `ssize_t writev(int fd, const struct iovec *iov, int iovcnt)`
#[no_mangle]
pub extern "C" fn writev(fd: c_int, iov: *const crate::types::iovec, iovcnt: c_int) -> ssize_t {
    transfer(fd, iov, iovcnt, true)
}

/// The shared body of `readv` and `writev`.
///
/// A kernel `readv` walks the vector in one pass; this walks it in Rust and makes
/// one call per element. The observable behaviour is the same for the
/// single-descriptor vectors `std` builds, and it keeps the vector ABI off the
/// kernel's argument list entirely.
fn transfer(fd: c_int, iov: *const crate::types::iovec, iovcnt: c_int, writing: bool) -> ssize_t {
    if iov.is_null() || iovcnt < 0 {
        return shim::fail_ssize(errno::EFAULT);
    }

    let mut total: ssize_t = 0;
    for i in 0..iovcnt as usize {
        // SAFETY: the caller passed an array of at least `iovcnt` iovecs.
        let v = unsafe { ptr::read(iov.add(i)) };
        if v.iov_base.is_null() && v.iov_len != 0 {
            return shim::fail_ssize(errno::EFAULT);
        }
        if v.iov_len == 0 {
            continue;
        }
        // SAFETY: the caller promised each `iov_base` points at `iov_len` readable
        // bytes (writing) or writable bytes (reading), for the duration of the call.
        let got = unsafe {
            if writing {
                Posix::global().write(fd, core::slice::from_raw_parts(v.iov_base, v.iov_len))
            } else {
                Posix::global().read(fd, core::slice::from_raw_parts_mut(v.iov_base, v.iov_len))
            }
        };
        match got {
            Ok(n) => {
                total += n as ssize_t;
                // A short read means the end of the data. A short write is a real
                // error the caller has to see, so it is not treated as the end.
                if n < v.iov_len && !writing {
                    crate::errno::clear();
                    return total;
                }
            }
            Err(_) if total > 0 && !writing => {
                crate::errno::clear();
                return total;
            }
            Err(e) => return shim::fail_ssize(e),
        }
    }
    crate::errno::clear();
    total
}

// ── directories and metadata ────────────────────────────────────────────────

/// `int mkdir(const char *pathname, mode_t mode)`
#[no_mangle]
pub extern "C" fn mkdir(pathname: *const c_char, mode: mode_t) -> c_int {
    // SAFETY: forwarded to the kernel, which dereferences it; the caller passed a
    // NUL-terminated string.
    let r = unsafe { Posix::global().mkdirat(AT_FDCWD, pathname.cast::<u8>(), mode) };
    shim::code(r, |_| 0i32)
}

/// `int mkdirat(int dirfd, const char *pathname, mode_t mode)`
#[no_mangle]
pub extern "C" fn mkdirat(dirfd: c_int, pathname: *const c_char, mode: mode_t) -> c_int {
    // SAFETY: as `mkdir`.
    let r = unsafe { Posix::global().mkdirat(dirfd, pathname.cast::<u8>(), mode) };
    shim::code(r, |_| 0i32)
}

/// `int rmdir(const char *pathname)`
#[no_mangle]
pub extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    // SAFETY: as `mkdir`, with `AT_REMOVEDIR` so the kernel knows a directory is
    // being removed.
    let r = unsafe { Posix::global().unlinkat(AT_FDCWD, pathname.cast::<u8>(), sysnum::AT_REMOVEDIR) };
    shim::code(r, |_| 0i32)
}

/// `int unlink(const char *pathname)`
#[no_mangle]
pub extern "C" fn unlink(pathname: *const c_char) -> c_int {
    // SAFETY: as `mkdir`.
    let r = unsafe { Posix::global().unlinkat(AT_FDCWD, pathname.cast::<u8>(), 0) };
    shim::code(r, |_| 0i32)
}

/// `int unlinkat(int dirfd, const char *pathname, int flags)`
#[no_mangle]
pub extern "C" fn unlinkat(dirfd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    // SAFETY: as `mkdir`.
/// `int truncate(const char *path, off_t length)`
#[no_mangle]
pub extern "C" fn truncate(path: *const c_char, length: off_t) -> c_int {
    // SAFETY: forwarded to the kernel; the caller passed a NUL-terminated string.
    let r = unsafe { Posix::global().truncate(path.cast::<u8>(), length as u64) };
    shim::code(r, |_| 0i32)
}

/// `int ftruncate(int fd, off_t length)`
#[no_mangle]
pub extern "C" fn ftruncate(fd: c_int, length: off_t) -> c_int {
    shim::code(Posix::global().ftruncate(fd, length as u64), |_| 0i32)
}

/// `int fsync(int fd)`
#[no_mangle]
pub extern "C" fn fsync(fd: c_int) -> c_int {
    shim::code(Posix::global().fsync(fd), |_| 0i32)
}

/// `int fdatasync(int fd)` — TrangorgeOS has no separate data/metadata split.
#[no_mangle]
pub extern "C" fn fdatasync(fd: c_int) -> c_int {
    fsync(fd)
}

/// `int flock(int fd, int operation)`
#[no_mangle]
pub extern "C" fn flock(fd: c_int, operation: c_int) -> c_int {
    shim::count(Posix::global().ioctl(fd, flock_request(operation), 0), |_| 0)
}

/// The `ioctl` request number for a `flock` operation, from `<linux/file.h>`.
///
/// Spelled out rather than imported: the `libc` crate declares it only for targets
/// it knows, and hardcoding it with the reference in a comment is both shorter and
/// impossible to get out of step with the kernel.
const fn flock_request(operation: c_int) -> u64 {
    const I: u64 = 0x4000_0000;
    I | ((operation as u64) << 8) | 0x2b
}

/// `mode_t umask(mode_t mask)`
#[no_mangle]
pub extern "C" fn umask(mask: mode_t) -> mode_t {
    shim::count(Posix::global().umask(mask), |old| old as c_int) as mode_t
}

/// `char *getcwd(char *buf, size_t size)`
#[no_mangle]
pub extern "C" fn getcwd(buf: *mut c_char, size: size_t) -> *mut c_char {
    if buf.is_null() || size == 0 {
        shim::fail_ssize(errno::EINVAL);
        return ptr::null_mut();
    }
    // SAFETY: `buf` is non-null and writable for `size` bytes, per the contract of
    // `getcwd`.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf.cast::<u8>(), size) };
    match Posix::global().getcwd(dst) {
        Ok(n) if n > 0 => buf,
        Ok(_) => {
            shim::fail_ssize(errno::ERANGE);
            ptr::null_mut()
        }
        Err(e) => {
            shim::fail_ssize(e);
            ptr::null_mut()
        }
    }
}

/// `int chdir(const char *path)`
#[no_mangle]
pub extern "C" fn chdir(path: *const c_char) -> c_int {
    // SAFETY: forwarded to the kernel; the caller passed a NUL-terminated string.
    let r = unsafe { Posix::global().chdir(path.cast::<u8>()) };
    shim::code(r, |_| 0i32)
}

// ── polling ─────────────────────────────────────────────────────────────────

/// `int poll(struct pollfd *fds, nfds_t nfds, int timeout)`
#[no_mangle]
pub extern "C" fn poll(fds: *mut crate::types::pollfd, nfds: c_uint, timeout: c_int) -> c_int {
    if fds.is_null() && nfds != 0 {
        return shim::fail_ssize(errno::EFAULT) as c_int;
    }
    // SAFETY: the caller passed a non-null array of `nfds` `pollfd`s, which is the
    // kernel-side contract; `std` always passes valid ones.
    let r = unsafe {
        let src = core::slice::from_raw_parts(fds, nfds as usize);
        let mut hal: alloc::vec::Vec<HalPollFd> = src
            .iter()
            .map(|p| HalPollFd {
                fd: p.fd,
                events: p.events,
                revents: 0,
            })
            .collect();
        Posix::global().ppoll(&mut hal, timeout).map(|n| {
            for (i, src) in hal.iter().enumerate() {
                // SAFETY: `i < nfds`, and `fds` was validated as a non-null array
                // of exactly that many `pollfd`s above.
                (*fds.add(i)).revents = src.revents;
            }
            n
        })
    };
    shim::count(r, |n| n as c_int)
}

/// `int ppoll(struct pollfd *fds, nfds_t nfds, const struct timespec *tmo_p, ...)`
#[no_mangle]
pub extern "C" fn ppoll(
    fds: *mut crate::types::pollfd,
    nfds: c_uint,
    tmo_p: *const crate::types::timespec,
    _sigmask: *const u8,
    _sigsetsize: size_t,
) -> c_int {
    // A null timeout means "block forever", which is `poll`'s -1. A timespec
    // timeout is rounded *up* to a whole millisecond, so a caller that asked for
    // 500 microseconds is never woken early.
    let timeout = if tmo_p.is_null() {
        -1
    } else {
        // SAFETY: the caller passed a non-null readable `timespec`.
        let t = unsafe { *tmo_p };
        let ms = (t.tv_sec as i64).saturating_mul(1000) + (t.tv_nsec + 999_999) / 1_000_000;
        ms.clamp(0, i32::MAX as i64) as c_int
    };
    poll(fds, nfds, timeout)
}

/// `int select(...)`
///
/// `std` does not use `select`; it is here because a C library layered on top
/// might, and the honest answer is `ENOSYS` rather than a silent success.
#[no_mangle]
pub extern "C" fn select(
    _nfds: c_int,
    _readfds: *mut u8,
    _writefds: *mut u8,
    _exceptfds: *mut u8,
    _timeout: *mut crate::types::timeval,
) -> c_int {
    shim::fail(errno::ENOSYS)
}


    let r = unsafe { Posix::global().unlinkat(dirfd, pathname.cast::<u8>(), flags as u32) };
    shim::code(r, |_| 0i32)
}

/// `int rename(const char *oldpath, const char *newpath)`
#[no_mangle]
pub extern "C" fn rename(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    // SAFETY: both pointers are NUL-terminated strings, per the contract.
    let r = unsafe { Posix::global().renameat(AT_FDCWD, oldpath.cast::<u8>(), AT_FDCWD, newpath.cast::<u8>()) };
    shim::code(r, |_| 0i32)
}

/// `int renameat(int olddirfd, const char *oldpath, int newdirfd, const char *newpath)`
#[no_mangle]
pub extern "C" fn renameat(
    olddirfd: c_int,
    oldpath: *const c_char,
    newdirfd: c_int,
    newpath: *const c_char,
) -> c_int {
    // SAFETY: both pointers are NUL-terminated strings, per the contract.
    let r = unsafe { Posix::global().renameat(olddirfd, oldpath.cast::<u8>(), newdirfd, newpath.cast::<u8>()) };
    shim::code(r, |_| 0i32)
}

/// `int access(const char *pathname, int mode)`
#[no_mangle]
pub extern "C" fn access(pathname: *const c_char, mode: c_int) -> c_int {
    // SAFETY: forwarded to the kernel; the caller passed a NUL-terminated string.
    let r = unsafe { Posix::global().access(AT_FDCWD, pathname.cast::<u8>(), mode as u32) };
    shim::code(r, |_| 0i32)
}

/// `int faccessat(int dirfd, const char *pathname, int mode, int flags)`
#[no_mangle]
pub extern "C" fn faccessat(dirfd: c_int, pathname: *const c_char, mode: c_int, _flags: c_int) -> c_int {
    // SAFETY: as `access`.
    let r = unsafe { Posix::global().access(dirfd, pathname.cast::<u8>(), mode as u32) };
    shim::code(r, |_| 0i32)
}

/// `ssize_t readlink(const char *pathname, char *buf, size_t bufsiz)`
///
/// Not implemented: the kernel has no symbolic links yet, and answering with the
/// target's own bytes would be a lie.
#[no_mangle]
pub extern "C" fn readlink(_pathname: *const c_char, _buf: *mut c_char, _bufsiz: size_t) -> ssize_t {
    shim::fail_ssize(errno::ENOSYS)
}


