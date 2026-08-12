use super::bitmap::{BitmapFrameAllocator, PAGE_SIZE};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub fn total_frames(memory_map: &MemoryMap) -> usize {
    let mut max_frame = 0usize;
    for region in memory_map.iter() {
        let end_frame = (region.range.end_addr() / PAGE_SIZE as u64) as usize;
        if end_frame > max_frame {
            max_frame = end_frame;
        }
    }
    max_frame
}

pub fn populate(allocator: &mut BitmapFrameAllocator<'_>, memory_map: &MemoryMap) {
    allocator.mark_all_used();

    for region in memory_map.iter() {
        if region.region_type != MemoryRegionType::Usable {
            continue;
        }
        let start_frame = (region.range.start_addr() / PAGE_SIZE as u64) as usize;
        let end_frame = (region.range.end_addr() / PAGE_SIZE as u64) as usize;
        if end_frame > start_frame {
            allocator.mark_range_free(start_frame, end_frame - start_frame);
        }
    }
}
