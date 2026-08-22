//! Klej jądra ←→ C (core-lang / edytor).
//!
//! Definiuje symbole `k_*`, których potrzebuje `libmm.a`, gdy do linkowania
//! trafia kod C core-lang (`bridge.c`, `loader.c`) oraz edytora (`editor.c`).
//! Jest to działa odpowiednik `kstd_glue.rs`, ale zawiera TYLKO funkcje
//! możliwe do zbudowania na istniejących modułach jądra — nie zależy od
//! niedeklarowanych jeszcze modułów (audio/userspace/trampoline_rings).
//!
//! Funkcje bez wsparcia w obecnym jądrze (procesy, IPC, tick) zwracają
//! wartości "puste" (−1 / 0), dzięki czemu mostek C linkuje się poprawnie.

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

/// `extern int32_t k_input_key(void);` (bridge.c br_key)
#[no_mangle]
pub extern "C" fn k_input_key() -> i32 {
    match crate::drivers::usb::class::hid::keyboard::take_char() {
        Some(c) => c as i32,
        None => -1,
    }
}

/// `extern uint32_t k_input_keycode(void);` (editor.c) — kod edytora,
/// 0 = brak zdarzenia, 0x100+ = EDK_* z editor.h.
#[no_mangle]
pub extern "C" fn k_input_keycode() -> u32 {
    // 1. PS/2 scancode-y wprost z terminala (strzałki, F-klawisze, ESC, ...).
    if let Some(k) = crate::terminal::pop_keycode() {
        return k;
    }

    // 2. Klawiatura USB HID (tylko znaki — mapuj na kody edytora).
    if let Some(c) = crate::drivers::usb::class::hid::keyboard::take_char() {
        return match c {
            b'\n' => 0x100, // EDK_ENTER
            8 => 0x101,     // EDK_BACKSPACE
            b'\t' => 0x10A, // EDK_TAB
            c if (c as u32) >= 32 && (c as u32) < 0x100 => c as u32,
            _ => 0,
        };
    }

    0
}

/// `extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);`
///
/// Czyta plik przez TFS (jedyny zamontowany system plików w tym buildzie).
/// Ścieżka może zawierać katalogi rozdzielone `/`; względna od katalogu
/// głównego (`ROOT_DIR`).
#[no_mangle]
pub extern "C" fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32 {
    if buf.is_null() || cap == 0 {
        return -1;
    }

    let dev = match crate::fs::root_device() {
        Some(d) => d,
        None => return -1,
    };

    let path = match cstr_to_str(path) {
        Some(value) => value,
        None => return -1,
    };

    let data = match tfs_read_path(dev, path) {
        Some(d) => d,
        None => return -1,
    };

    let n = data.len().min(cap as usize);
    unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), buf, n) };
    n as i32
}

/// `extern int32_t k_fs_exists(const char *path);`
#[no_mangle]
pub extern "C" fn k_fs_exists(path: *const u8) -> i32 {
    let dev = match crate::fs::root_device() {
        Some(d) => d,
        None => return 0,
    };

    let path = match cstr_to_str(path) {
        Some(value) => value,
        None => return 0,
    };

    if tfs_read_path(dev, path).is_some() {
        1
    } else {
        0
    }
}

/// Pomocnik: czyta plik przez TFS, obsługując proste ścieżki `a/b/c`.
fn tfs_read_path(dev: &dyn crate::fs::driver::block::BlockDevice, path: &str) -> Option<alloc::vec::Vec<u8>> {
    let trimmed = path.trim_start_matches('/');
    let mut parts = trimmed.split('/').filter(|p| !p.is_empty());
    let file = parts.next_back()?;

    let mut dir = crate::fs::tfs::ROOT_DIR;

    // Wszystkie człony poza ostatnim to katalogi — schodź w dół.
    for part in parts {
        dir = crate::fs::tfs::find_dir(dev, dir, part).ok()?;
    }

    crate::fs::tfs::read_file(dev, dir, file).ok()
}

/// `extern bool k_user_cstr(uint64_t ptr, char *buf, uint32_t cap);`
/// Kopiuje łańcuch z pamięci (ptr) do buf z limitem cap.
/// Zwraca true, gdy osiągnięto NUL.
#[no_mangle]
pub extern "C" fn k_user_cstr(ptr: u64, buf: *mut u8, cap: u32) -> bool {
    if ptr == 0 || buf.is_null() || cap == 0 {
        return false;
    }

    let src = ptr as *const u8;

    for i in 0..cap as usize {
        let c = unsafe { core::ptr::read_volatile(src.add(i)) };
        unsafe { *buf.add(i) = c };

        if c == 0 {
            return true;
        }
    }

    false
}

/// Procesy nie są jeszcze w tym buildzie — zwracamy 0.
#[no_mangle]
pub extern "C" fn k_getpid() -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn k_tick() -> u64 {
    0
}

#[no_mangle]
pub extern "C" fn k_world_cr3() -> u64 {
    0
}

#[no_mangle]
pub extern "C" fn k_kernel_cr3() -> u64 {
    use x86_64::registers::control::Cr3;

    Cr3::read().0.start_address().as_u64()
}

/// Spawn procesów jest niedostępny w tym buildzie.
#[no_mangle]
pub extern "C" fn k_spawn(_path: *const u8, _parent: u32, _cr3: u64) -> i64 {
    -1
}

#[no_mangle]
pub extern "C" fn k_ipc_send(_dst: u32, _a0: u64, _a1: u64) -> i32 {
    -1
}

#[no_mangle]
pub extern "C" fn k_ipc_recv(_out_a0: *mut u64, _out_a1: *mut u64) -> i32 {
    -1
}