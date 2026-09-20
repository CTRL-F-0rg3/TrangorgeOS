#![no_std]

// Re-export core foundations so sub-modules don't need to depend on kw-base directly if not needed.
extern crate kw_base;

pub mod utils;
pub mod kernel;

// Placeholder modules for future implementation
pub mod mem;
pub mod task;
pub mod fs;
pub mod net;
pub mod drivers;
pub mod ipc;
pub mod arch;