// kernel_Workspace/core/gluecore/src/ffi/c/extern_block.rs

// Raw C FFI declarations for the legacy kernel engine.
// These are strictly unsafe and must only be called through the `bridge` wrappers.

extern "C" {
    // --- Physical Memory Manager (PMM) ---
    pub fn pmm_alloc_frame(out_phys: *mut u64) -> bool;
    pub fn pmm_alloc_zero_frame(out_phys: *mut u64) -> bool;
    pub fn pmm_alloc_frames(count: usize, out_phys: *mut u64) -> bool;
    pub fn pmm_alloc_frames_aligned(count: usize, align_frames: usize, out_phys: *mut u64) -> bool;
    pub fn pmm_free_frame(phys: u64) -> bool;
    pub fn pmm_free_frames(phys: u64, count: usize) -> bool;

    // --- Virtual Memory Manager (VMM) ---
    pub fn vmm_alloc(bytes: usize, flags: u32, out_virt: *mut u64) -> bool;
    pub fn vmm_alloc_aligned(bytes: usize, align: usize, flags: u32, out_virt: *mut u64) -> bool;
    pub fn vmm_free(virt: u64, bytes: usize) -> bool;
    pub fn vmm_map_device(phys: u64, len: usize, out_virt: *mut u64) -> bool;
    pub fn vmm_unmap_device(virt: u64, len: usize) -> bool;

    // --- Kernel Heap ---
    pub fn kmalloc(size: usize) -> *mut u8;
    pub fn kzalloc(size: usize) -> *mut u8;
    pub fn krealloc(ptr: *mut u8, size: usize) -> *mut u8;
    pub fn kfree(ptr: *mut u8);
    pub fn kalloc_pages(pages: usize) -> *mut u8;
    pub fn kfree_pages(ptr: *mut u8, pages: usize);
    pub fn kvirt_to_phys(ptr: *const u8) -> u64;
    
    // --- Paging / Address Spaces ---
    pub fn paging_read_cr3() -> u64;
    pub fn paging_write_cr3(pml4_phys: u64);
    pub fn paging_map_page(virt: u64, phys: u64, flags: u64) -> bool;
    pub fn paging_unmap_page(virt: u64) -> bool;
    pub fn paging_flush_tlb_all();
    pub fn paging_flush_page(addr: u64);
}