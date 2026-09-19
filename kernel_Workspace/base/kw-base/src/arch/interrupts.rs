use core::sync::atomic::{AtomicUsize, Ordering};

#[inline(always)]
pub fn disable() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
}

#[inline(always)]
pub fn enable() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
}

#[inline(always)]
pub fn halt() {
    unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
}

#[inline(always)]
pub fn are_enabled() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        let flags: u64;
        unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem)); }
        (flags & (1 << 9)) != 0
    }
    #[cfg(not(target_arch = "x86_64"))]
    { false }
}

pub struct InterruptGuard {
    was_enabled: bool,
}

impl InterruptGuard {
    pub fn disable() -> Self {
        let was_enabled = are_enabled();
        disable();
        Self { was_enabled }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.was_enabled { enable(); }
    }
}