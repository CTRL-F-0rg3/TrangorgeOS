use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTable;

pub static PHYSICAL_MEMORY_OFFSET: Mutex<u64> = Mutex::new(0);

crate::test_module!({
    let offset = VirtAddr::new(*PHYSICAL_MEMORY_OFFSET.lock());
    let table = unsafe { active_level_4_table(offset) };
    let used = table.iter().filter(|entry| !entry.is_unused()).count();
    if used == 0 {
        return Err("level 4 page table has no active entries");
    }
    Ok("level 4 page table read via physical memory offset")
});

pub fn init(physical_memory_offset: u64) {
    *PHYSICAL_MEMORY_OFFSET.lock() = physical_memory_offset;
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
