use crate::core::{KResult, KernelError};

extern "C" {
    fn k_futex_wait(uaddr: *const u32, expected_val: u32, timeout_ns: u64) -> i32;
    fn k_futex_wake(uaddr: *const u32, wake_count: u32) -> i32;
}

pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;

pub fn wait(uaddr: *const u32, expected: u32, timeout_ns: u64) -> KResult<()> {
    let rc = unsafe { k_futex_wait(uaddr, expected, timeout_ns) };
    if rc == 0 { Ok(()) } else { Err(KernelError::from_code(rc)) }
}

pub fn wake(uaddr: *const u32, count: u32) -> KResult<u32> {
    let rc = unsafe { k_futex_wake(uaddr, count) };
    if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as u32) }
}