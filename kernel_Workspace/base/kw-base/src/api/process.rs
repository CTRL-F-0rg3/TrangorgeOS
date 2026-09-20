use crate::core::{KResult, KernelError};

extern "C" {
    fn k_proc_spawn(world_id: u32, entry: u64, stack: u64) -> i64;
    fn k_proc_exit(code: i32);
    fn k_proc_yield();
    fn k_proc_get_pid() -> u32;
}

pub fn spawn(world_id: u32, entry: u64, stack: u64) -> KResult<u64> {
    let rc = unsafe { k_proc_spawn(world_id, entry, stack) };
    if rc < 0 { Err(KernelError::from_code(rc as i32)) } else { Ok(rc as u64) }
}

pub fn exit(code: i32) -> ! {
    unsafe { k_proc_exit(code); }
    loop { core::hint::spin_loop(); }
}

pub fn yield_now() {
    unsafe { k_proc_yield(); }
}

pub fn get_pid() -> u32 {
    unsafe { k_proc_get_pid() }
}