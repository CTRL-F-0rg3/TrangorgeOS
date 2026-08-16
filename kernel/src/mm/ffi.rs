use core::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawMemEntry {
    pub base: u64,
    pub len: u64,
    pub typ: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MmBootParams {
    pub memmap: *const RawMemEntry,
    pub memmap_count: usize,
    pub kernel_phys_start: u64,
    pub kernel_phys_end: u64,
    pub initrd_phys_start: u64,
    pub initrd_phys_end: u64,
    pub boot_phys_offset: u64,
}

extern "C" {
    /* core */
    pub fn mm_init(params: *const MmBootParams) -> bool;
    pub fn mm_ready() -> bool;
    pub fn mm_total_ram() -> u64;
    pub fn mm_free_ram() -> u64;
    pub fn mm_dump();

    /* arch memory */
    pub fn arch_memory_ready() -> bool;
    pub fn arch_memory_reserve_range(base: u64, len: u64);
    pub fn arch_memory_boot_alloc(len: u64, align: u64, out: *mut u64) -> bool;

    /* paging */
    pub fn paging_init(boot_phys_offset: u64);
    pub fn paging_enable_nx();
    pub fn paging_read_cr3() -> u64;
    pub fn paging_aspace_switch(aspace: *mut c_void);

    /* pmm */
    pub fn pmm_init() -> bool;
    pub fn pmm_alloc_frame(out: *mut u64) -> bool;
    pub fn pmm_alloc_zero_frame(out: *mut u64) -> bool;
    pub fn pmm_alloc_frames(count: usize, out: *mut u64) -> bool;
    pub fn pmm_alloc_frames_aligned(count: usize,
                                    align: usize,
                                    out: *mut u64) -> bool;
    pub fn pmm_free_frame(phys: u64) -> bool;
    pub fn pmm_free_frames(phys: u64, count: usize) -> bool;

    /* vmm */
    pub fn vmm_init() -> bool;
    pub fn vmm_alloc(bytes: usize, flags: u32, out: *mut u64) -> bool;
    pub fn vmm_free(virt: u64, bytes: usize) -> bool;
    pub fn vmm_map_device(phys: u64, len: usize, out: *mut u64) -> bool;
    pub fn vmm_unmap_device(virt: u64, len: usize) -> bool;

    /* heap api */
    pub fn kmalloc(size: usize) -> *mut c_void;
    pub fn kzalloc(size: usize) -> *mut c_void;
    pub fn kcalloc(count: usize, size: usize) -> *mut c_void;
    pub fn krealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    pub fn kmalloc_aligned(size: usize, align: usize) -> *mut c_void;
    pub fn kfree(ptr: *mut c_void);
    pub fn kalloc_pages(pages: usize) -> *mut c_void;
    pub fn kfree_pages(ptr: *mut c_void, pages: usize);
    pub fn kvirt_to_phys(ptr: *mut c_void) -> u64;

    /* special */
    pub fn contig_alloc(bytes: usize,
                        align: usize,
                        out_phys: *mut u64,
                        out_virt: *mut *mut c_void) -> bool;
    pub fn contig_free(phys: u64, bytes: usize);
    pub fn dma_alloc_coherent(bytes: usize,
                              zone_max: u64,
                              out_phys: *mut u64,
                              out_virt: *mut *mut c_void) -> bool;
    pub fn dma_free_coherent(phys: u64, virt: *mut c_void, bytes: usize);

    /* process */
    pub fn aspace_subsystem_init() -> bool;
    pub fn aspace_create() -> *mut c_void;
    pub fn aspace_destroy(pa: *mut c_void);
    pub fn aspace_paging_handle(pa: *mut c_void) -> *mut c_void;
    pub fn aspace_map_anon(pa: *mut c_void,
                           hint: u64,
                           len: usize,
                           prot: u32) -> u64;
    pub fn aspace_unmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
    pub fn aspace_protect(pa: *mut c_void,
                          addr: u64,
                          len: usize,
                          prot: u32) -> bool;
    pub fn aspace_brk(pa: *mut c_void, new_brk: u64) -> u64;
    pub fn mmap(pa: *mut c_void,
                addr: u64,
                len: usize,
                prot: u32,
                flags: u32) -> u64;
    pub fn munmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
}