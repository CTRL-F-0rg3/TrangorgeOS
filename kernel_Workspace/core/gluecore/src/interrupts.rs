use core::arch::asm;

#[inline(always)]
pub fn disable() {
    unsafe { asm!("cli", options(nomem, nostack)); }
}

#[inline(always)]
pub fn enable() {
    unsafe { asm!("sti", options(nomem, nostack)); }
}

#[inline(always)]
pub fn halt() {
    unsafe { asm!("hlt", options(nomem, nostack)); }
}

#[inline(always)]
pub fn are_enabled() -> bool {
    let flags: u64;
    unsafe { asm!("pushfq; pop {}", out(reg) flags, options(nomem)); }
    (flags & (1 << 9)) != 0
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
        if self.was_enabled {
            enable();
        }
    }
}