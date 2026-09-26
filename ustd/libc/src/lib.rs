//! `tgs-libc` — the C ABI that upstream Rust `std` links against.
//!
//! `std` is a compiled artifact, not a trait. To run it on TrangorgeOS the
//! *unmodified* `std` source has to be built for a TrangorgeOS target, and
//! everything it calls out to has to exist at link time: the `malloc` family its
//! allocator sits on, `read`/`write`/`open` for `std::fs` and `std::io`, the
//! clocks behind `std::time`, the `main` shim, `__errno_location`, and a handful
//! of `__cxa_*` hooks. This crate is exactly that list, and nothing else.
//!
//! # What is *not* here
//!
//! No `libc` shim. The target spec in
//! `../targets/x86_64-trangorge-uspace.json` declares `target_os = "linux"`, so
//! `std` resolves its `libc` dependency to the real `libc` crate from
//! crates.io, and that crate only *declares* the functions — the definitions
//! come from here. That is why this crate is a few hundred lines rather than a
//! few thousand: the type declarations come free with the `libc` crate, and only
//! the bodies have to be written.
//!
//! The cost of that choice is stated plainly in `../README.md`: `target_os` is
//! not `trangorge`, because upstream `std` has no `trangorge` platform
//! selection. Getting a real `std` fork would remove the need for this crate
//! almost entirely, and is the right long-term move.
//!
//! # The `sysroot` feature
//!
//! Every `#[no_mangle]` symbol here is behind the `sysroot` feature. That is not
//! ceremony: without it, `cargo test` on the host would link this crate's
//! `malloc` and `write` against the test binary and shadow the host libc's,
//! which fails at link time in a way that has nothing to do with the code.
//! `build.sh` passes `--features sysroot`; `cargo test` does not.
//!
//! # Layering
//!
//! ```text
//!   upstream std  ->  tgs-libc  ->  tgs-hal  ->  int 0x80  ->  kernel
//! ```

#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

pub mod types;

#[cfg(feature = "sysroot")]
pub mod auxv;
#[cfg(feature = "sysroot")]
pub mod cxa;
#[cfg(feature = "sysroot")]
pub mod env;
#[cfg(feature = "sysroot")]
pub mod errno;
#[cfg(feature = "sysroot")]
pub mod fs;
#[cfg(feature = "sysroot")]
pub mod mem;
#[cfg(feature = "sysroot")]
pub mod net;
#[cfg(feature = "sysroot")]
pub mod process;
#[cfg(feature = "sysroot")]
mod shim;
#[cfg(feature = "sysroot")]
pub mod term;
#[cfg(feature = "sysroot")]
pub mod thread;
#[cfg(feature = "sysroot")]
pub mod time;

#[cfg(feature = "sysroot")]
pub use mem::System;
