pub mod config;
pub mod heap;
pub mod physical;
pub mod stats;
pub mod traits;
pub mod virt;

use bootloader::bootinfo::MemoryMap;

#[global_allocator]
pub static ALLOCATOR: heap::LockedHeap = heap::LockedHeap::new();

crate::test_module!({
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    let boxed = Box::new(0xDEAD_BEEFu32);
    if *boxed != 0xDEAD_BEEF {
        return Err("Box<u32> did not round-trip through the heap correctly");
    }
    drop(boxed);

    let mut values: Vec<u32> = Vec::new();
    for i in 0..256 {
        values.push(i);
    }
    if values.len() != 256 {
        return Err("Vec did not grow to the expected length");
    }
    for (i, value) in values.iter().enumerate() {
        if *value != i as u32 {
            return Err("Vec contents were corrupted after growth");
        }
    }
    drop(values);

    Ok("real heap-backed Box and Vec allocation verified end-to-end")
});

pub fn init(memory_map: &'static MemoryMap) {
    physical::init(memory_map);

    let heap_start = virt::adress_space::HEAP_REGION
        .reserve(config::HEAP_INITIAL_SIZE, config::PAGE_SIZE)
        .expect("failed to reserve virtual address range for the kernel heap");

    virt::mapper::map_range(heap_start, config::HEAP_INITIAL_SIZE)
        .expect("failed to map physical frames for the kernel heap");

    unsafe {
        ALLOCATOR.init(heap_start.as_u64() as usize, config::HEAP_INITIAL_SIZE);
    }
}
