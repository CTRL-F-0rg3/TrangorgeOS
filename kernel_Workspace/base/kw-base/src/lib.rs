#![no_std]

extern crate kstd_base;
extern crate kstd_alloc;
extern crate kstd_core;
extern crate kstd_data;
extern crate kstd_io;
extern crate kstd_os_workspace;
extern crate gluecore;

#[macro_use]
pub mod core;
pub mod utils;

pub use self::core::*;
pub use utils::*;