//! Shared-memory regions and their mappings.

use crate::rights::Rights;

/// A kernel-owned shared-memory region.
#[derive(Debug, Clone, Copy)]
pub struct ShmemRegion {
    pub id: u64,
    pub phys_base: u64,
    pub page_count: usize,
}

/// A mapping of (part of) a region into a layer's address space.
#[derive(Debug, Clone, Copy)]
pub struct ShmemMapping {
    pub region_id: u64,
    pub virt_addr: u64,
    pub offset_pages: u32,
    pub length_pages: u32,
    pub rights: Rights,
}

impl ShmemRegion {
    pub const fn new(id: u64, phys_base: u64, page_count: usize) -> Self {
        Self {
            id,
            phys_base,
            page_count,
        }
    }

    /// Validate that `[offset, offset+length)` lies within the region.
    pub const fn validate(&self, offset: u32, length: u32) -> bool {
        (offset as usize + length as usize) <= self.page_count
    }
}
