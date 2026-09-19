use kstd_base::{Status, Size};

pub trait Allocator {
    fn alloc(&self, layout: Layout) -> Result<*mut u8, Status>;
    fn dealloc(&self, ptr: *mut u8, layout: Layout);
    fn realloc(&self, ptr: *mut u8, old: Layout, new_size: usize) -> Result<*mut u8, Status> {
        let new_ptr = self.alloc(Layout::new(new_size, old.align()))?;
        if !ptr.is_null() {
            unsafe {
                kstd_base::intrinsics::memcpy(new_ptr, ptr, old.size().min(new_size));
            }
            self.dealloc(ptr, old);
        }
        Ok(new_ptr)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    size: usize,
    align: usize,
}

impl Layout {
    pub const fn new(size: usize, align: usize) -> Self {
        Self { size, align }
    }

    pub const fn from_size(size: usize) -> Self {
        Self { size, align: 8 }
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn align(&self) -> usize {
        self.align
    }

    pub fn padded(&self) -> usize {
        (self.size + self.align - 1) & !(self.align - 1)
    }
}