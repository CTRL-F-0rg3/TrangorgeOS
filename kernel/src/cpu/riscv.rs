//! RISC-V CPU backend: SBI calls and hart identification.
//!
//! The kernel currently runs as the single S-mode hart under OpenSBI. SMP
//! bring-up (SBI HSM `hart_start`) is a future milestone; until then the
//! boot hart is 0 by definition.

/// Legacy/base SBI extension id for "System Reset" (SRST).
const SBI_EID_SRST: usize = 0x5352_5354;
const SBI_FID_SYSTEM_RESET: usize = 0;
/// SRST reset types.
const SRST_TYPE_SHUTDOWN: u32 = 0;
const SRST_TYPE_COLD_REBOOT: u32 = 1;

/// Early CPU init (single hart; no AP state to track yet).
pub fn init() {
    // Nothing to do — reserved as the hook for future per-hart state
    // (sscratch/tp setup once SMP via SBI HSM lands).
}

/// Hart running this code (0 until SMP bring-up exists).
pub fn current_hart() -> u64 {
    0
}

/// Power the machine off through the SBI System Reset extension.
pub fn poweroff() -> ! {
    system_reset(SRST_TYPE_SHUTDOWN)
}

/// Reboot through the SBI System Reset extension.
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
            in("a2") 0usize, // SRST reason: none
            in("a6") SBI_FID_SYSTEM_RESET,
            in("a7") SBI_EID_SRST,
            options(nomem, nostack)
        );
    }
    // If SBI refused (older firmware), just idle forever.
    let _ = err;
    crate::hlt_loop()
}
