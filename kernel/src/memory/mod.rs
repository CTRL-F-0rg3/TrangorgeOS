pub mod frame_allocator;
pub mod mapping;
pub mod paging;

use bootloader::bootinfo::MemoryMap;
use frame_allocator::BootInfoFrameAllocator;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;

pub static PHYSICAL_MEMORY_OFFSET: Mutex<u64> = Mutex::new(0);
pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

pub fn init(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    *PHYSICAL_MEMORY_OFFSET.lock() = physical_memory_offset;

    let offset = VirtAddr::new(physical_memory_offset);
    let mapper = unsafe { paging::init(offset) };
    *MAPPER.lock() = Some(mapper);

    let allocator = unsafe { BootInfoFrameAllocator::init(memory_map) };
    *FRAME_ALLOCATOR.lock() = Some(allocator);
}
