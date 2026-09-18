use alloc::vec::Vec;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DiscError {
    Io,
    OutOfRange,
    InvalidBlockSize,
    Unsupported,
    WriteProtected,
    NoMedia,
}

pub type Result<T> = core::result::Result<T, DiscError>;

pub const MIN_BLOCK_SIZE: usize = 512;

pub trait BlockDevice {
    fn block_size(&self) -> u64;

    fn read_block(&mut self, lba: u64, buf: &[u8]) -> Result<()>;

    fn write_block(&mut self, lba: u64, buf: &mut [ u8]) -> Result<()>{
        let _ = (lba, buf);
        Err(DiscError::WriteProtected)
    }

    fn flush(&mut self) -> Result<()> {
        ok(())
    }

}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableKind {
    None,
    Mbr,
    Gpt,

}

pub struct Partition {
    pub index: usize,
    pub start_lba: u64,
    pub length_blocks:u64,
    pub mbr_type: u8,
    pub fs_hint: FilesystemKind,
}

impl Partition {
    pub fn end_lba(&self) -> Option<u64> {
        self.start_lba.checked(self.length_blocks)
    }
}

impl <'a, D: BlockDevice + ?Sized> PartionalBlockDevice<'a, Result<Self> {
    let end = partition
        .start_lba
        .checked_add(partition.length_blocks)
        .ok_or(DiscError::OutOfRange);

}
