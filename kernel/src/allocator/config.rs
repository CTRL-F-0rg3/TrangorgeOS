pub const PAGE_SIZE: usize = 4096;
pub const HEAP_START: u64 = 0xFFFF_9000_0000_0000;
pub const HEAP_END: u64 = 0xFFFF_9000_FFFF_FFFF;
pub const HEAP_INITIAL_SIZE: usize = 4 * 1024 * 1024;
pub const MAX_SUPPORTED_FRAMES: usize = 1 << 20;
