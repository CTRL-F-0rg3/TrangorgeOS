use tg_ipc_abi::opcodes::SysOpcode;
use super::raw;

pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32 = 1 << 2;

pub const MAP_PRIVATE: u32 = 1 << 0;
pub const MAP_ANONYMOUS: u32 = 1 << 1;

pub unsafe fn mmap(addr: u64, len: u64, prot: u32, flags: u32) -> u64 {
    raw::syscall6(
        SysOpcode::MemMap,
        addr,
        len,
        prot as u64,
        flags as u64,
        0, 
        0  
    )
}

pub unsafe fn munmap(addr: u64, len: u64) -> u64 {
    raw::syscall2(SysOpcode::MemUnmap, addr, len)
}