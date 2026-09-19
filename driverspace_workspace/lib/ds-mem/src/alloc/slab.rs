pub struct SlabAllocator {
    object_size: usize,
    free_head: usize,
}

impl SlabAllocator {
    pub const fn new(object_size: usize) -> Self {
        Self {
            object_size,
            free_head: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<usize> {
        // TODO: Pop address from free_head linked list
        Some(0)
    }

    pub fn free(&mut self, ptr: usize) {
        // TODO: Push address to free_head linked list
    }
}