use core::ptr::NonNull;

pub const SLAB_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct SlabNode {
    next: Option<NonNull<SlabNode>>,
}

pub struct SlabBucket {
    pub block_size: usize,
    free_list: Option<NonNull<SlabNode>>,
}

unsafe impl Send for SlabBucket {}

crate::test_module!({
    static mut SCRATCH: [u64; 32] = [0; 32];
    let mut bucket = SlabBucket::new(32);
    let base = &raw mut SCRATCH as *mut u8;

    unsafe {
        for i in 0..4 {
            bucket.push_block(base.add(i * 32));
        }
    }

    let a = match bucket.allocate() {
        Some(ptr) => ptr,
        None => return Err("slab bucket returned None after blocks were pushed"),
    };
    let b = match bucket.allocate() {
        Some(ptr) => ptr,
        None => return Err("slab bucket ran out of blocks too early"),
    };
    if a == b {
        return Err("slab bucket handed out the same block twice");
    }

    unsafe {
        bucket.deallocate(a);
    }
    let reused = match bucket.allocate() {
        Some(ptr) => ptr,
        None => return Err("slab bucket failed to reuse a freed block"),
    };
    if reused != a {
        return Err("slab bucket did not reuse the most recently freed block");
    }

    Ok("slab bucket push/allocate/deallocate/reuse verified")
});

impl SlabBucket {
    pub const fn new(block_size: usize) -> Self {
        Self {
            block_size,
            free_list: None,
        }
    }

    pub unsafe fn push_block(&mut self, ptr: *mut u8) {
        let node_ptr = ptr as *mut SlabNode;
        unsafe {
            (*node_ptr).next = self.free_list;
        }
        self.free_list = NonNull::new(node_ptr);
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
        unsafe {
            (*node_ptr).next = self.free_list;
        }
        self.free_list = NonNull::new(node_ptr);
    }
}
