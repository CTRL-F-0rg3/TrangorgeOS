// kw-libs/src/mem/virtual/aspace.rs

use kstd_base::{VirtAddr, Size};
use kw_base::core::{KResult, KernelError};
use super::vma::{Vma, VmaType};

pub struct AddressSpace {
    pub cr3_phys: u64,
    vma_head: *mut Vma,
    vma_count: usize,
}

unsafe impl Send for AddressSpace {}
unsafe impl Sync for AddressSpace {}

impl AddressSpace {
    pub fn new(cr3: u64) -> Self {
        Self {
            cr3_phys: cr3,
            vma_head: core::ptr::null_mut(),
            vma_count: 0,
        }
    }

    pub fn map_region(&mut self, start: VirtAddr, size: usize, flags: u32, vma_type: VmaType) -> KResult<()> {
        let end = VirtAddr(start.0 + size as u64);
        
        // Check for overlaps
        let mut current = self.vma_head;
        while !current.is_null() {
            let vma = unsafe { &*current };
            if vma.overlaps(start, end) {
                return Err(KernelError::AlreadyExists);
            }
            current = vma.next;
        }

        // Allocate new VMA
        let layout = core::alloc::Layout::new::<Vma>();
        let ptr = unsafe { kstd_alloc::KernelHeap.alloc(layout) }.map_err(|_| KernelError::NoMemory)? as *mut Vma;
        
        unsafe {
            ptr.write(Vma::new(start, end, flags, vma_type));
            (*ptr).next = self.vma_head;
            self.vma_head = ptr;
            self.vma_count += 1;
        }

        // TODO: Actually map pages in the page table via kw_base::mem::paging
        Ok(())
    }

    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        let mut current = self.vma_head;
        while !current.is_null() {
            let vma = unsafe { &*current };
            if vma.contains(addr) {
                return Some(vma);
            }
            current = vma.next;
        }
        None
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let mut current = self.vma_head;
        while !current.is_null() {
            let next = unsafe { (*current).next };
            let layout = core::alloc::Layout::new::<Vma>();
            unsafe { kstd_alloc::KernelHeap.dealloc(current as *mut u8, layout); }
            current = next;
        }
    }
}