use crate::core::{KResult, KernelError, Handle};
use crate::fs::Vfs;

pub fn open(path_ptr: *const u8, flags: u32) -> KResult<u32> {
    if path_ptr.is_null() { return Err(KernelError::BadAddress); }
    let path = unsafe { core::ffi::CStr::from_ptr(path_ptr as *const i8) }
        .to_str().map_err(|_| KernelError::InvalidArg)?;
    let handle = Vfs::open(path, flags)?;
    Ok(handle.0)
}

pub fn read(fd: u32, buf: *mut u8, len: usize) -> KResult<usize> {
    if buf.is_null() { return Err(KernelError::BadAddress); }
    let slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    Vfs::read(Handle(fd as i32), slice)
}

pub fn write(fd: u32, buf: *const u8, len: usize) -> KResult<usize> {
    if buf.is_null() { return Err(KernelError::BadAddress); }
    let slice = unsafe { core::slice::from_raw_parts(buf, len) };
    Vfs::write(Handle(fd as i32), slice)
}

pub fn close(fd: u32) -> KResult<()> {
    extern "C" { fn k_fs_close(fd: i32) -> i32; }
    let rc = unsafe { k_fs_close(fd as i32) };
    if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(()) }
}