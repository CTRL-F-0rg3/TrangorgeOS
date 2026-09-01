const SBI_EID_SRST: usize = 0x5352_5354;
const SBI_FID_SYSTEM_RESET: usize = 0;

const SRST_TYPE_SHUTDOWN: u32 = 0;
const SRST_TYPE_COLD_REBOOT: u32 = 1;

pub fn init() {

}


pub fn current_hart() -> u64 {
    0
}


pub fn poweroff() -> ! {
    system_reset(SRST_TYPE_SHUTDOWN)
}


pub fn reboot() -> ! {
    system_reset(SRST_TYPE_COLD_REBOOT)
}

fn system_reset(reset_type: u32) -> ! {
    let err: i64;
    let _val: i64;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") reset_type as usize => err,
            inlateout("a1") 0usize => _val,
            in("a2") 0usize, 
            in("a6") SBI_FID_SYSTEM_RESET,
            in("a7") SBI_EID_SRST,
            options(nomem, nostack)
        );
    }

    let _ = err;
    crate::hlt_loop()
}
