use core::alloc::Layout;
use core::ptr::NonNull;

pub trait SubAllocator {
    unsafe fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>>;
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout);
}
