#![no_std]

//! `audio` — audio jack / codec bridge, extracted from `kernel/src/audio`.
//!
//! Concrete actions: init the codec, poll the headphone jack, query state,
//! toggle the amplifier, and play a physical buffer.

extern "C" {
    fn ad_init(base: u32) -> bool;
    fn ad_jack_present() -> bool;
    fn ad_set_amp(on: bool);
    fn ad_play_phys(phys: u64, len: u32) -> bool;
    fn ad_stop();
}

static mut AMP_ON: bool = false;
static mut PRESENT: bool = false;

pub fn init(base: u32) -> bool {
    unsafe { ad_init(base) }
}

pub fn poll_jack() -> bool {
    let now = unsafe { ad_jack_present() };

    unsafe {
        if now != PRESENT {
            PRESENT = now;
            return true;
        }
    }

    false
}

pub fn query() -> u32 {
    unsafe { (PRESENT as u32) | ((AMP_ON as u32) << 1) }
}

pub fn set_amp(on: bool) {
    unsafe { AMP_ON = on; }
    unsafe { ad_set_amp(on); }
}

pub fn play_phys(phys: u64, len: u32) -> bool {
    unsafe { ad_play_phys(phys, len) }
}

pub fn stop() {
    unsafe { ad_stop(); }
}
