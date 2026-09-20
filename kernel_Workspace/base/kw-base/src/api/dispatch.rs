use crate::core::KernelError;
use super::{fs_ops, memory, process, net_ops};

// Native TrangorgeOS Syscall Numbers
pub const SYS_READ: u64 = 1;
pub const SYS_WRITE: u64 = 2;
pub const SYS_OPEN: u64 = 3;
pub const SYS_CLOSE: u64 = 4;
pub const SYS_MMAP: u64 = 10;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_SPAWN: u64 = 20;
pub const SYS_EXIT: u64 = 21;
pub const SYS_YIELD: u64 = 22;
pub const SYS_GETPID: u64 = 23;
pub const SYS_SOCKET: u64 = 30;
pub const SYS_SEND: u64 = 31;
pub const SYS_RECV: u64 = 32;

// Main syscall dispatcher. Called from the trampoline/assembly entry point.
// Returns i64: positive for success/data, negative for KernelError codes.
pub fn dispatch(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let res: Result<i64, KernelError> = match num {
        SYS_READ => fs_ops::read(a0 as u32, a1 as *mut u8, a2 as usize).map(|v| v as i64),
        SYS_WRITE => fs_ops::write(a0 as u32, a1 as *const u8, a2 as usize).map(|v| v as i64),
        SYS_OPEN => fs_ops::open(a0 as *const u8, a1 as u32).map(|v| v as i64),
        SYS_CLOSE => fs_ops::close(a0 as u32).map(|_| 0),
        
        SYS_MMAP => memory::mmap(a0, a1 as usize, a2 as u32, a3 as u32).map(|v| v as i64),
        SYS_MUNMAP => memory::munmap(a0, a1 as usize).map(|_| 0),
        
        SYS_SPAWN => process::spawn(a0 as u32, a1, a2).map(|v| v as i64),
        SYS_EXIT => { process::exit(a0 as i32); },
        SYS_YIELD => { process::yield_now(); Ok(0) },
        SYS_GETPID => Ok(process::get_pid() as i64),
        
        SYS_SOCKET => net_ops::socket(a0 as u32, a1 as u32, a2 as u32).map(|v| v as i64),
        SYS_SEND => net_ops::send(a0 as u32, a1 as *const u8, a2 as u32, a3 as u32).map(|v| v as i64),
        SYS_RECV => net_ops::recv(a0 as u32, a1 as *mut u8, a2 as u32, a3 as u32).map(|v| v as i64),

        _ => Err(KernelError::NotSupported),
    };

    match res {
        Ok(v) => v,
        Err(e) => e.code() as i64,
    }
}