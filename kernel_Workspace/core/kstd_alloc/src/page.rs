use kstd_base::{Status, PhysAddr, VirtAddr, PAGE_SIZE};
use crate::traits::Layout;

extern "C" {
    fn kalloc_pages(order: usize) -> *mut u8;
    fn kfree_pages(ptr: *mut u8, pages: usize);
}

pub fn alloc_pages(order: usize) -> Result<VirtAddr, Status> {
    let ptr = unsafe { kalloc_pages(order) };
    if ptr.is_null() {
        Err(Status::OutOfMemory)
    } else {
        Ok(VirtAddr(ptr as u64))
    }
}

pub fn free_pages(addr: VirtAddr, order: usize) {
    let pages = 1 << order;
    unsafe { kfree_pages(addr.0 as *mut u8, pages) };
}

pub fn alloc_single_page() -> Result<VirtAddr, Status> {
    alloc_pages(0)
}

pub fn free_single_page(addr: VirtAddr) {
    free_pages(addr, 0)
}

pub fn pages_for_size(size: usize) -> usize {
    (size + PAGE_SIZE - 1) / PAGE_SIZE
}