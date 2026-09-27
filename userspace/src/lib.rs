//! TrangorgeOS **userspace** — the nested system.
//!
//! Userspace owns three things, and this crate is all of them:
//!
//! * [`layout`] — the on-disk hierarchy (`/kernel/userspace/user/root/...`),
//! * [`bootstrap`] — creating that hierarchy on first boot,
//! * [`apps`] — the replaceable applications that live in it: the shell and a
//!   standalone terminal.
//!
//! # The shell is a program, not the system
//!
//! The shell lives at `user/root/shell/` on the volume, and what runs is
//! whatever is in that directory. Replacing the shell means replacing that
//! directory — not editing a `match` arm somewhere. The terminal is the same
//! idea as a separate program, which is why the two are separate modules rather
//! than one `execute()` with a mode flag.
//!
//! # no_std
//!
//! `cfg_attr(not(test), no_std)`, so the kernel can link this crate and run the
//! shell directly. That is the whole point: a shell living only on the host is
//! a shell nobody boots into. `std` is used only where `alloc` has no
//! equivalent — `BTreeMap`/`BTreeSet` live in `alloc::collections`, and
//! `mem::take` is spelled out where it is not available. The result is one
//! implementation of the shell that both the boot self-test and a real login
//! exercise.
//!
//! # What this crate does not own
//!
//! `allde` is a separate contribution and is left alone. So is the kernel's
//! pre-userspace boot shell, which runs before userspace exists. If something in
//! here seems to need a change in either, the layering is wrong and this crate
//! should grow instead.

#![cfg_attr(not(test), no_std)]

// `alloc::format` and `alloc::vec` are not in the `no_std` prelude, and every
// module in this crate builds strings and vectors. `#[macro_use]` on the extern
// crate brings both macros into scope for this crate *and its submodules*,
// which a plain `use` would not: a `use` in the crate root is not inherited.
#[macro_use]
extern crate alloc;

pub mod apps;
pub mod bootstrap;
pub mod commands;
pub mod layout;
pub mod login;

#[cfg(test)]
mod tests;

pub use apps::shell::{Fs, Sys};
pub use apps::terminal::Terminal;
pub use apps::shell::Shell;
pub use bootstrap::{build_tree, MemoryVolume, TreeReport, Volume};
pub use commands::registry;
pub use layout::{normalize, required_dirs, ROOT_USER, RUNG_MAX, RUNG_MIN};
pub use login::{Account, Accounts, LoginError, Session};
