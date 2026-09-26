//! Reaching the kernel.
//!
//! The whole crate funnels through [`Syscall::call`]. Two implementations
//! exist:
//!
//! * [`Gate`] — the real one. On x86_64 that is a single `int 0x80`; on other
//!   architectures it reports `ENOSYS`, which keeps the crate buildable (and
//!   unit-testable) on the host.
//! * [`Mock`] — a scriptable fake kernel, available with `cfg(test)` or the
//!   `mock` feature. It hands out *real* host addresses for `mmap`, so an
//!   allocator's free lists can be exercised for real instead of being mocked
//!   out one layer up.
//!
//! # Calling convention
//!
//! | Register | Role |
//! |----------|------|
//! | `rax` | syscall number in, return value out |
//! | `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` | arguments 0..5 (Linux order: `rcx` is skipped because `int` clobbers it) |
//!
//! Six arguments is the Linux maximum and therefore the maximum `std` ever
//! needs.

use crate::errno::{Errno, Result};
use crate::sysnum;

// The mock needs a growable log and interior mutability. `alloc` is linked in
// unconditionally (the userspace program has a heap), but under `cfg(test)` the
// crate is a normal `std` crate, so the import has to come from the right one.
#[cfg(not(test))]
use alloc::{vec, vec::Vec};
#[cfg(test)]
use std::{vec, vec::Vec};

use core::cell::{Cell, RefCell};

#[cfg(any(test, feature = "mock"))]
use crate::heap::PAGE;

/// One call into the kernel.
///
/// # Safety
///
/// Implementations must pass the argument words through unchanged and must not
/// dereference the pointers inside them — that is the job of the layer above.
pub unsafe trait Syscall {
    /// Perform one syscall.
    ///
    /// # Safety
    ///
    /// `args` may contain pointers into the caller's address space. An
    /// implementation must not dereference them and must not retain `args`
    /// after returning. The result is a raw `rax` value: non-negative for
    /// success, `-errno` for failure.
    unsafe fn call(&self, num: u64, args: [u64; 6]) -> i64;
}

/// The real gate: `int 0x80`, the vector the kernel installs at DPL 3 in
/// `kernel/src/trampoline_rings/arch/x86_64/trampoline_rings.rs`
/// (`set_gate(0x80, hyp, 3)`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Gate;

/// The one and only gate, as a `const` so it can be referenced from other
/// `const` items — the system heap needs exactly that.
pub const GATE: Gate = Gate;

impl Gate {
    /// The process-wide gate. The gate is stateless, so there is exactly one.
    pub const GLOBAL: Gate = Gate;

    /// Give up the rest of the time slice.
    #[inline]
    pub fn yield_now(&self) {
        // SAFETY: a zero-argument syscall carrying no pointers.
        unsafe { self.call(sysnum::YIELD, [0; 6]) };
    }

    /// Print a NUL-terminated string on the kernel console.
    ///
    /// # Safety
    ///
    /// `msg` must point at a NUL-terminated string in readable memory.
    #[inline]
    pub unsafe fn log(&self, msg: *const u8) {
        // SAFETY: forwarded from this function's contract.
        unsafe { self.call(sysnum::LOG, [msg as u64, 0, 0, 0, 0, 0]) };
    }
}

