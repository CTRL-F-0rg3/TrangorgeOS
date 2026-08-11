
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

pub const KERNEL_HEAP_START: usize = 0xFFFF_9000_0000_0000;

pub const KERNEL_HEAP_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

pub const HEAP_MAX_ORDER: usize = 22; 
pub const HEAP_MIN_ORDER: usize = 4;  

pub const PHYS_MAX_ORDER: usize = 18; 