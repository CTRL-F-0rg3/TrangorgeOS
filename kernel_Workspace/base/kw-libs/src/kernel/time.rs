// High-level time API for libraries and drivers.

pub fn ticks() -> u64 {
    kw_base::hal::timers::get_ticks()
}

pub fn frequency() -> u64 {
    kw_base::hal::timers::get_frequency()
}

pub fn sleep_ms(ms: u32) {
    kw_base::hal::timers::sleep_ms(ms);
}

pub fn sleep_ticks(ticks: u64) {
    let start = ticks();
    while ticks() - start < ticks {
        core::hint::spin_loop();
    }
}