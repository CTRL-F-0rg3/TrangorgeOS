use crate::core::{KResult, KernelError};

extern "C" {
    fn k_timer_get_ticks() -> u64;
    fn k_timer_get_frequency() -> u64;
    fn k_timer_sleep_ms(ms: u32);
    fn k_timer_set_oneshot(ns: u64) -> bool;
}

pub fn get_ticks() -> u64 {
    unsafe { k_timer_get_ticks() }
}

pub fn get_frequency() -> u64 {
    unsafe { k_timer_get_frequency() }
}

pub fn sleep_ms(ms: u32) {
    unsafe { k_timer_sleep_ms(ms); }
}

pub fn set_oneshot(ns: u64) -> KResult<()> {
    if unsafe { k_timer_set_oneshot(ns) } {
        Ok(())
    } else {
        Err(KernelError::IoError)
    }
}