#[cfg(target_arch = "x86_64")]
unsafe impl Syscall for Gate {
    unsafe fn call(&self, num: u64, args: [u64; 6]) -> i64 {
        let ret: i64;
        // SAFETY: `int 0x80` is unconditionally available on x86_64, the
        // kernel installs the gate at DPL 3 during boot, and the caller
        // guarantees `args` holds only integers and pointers it owns.
        unsafe {
            core::arch::asm!(
                "int 0x80",
                inlateout("rax") num => ret,
                in("rdi") args[0],
                in("rsi") args[1],
                in("rdx") args[2],
                in("r10") args[3],
                in("r8") args[4],
                in("r9") args[5],
                options(nostack),
            );
        }
        ret
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe impl Syscall for Gate {
    unsafe fn call(&self, _num: u64, _args: [u64; 6]) -> i64 {
        // Building for an architecture with no TrangorgeOS syscall instruction
        // is a build mistake rather than a runtime condition. Report
        // "not implemented" instead of aborting: `std`'s startup probes several
        // calls and tolerates `ENOSYS`.
        Errno::ENOSYS.to_raw()
    }
}

/// The process-wide gate, for code that cannot be handed one.
pub const fn global() -> &'static Gate {
    &Gate::GLOBAL
}

/// Decode a raw `rax` into a [`Result`].
#[inline]
pub fn check(raw: i64) -> Result<usize> {
    Errno::from_raw(raw)
}

/// Erase a pointer's address for a syscall argument.
///
/// # Safety
///
/// The callee may dereference the result, so the pointee must stay valid for
/// the duration of the call.
#[inline]
pub fn addr_of<T>(p: *const T) -> u64 {
    p as u64
}

/// Erase a mutable pointer for a syscall argument.
///
/// # Safety
///
/// See [`addr_of`]; in addition the callee may write through the pointee.
#[inline]
pub fn addr_of_mut<T>(p: *mut T) -> u64 {
    p as u64
}

/// A scripted fake kernel, for the tests in this crate and in `tgs-libc`.
///
/// It is deliberately *not* a pure function of its input: `MMAP` returns a
/// pointer into a real host allocation, so a caller can genuinely write
/// through the address it is handed and a test can assert the bytes arrived.
/// That is the only honest way to test an allocator without one.
///
/// Usable as a `&dyn Syscall` directly — the state lives behind a `RefCell`,
/// so the same value can be scripted, handed to the code under test, and
/// inspected afterwards.
#[cfg(any(test, feature = "mock"))]
pub struct Mock {
    state: RefCell<State>,
    /// The simulated address space, leaked so the addresses handed out stay
    /// valid for as long as anything the test allocated through them.
    arena: &'static mut [u8],
    /// The next free byte in [`Mock::arena`].
    next: Cell<usize>,
    /// How many bytes one simulated `mmap` hands out.
    chunk: usize,
}

#[cfg(any(test, feature = "mock"))]
/// The scriptable state of a [`Mock`]: what it has been asked to return, and
/// everything it has been asked.
///
/// Public because it is the scripting surface: a test reaches it through
/// [`Mock::with`], never by naming a field directly.
#[cfg(any(test, feature = "mock"))]
pub struct State {
    /// Every call made, in order, as `(number, args)`.
    log: Vec<(u64, [u64; 6])>,
    /// Queued results per syscall number.
    returns: Vec<(u64, Vec<i64>)>,
    /// When a call to a number arrives, copy these bytes into the pointer the
    /// caller passed in the given argument. This is how the tests exercise the
    /// syscalls that report data *through* a pointer — `clock_gettime`,
    /// `fstat`, `getdents64` — without a kernel to do it.
    fill: Vec<(u64, usize, Vec<u8>)>,
    /// When set, every call fails with this error regardless of its number.
    fail: Option<Errno>,
}

#[cfg(any(test, feature = "mock"))]
impl core::fmt::Debug for Mock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mock")
            .field("calls", &self.state.borrow().log.len())
            .field("chunk", &self.chunk)
            .finish()
    }
}

#[cfg(any(test, feature = "mock"))]
impl Mock {
    /// A mock whose simulated address space is `chunk` bytes, rounded up to a
    /// page so the addresses it hands out are page-aligned like a real
    /// kernel's.
    pub fn new(chunk: usize) -> Self {
        let chunk = chunk.max(4096).next_multiple_of(4096);
        // One extra page so that rounding the base *up* to a page boundary can
        // never push the end of the usable region past the end of the
        // allocation. Without the slack a `Vec` that happened to start 16 bytes
        // into a page would leave 4080 bytes of "arena" that were never ours,
        // and the first write through a returned address would be a wild one.
        let raw: &'static mut [u8] = vec![0u8; chunk + PAGE].leak();
        let base = raw.as_ptr() as usize;
        let aligned = (base + PAGE - 1) & !(PAGE - 1);
        let off = aligned - base;

        Self {
            state: RefCell::new(State {
                log: Vec::new(),
                returns: Vec::new(),
                fill: Vec::new(),
                fail: None,
            }),
            arena: &mut raw[off..off + chunk],
            next: Cell::new(aligned),
            chunk,
        }
    }

    /// A mock of the default size, 1 MiB of simulated address space.
    pub fn with_default_arena() -> Self {
        Self::new(1024 * 1024)
    }

