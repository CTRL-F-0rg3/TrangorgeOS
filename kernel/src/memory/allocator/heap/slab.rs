use core::ptr::NonNull;
use core::alloc::Layout;

pub const SLAB_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024];

struct SlabNode {
    next: Option<NonNull<SlabNode>>,
}

pub struct SlabBucket {
    block_size : usize,
    free_list: Option<NonNull<SlabNode>>,
}

impl SlabBucket {
    pub const fn new(block_size: usize) -> Self {
        Self {
            block_size,
            free_list: None,
        }
        pub unsafe fn push_block(&mut self, *mut u8) {
            let mode_ptr = ptr  as *mut SlabNode;
            (*node_ptr).next = self.free_list;
            self.free_list = NonNull::new(none_ptr);

        }

        pub fn allocate(&mut self) -> Option<NonNull<u8>> {
            let node = self.free_list?;
            unsafe {
                self.free_list = node.as_ref().next;
            }
            Some(node.cast::<u8>())
        }
        pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>) {
            let node_ptr = ptr.as_ptr() as *mut SlabNode;
            (*node_ptr).next = self.free_list;
            self.free_list = NonNull::new(node_ptr);

        }

    }
}
