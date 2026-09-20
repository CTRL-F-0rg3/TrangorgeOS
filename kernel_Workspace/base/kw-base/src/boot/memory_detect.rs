use super::multiboot::*;
use kstd_base::{PhysAddr, Size};

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: PhysAddr,
    pub size: Size,
    pub is_usable: bool,
}

pub fn extract_memory_map(info: &BootInfo) -> (usize, [MemoryRegion; 32]) {
    let mut regions = [MemoryRegion { base: PhysAddr(0), size: Size(0), is_usable: false }; 32];
    let mut count = 0;

    let tag_ptr = info.find_tag(TAG_TYPE_MMAP);
    if tag_ptr.is_null() { return (0, regions); }

    let mmap = unsafe { &*(tag_ptr as *const MmapTag) };
    let mut entry_ptr = unsafe { (tag_ptr as *const u8).add(core::mem::size_of::<MmapTag>()) };
    let end_ptr = unsafe { (tag_ptr as *const u8).add(mmap.tag.size as usize) };

    while entry_ptr < end_ptr && count < 32 {
        let entry = unsafe { &*(entry_ptr as *const MmapEntry) };
        regions[count] = MemoryRegion {
            base: PhysAddr(entry.base_addr),
            size: Size(entry.length as usize),
            is_usable: entry.entry_type == 1,
        };
        count += 1;
        entry_ptr = unsafe { entry_ptr.add(mmap.entry_size as usize) };
    }

    (count, regions)
}