use x86_64::instructions::interrupts;

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
