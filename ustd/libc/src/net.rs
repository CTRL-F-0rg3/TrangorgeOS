//! Sockets.
//!
//! `std::net` is a large module and this is a thin shim over it, because
//! TrangorgeOS's network stack is not in userspace yet: the kernel has a NIC
//! driver (`kernel/src/nic/`, `virtio`) and driver-space networking is an explicit
//! project priority, but there is no ring-3 socket ABI. Every call here reaches
//! the `0x3000` block in [`tgs_hal::sysnum`] and reports what the kernel says —
//! including `ENOSYS`, which is the correct answer today and a working answer
//! once `kernel/src/userspace/process/syscall.rs` grows the `net` arms.
//!
//! `getaddrinfo` is the exception: it is the one entry point with no syscall behind
//! it, because name resolution is a policy question — a hosts file, DNS, or
//! neither — that belongs to the kernel. It answers `EAI_NONAME` for everything,
//! so a caller that already has a numeric address (which is what
//! `std::net::ToSocketAddrs` parses before it ever asks) works with no resolver
//! at all, and a caller that wants a name finds out immediately.

use core::ptr;

use tgs_hal::errno;
use tgs_hal::posix::Posix;
use tgs_hal::sysnum;

use crate::shim;
use crate::types::{c_char, c_int, c_void, size_t, ssize_t};

/// `int socket(int domain, int type, int protocol)`
#[no_mangle]
pub extern "C" fn socket(domain: c_int, kind: c_int, protocol: c_int) -> c_int {
    shim::count(Posix::global().socket(domain, kind, protocol), |fd| fd as c_int)
}

/// `int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen)`
#[no_mangle]
pub extern "C" fn connect(fd: c_int, addr: *const u8, addrlen: u32) -> c_int {
    if addr.is_null() {
        return shim::fail(errno::EFAULT) as c_int;
    }
    // SAFETY: forwarded to the kernel, which reads `addrlen` bytes; the caller
    // passed a real socket address.
    let r = unsafe { Posix::global().connect(fd, addr, addrlen) };
    shim::code(r, |_| 0i32)
}

/// `ssize_t send(int sockfd, const void *buf, size_t len, int flags)`
#[no_mangle]
pub extern "C" fn send(fd: c_int, buf: *const u8, len: size_t, flags: c_int) -> ssize_t {
    if buf.is_null() && len != 0 {
        return shim::fail_ssize(errno::EFAULT);
    }
    // SAFETY: `buf` is non-null and readable for `len` bytes, per the contract.
    let src = unsafe { core::slice::from_raw_parts(buf, len) };
    shim::count(Posix::global().send(fd, src, flags), |n| n as ssize_t)
}

/// `ssize_t recv(int sockfd, void *buf, size_t len, int flags)`
#[no_mangle]
pub extern "C" fn recv(fd: c_int, buf: *mut u8, len: size_t, flags: c_int) -> ssize_t {
    if buf.is_null() && len != 0 {
        return shim::fail_ssize(errno::EFAULT);
    }
    // SAFETY: `buf` is non-null and writable for `len` bytes, per the contract.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    shim::count(Posix::global().recv(fd, dst, flags), |n| n as ssize_t)
}

/// `ssize_t sendto(...)`
#[no_mangle]
pub extern "C" fn sendto(
    fd: c_int,
    buf: *const u8,
    len: size_t,
    flags: c_int,
    _dest: *const u8,
    _addrlen: u32,
) -> ssize_t {
    send(fd, buf, len, flags)
}

/// `ssize_t recvfrom(...)`
#[no_mangle]
pub extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut u8,
    len: size_t,
    flags: c_int,
    _src: *mut u8,
    _addrlen: *mut u32,
) -> ssize_t {
    recv(fd, buf, len, flags)
}

