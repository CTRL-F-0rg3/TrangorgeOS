pub mod paging;

use spin::Mutex;

pub static PHYSICAL_MEMORY_OFFSET: Mutex<u64> = Mutex::new(0);

pub fn init(physical_memory_offset: u64) {
    *PHYSICAL_MEMORY_OFFSET.lock() = physical_memory_offset;
}
