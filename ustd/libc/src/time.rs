//! Clocks, and the one thing a `std` program cannot start without.
//!
//! `std::time::Instant` calls `clock_gettime(CLOCK_MONOTONIC)` the first time it
//! is asked for anything, and `HashMap`'s `RandomState` calls `getrandom` the
//! first time a map is built. A program that gets either of those wrong does not
//! fail at that line; it fails somewhere unrelated and much later, which is why
//! both are implemented here rather than stubbed.

use core::ptr;

use tgs_hal::posix::{Posix, Timespec as HalTimespec};
use tgs_hal::sysnum;

use crate::shim;
use crate::types::{c_int, c_long, time_t, timespec, timeval};

/// `int clock_gettime(clockid_t clk_id, struct timespec *tp)`
#[no_mangle]
pub extern "C" fn clock_gettime(clk_id: c_int, tp: *mut timespec) -> c_int {
    if tp.is_null() {
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    // SAFETY: `tp` is non-null and, per the contract of `clock_gettime`, points
    // at writable storage for a `timespec`.
    let r = Posix::global().clock_gettime(clk_id).map(|ts| {
        // SAFETY: as above; the kernel wrote into this same address.
        unsafe { ptr::write(tp, timespec { tv_sec: ts.sec, tv_nsec: ts.nsec }) };
    });
    shim::code(r, |_| 0i32)
}

/// `int clock_gettime64(...)` — the LFS64 spelling some 32-bit-ish call sites use.
#[no_mangle]
pub extern "C" fn clock_gettime64(clk_id: c_int, tp: *mut timespec) -> c_int {
    clock_gettime(clk_id, tp)
}

/// `int nanosleep(const struct timespec *req, struct timespec *rem)`
#[no_mangle]
pub extern "C" fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int {
    if req.is_null() {
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    // SAFETY: `req` is non-null and readable per the contract of `nanosleep`.
    let want = unsafe { *req };
    let ts = HalTimespec {
        sec: want.tv_sec,
        nsec: want.tv_nsec,
    };
    let r = Posix::global().nanosleep(&ts).map(|left| {
        if let Some(slot) = rem_non_null(rem) {
            // SAFETY: `slot` is non-null and writable, per the caller.
            unsafe { ptr::write(slot, timespec { tv_sec: left.sec, tv_nsec: left.nsec }) };
        }
    });
    shim::code(r, |_| 0i32)
}

/// `int usleep(useconds_t usec)`
#[no_mangle]
pub extern "C" fn usleep(usec: tgs_hal::posix::Timespec) -> c_int {
    let sec = usec.sec;
    let nsec = usec.nsec * 1000;
    let req = timespec {
        tv_sec: sec,
        tv_nsec: nsec,
    };
    nanosleep(&req, core::ptr::null_mut())
}

/// `int gettimeofday(struct timeval *tv, void *tz)`
#[no_mangle]
pub extern "C" fn gettimeofday(tv: *mut timeval, _tz: *mut u8) -> c_int {
    if tv.is_null() {
        return shim::fail(tgs_hal::errno::EINVAL);
    }
    let r = Posix::global()
        .clock_gettime(sysnum::CLOCK_REALTIME)
        .map(|ts| {
            // SAFETY: `tv` is non-null and writable, per the contract of
            // `gettimeofday`.
            unsafe {
                ptr::write(
                    tv,
                    timeval {
                        tv_sec: ts.sec,
                        tv_usec: ts.nsec / 1000,
                    },
                )
            };
        });
    shim::code(r, |_| 0i32)
}

/// `time_t time(time_t *t)`
#[no_mangle]
pub extern "C" fn time(t: *mut time_t) -> time_t {
    let now = Posix::global()
        .clock_gettime(sysnum::CLOCK_REALTIME)
        .map_or(0, |ts| ts.sec);
    if let Some(slot) = rem_non_null(t.cast::<timespec>()) {
        // SAFETY: the caller passed a writable `time_t *`, per the contract of
        // `time`.
        unsafe { ptr::write(slot.cast::<time_t>(), now) };
    }
    now
}

/// `int getrandom(void *buf, size_t buflen, unsigned int flags)`
///
/// `HashMap` cannot be built without this, so it is not a stub. The kernel is
/// asked first; if it answers `ENOSYS` the bytes come from a SplitMix64 seeded
/// off the time stamp counter, which is good enough to keep a hasher from
/// degenerating and is documented in `../README.md` as not cryptographic.
#[no_mangle]
pub extern "C" fn getrandom(buf: *mut u8, buflen: usize, flags: u32) -> isize {
    if buf.is_null() {
        return shim::fail_ssize(tgs_hal::errno::EINVAL);
    }
    // SAFETY: `buf` is non-null and, per the contract of `getrandom`, writable
    // for `buflen` bytes.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf, buflen) };

    if flags & sysnum::GRND_RANDOM != 0 {
        // "Do not use hardware entropy": the fallback below is all there is.
        return fallback_entropy(dst);
    }
    match Posix::global().getrandom(dst) {
        Ok(n) => n as isize,
        Err(tgs_hal::errno::ENOSYS) => fallback_entropy(dst),
        Err(e) => shim::fail_ssize(e),
    }
}

/// Fill `dst` from a SplitMix64 stream seeded off the cycle counter.
fn fallback_entropy(dst: &mut [u8]) -> isize {
    let mut state = seed();
    for chunk in dst.chunks_mut(8) {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let bytes = z.to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    dst.len() as isize
}

/// A seed for the fallback stream.
fn seed() -> u64 {
    #[cfg(target_arch = "x86_64")]
    let tsc: u64 = {
        let lo: u64;
        let hi: u64;
        // SAFETY: `rdtsc` takes no inputs and clobbers nothing else, and is
        // available on every x86_64 part. The `lfence`s stop the value being
        // hoisted above earlier work, which is what would make two calls in a row
        // return the same number.
        unsafe {
            core::arch::asm!(
                "lfence",
                "rdtsc",
                "lfence",
                lateout("rax") lo,
                lateout("rdx") hi,
                options(nomem, nostack, preserves_flags),
            );
        }
        lo | (hi << 32)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let tsc = 0x5DEE_CE66_0000_0000;

    // Address-space-layout randomisation: the load address of a function differs
    // between runs and is a cheap second source of variation.
    let stack = &tsc as *const u64 as u64;
    tsc ^ stack.rotate_left(17) ^ (tsc >> 32)
}

/// `int rem_non_null`-style null check, so every shim shares one spelling.
fn rem_non_null<T>(p: *mut T) -> Option<*mut T> {
    if p.is_null() {
        None
    } else {
        Some(p)
    }
}

/// `clock_t clock(void)` — CPU time, which on a single-task kernel with no
/// accounting is the monotonic clock.
#[no_mangle]
pub extern "C" fn clock() -> c_long {
    Posix::global()
        .clock_gettime(sysnum::CLOCK_MONOTONIC)
        .map_or(0, |ts| ts.sec as c_long * 1_000_000 + ts.nsec as c_long / 1000)
}
