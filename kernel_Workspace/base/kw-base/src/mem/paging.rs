use crate::core::{PhysAddr, VirtAddr, KResult, KernelError};
use gluecore::context;

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_WRITE_THROUGH: u64 = 1 << 3;
pub const PAGE_CACHE_DISABLE: u64 = 1 << 4;
pub const PAGE_HUGE: u64 = 1 << 7;
pub const PAGE_GLOBAL: u64 = 1 << 8;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

extern "C" {
    fn paging_map_page(virt: u64, phys: u64, flags: u64) -> bool;
    fn paging_unmap_page(virt: u64) -> bool;
}

pub fn map_page(virt: VirtAddr, phys: PhysAddr, flags: u64) -> KResult<()> {
    if unsafe { paging_map_page(virt.0, phys.0, flags) } {
        Ok(())
    } else {
        Err(KernelError::NoMemory)
    }
}

pub fn unmap_page(virt: VirtAddr) -> KResult<()> {
    if unsafe { paging_unmap_page(virt.0) } {
        Ok(())
    } else {
        Err(KernelError::InvalidArg)
    }
}

pub fn flush_tlb_page(virt: VirtAddr) {
    context::flush_page(virt);
}

pub fn flush_tlb_all() {
    context::flush_tlb_all();
}