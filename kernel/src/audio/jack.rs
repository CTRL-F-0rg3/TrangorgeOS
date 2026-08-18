use crate::drivers::audiodriver as hal;

static mut AMP_ON: bool = false;
static mut PRESENT: bool = false;

pub fn init(base: u32) -> bool {
    hal::init(base)
}

pub fn poll_jack() -> bool {
    let now = hal::jack_present();

    unsafe {
        if now != PRESENT {
            PRESENT = now;
            return true;
        }
    }

    false
}

pub fn query() -> u32 {
    unsafe {
        (PRESENT as u32) | ((AMP_ON as u32) << 1)
    }
}

pub fn set_amp(on: bool) {
    unsafe { AMP_ON = on; }
    hal::set_amp(on);
}

pub fn play_phys(phys: u64, len: u32) -> bool {
    hal::play_phys(phys, len)
}

pub fn stop() {
    hal::stop();
}