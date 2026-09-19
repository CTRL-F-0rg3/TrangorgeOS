// kw-libs/src/mem/virtual/vma.rs

use kstd_base::{VirtAddr, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmaType {
    Anonymous,
    FileBacked,
    DeviceMapped,
    Stack,
    Heap,
}

pub struct Vma {
    pub start: VirtAddr,
    pub end: VirtAddr,
    pub flags: u32, // PROT_READ | PROT_WRITE | PROT_EXEC
    pub vma_type: VmaType,
    pub file_offset: u64, // If FileBacked
    pub next: *mut Vma,   // For linked list in AddressSpace
}

impl Vma {
    pub fn new(start: VirtAddr, end: VirtAddr, flags: u32, vma_type: VmaType) -> Self {
        Self {
            start,
            end,
            flags,
            vma_type,
            file_offset: 0,
            next: core::ptr::null_mut(),
        }
    }

    pub fn size(&self) -> usize {
        (self.end.0 - self.start.0) as usize
    }

    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr.0 >= self.start.0 && addr.0 < self.end.0
    }

    pub fn overlaps(&self, start: VirtAddr, end: VirtAddr) -> bool {
        self.start.0 < end.0 && start.0 < self.end.0
    }
}