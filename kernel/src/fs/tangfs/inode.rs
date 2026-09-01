use super::TangFs;
use crate::fs::vfs::{Inode as VfsInode, DirEntry, FileType, Result};
use alloc::string::String;
use alloc::vec::Vec;

#[repr(C, packed)]
pub struct Inode {
    pub mode: u16,
    pub uid: u16,
    pub gid: u16,
    pub link_count: u16,
    pub size: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub crtime: u64,
    pub flags: u32,
    pub generation: u32,
    pub xattr_block: u64,
    pub extents: [Extent; 10],
    pub indirect_extent: u64,
    pub checksum: u32,
    pub reserved: [u8; 3840],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Extent {
    pub logical_block: u64,
    pub physical_block: u64,
    pub length: u32,
    pub flags: u32,
}

pub struct InodeHandle {
    pub fs: *const TangFs,
    pub ino: u64,
}

impl InodeHandle {
    fn load_inode(&self) -> Result<Inode> {
        let fs = unsafe { &*self.fs };
        let block = self.ino / 8; 
        let offset = (self.ino % 8) as usize * 512;
        
        let buf = fs.read_block(block + 1)?; 
        
        let inode: Inode = unsafe {
            core::ptr::read_unaligned(buf.as_ptr().add(offset) as *const Inode)
        };
        
        Ok(inode)
    }
    
    fn save_inode(&self, inode: &Inode) -> Result<()> {
        let fs = unsafe { &*self.fs };
        let block = self.ino / 8;
        let offset = (self.ino % 8) as usize * 512;
        
        let mut buf = fs.read_block(block + 1)?;
        
        unsafe {
            core::ptr::write_unaligned(
                buf.as_mut_ptr().add(offset) as *mut Inode,
                *inode,
            );
        }
        
        fs.write_block(block + 1, &buf)?;
        
        Ok(())
    }
}

impl VfsInode for InodeHandle {
    fn lookup(&self, name: &str) -> Result<Box<dyn VfsInode>> {
        let inode = self.load_inode()?;
        
        if inode.mode & 0xF000 != 0x4000 {
            return Err("Not a directory");
        }
        
        let child_ino = super::btree::lookup_dir_entry(
            unsafe { &*self.fs },
            self.ino,
            name,
        )?;
        
        Ok(Box::new(InodeHandle {
            fs: self.fs,
            ino: child_ino,
        }))
    }
    
    fn readdir(&self) -> Result<Vec<DirEntry>> {
        let inode = self.load_inode()?;
        
        if inode.mode & 0xF000 != 0x4000 {
            return Err("Not a directory");
        }
        
        super::btree::read_dir_entries(unsafe { &*self.fs }, self.ino)
    }
    
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        super::file::read_file(unsafe { &*self.fs }, &self.load_inode()?, offset, buf)
    }
    
    fn write(&self, offset: u64, data: &[u8]) -> Result<usize> {
        let mut inode = self.load_inode()?;
        let written = super::file::write_file(unsafe { &*self.fs }, &mut inode, offset, data)?;
        self.save_inode(&inode)?;
        Ok(written)
    }
    
    fn stat(&self) -> Result<crate::fs::vfs::Stat> {
        let inode = self.load_inode()?;
        
        Ok(crate::fs::vfs::Stat {
            ino: self.ino,
            mode: inode.mode,
            size: inode.size,
            atime: inode.atime,
            mtime: inode.mtime,
            ctime: inode.ctime,
        })
    }
}