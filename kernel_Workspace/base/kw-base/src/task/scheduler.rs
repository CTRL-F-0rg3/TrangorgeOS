use crate::core::{KResult, KernelError};
use gluecore::context;

pub fn yield_now() {
    unsafe {
        core::arch::asm!("int 0x30", options(nomem, nostack)); // Trigger scheduler IPI/interrupt
    }
}

pub fn halt_cpu() {
    loop {
        crate::arch::interrupts::enable();
        crate::arch::interrupts::halt();
    }
}

pub fn set_current_priority(prio: i32) -> KResult<()> {
    if prio < -20 || prio > 19 {
        return Err(KernelError::InvalidArg);
    }
    // Hook into legacy kernel scheduler via gluecore if needed
    Ok(())
}