//! Buddy allocator - backend sterty jądra: duże alokacje bezpośrednio
//! oraz świeże strony dla kubełków slab, gdy te są puste.

use core::cmp::{max, min};
use core::ptr::NonNull;

pub use crate::allocator::config::{HEAP_MAX_ORDER as MAX_ORDER, HEAP_MIN_ORDER as MIN_ORDER};

struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

pub struct BuddyAllocator {
    free_lists: [Option<NonNull<FreeNode>>; MAX_ORDER + 1],
    total_bytes: usize,
    allocated_bytes: usize,
}

// Sterta jest zawsze chroniona muteksem (`LockedHeap`), więc ręczne
// oznaczenie jako `Send` jest tu bezpieczne mimo surowych wskaźników.
unsafe impl Send for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [None; MAX_ORDER + 1],
            total_bytes: 0,
            allocated_bytes: 0,
        }
    }

    /// # Bezpieczeństwo
    /// Region musi być poprawnie zmapowaną, niewykorzystywaną pamięcią,
    /// dostępną przez cały czas życia alokatora.
    pub unsafe fn add_memory(&mut self, mut start_addr: usize, mut size: usize) {
        let min_align = 1usize << MIN_ORDER;
        if start_addr % min_align != 0 {
            let offset = min_align - (start_addr % min_align);
            if size <= offset { return; }
            start_addr += offset;
            size -= offset;
        }

        self.total_bytes += size;

        let mut current_addr = start_addr;
        let mut remaining_size = size;

        while remaining_size >= (1usize << MIN_ORDER) {
            let mut order = MAX_ORDER;
            while order > MIN_ORDER {
                let block_size = 1usize << order;
                if block_size <= remaining_size && current_addr % block_size == 0 {
                    break;
                }
                order -= 1;
            }

            let block_ptr = current_addr as *mut FreeNode;
            self.push_free_node(order, block_ptr);

            let allocated_size = 1usize << order;
            current_addr += allocated_size;
            remaining_size -= allocated_size;
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let required_size = max(size, max(align, 1usize << MIN_ORDER));
        let target_order = self.size_to_order(required_size)?;

        let mut current_order = target_order;
        while current_order <= MAX_ORDER {
            if let Some(node_ptr) = self.pop_free_node(current_order) {
                let mut split_order = current_order;
                while split_order > target_order {
                    split_order -= 1;
                    let buddy_addr = (node_ptr.as_ptr() as usize) + (1usize << split_order);
                    unsafe { self.push_free_node(split_order, buddy_addr as *mut FreeNode); }
                }
                self.allocated_bytes += 1usize << target_order;
                return Some(node_ptr.cast::<u8>());
            }
            current_order += 1;
        }
        None
    }

    /// # Bezpieczeństwo
    /// `ptr`, `size`, `align` muszą dokładnie odpowiadać wcześniejszemu
    /// wywołaniu `allocate`, które zwróciło ten wskaźnik.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, size: usize, align: usize) {
        let required_size = max(size, max(align, 1usize << MIN_ORDER));
        let mut order = match self.size_to_order(required_size) {
            Some(o) => o,
            None => return,
        };
        self.allocated_bytes -= 1usize << order;

        let mut current_ptr = ptr.as_ptr() as usize;

        while order < MAX_ORDER {
            let block_size = 1usize << order;
            let buddy_addr = current_ptr ^ block_size;

            if self.remove_free_node_if_exists(order, buddy_addr as *mut FreeNode) {
                current_ptr = min(current_ptr, buddy_addr);
                order += 1;
            } else {
                break;
            }
        }

        self.push_free_node(order, current_ptr as *mut FreeNode);
    }

    fn size_to_order(&self, size: usize) -> Option<usize> {
        if size == 0 { return Some(MIN_ORDER); }
        let mut order = MIN_ORDER;
        while (1usize << order) < size {
            order += 1;
            if order > MAX_ORDER { return None; }
        }
        Some(order)
    }

    unsafe fn push_free_node(&mut self, order: usize, ptr: *mut FreeNode) {
        (*ptr).next = self.free_lists[order];
        self.free_lists[order] = NonNull::new(ptr);
    }

    fn pop_free_node(&mut self, order: usize) -> Option<NonNull<FreeNode>> {
        let head = self.free_lists[order]?;
        unsafe { self.free_lists[order] = head.as_ref().next; }
        Some(head)
    }

    fn remove_free_node_if_exists(&mut self, order: usize, target: *mut FreeNode) -> bool {
        let mut current = &mut self.free_lists[order];
        while let Some(node) = *current {
            if node.as_ptr() == target {
                unsafe { *current = node.as_ref().next; }
                return true;
            }
            unsafe { current = &mut (*node.as_ptr()).next; }
        }
        false
    }

    pub fn total_bytes(&self) -> usize { self.total_bytes }
    pub fn allocated_bytes(&self) -> usize { self.allocated_bytes }
    pub fn free_bytes(&self) -> usize { self.total_bytes - self.allocated_bytes }
}