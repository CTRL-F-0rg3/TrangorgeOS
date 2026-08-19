extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

fn cstr_len(p: *const u8) -> usize {
    let mut n = 0;
    unsafe {
        while *p.add(n) != 0 {
            n += 1;
        }
    }
    n
}

fn cstr_to_str<'a>(p: *const u8) -> &'a str {
    let n = cstr_len(p);
    unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, n)) }
}

#[no_mangle]
pub extern "C" fn k_input_key() -> i32 {
    match crate::drivers::usb::class::hid::keyboard::take_char() {
        Some(c) => c as i32,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32 {
    let fs = match crate::fs::vfs::root() {
        Some(f) => f,
        None => return -1,
    };

    let s = cstr_to_str(path);
    let slice = unsafe { core::slice::from_raw_parts_mut(buf, cap as usize) };

    match fs.read_path(s, slice) {
        Some(n) => n as i32,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn k_fs_exists(path: *const u8) -> i32 {
    let fs = match crate::fs::vfs::root() {
        Some(f) => f,
        None => return 0,
    };

    let mut tmp = [0u8; 64];

    if fs.read_path(cstr_to_str(path), &mut tmp).is_some() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn k_audio_play(phys: u64, len: u32) -> i32 {
    if crate::audio::jack::play_phys(phys, len) { 0 } else { -1 }
}

#[no_mangle]
pub extern "C" fn k_audio_stop() -> i32 {
    crate::audio::jack::stop();
    0
}

#[no_mangle]
pub extern "C" fn k_audio_jack() -> i32 {
    unsafe {
        crate::audio::jack::poll_jack();
        if crate::audio::jack::query() & 1 != 0 { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn k_audio_amp(on: i32) -> i32 {
    crate::audio::jack::set_amp(on != 0);
    0
}