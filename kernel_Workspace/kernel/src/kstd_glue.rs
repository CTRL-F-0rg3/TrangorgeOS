const KSTD_PATH_MAX: usize = 256;

fn cstr_to_str<'a>(p: *const u8) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }

    let mut n = 0usize;

    unsafe {
        while n < KSTD_PATH_MAX {
            if *p.add(n) == 0 {
                return core::str::from_utf8(core::slice::from_raw_parts(p, n)).ok();
            }

            n += 1;
        }
    }

    None
}

#[no_mangle]
pub extern "C" fn k_input_key() -> i32 {
    match crate::drivers::usb::class::hid::keyboard::take_char() {
        Some(c) => c as i32,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn k_input_keycode() -> u32 {
    if let Some(k) = crate::terminal::pop_keycode() {
        return k;
    }

    if let Some(c) = crate::drivers::usb::class::hid::keyboard::take_char() {
        return match c {
            b'\n' => 0x100, 
            8 => 0x101,     
            b'\t' => 0x10A, 
            c if (c as u32) >= 32 && (c as u32) < 0x100 => c as u32,
            _ => 0,
        };
    }

    0
}
#[no_mangle]
pub extern "C" fn k_getpid() -> u32 {
    let w = crate::trampoline_rings::current();
    crate::process::proc::by_world(w).map(|p| p.pid).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn k_world_cr3() -> u64 {
    crate::trampoline_rings::world_cr3(crate::trampoline_rings::current())
}

#[no_mangle]
pub extern "C" fn k_kernel_cr3() -> u64 {
    extern "C" { fn paging_read_cr3() -> u64; }
    unsafe { paging_read_cr3() }
}

#[no_mangle]
pub extern "C" fn k_spawn(path: *const u8, parent: u32, cr3: u64) -> i64 {
    crate::process::syscall::do_spawn(cr3, path as u64, parent)
}

#[no_mangle]
pub extern "C" fn k_ipc_send(dst: u32, a0: u64, a1: u64) -> i32 {
    let w = crate::trampoline_rings::current();
    let me = crate::process::proc::by_world(w).map(|p| p.pid).unwrap_or(0);

    if crate::process::proc::send(dst, crate::process::proc::IpcMsg {
        from: me, a0, a1, a2: 0, a3: 0,
    }) { 0 } else { -1 }
}

#[no_mangle]
pub extern "C" fn k_ipc_recv(out_a0: *mut u64, out_a1: *mut u64) -> i32 {
    let w = crate::trampoline_rings::current();
    let me = match crate::process::proc::by_world(w) {
        Some(p) => p.pid,
        None => return -1,
    };

    match crate::process::proc::recv(me) {
        Some(m) => {
            unsafe {
                *out_a0 = m.a0;
                *out_a1 = m.a1;
            }
            m.from as i32
        }
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn k_tick() -> u64 {
    crate::process::syscall::tick_get()
}
#[no_mangle]
pub extern "C" fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32 {
    if buf.is_null() || cap == 0 {
        return -1;
    }

    let fs = match crate::fs::vfs::root() {
        Some(f) => f,
        None => return -1,
    };

    let path = match cstr_to_str(path) {
        Some(value) => value,
        None => return -1,
    };

    let slice = unsafe { core::slice::from_raw_parts_mut(buf, cap as usize) };

    match fs.read_path(path, slice) {
        Some(n) if n <= i32::MAX as usize => n as i32,
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn k_fs_exists(path: *const u8) -> i32 {
    let fs = match crate::fs::vfs::root() {
        Some(f) => f,
        None => return 0,
    };

    let path = match cstr_to_str(path) {
        Some(value) => value,
        None => return 0,
    };

    let mut tmp = [0u8; 64];

    if fs.read_path(path, &mut tmp).is_some() {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn k_audio_play(phys: u64, len: u32) -> i32 {
    if crate::audio::jack::play_phys(phys, len) {
        0
    } else {
        -1
    }
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

        if crate::audio::jack::query() & 1 != 0 {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn k_audio_amp(on: i32) -> i32 {
    crate::audio::jack::set_amp(on != 0);
    0
}
