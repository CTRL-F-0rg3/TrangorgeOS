//! The auxiliary vector and the environment block.
//!
//! At `_start` the kernel hands over a stack laid out the ELF way: `argc`, then
//! `argv[argc + 1]` pointers, then `envp[]`, then the auxiliary vector, then the
//! strings. `std` reads three things out of it and nothing more — the program
//! name (`AT_EXECFN`), the page size (`AT_PAGESZ`) and a random seed
//! (`AT_RANDOM`) — and `getenv` needs the whole environment block.
//!
//! `tgs-rt::_start` walks the stack once and records what it finds here, so that
//! `getauxval` and `getenv` are pure lookups afterwards.

use core::ptr;

use tgs_hal::errno::ENOSYS;

use crate::types::c_char;

/// `AT_NULL` — the auxv terminator.
pub const AT_NULL: usize = 0;
/// `AT_PAGESZ` — the target's page size.
pub const AT_PAGESZ: usize = 6;
/// `AT_PHDR` — the program headers, in memory.
pub const AT_PHDR: usize = 3;
/// `AT_PHENT` — the size of one program header.
pub const AT_PHENT: usize = 4;
/// `AT_PHNUM` — how many program headers there are.
pub const AT_PHNUM: usize = 5;
/// `AT_ENTRY` — the ELF entry point.
pub const AT_ENTRY: usize = 9;
/// `AT_RANDOM` — 16 bytes the kernel chose, used to seed hashers.
pub const AT_RANDOM: usize = 25;
/// `AT_EXECFN` — the path the program was loaded from.
pub const AT_EXECFN: usize = 31;
/// `AT_SECURE` — set by a set-uid execve.
pub const AT_SECURE: usize = 23;

/// The maximum number of auxiliary entries kept.
///
/// The kernel builds at most a couple of dozen, and a fixed cap means a
/// malformed auxv cannot make this walk off the stack.
const MAX_ENTRIES: usize = 32;

/// What `_start` recorded.
#[derive(Debug, Clone, Copy)]
pub struct Auxv {
    /// `(key, value)` pairs, terminated by `AT_NULL`.
    pub entries: [(usize, usize); MAX_ENTRIES],
    /// How many pairs are real.
    pub count: usize,
    /// `environ`, or null if the program has no environment.
    pub environ: *mut *const c_char,
    /// `argv`, or null if the program has no arguments.
    pub argv: *const *const c_char,
    /// `argc`.
    pub argc: isize,
}

impl Auxv {
    /// An empty record, for a program whose stack did not look like one.
    pub const fn empty() -> Self {
        Self {
            entries: [(AT_NULL, 0); MAX_ENTRIES],
            count: 0,
            environ: ptr::null_mut(),
            argv: ptr::null(),
            argc: 0,
        }
    }

    /// Record one pair, ignoring anything past the cap.
    pub fn push(&mut self, key: usize, value: usize) {
        if key == AT_NULL || self.count == MAX_ENTRIES {
            return;
        }
        self.entries[self.count] = (key, value);
        self.count += 1;
    }

    /// The value for `key`, or 0.
    pub fn get(&self, key: usize) -> usize {
        self.entries[..self.count]
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(0, |(_, v)| *v)
    }

    /// The value for `key`, or [`ENOSYS`] as a raw `long`.
    ///
    /// The odd return value is deliberate and matches glibc: a caller cannot
    /// distinguish "absent" from "present and zero" this way, which is why
    /// glibc's own headers discourage using it for anything real.
    pub fn get_or_missing(&self, key: usize) -> usize {
        if self.get(key) == 0 {
            ENOSYS.0 as usize
        } else {
            self.get(key)
        }
    }

    /// True once anything has been recorded.
    pub fn is_empty(&self) -> bool {
        self.count == 0 && self.argv.is_null()
    }
}

/// The recorded state, filled in by `tgs-rt::_start` before `main` runs.
static mut AUXV: Auxv = Auxv::empty();

/// Store the auxv `tgs-rt::_start` walked out of the stack.
///
/// # Safety
///
/// Must be called exactly once, before any thread of the program runs. `tgs-rt`
/// does it from `_start`, which is the only entry point.
pub unsafe fn set(auxv: Auxv) {
    // SAFETY: the caller promises this runs once, before anything else reads it.
    unsafe { AUXV = auxv };
}

/// The recorded auxv.
pub fn auxv() -> &'static Auxv {
    // SAFETY: read-only access to a value that is written once, before `main`.
    unsafe { &*(&raw const AUXV) }
}

/// `unsigned long getauxval(unsigned long type)`
#[no_mangle]
pub extern "C" fn getauxval(kind: usize) -> usize {
    auxv().get_or_missing(kind)
}

/// `char **environ`
///
/// Declared here rather than filled in by the kernel, because there is no
/// dynamic loader to do it: `tgs-rt::_start` parses the environment block off
/// the stack and points this at it.
#[no_mangle]
pub static mut environ: *mut *const c_char = ptr::null_mut();
