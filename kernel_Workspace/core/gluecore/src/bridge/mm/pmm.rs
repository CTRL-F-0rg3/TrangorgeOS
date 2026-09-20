// kernel_Workspace/core/gluecore/src/bridge/mm/pmm.rs

use kstd_base::{PhysAddr, Size, Status};
use crate::ffi::c::extern_block;

/// Allocates a single physical frame (4KB).
pub fn alloc_frame() -> Result<PhysAddr, Status> {
    let mut phys: u64 = 0;
    if unsafe { extern_block::pmm_alloc_frame(&mut phys) } {
        Ok(PhysAddr(phys))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Allocates a single zeroed physical frame.
pub fn alloc_zero_frame() -> Result<PhysAddr, Status> {
    let mut phys: u64 = 0;
    if unsafe { extern_block::pmm_alloc_zero_frame(&mut phys) } {
        Ok(PhysAddr(phys))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Allocates contiguous physical frames.
pub fn alloc_frames(count: usize) -> Result<PhysAddr, Status> {
    let mut phys: u64 = 0;
    if unsafe { extern_block::pmm_alloc_frames(count, &mut phys) } {
        Ok(PhysAddr(phys))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Allocates contiguous physical frames with a specific alignment (in frames).
pub fn alloc_frames_aligned(count: usize, align_frames: usize) -> Result<PhysAddr, Status> {
    let mut phys: u64 = 0;
    if unsafe { extern_block::pmm_alloc_frames_aligned(count, align_frames, &mut phys) } {
        Ok(PhysAddr(phys))
    } else {
        Err(Status::OutOfMemory)
    }
}

/// Frees a single physical frame.
pub fn free_frame(addr: PhysAddr) -> Result<(), Status> {
    if unsafe { extern_block::pmm_free_frame(addr.0) } {
        Ok(())
    } else {
        Err(Status::InvalidArgument)
    }
}

/// Frees contiguous physical frames.
pub fn free_frames(addr: PhysAddr, count: usize) -> Result<(), Status> {
    if unsafe { extern_block::pmm_free_frames(addr.0, count) } {
        Ok(())
    } else {
        Err(Status::InvalidArgument)
    }
}