// Re-exports for external C/Ada/Nim bindings if needed, 
// or centralized FFI types that don't fit into specific bridges.
use kstd_base::{PhysAddr, VirtAddr};

#[repr(C)]
pub struct CPhysRange {
    pub start: u64,
    pub end: u64,
}

impl From<CPhysRange> for (PhysAddr, usize) {
    fn from(val: CPhysRange) -> Self {
        (PhysAddr(val.start), (val.end - val.start) as usize)
    }
}