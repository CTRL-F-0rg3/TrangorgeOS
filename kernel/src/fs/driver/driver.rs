use crate::fs::disc::{BlockDevice, DiscError, Result};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use spin::Mutex;

pub struct BlockDeviceRegistry {
    devices: Vec<Box<dyn BlockDevice + Send>>,
}

impl BlockDeviceRegistry {
    pub const fn new() -> Self {
        BlockDeviceRegistry {
            devices: Vec::new(),
        }
    }

    pub fn register(&mut self, device: Box<dyn BlockDevice + Send>) -> usize {
        self.devices.push(device);
        self.devices.len() - 1
    }

    pub fn get(&mut self, index: usize) -> Option<&mut (dyn BlockDevice + Send + 'static)> {
        self.devices.get_mut(index).map(|b| b.as_mut())
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }
}

pub static REGISTRY: Mutex<BlockDeviceRegistry> = Mutex::new(BlockDeviceRegistry::new());

pub struct RamDisk {
    data: Vec<u8>,
    block_size: u64,
}

impl RamDisk {
    pub fn new(size_bytes: usize, block_size: u64) -> Self {
        RamDisk {
            data: vec![0u8; size_bytes],
            block_size,
        }
    }
}

impl BlockDevice for RamDisk {
    fn block_size(&self) -> u64 {
        self.block_size
    }

    fn block_count(&self) -> u64 {
        self.data.len() as u64 / self.block_size
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> Result<()> {
        let offset = (lba * self.block_size) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(DiscError::OutOfRange);
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> Result<()> {
        let offset = (lba * self.block_size) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(DiscError::OutOfRange);
        }
        self.data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}