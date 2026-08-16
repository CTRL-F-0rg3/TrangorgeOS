pub struct IrqGuard {
    flags: u64,
}

impl IrqGuard {
    pub fn lock() -> Self {
        let flags: u64;

        unsafe {
            core::arch::asm!("pushfq", "pop {0}", "cli", out(reg) flags);
        }

        Self { flags }
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        unsafe {
            core::arch::asm!("push {0}", "popfq", in(reg) self.flags);
        }
    }
}