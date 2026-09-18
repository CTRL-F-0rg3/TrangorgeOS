pub mod superblock;
pub mod btree;
pub mod extent;
pub mod journal;
pub mod inode;
pub mod dir;
pub mod file;
pub mod format;

use crate::fs::vfs::{FileSystem, Inode as VfsInode, DirEntry, FileType, Result};
use alloc::vec::Vec;
use alloc::string::String;
use core::cell::RefCell;

pub use superblock::Superblock;
pub use inode::Inode;

pub struct TangFs {
    device: &'static dyn crate::fs::driver::BlockDevice,
    superblock: RefCell<Superblock>,
    journal: RefCell<journal::Journal>,
    block_cache: RefCell<alloc::collections::BTreeMap<u64, alloc::vec::Vec<u8>>>,
}

impl TangFs {
    pub fn mount(device: &'static dyn crate::fs::driver::BlockDevice) -> Result<Self> {
        let sb = Superblock::read(device)?;
        
        if &sb.magic != b"TANGFS01" {
            return Err("Invalid TangFS magic");
        }
        
        if sb.version > 0x0100 {
            return Err("Unsupported TangFS version");
        }
        
        let journal = journal::Journal::open(device, &sb)?;
        
        journal.replay()?;
        
        Ok(Self {
            device,
            superblock: RefCell::new(sb),
            journal: RefCell::new(journal),
            block_cache: RefCell::new(alloc::collections::BTreeMap::new()),
        })
    }
    
    pub fn read_block(&self, block: u64) -> Result<Vec<u8>> {
        let mut cache = self.block_cache.borrow_mut();
        
        if let Some(cached) = cache.get(&block) {
            return Ok(cached.clone());
        }
        
        let mut buf = vec![0u8; 4096];
        self.device.read_blocks(block, 1, &mut buf)?;
        
        cache.insert(block, buf.clone());
        Ok(buf)
    }
    
    pub fn write_block(&self, block: u64, data: &[u8]) -> Result<()> {
        if data.len() != 4096 {
            return Err("Block size mismatch");
        }
        

        self.journal.borrow_mut().write_block(block, data)?;
        
        self.device.write_blocks(block, 1, data)?;
        
        let mut cache = self.block_cache.borrow_mut();
        cache.insert(block, data.to_vec());
        
        Ok(())
    }
}

impl FileSystem for TangFs {
    fn root_inode(&self) -> Box<dyn VfsInode> {
        let sb = self.superblock.borrow();
        Box::new(inode::InodeHandle {
            fs: self as *const TangFs,
            ino: sb.root_inode,
        })
    }
    
    fn statfs(&self) -> Result<crate::fs::vfs::StatFs> {
        let sb = self.superblock.borrow();
        Ok(crate::fs::vfs::StatFs {
            total_blocks: sb.total_blocks,
            free_blocks: sb.free_blocks,
            block_size: sb.block_size,
            total_inodes: sb.total_blocks / 256, 
            free_inodes: sb.free_blocks / 256,
        })
    }
}