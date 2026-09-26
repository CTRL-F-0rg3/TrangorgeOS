//! The C scalar and pointer types, spelled out once.
//!
//! `std` links against these definitions by *name and layout*, not by Rust
//! type: the `libc` crate declares `c_int` as its own `i32` and the linker only
//! sees the `extern "C" fn` symbols. Defining them here rather than depending on
//! the `libc` crate is what keeps this crate buildable for a host test run, and
//! it doubles as a readable statement of the ABI.

// The kernel is 64-bit only (`x86_64-trangorge-uspace.json`), so LP64 applies.
#![allow(dead_code)]
// The names are the C ones on purpose: this module is the ABI, and a reader
// looking for `c_ulong` has to find it under that name.
#![allow(non_camel_case_types)]

/// `int`
pub type c_int = i32;
/// `unsigned int`
pub type c_uint = u32;
/// `long`
pub type c_long = i64;
/// `unsigned long`
pub type c_ulong = u64;
/// `long long`
pub type c_longlong = i64;
/// `unsigned long long`
pub type c_ulonglong = u64;
/// `short`
pub type c_short = i16;
/// `unsigned short`
pub type c_ushort = u16;
/// `char`
pub type c_char = i8;
/// `void`
pub type c_void = core::ffi::c_void;
/// `unsigned char`
pub type c_uchar = u8;
/// `float`
pub type c_float = f32;
/// `double`
pub type c_double = f64;
/// `size_t`
pub type size_t = usize;
/// `ssize_t`
pub type ssize_t = isize;
/// `ptrdiff_t`
pub type ptrdiff_t = isize;
/// `off_t`
pub type off_t = i64;
/// `mode_t`
pub type mode_t = u32;
/// `pid_t`
pub type pid_t = i32;
/// `uid_t`
pub type uid_t = u32;
/// `gid_t`
pub type gid_t = u32;
/// `clockid_t`
pub type clockid_t = i32;
/// `time_t`
pub type time_t = i64;
/// `suseconds_t`
pub type suseconds_t = i64;

/// `struct timespec`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct timespec {
    /// Whole seconds.
    pub tv_sec: time_t,
    /// Nanoseconds.
    pub tv_nsec: c_long,
}

/// `struct timeval`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct timeval {
    /// Whole seconds.
    pub tv_sec: time_t,
    /// Microseconds.
    pub tv_usec: suseconds_t,
}

/// `struct iovec`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct iovec {
    /// The buffer.
    pub iov_base: *mut u8,
    /// Its length.
    pub iov_len: size_t,
}

/// `struct pollfd`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct pollfd {
    /// The descriptor to watch.
    pub fd: c_int,
    /// What to wait for.
    pub events: c_short,
    /// What happened.
    pub revents: c_short,
}

/// `DIR`, the opaque handle `opendir` returns.
///
/// Only ever handled by pointer, so the contents do not matter — but the type
/// has to be distinct from `FILE` and `void *` or a caller could confuse them.
#[repr(C)]
pub struct DIR {
    _private: [u8; 0],
}

/// `FILE`, the opaque handle `std`'s stdio layer would use.
#[repr(C)]
pub struct FILE {
    _private: [u8; 0],
}

/// `struct utsname`, 6 × 65 bytes, exactly as the kernel writes it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct utsname {
    /// Operating-system name.
    pub sysname: [c_char; 65],
    /// Network-node name.
    pub nodename: [c_char; 65],
    /// Release.
    pub release: [c_char; 65],
    /// Version.
    pub version: [c_char; 65],
    /// Hardware type.
    pub machine: [c_char; 65],
    /// Domain name.
    pub domainname: [c_char; 65],
}
