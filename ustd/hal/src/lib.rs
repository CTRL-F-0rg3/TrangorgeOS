//! `tgs-hal` — the syscall ABI of TrangorgeOS ring 3.
//!
//! This is the bottom of the userspace standard-library stack. It knows three
//! things and nothing else:
//!
//! 1. **which** syscall number means what ([`sysnum`]),
//! 2. **how** to reach the kernel ([`gate::Syscall`], an `int 0x80` instruction
//!    on x86_64), and how to read a result back,
//! 3. **how** to give `std` the POSIX-shaped primitives it expects on top of
//!    that: an open-file table ([`fd`]), a `malloc`-able heap ([`heap`]) and
//!    the `getdents64` record layout ([`dirent`]).
//!
//! # Why a trait
//!
//! Every piece of logic in this crate — the fd table, the heap's free lists,
//! the dirent codec, the errno mapping — is written against
//! [`gate::Syscall`] instead of an `asm!` block. The production
//! implementation is [`gate::Gate`]; [`gate::Mock`] is a fake kernel that the
//! unit tests in `src/tests.rs` drive. That is what makes the allocator's free
//! lists and the fd table's lifetime rules actually verifiable on a host
//! instead of "trust me, it worked in QEMU".
//!
//! # Error convention
//!
//! The kernel returns `-errno` in `rax` for a failed call and a non-negative
//! value for success — the Linux convention, which is what upstream `std`
//! expects. The legacy TrangorgeOS code paths return `u64::MAX` for "call I do
//! not know"; [`errno::from_raw`] normalises that to [`errno::ENOSYS`] so an
//! unimplemented kernel syscall surfaces as `ErrorKind::Unsupported` instead
//! of a bogus permission error.
//!
//! # Layering rule
//!
//! `tgs-hal` never calls into `std`, never allocates through `alloc` from the
//! heap it provides ([`heap`] is allocation-free by construction) and never
//! formats anything. It is safe to call from an allocator, which is exactly
//! what `tgs-libc` does.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod dirent;
pub mod errno;
pub mod fd;
pub mod gate;
pub mod heap;
pub mod posix;
pub mod sysnum;

#[cfg(test)]
mod tests;

pub use errno::{Errno, Result};
pub use gate::{Gate, Mock, Syscall};
pub use posix::Posix;