    /// Run `f` against the mock's mutable state.
    pub fn with<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        f(&mut self.state.borrow_mut())
    }

    /// The simulated address space, read-only.
    pub fn arena(&self) -> &[u8] {
        self.arena
    }

    /// The simulated address space, mutable.
    pub fn arena_mut(&mut self) -> &mut [u8] {
        self.arena
    }

    /// How many bytes the simulated address space holds.
    pub fn arena_len(&self) -> usize {
        self.arena.len()
    }

    /// The address the next simulated `mmap` will hand out.
    pub fn next_address(&self) -> usize {
        self.next.get()
    }

    /// Every call made, in order, as `(number, args)`.
    pub fn log(&self) -> Vec<(u64, [u64; 6])> {
        self.state.borrow().log.clone()
    }

    /// The arguments of every call made to `num` so far, in order.
    pub fn args_of(&self, num: u64) -> Vec<[u64; 6]> {
        self.state
            .borrow()
            .log
            .iter()
            .filter(|(n, _)| *n == num)
            .map(|(_, a)| *a)
            .collect()
    }

    /// The arguments of the most recent call to `num`.
    pub fn last_args(&self, num: u64) -> Option<[u64; 6]> {
        self.state
            .borrow()
            .log
            .iter()
            .rev()
            .find(|(n, _)| *n == num)
            .map(|(_, a)| *a)
    }

    /// How many times `num` was called.
    pub fn count_of(&self, num: u64) -> usize {
        self.state
            .borrow()
            .log
            .iter()
            .filter(|(n, _)| *n == num)
            .count()
    }

    /// The last error any call reported, if the mock was told to fail.
    pub fn failure(&self) -> Option<Errno> {
        self.state.borrow().fail
    }
}

#[cfg(any(test, feature = "mock"))]
impl State {
    /// Queue `ret` as the result of the next call to `num`.
    pub fn push_ret(&mut self, num: u64, ret: i64) {
        if let Some((_, q)) = self.returns.iter_mut().find(|(n, _)| *n == num) {
            q.push(ret);
        } else {
            self.returns.push((num, vec![ret]));
        }
    }

    /// Make every subsequent call fail with `err`.
    pub fn fail_all(&mut self, err: Errno) {
        self.fail = Some(err);
    }

    /// Stop failing.
    pub fn succeed_again(&mut self) {
        self.fail = None;
    }

    /// Forget the call log, keeping the scripted results.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    /// When the next call to `num` arrives, copy `bytes` into the pointer the
    /// caller passed in argument `arg`.
    pub fn push_fill(&mut self, num: u64, arg: usize, bytes: &[u8]) {
        self.fill.push((num, arg, bytes.to_vec()));
    }
}

#[cfg(any(test, feature = "mock"))]
unsafe impl Syscall for Mock {
    unsafe fn call(&self, num: u64, args: [u64; 6]) -> i64 {
        let mut s = self.state.borrow_mut();
        s.log.push((num, args));

        if let Some(e) = s.fail {
            return e.to_raw();
        }

        // Report data through the caller's pointer, the way a real kernel does
        // for `clock_gettime`, `fstat` and `getdents64`. This happens before the
        // scripted result is consulted, because a real kernel writes the data
        // *and* returns a length: the two are independent.
        if let Some(i) = s.fill.iter().position(|(n, _, _)| *n == num) {
            let (_, arg, bytes) = s.fill.remove(i);
            let dst = args[arg] as *mut u8;
            // SAFETY: the mock is only ever pointed at a live buffer the code
            // under test owns — that is what the test scripted it with. The
            // copy is bounded by the scripted length, and the buffer was
            // allocated by the same test, so it is at least that long.
            unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len()) };
        }

        // A scripted result wins over the built-in behaviour, so a test can make
        // `mmap` return a specific address — or address 0, to check that the heap
        // notices — without the mock needing a special case.
        let queued = s
            .returns
            .iter_mut()
            .find(|(n, _)| *n == num)
            .and_then(|(_, q)| if q.is_empty() { None } else { Some(q.remove(0)) });

        if let Some(v) = queued {
            return v;
        }

        if num == sysnum::MMAP {
            let addr = self.next.get();
            self.next.set(addr + self.chunk);
            return addr as i64;
        }

        // An unscripted call succeeds with 0, which is what the majority of a
        // `std` startup sequence expects to see.
        0
    }
}
