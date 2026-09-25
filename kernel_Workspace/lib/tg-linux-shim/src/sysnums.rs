use tg_ipc_abi::opcodes::SysOpcode;

pub const LINUX_READ: u32 = 0;
pub const LINUX_WRITE: u32 = 1;
pub const LINUX_OPEN: u32 = 2;
pub const LINUX_CLOSE: u32 = 3;
pub const LINUX_STAT: u32 = 4;
pub const LINUX_FSTAT: u32 = 5;
pub const LINUX_MMAP: u32 = 9;
pub const LINUX_MPROTECT: u32 = 10;
pub const LINUX_MUNMAP: u32 = 11;
pub const LINUX_BRK: u32 = 12;
pub const LINUX_EXIT: u32 = 60;
pub const LINUX_CLOCK_GETTIME: u32 = 228;

pub fn translate(linux_num: u32) -> Option<SysOpcode> {
    match linux_num {
        LINUX_READ => Some(SysOpcode::IoRead),
        LINUX_WRITE => Some(SysOpcode::IoWrite),
        LINUX_OPEN => Some(SysOpcode::IoOpen),
        LINUX_CLOSE => Some(SysOpcode::IoClose),
        LINUX_MMAP => Some(SysOpcode::MemMap),
        LINUX_MUNMAP => Some(SysOpcode::MemUnmap),
        LINUX_EXIT => Some(SysOpcode::ThreadExit),
        LINUX_CLOCK_GETTIME => Some(SysOpcode::ClockGettime),
        _ => None,
    }
}