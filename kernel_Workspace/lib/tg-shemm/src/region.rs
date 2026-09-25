use tg_cap_core::{ObjectId, Rights};
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct ShmemMapping {
    pub region_id: ObjectId,
    pub virt_addr: u64,
    pub offset_pages: u32,
    pub length_pages: u32,
    pub rights: Rights,
}

pub struct ShmemRegion {
    pub id: ObjectId,
    pub phys_base: u64,
    pub page_count: usize,
    pub map_count: AtomicU32,
}

impl ShmemRegion {
    pub fn new(id: ObjectId, phys_base: u64, page_count: usize) -> Self {
        Self {
            id,
            phys_base,
            page_count,
            map_count: AtomicU32::new(0),
        }
    }

    pub fn validate_mapping(&self, offset: u32, length: u32) -> bool {
        (offset as usize + length as usize) <= self.page_count
    }

    pub fn phys_for_offset(&self, offset_pages: u32) -> u64 {
        self.phys_base + (offset_pages as u64 * 4096)
    }

    pub fn add_mapping(&self) {
        self.map_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_mapping(&self) -> u32 {
        self.map_count.fetch_sub(1, Ordering::Release)
    }

    pub fn active_mappings(&self) -> u32 {
        self.map_count.load(Ordering::Acquire)
    }
}