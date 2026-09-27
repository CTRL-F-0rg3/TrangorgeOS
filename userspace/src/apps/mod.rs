//! The applications userspace ships.
//!
//! Each one is a separate program living in its own directory under the user's
//! home (`shell/`, `terminal/`). That is what makes them replaceable: what runs
//! is whatever is in the directory, so swapping the shell means swapping the
//! directory, not editing a dispatch table.
//!
//! # Why the shell and the terminal are different things
//!
//! A terminal *runs* commands and shows their output. A shell *interprets* what
//! the user types and hands lines to something. They overlap heavily in a simple
//! system, which is exactly why separating them pays off: a user who wants a
//! different command interpreter replaces the shell, and a user who wants a
//! different window for running commands replaces the terminal. Neither
//! replacement disturbs the other.
//!
//! # What is not here
//!
//! No windowing. `allde` owns that, and it is someone else's crate; see
//! [`crate::layout`]'s ownership note. These two are text-mode programs that
//! own a text buffer and a cursor position, which is enough to run headless and
//! be driven by a test.

pub mod shell;
pub mod terminal;
