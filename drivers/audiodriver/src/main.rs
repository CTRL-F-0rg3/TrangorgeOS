fn main() {
    println!("Hello, world!");
}

extern "C" {
    fn audio_play_phys(phys: u64, len: u32) -> i32;
    fn audio_stop() -> i32;
    fn audio_jack_present() -> i32;
    fn audio_set_amp(on: i32) -> i32;
}

pub fn play_phys(phys: u64, len: u32) -> bool { unsafe { audio_play_phys(phys, len) == 0 } }
pub fn stop() { unsafe { let _ = audio_stop(); } }
pub fn jack_present() -> bool { unsafe { audio_jack_present() != 0 } }
pub fn set_amp(on: bool) { unsafe { let _ = audio_set_amp(on as i32); } }