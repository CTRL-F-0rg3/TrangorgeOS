use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

crate::test_module!({
    let mut guard = crate::memory::FRAME_ALLOCATOR.lock();
    let allocator = match guard.as_mut() {
        Some(allocator) => allocator,
        None => return Err("frame allocator not initialized"),
    };

    let mut frames: [Option<PhysFrame>; 8] = [None; 8];
    for slot in frames.iter_mut() {
        *slot = allocator.allocate_frame();
    }

    for i in 0..frames.len() {
        let Some(frame) = frames[i] else {
            return Err("frame allocator ran out of usable frames during test");
        };
        if frame.start_address().as_u64() % 4096 != 0 {
            return Err("allocated frame is not 4KiB aligned");
        }
        for j in (i + 1)..frames.len() {
            if frames[j] == Some(frame) {
                return Err("frame allocator returned the same frame twice");
            }
        }
    }

    Ok("frame allocator returned 8 distinct, aligned physical frames")
});

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
