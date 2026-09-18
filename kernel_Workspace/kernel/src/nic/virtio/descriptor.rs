pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtqDescriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}
