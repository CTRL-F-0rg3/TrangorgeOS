use tg_ipc_abi::opcodes::SysOpcode;
use super::raw;

pub const CLOCK_MONOTONIC: u32 = 0;
pub const CLOCK_REALTIME: u32 = 1;

#[repr(C)]
pub struct Timespec {
    pub tv_sec: u64,
    pub tv_nsec: u64,
}

pub unsafe fn clock_gettime(clock_id: u32, tp: *mut Timespec) -> u64 {
    raw::syscall2(SysOpcode::ClockGettime, clock_id as u64, tp as u64)
}