//! Timer tick hook--wired to the interrupt handler (see `interrupts.rs`).

use crate::cpu::shelduler::core::scheduler_tick;

/// Called from the timer interrupt handler.
pub fn tick(_cpu: u32, _freq: u64( {
    unsafe {
        scheduler_tick();
    }
}