/// `int bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen)`
#[no_mangle]
pub extern "C" fn bind(_fd: c_int, _addr: *const u8, _addrlen: u32) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int listen(int sockfd, int backlog)`
#[no_mangle]
pub extern "C" fn listen(_fd: c_int, _backlog: c_int) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen)`
#[no_mangle]
pub extern "C" fn accept(_fd: c_int, _addr: *mut u8, _addrlen: *mut u32) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int setsockopt(...)`
#[no_mangle]
pub extern "C" fn setsockopt(
    _fd: c_int,
    _level: c_int,
    _optname: c_int,
    _optval: *const c_void,
    _optlen: u32,
) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int getsockopt(...)`
#[no_mangle]
pub extern "C" fn getsockopt(
    _fd: c_int,
    _level: c_int,
    _optname: c_int,
    _optval: *mut c_void,
    _optlen: *mut u32,
) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int shutdown(int sockfd, int how)`
#[no_mangle]
pub extern "C" fn shutdown(_fd: c_int, _how: c_int) -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}

/// `int gethostname(char *name, size_t len)`
#[no_mangle]
pub extern "C" fn gethostname(name: *mut c_char, len: size_t) -> c_int {
    const HOST: &[u8] = b"tgs\0";
    if name.is_null() {
        return shim::fail(errno::EFAULT) as c_int;
    }
    let n = HOST.len().min(len);
    // SAFETY: the caller passed a non-null buffer of `len` writable bytes.
    unsafe { ptr::copy_nonoverlapping(HOST.as_ptr(), name.cast::<u8>(), n) };
    0
}

/// The `EAI_*` values `std` compares against.
pub mod eai {
    use crate::types::c_int;

    /// Success.
    pub const EAI_OK: c_int = 0;
    /// The name does not resolve.
    pub const EAI_NONAME: c_int = -2;
    /// The service does not resolve.
    pub const EAI_SERVICE: c_int = -8;
}

/// `struct addrinfo`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct addrinfo {
    /// `AI_PASSIVE` and friends.
    pub ai_flags: c_int,
    /// `AF_INET` or `AF_INET6`.
    pub ai_family: c_int,
    /// `SOCK_STREAM` or `SOCK_DGRAM`.
    pub ai_socktype: c_int,
    /// `IPPROTO_TCP` or `IPPROTO_UDP`.
    pub ai_protocol: c_int,
    /// Length of `ai_addr`.
    pub ai_addrlen: u32,
    /// The address.
    pub ai_addr: *mut u8,
    /// Canonical name, or null.
    pub ai_canonname: *mut c_char,
    /// The next result, or null.
    pub ai_next: *mut addrinfo,
}

/// `int getaddrinfo(const char *node, const char *service, const struct addrinfo *hints, struct addrinfo **res)`
#[no_mangle]
pub extern "C" fn getaddrinfo(
    _node: *const c_char,
    _service: *const c_char,
    _hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    if !res.is_null() {
        // SAFETY: the caller passed a non-null result pointer, per the contract.
        unsafe { *res = ptr::null_mut() };
    }
    eai::EAI_NONAME
}

/// `void freeaddrinfo(struct addrinfo *res)`
///
/// Nothing is ever handed out, so there is nothing to free. A shim that called
/// `free` here would push a pointer it never allocated onto `malloc`'s free list.
#[no_mangle]
pub extern "C" fn freeaddrinfo(_res: *mut addrinfo) {}

/// `const char *gai_strerror(int errcode)`
#[no_mangle]
pub extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    match errcode {
        eai::EAI_OK => b"Success\0".as_ptr().cast(),
        eai::EAI_NONAME => b"Name or service not known\0".as_ptr().cast(),
        eai::EAI_SERVICE => b"Service not available for socket type\0".as_ptr().cast(),
        _ => b"Unknown error\0".as_ptr().cast(),
    }
}

/// The first number of the socket block, so a C caller can see which syscalls
/// this shim expects the kernel to grow.
pub const NET_BLOCK_FIRST: u64 = sysnum::NET_FIRST;

/// `int getaddrinfo_a(...)` — the GNU all-synchronous spelling.
#[no_mangle]
pub extern "C" fn getaddrinfo_a(_req: *const c_void, _res: *mut *mut addrinfo) -> c_int {
    eai::EAI_NONAME
}
