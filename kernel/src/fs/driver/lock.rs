use x86_64::instructions::interrupts;

/// RAII guard that disables interrupts while held and restores the previous
/// interrupt state on drop.
///
/// Uses the `x86_64` crate's `sti`/`cli` wrappers, which do not touch the
/// stack (the hand-written `pushfq`/`popfq` version could leave the stack
/// misaligned when an interrupt fired on re-enable).
pub struct IrqGuard {
    were_enabled: bool,
}

impl IrqGuard {
    pub fn lock() -> Self {
        let were_enabled = interrupts::are_enabled();

        if were_enabled {
            interrupts::disable();
        }

        Self { were_enabled }
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        if self.were_enabled {
            interrupts::enable();
        }
    }
}

