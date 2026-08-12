
pub mod errno;
pub mod ktable;
pub mod syscall;
pub mod types;

pub const ABI_VERSION_MAJOR: u32 = 0;

pub const ABI_VERSION_MINOR: u32 = 1;

pub const ABI_MAGIC: u64 = 0x5452_474F_5247_4500;