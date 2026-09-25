use tg_ipc_abi::opcodes::SysOpcode;
use super::raw;

pub unsafe fn read(fd: u64, buf: *mut u8, count: u64) -> u64 {
    raw::syscall3(SysOpcode::IoRead, fd, buf as u64, count)
}

pub unsafe fn write(fd: u64, buf: *const u8, count: u64) -> u64 {
    raw::syscall3(SysOpcode::IoWrite, fd, buf as u64, count)
}

pub unsafe fn open(path: *const u8, flags: u32, mode: u32) -> u64 {
    raw::syscall3(SysOpcode::IoOpen, path as u64, flags as u64, mode as u64)
}

pub unsafe fn close(fd: u64) -> u64 {
    raw::syscall1(SysOpcode::IoClose, fd)
}