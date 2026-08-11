use crate::memory::{FRAME_ALLOCATOR, MAPPER};
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags};

crate::test_module!({
    let mut mapper_guard = MAPPER.lock();
    let mut allocator_guard = FRAME_ALLOCATOR.lock();

    let mapper = match mapper_guard.as_mut() {
        Some(mapper) => mapper,
        None => return Err("mapper not initialized"),
    };
    let allocator = match allocator_guard.as_mut() {
        Some(allocator) => allocator,
        None => return Err("frame allocator not initialized"),
    };

    let test_addr = VirtAddr::new(0x_4444_4444_0000);
    let page: Page = Page::containing_address(test_addr);

    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return Err("no free frame available for mapping test"),
    };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let map_result = unsafe { mapper.map_to(page, frame, flags, allocator) };
    let flush = match map_result {
        Ok(flush) => flush,
        Err(_) => return Err("map_to failed to establish the mapping"),
    };
    flush.flush();

    let ptr = test_addr.as_mut_ptr::<u64>();
    unsafe {
        core::ptr::write_volatile(ptr, 0xCAFE_BABE_u64);
    }
    let read_back = unsafe { core::ptr::read_volatile(ptr) };

    if let Ok((_, unmap_flush)) = mapper.unmap(page) {
        unmap_flush.flush();
    }

    if read_back != 0xCAFE_BABE {
        return Err("write/read through freshly mapped page did not round-trip");
    }

    Ok("mapped a fresh page, wrote through it, read back correctly, unmapped")
});
