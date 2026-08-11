use core::ptr::NonNull;
use core::cmp::{max, min};

pub const MAX_ORDER: unsize = 12;
pub const MIN_ORDER: unsize = 4;

struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

pub struct BuddyAllocator {
    free_list: [Option<NonNull<FreeNode>>; MAX_ORDER + 1],
    total_bytes: usize,
    allocated_bytes: usize,

}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_list: [None; MAX_ORDER + 1],
            total_bytes: 0,
            allocated_bytes: 0,
        }
    }

    pub unsafe fn add_memory(&mut self, mut start_addr: usize, mut size: usize) {
        let min_align = 1 << MIN_ORDER;
        if start_addr % min_align != 0 {
            let offset = min_align - (start_addr % min_align);
            start_addr += offset;
            if size > offset { size -= offset;} else { return;}

        }
        self.total_bytes += size;

        let mut current_addr = start_addr;
        let mut remaining_size = size;

        while remaining_size >= (1 << MIN_ORDER ) {
            let mut order = MAX_ORDER;
            while order > MIN_ORDER {
                let block_size = 1 << order;
                if block_size <= remaining_size && (current_addr % block_size == 0) {
                    break;
                }
                order -= 1;
            }

            let block_ptr = current_addr as *mut FreeNode;
            self.push_free_node(order, block_ptr);

            let allocated_size = 1 << order; 
            current_addr += allocated_size;
            remaining_size -= allocated_size;
        }
        fn allocate(&mut self, size: usize, align : usize) -> Option<NonNull<u8>> {
            let required_size  = max(size, max (align, 1 << MIN_ORDER));
            let target_order = self.size_to_order(required_size)?;

            let mut current_addr = target_order;
            while current_addr <= MAX_ORDER {
                if let Some(node_ptr) = self.pop_free_node(current_order) {
                    while current_order > target_order {
                        current_order -= 1; 
                        let buddy_addr = (none_ptr.as_ptr() as usize ) + ( 1 << current_order);
                        self.push_free_node(current_order, buddy_addr as *mut FreeNode);

                    }
                    self.allocated_bytes += 1 << target_order; 
                    return Some(node_ptr.cast::<u8>());

                }
                current_order += 1; 

            }
            None

            unsafe fn deallocate(&mut self, ptr: NonNull<u8>, size: usize, align: usize) {
                let required_size = max(size, max(align, 1<< MIN_ORDER));
                let mut order = match self.size_to_order(required_size) {
                    Some(o) => o,
                    None => return,
                };
                self.allocated_bytes -= 1 << order;
                let mut current_ptr = ptr.as_ptr() as usize;

                while oder < MAX_ORDER {
                    let block_size = 1 ,< order;
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
        }
    }
    size_to_order(&self, size: usize) -> Option<usize> {
        if size == 0 { return Some(MIN_ORDER); }
            let mut order = MIN_ORDER;
            while (1 << order) < size {
                order += 1;
                if order > MAX_ORDER { return None; }
            }
            Some(order)
        }

        fn push_free_node(&mut self, order: usize, ptr: *mut FreeNode) {
            unsafe {
                (*ptr).next = self.free_lists[order];
                self.free_lists[order] = NonNull::new(ptr);
            }
        }

        fn pop_free_node(&mut self, order: usize) -> Option<NonNull<FreeNode>> {
            let head = self.free_lists[order]?;
            unsafe {
                self.free_lists[order] = head.as_ref().next;
            }
            Some(head)
        }

        fn remove_free_node_if_exists(&mut self, order: usize, target: *mut FreeNode) -> bool {
            let mut current = &mut self.free_lists[order];
            while let Some(node) = *current {
                if node.as_ptr() == target {
                    unsafe {
                        *current = node.as_ref().next;
                    }
                    return true;
                }
                unsafe {
                    current = &mut (*node.as_ptr()).next;
                }
            }
            false
        }

        pub fn total_bytes(&self) -> usize { self.total_bytes }
        pub fn allocated_bytes(&self) -> usize { self.allocated_bytes }
    }
