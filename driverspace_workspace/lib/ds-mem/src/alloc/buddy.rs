use core::alloc::Layout;

const MAX_ORDER: usize = 10; // Np. do 4MB przy stronie 4KB

pub struct BuddyAllocator {
    base_addr: usize,
    total_pages: usize,
    free_lists: [usize; MAX_ORDER + 1],
}

impl BuddyAllocator {
    pub const fn new(base_addr: usize, total_pages: usize) -> Self {
        Self {
            base_addr,
            total_pages,
            free_lists: [0; MAX_ORDER + 1],
        }
    }

    pub fn alloc_pages(&mut self, order: usize) -> Option<usize> {
        if order > MAX_ORDER {
            return None;
        }
        // TODO: Implement actual buddy allocation (find free block, split if necessary)
        Some(self.base_addr)
    }

    pub fn free_pages(&mut self, addr: usize, order: usize) {
        // TODO: Implement buddy merge logic
    }
}