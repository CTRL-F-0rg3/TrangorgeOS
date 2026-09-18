use super::TangFs;
use crate::fs::vfs::Result;
use alloc::vec::Vec;

pub struct Journal {
    device: &'static dyn crate::fs::driver::BlockDevice,
    journal_start: u64,
    journal_size: u32,
    current_transaction: u64,
}

impl Journal {
    pub fn open(device: &'static dyn crate::fs::driver::BlockDevice, sb: &super::Superblock) -> Result<Self> {
        Ok(Self {
            device,
            journal_start: sb.journal_inode,
            journal_size: sb.journal_size,
            current_transaction: 0,
        })
    }
    
    pub fn write_block(&mut self, block: u64, data: &[u8]) -> Result<()> {
        let journal_offset = self.journal_start + (self.current_transaction % self.journal_size as u64);
        
        let mut journal_entry = Vec::with_capacity(4096);
        journal_entry.extend_from_slice(&self.current_transaction.to_le_bytes());
        journal_entry.extend_from_slice(&block.to_le_bytes());
        journal_entry.extend_from_slice(data);
        
        journal_entry.resize(4096, 0);
        
        self.device.write_blocks(journal_offset, 1, &journal_entry)?;
        
        self.current_transaction += 1;
        
        Ok(())
    }
    
    pub fn replay(&self) -> Result<()> {
        Ok(())
    }
}