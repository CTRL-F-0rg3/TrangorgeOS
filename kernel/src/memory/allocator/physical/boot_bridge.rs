
use super::bitmap::BitmapFrameAllocator;
use crate::allocator::config::PAGE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Usable,
    Reserved,
    AcpiReclaimable,
    BadMemory,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub end_addr: u64, // wyłącznie (exclusive)
    pub kind: RegionKind,
}

pub fn frames_required(regions: impl Iterator<Item = MemoryRegion>) -> usize {
    let max_addr = regions.map(|r| r.end_addr).max().unwrap_or(0);
    ((max_addr as usize) + PAGE_SIZE - 1) / PAGE_SIZE
}

pub fn build_allocator<'a>(
    bitmap_storage: &'a mut [u64],
    regions: impl Iterator<Item = MemoryRegion>,
) -> BitmapFrameAllocator<'a> {
    let total_frames = bitmap_storage.len() * 64;
    let mut allocator = BitmapFrameAllocator::new(bitmap_storage, total_frames);
    allocator.mark_all_used();

    for region in regions {
        if region.kind != RegionKind::Usable {
            continue;
        }
        let start_frame = (region.start_addr as usize) / PAGE_SIZE;
        let end_frame = ((region.end_addr as usize) + PAGE_SIZE - 1) / PAGE_SIZE;
        allocator.mark_range_free(start_frame, end_frame - start_frame);
    }

    allocator
}