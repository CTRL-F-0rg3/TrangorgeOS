use kstd_base::{PhysAddr, VirtAddr, Status};

extern "C" {
    fn paging_read_cr3() -> u64;
    fn paging_write_cr3(pml4_phys: u64);
    fn paging_flush_tlb_all();
    fn paging_flush_page(addr: u64);
    fn context_switch_save_restore(old_ctx: *mut u8, new_ctx: *mut u8);
}

pub fn read_cr3() -> PhysAddr {
    PhysAddr(unsafe { paging_read_cr3() })
}

pub fn write_cr3(pml4: PhysAddr) {
    unsafe { paging_write_cr3(pml4.0); }
}

pub fn flush_tlb_all() {
    unsafe { paging_flush_tlb_all(); }
}

pub fn flush_page(addr: VirtAddr) {
    unsafe { paging_flush_page(addr.0); }
}

pub unsafe fn switch_context(old_ctx: *mut u8, new_ctx: *mut u8) {
    context_switch_save_restore(old_ctx, new_ctx);
}