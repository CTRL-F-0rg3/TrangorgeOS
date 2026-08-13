use crate::fs::disc::BlockDevice;
use crate::fs::share::{FsError, FsResult};
use alloc::vec::Vec;

pub const EXT4_SUPERBLOCK_OFFSET: u64 = 1024;
pub const EXT4_MAGIC_OFFSET: usize = 56;
pub const EXT4_MAGIC: u16 = 0xEF53;

pub fn is_ext4<D: BlockDevice + ?Sized>(device: &mut D) -> FsResult<bool> {
    let mut buf = [0u8; 1024];
    let sector = EXT4_SUPERBLOCK_OFFSET / device.block_size();
    device.read_block(sector, &mut buf).map_err(|_| FsError::Io)?;

    let magic_offset = (EXT4_SUPERBLOCK_OFFSET % device.block_size()) as usize + EXT4_MAGIC_OFFSET;
    if magic_offset + 2 > buf.len() {
        return Err(FsError::Io);
    }
    let magic = u16::from_le_bytes([buf[magic_offset], buf[magic_offset + 1]]);
    Ok(magic == EXT4_MAGIC)
}

pub fn mount<D: BlockDevice + ?Sized>(_device: &mut D) -> FsResult<()> {
    Err(FsError::Unsupported)
}

struct TestDisc {
    data: Vec<u8>,
}

impl BlockDevice for TestDisc {
    fn block_size(&self) -> u64 {
        512
    }

    fn block_count(&self) -> u64 {
        (self.data.len() / 512) as u64
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> crate::fs::disc::Result<()> {
        let offset = (lba * 512) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(crate::fs::disc::DiscError::OutOfRange);
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }
}

crate::test_module!({
    use alloc::vec;

    let mut data = vec![0u8; 512 * 4];
    let magic_pos = 1024 + EXT4_MAGIC_OFFSET;
    data[magic_pos] = 0x53;
    data[magic_pos + 1] = 0xEF;

    let mut disc = TestDisc { data };
    match is_ext4(&mut disc) {
        Ok(true) => {}
        Ok(false) => return Err("failed to detect a synthetic ext4 superblock magic"),
        Err(_) => return Err("is_ext4 returned an IO error on a valid synthetic buffer"),
    }

    if mount(&mut disc).is_ok() {
        return Err("mount() should not claim success - ext4 mounting is not implemented");
    }

    Ok("ext4 superblock magic detection verified (mount intentionally unsupported)")
});
