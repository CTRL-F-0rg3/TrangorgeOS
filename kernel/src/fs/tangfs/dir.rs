#[repr(C, packed)]
pub struct DirEntry {
    pub inode: u64,
    pub name_len: u16,
    pub file_type: u8,
    pub pad: u8,
    pub name: [u8; 256],
}