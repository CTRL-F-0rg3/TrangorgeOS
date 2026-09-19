use crate::core::{KResult, KernelError, Handle, FileStat};
use kstd_base::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType { Regular, Directory, CharDevice, BlockDevice, Pipe, Socket }

pub trait Inode {
    fn stat(&self) -> KResult<FileStat>;
    fn read(&self, offset: u64, buf: &mut [u8]) -> KResult<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> KResult<usize>;
    fn lookup(&self, name: &str) -> KResult<Handle>;
}

pub trait File {
    fn inode(&self) -> &dyn Inode;
    fn flags(&self) -> u32;
}

pub trait FileSystem {
    fn name(&self) -> &str;
    fn root_inode(&self) -> Handle;
    fn sync(&self) -> KResult<()>;
}

pub struct Vfs;

impl Vfs {
    pub fn open(path: &str, flags: u32) -> KResult<Handle> {
        // Route to legacy kernel VFS via FFI if needed, or internal registry
        extern "C" { fn k_fs_open(path: *const u8, flags: u32) -> i32; }
        let rc = unsafe { k_fs_open(path.as_ptr(), flags) };
        if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(Handle(rc as u32)) }
    }

    pub fn read(fd: Handle, buf: &mut [u8]) -> KResult<usize> {
        extern "C" { fn k_fs_read(fd: i32, buf: *mut u8, cap: u32) -> i32; }
        let rc = unsafe { k_fs_read(fd.0 as i32, buf.as_mut_ptr(), buf.len() as u32) };
        if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as usize) }
    }

    pub fn write(fd: Handle, buf: &[u8]) -> KResult<usize> {
        extern "C" { fn k_fs_write(fd: i32, buf: *const u8, len: u32) -> i32; }
        let rc = unsafe { k_fs_write(fd.0 as i32, buf.as_ptr(), buf.len() as u32) };
        if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as usize) }
    }
}