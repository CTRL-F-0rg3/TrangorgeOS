// kw-libs/src/mem/allocators/slab.rs

use kw_base::core::{KResult, KernelError};
use kstd_base::Size;

const SLAB_OBJ_MIN_SIZE: usize = 8;

struct SlabPage {
    next: *mut SlabPage,
    free_head: *mut u8,
    in_use: u32,
}

pub struct SlabCache {
    obj_size: usize,
    align: usize,
    objs_per_page: usize,
    partial_pages: *mut SlabPage,
    full_pages: *mut SlabPage,
}

unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}

impl SlabCache {
    pub fn new(obj_size: usize, align: usize) -> Self {
        let actual_size = if obj_size < core::mem::size_of::<*mut u8>() {
            core::mem::size_of::<*mut u8>()
        } else {
            obj_size
        };
        
        let aligned_size = (actual_size + align - 1) & !(align - 1);
        let objs_per_page = (4096 - core::mem::size_of::<SlabPage>()) / aligned_size;

        Self {
            obj_size: aligned_size,
            align,
            objs_per_page,
            partial_pages: core::ptr::null_mut(),
            full_pages: core::ptr::null_mut(),
        }
    }

    fn alloc_page(&self) -> KResult<*mut SlabPage> {
        let layout = core::alloc::Layout::from_size_align(4096, 4096).unwrap();
        let ptr = unsafe { kstd_alloc::KernelHeap.alloc(layout) }.map_err(|_| KernelError::NoMemory)?;
        
        let page = ptr as *mut SlabPage;
        unsafe {
            (*page).next = core::ptr::null_mut();
            (*page).in_use = 0;
            
            // Build free list inside the page
            let data_start = ptr.add(core::mem::size_of::<SlabPage>());
            let mut current = data_start;
            for i in 0..self.objs_per_page - 1 {
                let next = current.add(self.obj_size);
                *(current as *mut *mut u8) = next;
                current = next;
            }
            *(current as *mut *mut u8) = core::ptr::null_mut();
            (*page).free_head = data_start;
        }
        
        Ok(page)
    }

    pub fn alloc(&mut self) -> KResult<*mut u8> {
        if self.partial_pages.is_null() {
            let page = self.alloc_page()?;
            unsafe {
                (*page).next = self.partial_pages;
                self.partial_pages = page;
            }
        }

        let page = self.partial_pages;
        let obj = unsafe { (*page).free_head };
        
        if obj.is_null() {
            return Err(KernelError::NoMemory);
        }

        unsafe {
            (*page).free_head = *(obj as *mut *mut u8);
            (*page).in_use += 1;

            if (*page).in_use as usize == self.objs_per_page {
                self.partial_pages = (*page).next;
                (*page).next = self.full_pages;
                self.full_pages = page;
            }
        }

        Ok(obj)
    }

    pub fn free(&mut self, obj: *mut u8) {
        // Find the page this object belongs to
        let page_addr = (obj as usize) & !(4096 - 1);
        let page = page_addr as *mut SlabPage;

        unsafe {
            *(obj as *mut *mut u8) = (*page).free_head;
            (*page).free_head = obj;
            (*page).in_use -= 1;

            if (*page).in_use == 0 {
                // TODO: Remove from partial/full list and free page
                // For now, keep it in partial list
            } else if (*page).in_use as usize == self.objs_per_page - 1 {
                // Moved from full to partial
                // TODO: Properly relink lists
            }
        }
    }
}