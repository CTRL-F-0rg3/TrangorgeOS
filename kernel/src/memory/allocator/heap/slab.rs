

use core::ptr::NonNull;

pub const SLAB_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Rozmiar porcji pobieranej jedn
pub const SLAB_REFILL_SIZE: usize = 4096;

struct SlabNode {
    next: Option<NonNull<SlabNode>>,
}

pub struct SlabBucket {
    pub(crate) block_size: usize,
    free_list: Option<NonNull<SlabNode>>,
}

impl SlabBucket {
    pub const fn new(block_size: usize) -> Self {
        Self { block_size, free_list: None }
    }

    pub unsafe fn push_block(&mut self, ptr: *mut u8) {
        let node_ptr = ptr as *mut SlabNode;
        (*node_ptr).next = self.free_list;
        self.free_list = NonNull::new(node_ptr);
    }

    pub unsafe fn refill(&mut self, chunk: NonNull<u8>, chunk_size: usize) {
        let count = chunk_size / self.block_size;
        for i in 0..count {
            let block_ptr = chunk.as_ptr().add(i * self.block_size);
            self.push_block(block_ptr);
        }
    }

    pub fn allocate(&mut self) -> Option<NonNull<u8>> {
        let node = self.free_list?;
        unsafe { self.free_list = node.as_ref().next; }
        Some(node.cast::<u8>())
    }

    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>) {
        self.push_block(ptr.as_ptr());
    }
}