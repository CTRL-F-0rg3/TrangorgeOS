// x86_64 Linux Syscall Numbers (subset for ABI translation)
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_STAT: u64 = 4;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MPROTECT: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_IOCTL: u64 = 16;
pub const SYS_ACCESS: u64 = 21;
pub const SYS_GETPID: u64 = 39;
pub const SYS_SOCKET: u64 = 41;
pub const SYS_CONNECT: u64 = 42;
pub const SYS_EXIT: u64 = 60;
pub const SYS_CLONE: u64 = 56;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_EXIT_GROUP: u64 = 231;

// Translate Linux syscall number to native TrangorgeOS IPC/Service opcode
#[inline]
pub fn map_syscall_to_native_op(sys_num: u64) -> Option<u32> {
    match sys_num {
        SYS_READ | SYS_WRITE => Some(0x1001), // Native FS/IO opcode
        SYS_MMAP | SYS_MUNMAP | SYS_BRK => Some(0x2001), // Native MM opcode
        SYS_GETPID => Some(0x3001), // Native Proc opcode
        SYS_EXIT | SYS_EXIT_GROUP => Some(0x3002),
        SYS_SOCKET | SYS_CONNECT => Some(0x4001), // Native Net opcode
        _ => None, // Unhandled, requires full POSIX shim
    }
}