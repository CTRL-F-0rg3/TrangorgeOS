//! Process lifetime, identity and scheduling.
//!
//! The one that matters most is `exit_group`: `std::process::exit`, the end of
//! every `main`, and the panic path all end up here. It has to run the `atexit`
//! handlers first and it must never return.

use core::ptr;

use tgs_hal::posix::Posix;
use tgs_hal::sysnum;
use tgs_hal::Syscall;

use crate::cxa;
use crate::shim;
use crate::types::{
    c_char, c_int, c_long, c_uint, c_void, c_ulong, gid_t, pid_t, size_t, uid_t,
};

/// `_exit(int status)` — terminate without unwinding or running handlers.
#[no_mangle]
pub extern "C" fn _exit(status: c_int) -> ! {
    Posix::global().exit_group(status)
}

/// `void exit(int status)`
///
/// Runs the `atexit` handlers, then flushes nothing — there is no buffered stdio
/// here, `write` goes straight to the descriptor — and terminates.
#[no_mangle]
pub extern "C" fn exit(status: c_int) -> ! {
    cxa::run_exit_handlers();
    _exit(status)
}

/// `void abort(void)`
///
/// `SIGABRT` with the default disposition, which on TrangorgeOS is the same as
/// exiting with 134. The distinction is recorded in the message rather than the
/// exit status, because the kernel has no signal machinery yet.
#[no_mangle]
pub extern "C" fn abort() -> ! {
    eprintln_to_stderr("tgs-rt: abort");
    cxa::run_exit_handlers();
    Posix::global().exit_group(134)
}

/// `int raise(int sig)`
///
/// There are no signals to raise, so this reports `ENOSYS` rather than
/// pretending: a caller that relies on `raise` for control flow would otherwise
/// carry on after what it thought was a fatal event.
#[no_mangle]
pub extern "C" fn raise(sig: c_int) -> c_int {
    let _ = sig;
    shim::fail(tgs_hal::errno::ENOSYS)
}

/// `pid_t getpid(void)`
#[no_mangle]
pub extern "C" fn getpid() -> pid_t {
    Posix::global().getpid().unwrap_or(0) as pid_t
}

/// `pid_t gettid(void)`
#[no_mangle]
pub extern "C" fn gettid() -> pid_t {
    Posix::global().gettid().unwrap_or(0) as pid_t
}

/// `pid_t getppid(void)` — the TrangorgeOS process table has no parent link yet.
#[no_mangle]
pub extern "C" fn getppid() -> pid_t {
    0
}

/// `uid_t getuid(void)` — every process runs as the session owner.
#[no_mangle]
pub extern "C" fn getuid() -> uid_t {
    0
}

/// `uid_t geteuid(void)`
#[no_mangle]
pub extern "C" fn geteuid() -> uid_t {
    getuid()
}

/// `gid_t getgid(void)`
#[no_mangle]
pub extern "C" fn getgid() -> gid_t {
    0
}

/// `gid_t getegid(void)`
#[no_mangle]
pub extern "C" fn getegid() -> gid_t {
    getgid()
}

/// `int sched_yield(void)`
#[no_mangle]
pub extern "C" fn sched_yield() -> c_int {
    shim::code(Posix::global().sched_yield(), |_| 0i32)
}

