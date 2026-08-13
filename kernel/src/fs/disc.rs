use crate::fs::share::FilesystemKind;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn block_count(&self) -> u64;

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> Result<()>;

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> Result<()> {
        let _ = (lba, buf);
        Err(DiscError::WriteProtected)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTableKind {
    None,
    Mbr,
    Gpt,
}

#[derive(Debug, Clone, Copy)]
pub struct Partition {
    pub index: usize,
    pub start_lba: u64,
    pub length_blocks: u64,
    pub mbr_type: u8,
    pub fs_hint: FilesystemKind,
}

impl Partition {
    pub fn end_lba(&self) -> Option<u64> {
        self.start_lba.checked_add(self.length_blocks)
    }
}

pub struct PartitionBlockDevice<'a, D: BlockDevice + ?Sized> {
    device: &'a mut D,
    partition: Partition,
}

impl<'a, D: BlockDevice + ?Sized> PartitionBlockDevice<'a, D> {
    pub fn new(device: &'a mut D, partition: Partition) -> Result<Self> {
        partition
            .start_lba
            .checked_add(partition.length_blocks)
            .ok_or(DiscError::OutOfRange)?;
        Ok(PartitionBlockDevice { device, partition })
    }

    fn translate(&self, lba: u64) -> Result<u64> {
        if lba >= self.partition.length_blocks {
            return Err(DiscError::OutOfRange);
        }
        self.partition
            .start_lba
            .checked_add(lba)
            .ok_or(DiscError::OutOfRange)
    }
}

impl<'a, D: BlockDevice + ?Sized> BlockDevice for PartitionBlockDevice<'a, D> {
    fn block_size(&self) -> u64 {
        self.device.block_size()
    }

    fn block_count(&self) -> u64 {
        self.partition.length_blocks
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> Result<()> {
        let absolute = self.translate(lba)?;
        self.device.read_block(absolute, buf)
    }

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> Result<()> {
        let absolute = self.translate(lba)?;
        self.device.write_block(absolute, buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.device.flush()
    }
}

pub const MBR_SIGNATURE_OFFSET: usize = 510;
pub const MBR_PARTITION_TABLE_OFFSET: usize = 446;
pub const MBR_PARTITION_ENTRY_SIZE: usize = 16;
pub const MBR_MAX_PARTITIONS: usize = 4;

pub fn read_mbr_partitions<D: BlockDevice + ?Sized>(device: &mut D) -> Result<Vec<Partition>> {
    let mut sector = [0u8; 512];
    device.read_block(0, &mut sector)?;

    if sector[MBR_SIGNATURE_OFFSET] != 0x55 || sector[MBR_SIGNATURE_OFFSET + 1] != 0xAA {
        return Err(DiscError::Unsupported);
    }

    let mut partitions = Vec::new();
    for i in 0..MBR_MAX_PARTITIONS {
        let entry_offset = MBR_PARTITION_TABLE_OFFSET + i * MBR_PARTITION_ENTRY_SIZE;
        let entry = &sector[entry_offset..entry_offset + MBR_PARTITION_ENTRY_SIZE];

        let partition_type = entry[4];
        if partition_type == 0 {
            continue;
        }

        let start_lba = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as u64;
        let length_blocks =
            u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as u64;

        partitions.push(Partition {
            index: i,
            start_lba,
            length_blocks,
            mbr_type: partition_type,
            fs_hint: fs_hint_from_mbr_type(partition_type),
        });
    }

    Ok(partitions)
}

fn fs_hint_from_mbr_type(mbr_type: u8) -> FilesystemKind {
    match mbr_type {
        0x0B | 0x0C | 0x0E => FilesystemKind::Fat32,
        0x83 => FilesystemKind::Ext4,
        _ => FilesystemKind::Unknown,
    }
}

struct MemoryDisc {
    data: Vec<u8>,
}

impl BlockDevice for MemoryDisc {
    fn block_size(&self) -> u64 {
        512
    }

    fn block_count(&self) -> u64 {
        (self.data.len() / 512) as u64
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> Result<()> {
        let offset = (lba * 512) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(DiscError::OutOfRange);
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> Result<()> {
        let offset = (lba * 512) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(DiscError::OutOfRange);
        }
        self.data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

crate::test_module!({
    let mut data = vec![0u8; 512 * 4];
    data[510] = 0x55;
    data[511] = 0xAA;

    let entry_offset = 446;
    data[entry_offset + 4] = 0x0C;
    data[entry_offset + 8..entry_offset + 12].copy_from_slice(&1u32.to_le_bytes());
    data[entry_offset + 12..entry_offset + 16].copy_from_slice(&2u32.to_le_bytes());

    let mut disc = MemoryDisc { data };
    let partitions = match read_mbr_partitions(&mut disc) {
        Ok(p) => p,
        Err(_) => return Err("failed to parse a valid synthetic MBR"),
    };

    if partitions.len() != 1 {
        return Err("expected exactly one non-empty partition entry");
    }
    let part = partitions[0];
    if part.start_lba != 1 || part.length_blocks != 2 {
        return Err("parsed partition geometry did not match the synthetic MBR");
    }
    if part.fs_hint != FilesystemKind::Fat32 {
        return Err("MBR type 0x0C should hint at FAT32");
    }

    let mut view = match PartitionBlockDevice::new(&mut disc, part) {
        Ok(view) => view,
        Err(_) => return Err("failed to construct a partition-relative block device"),
    };
    let mut buf = [0u8; 512];
    if view.read_block(0, &mut buf).is_err() {
        return Err("partition-relative read failed within bounds");
    }
    if view.read_block(2, &mut buf).is_ok() {
        return Err("partition-relative read succeeded past the partition end");
    }

    Ok("MBR parsing + partition-relative block device verified")
});