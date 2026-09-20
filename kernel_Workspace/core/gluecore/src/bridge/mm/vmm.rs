// kernel_Workspace/core/gluecore/src/bridge/mm/vmm.rs

use kstd_base::{VirtAddr, PhysAddr, Size, Status};
use crate::ffi::c::extern_block;

// VMM Flags (matching C-side definitions)
pub const VMM_FLAG_READ: u32 = 1 << 0;
pub const VMM_FLAG_WRITE: u32 = 1 << 1;
pub const VMM_FLAG_EXEC: u32 = 1 << 2;
pub const VMM_FLAG_USER: u32 = 1 << 3;
pub const VMM_FLAG_DEVICE: u32 = 1 << 4; // Uncacheable MMIO

/// Allocates virtual memory and maps it to physical frames.
pub fn alloc(bytes: usize, flags: u32) -> Result<VirtAddr, Status> {
    let mut virt: u64 = 0;
    if unsafe { extern_block::vmm_alloc(bytes, flags, &mut virt) } {
        Ok(VirtAddr(virt))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Allocates virtual memory with a specific alignment.
pub fn alloc_aligned(bytes: usize, align: usize, flags: u32) -> Result<VirtAddr, Status> {
    let mut virt: u64 = 0;
    if unsafe { extern_block::vmm_alloc_aligned(bytes, align, flags, &mut virt) } {
        Ok(VirtAddr(virt))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Frees previously allocated virtual memory.
pub fn free(addr: VirtAddr, bytes: usize) -> Result<(), Status> {
    if unsafe { extern_block::vmm_free(addr.0, bytes) } {
        Ok(())
    } else {
        Err(Status::InvalidArgument)
    }
}

/// Maps a physical device region (MMIO) into the kernel virtual space.
pub fn map_device(phys: PhysAddr, len: usize) -> Result<VirtAddr, Status> {
    let mut virt: u64 = 0;
    if unsafe { extern_block::vmm_map_device(phys.0, len, &mut virt) } {
        Ok(VirtAddr(virt))
    } else {
        Err(Status::BadAddress)
    }
}

/// Unmaps a device region.
pub fn unmap_device(addr: VirtAddr, len: usize) -> Result<(), Status> {
    if unsafe { extern_block::vmm_unmap_device(addr.0, len) } {
        Ok(())
    } else {
        Err(Status::InvalidArgument)
    }
}

/// Translates a kernel virtual address to a physical address.
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    let ptr = vaddr.0 as *const u8;
    PhysAddr(unsafe { extern_block::kvirt_to_phys(ptr) })
}