/// `int sched_getaffinity(pid_t pid, size_t size, cpu_set_t *mask)`
///
/// `std` reads this to size its thread-affinity assumptions, so a lying answer
/// makes it believe there are more CPUs than there are. Reporting one CPU is the
/// conservative answer.
#[no_mangle]
pub extern "C" fn sched_getaffinity(
    _pid: pid_t,
    size: size_t,
    mask: *mut usize,
) -> c_int {
    const WORD: usize = core::mem::size_of::<usize>();
    if mask.is_null() {
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    if size < WORD {
        // `EINVAL`, as the kernel would: the caller's buffer is smaller than one
        // bitmask word.
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    // SAFETY: the caller passed a non-null buffer of at least one word, which is
    // what the kernel-side contract requires.
    unsafe { ptr::write(mask, 1) };
    shim::fail(tgs_hal::errno::ENOSYS)
}

/// `int prctl(int option, ...)`
///
/// The only option TrangorgeOS understands is `PR_SET_NAME` (15), which is a
/// pure label; the kernel has no threads to name, so it is accepted and dropped.
#[no_mangle]
pub extern "C" fn prctl(option: c_int, _a: c_ulong, _b: c_ulong, _c: c_ulong, _d: c_ulong) -> c_int {
    match option {
        15 => 0,
        _ => shim::fail(tgs_hal::errno::ENOSYS),
    }
}

/// `int syscall(long number, ...)`
///
/// The generic escape hatch. Variadic, so only the fixed six-argument shape can
/// be reconstructed reliably; anything else is refused rather than guessed at.
#[no_mangle]
pub extern "C" fn syscall(number: c_long, a: c_ulong, b: c_ulong, c: c_ulong, d: c_ulong, e: c_ulong, f: c_ulong) -> c_long {
    // SAFETY: forwarding integers only. The `Syscall` contract is that a gate does
    // not dereference the argument words, and `Gate` only reads them as numbers.
    let raw = unsafe {
        tgs_hal::gate::GATE.call(number as u64, [a, b, c, d, e, f])
    };
    if raw < 0 {
        shim::fail_long(tgs_hal::errno::Errno::from_raw(raw).err().unwrap_or(tgs_hal::errno::EIO))
    } else {
        crate::errno::clear();
        raw
    }
}

/// `int getpagesize(void)`
#[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    tgs_hal::heap::PAGE as c_int
}

/// `long sysconf(int name)` — only `_SC_PAGESIZE` is answered.
///
/// `_SC_NPROCESSORS_ONLN` returns 1, and `std::thread::available_parallelism`
/// asks for exactly that, so answering it wrongly would let a program size a
/// thread pool for a machine that does not exist.
#[no_mangle]
pub extern "C" fn sysconf(name: c_int) -> c_long {
    const SC_NPROCESSORS_ONLN: c_int = 84;
    const SC_PAGESIZE: c_int = 30;
    match name {
        SC_PAGESIZE => tgs_hal::heap::PAGE as c_long,
        SC_NPROCESSORS_ONLN => 1,
        _ => {
            // -1 with `errno` set is how a C caller detects "no answer"; TrangorgeOS
            // uses the POSIX value of -1 rather than glibc's.
            shim::fail(tgs_hal::errno::EINVAL);
            -1
        }
    }
}

/// `int uname(struct utsname *buf)`
#[no_mangle]
pub extern "C" fn uname(buf: *mut crate::types::utsname) -> c_int {
    if buf.is_null() {
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    // SAFETY: `buf` is non-null and, per the contract of `uname`, points at
    // writable storage for a whole `utsname`.
    unsafe {
        let b = &mut *buf;
        copy_fixed(&mut b.sysname, b"TrangorgeOS");
        copy_fixed(&mut b.nodename, b"tgs");
        copy_fixed(&mut b.release, b"0.182");
        copy_fixed(&mut b.version, b"TrangorgeOS userspace (ustd)");
        copy_fixed(&mut b.machine, b"x86_64");
        copy_fixed(&mut b.domainname, b"(none)");
    }
    shim::fail(tgs_hal::errno::ENOSYS)
}

/// Copy `src` into a fixed-size, NUL-padded C string field.
fn copy_fixed(dst: &mut [c_char; 65], src: &[u8]) {
    let n = src.len().min(dst.len() - 1);
    // SAFETY: `dst` is a mutable array reference, so every index below is in
    // bounds by construction, and `i8` is the right type for a `char` field.
    unsafe {
        for (i, &b) in src[..n].iter().enumerate() {
            *dst.get_unchecked_mut(i) = b as c_char;
        }
        for slot in dst[n..].iter_mut() {
            *slot = 0;
        }
    }
}

/// `int set_tid_address(int *tidptr)` — no threads, so nothing to clear.
#[no_mangle]
pub extern "C" fn set_tid_address(_tidptr: *mut c_int) -> c_int {
    shim::fail(tgs_hal::errno::ENOSYS)
}

/// `void *dlopen(const char *filename, int flags)` — no dynamic loader.
#[no_mangle]
pub extern "C" fn dlopen(_filename: *const c_char, _flags: c_int) -> *mut c_void {
    crate::errno::set_errno(tgs_hal::errno::ENOSYS);
    ptr::null_mut()
}

/// `char *dlerror(void)`
#[no_mangle]
pub extern "C" fn dlerror() -> *const c_char {
    static MSG: [u8; 7] = *b"no dlr\0";
    MSG.as_ptr().cast()
}

/// Write a message to descriptor 2, for a fatal path that has no `std` yet.
fn eprintln_to_stderr(msg: &str) {
    let _ = Posix::global().write(2, msg.as_bytes());
}

/// `int atoi(const char *nptr)` — libc's own, because `std` does not provide it.
#[no_mangle]
pub extern "C" fn atoi(nptr: *const c_char) -> c_int {
    let mut p = nptr;
    if p.is_null() {
        return 0;
    }
    // SAFETY: the caller passed a NUL-terminated string.
    unsafe {
        while *p == b' ' as c_char {
            p = p.add(1);
        }
        let neg = *p == b'-' as c_char;
        if neg || *p == b'+' as c_char {
            p = p.add(1);
        }
        let mut v: c_int = 0;
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            v = v.saturating_mul(10).saturating_add((*p as c_int) - ('0' as c_int));
            p = p.add(1);
        }
        if neg {
            -v
        } else {
            v
        }
    }
}

/// `int kill(pid_t pid, int sig)` — no signals, and no lying about it.
#[no_mangle]
pub extern "C" fn kill(_pid: pid_t, _sig: c_int) -> c_int {
    shim::fail(tgs_hal::errno::ENOSYS)
}

/// `int tgs_cap_query(uint8_t id)` — the capability API, for a program that wants
/// to ask the kernel what it may do rather than finding out by being denied.
#[no_mangle]
pub extern "C" fn tgs_cap_query(id: c_uint) -> c_int {
    // SAFETY: the gate does not dereference its arguments.
    let raw = unsafe {
        tgs_hal::gate::GATE.call(sysnum::TGS_CAP_QUERY, [id as u64, 0, 0, 0, 0, 0])
    };
    raw as c_int
}
