use x86_64::registers::control::Cr3;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::{PhysAddr, VirtAddr};

crate::test_module!({
    let offset = VirtAddr::new(*crate::memory::PHYSICAL_MEMORY_OFFSET.lock());

    let level4 = unsafe { active_level_4_table(offset) };
    let used = level4.iter().filter(|entry| !entry.is_unused()).count();
    if used == 0 {
        return Err("level 4 page table has no active entries");
    }

    let vga_virt = offset + 0xb8000u64;
    match translate_addr(vga_virt, offset) {
        Some(phys) if phys.as_u64() == 0xb8000 => {}
        Some(_) => return Err("translate_addr resolved VGA offset to the wrong physical address"),
        None => return Err("translate_addr failed to resolve a mapped VGA address"),
    }

    Ok("level 4 table populated, translate_addr verified against known VGA physical address")
});

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

pub fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    unsafe { translate_addr_inner(addr, physical_memory_offset) }
}

unsafe fn translate_addr_inner(
    addr: VirtAddr,
    physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for (level, &index) in table_indexes.iter().enumerate() {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];

        if entry.is_unused() {
            return None;
        }

        if entry.flags().contains(Flags::HUGE_PAGE) {
            let huge_frame_addr = entry.addr();
            let page_offset = match level {
                1 => addr.as_u64() & 0x3fff_ffff,
                2 => addr.as_u64() & 0x1f_ffff,
                _ => return None,
            };
            return Some(huge_frame_addr + page_offset);
        }

        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => unreachable!(),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
