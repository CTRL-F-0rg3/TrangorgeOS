use crate::allocator::config::PAGE_SIZE;
use crate::allocator::physical::{self, GlobalFrameAllocator};
use crate::memory::MAPPER;
use x86_64::VirtAddr;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, Size4KiB};

#[derive(Debug)]
pub enum MapError {
    MapperNotInitialized,
    NoFreeFrame,
    MapToFailed,
}

pub fn map_range(start: VirtAddr, size: usize) -> Result<(), MapError> {
    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().ok_or(MapError::MapperNotInitialized)?;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let page_count = size.div_ceil(PAGE_SIZE);
    let mut allocator = GlobalFrameAllocator;

    for i in 0..page_count {
        let page: Page<Size4KiB> = Page::containing_address(start + (i * PAGE_SIZE) as u64);
        let frame = physical::allocate_frame().ok_or(MapError::NoFreeFrame)?;
        let flush = unsafe {
            mapper
                .map_to(page, frame, flags, &mut allocator)
                .map_err(|_| MapError::MapToFailed)?
        };
        flush.flush();
    }

    Ok(())
}
