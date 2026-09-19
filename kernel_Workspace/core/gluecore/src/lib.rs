// kernel_Workspace/core/gluecore/src/lib.rs
#![no_std]

pub use kstd_base;
pub use kstd_alloc;
pub use kstd_core;
pub use kstd_data;
pub use kstd_io;

pub mod ffi;
pub mod bridge;

pub use bridge::mm;
pub use bridge::caps;