#![no_std]

pub mod panic;
pub mod sync;
pub mod time;

pub use sync::SpinLock;
pub use time::{kernel_tick, rdtsc};