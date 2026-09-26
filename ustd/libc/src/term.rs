//! Terminals, and the `tcgetattr` question `std` asks at start-up.
//!
//! `std`'s `main` shim calls `isatty(0)`/`isatty(1)` to decide whether to
//! line-buffer `stdin`, and installs a `SIGPIPE` disposition via `sigaction`. Both
//! have to answer honestly: an `isatty` that claims a terminal on a serial
//! console makes every prompt appear on one line, and a `sigaction` that fails
//! makes `std` fall back to its default, which is fine but worth knowing.
//!
//! The answer here is that descriptors 0/1/2 are *not* ttys. The kernel's
//! `TGS_UI_OPEN` path is how a program gets the framebuffer, and a program that
//! wants the console writes to descriptor 1. There is no line discipline yet.

use tgs_hal::errno;

use crate::shim;
use crate::types::c_int;

/// `int isatty(int fd)`
#[no_mangle]
pub extern "C" fn isatty(fd: c_int) -> c_int {
    // The kernel has no line discipline and no tty driver, so nothing is a
    // terminal. Reporting 0 is the truth; reporting 1 would make `std` line-
    // buffer output into the middle of a line.
    let _ = fd;
    0
}

/// `int ttyname(int fd)`
#[no_mangle]
pub extern "C" fn ttyname(fd: c_int) -> *mut crate::types::c_char {
    // `isatty` said no, so there is no name to give.
    let _ = fd;
    shim::fail_ssize(errno::ENOTTY);
    core::ptr::null_mut()
}

/// `int tcgetattr(int fd, struct termios *termios_p)`
#[no_mangle]
pub extern "C" fn tcgetattr(_fd: c_int, _t: *mut u8) -> c_int {
    shim::fail(errno::ENOTTY)
}

/// `int tcsetattr(int fd, int optional_actions, const struct termios *termios_p)`
#[no_mangle]
pub extern "C" fn tcsetattr(_fd: c_int, _actions: c_int, _t: *const u8) -> c_int {
    shim::fail(errno::ENOTTY)
}

/// `int tcflush(int fd, int queue_selector)`
#[no_mangle]
pub extern "C" fn tcflush(_fd: c_int, _selector: c_int) -> c_int {
    shim::fail(errno::ENOTTY)
}

/// `unsigned int sleep(unsigned int seconds)`
///
/// Implemented by spinning on the monotonic clock rather than by blocking. That
/// is a deliberate simplification: a blocking `nanosleep` would need the kernel
/// to switch worlds, and the current world scheduler runs one world to completion
/// per `HYPER_YIELD`, so a sleeping task would stop the machine. Spinning burns
/// a core; the fix belongs in the kernel's scheduler, not here.
#[no_mangle]
pub extern "C" fn sleep(seconds: c_int) -> c_int {
    if seconds <= 0 {
        return 0;
    }
    let p = tgs_hal::posix::Posix::global();
    // Re-read the clock each round rather than trusting one `nanosleep`, because
    // `nanosleep` returns early when it is interrupted.
    let started = p.clock_gettime(tgs_hal::sysnum::CLOCK_MONOTONIC);
    let start_ns = started.map_or(0, |t| t.to_nanos());
    let target = start_ns + (seconds as i128) * 1_000_000_000;

    loop {
        let now = p
            .clock_gettime(tgs_hal::sysnum::CLOCK_MONOTONIC)
            .map_or(start_ns, |t| t.to_nanos());
        if now >= target {
            return 0;
        }
        let _ = p.nanosleep(&tgs_hal::posix::Timespec {
            sec: 0,
            nsec: 1_000_000,
        });
    }
}

/// `unsigned int usleep(unsigned int usec)` — the busy-wait spelling.
#[no_mangle]
pub extern "C" fn usleep_ticks(usec: c_int) -> c_int {
    if usec <= 0 {
        return 0;
    }
    let p = tgs_hal::posix::Posix::global();
    let _ = p.nanosleep(&tgs_hal::posix::Timespec {
        sec: (usec / 1_000_000) as i64,
        nsec: ((usec % 1_000_000) as i64) * 1000,
    });
    0
}
