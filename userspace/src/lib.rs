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
//! directory's contents — not editing a `match` arm somewhere. The terminal is
//! the same idea as a separate program, which is why the two are separate
//! modules rather than one `execute()` with a mode flag.
//!
//! # What this crate does not own
//!
//! `allde` is a separate contribution and is left alone. So is the kernel's own
//! boot shell, which still runs before userspace exists. If something in here
//! seems to need a change in either, the layering is wrong and this crate
//! should grow instead.

pub mod apps;
pub mod bootstrap;
pub mod layout;
pub mod login;

#[cfg(test)]
mod tests;

pub use apps::terminal::Terminal;
pub use apps::shell::Shell;
pub use bootstrap::build_tree;
pub use layout::{normalize, required_dirs, ROOT_USER, RUNG_MAX, RUNG_MIN};
pub use login::{Account, Session, LoginError};
