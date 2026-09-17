#![no_std]

use core::arch::asm;

/* ------------------------------------------------------------------ */
/*  Numery wywolan systemowych (rax = numer, int 0x80)                 */
/* ------------------------------------------------------------------ */

pub const SYS_YIELD: u64 = 1;
pub const SYS_LOG: u64 = 2;
pub const SYS_EXIT: u64 = 0x1000;
pub const SYS_SPAWN: u64 = 0x1001;
pub const SYS_GETPID: u64 = 0x1002;
pub const SYS_IPC_SEND: u64 = 0x1010;
pub const SYS_IPC_RECV: u64 = 0x1011;
pub const SYS_OPEN: u64 = 0x1020;
pub const SYS_READ: u64 = 0x1021;
pub const SYS_WRITE: u64 = 0x1022;
pub const SYS_CLOSE: u64 = 0x1023;
pub const SYS_WAIT: u64 = 0x1024;
pub const SYS_READDIR: u64 = 0x1025;
pub const SYS_KEY: u64 = 0x1040;
pub const SYS_RUNCL: u64 = 0x1050;
pub const SYS_UI_OPEN: u64 = 0x1060;

/* ------------------------------------------------------------------ */
/*  Prymitywy wywolan                                                  */
/* ------------------------------------------------------------------ */

#[inline]
fn sc0(n: u64) -> u64 {
    let r: u64;
    unsafe { asm!("int 0x80", in("rax") n, lateout("rax") r); }
    r
}

#[inline]
fn sc1(n: u64, a0: u64) -> u64 {
    let r: u64;
    unsafe { asm!("int 0x80", in("rax") n, in("rdi") a0, lateout("rax") r); }
    r
}

#[inline]
fn sc3(n: u64, a0: u64, a1: u64, a2: u64) -> u64 {
    let r: u64;
    unsafe {
        asm!("int 0x80", in("rax") n, in("rdi") a0,
             in("rsi") a1, in("rdx") a2, lateout("rax") r);
    }
    r
}

/// Kopiuje `s` do lokalnego bufora zakonczonego NUL.
///
/// Jadro czyta sciezki/teksty jako C-stringi, a `&str` z Rusta nie ma bajtu
/// NUL na koncu, dlatego kazda sciezka przechodzi przez ten pomocnik.
fn cstr(s: &str) -> ([u8; 256], usize) {
    let mut buf = [0u8; 256];
    let n = s.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&s.as_bytes()[..n]);
    (buf, n + 1)
}

/* ------------------------------------------------------------------ */
/*  Podstawowe operacje                                                 */
/* ------------------------------------------------------------------ */

pub fn log(s: &str) {
    let (buf, _len) = cstr(s);
    sc1(SYS_LOG, buf.as_ptr() as u64);
}

/// Wypisuje tekst na konsole uzytkownika (na razie tym samym kanalem co `log`).
pub fn print(s: &str) {
    log(s);
}

pub fn yield_cpu() { sc0(SYS_YIELD); }

pub fn exit(code: i32) -> ! {
    sc1(SYS_EXIT, code as u64);
    loop {}
}

pub fn getpid() -> u32 { sc0(SYS_GETPID) as u32 }

pub fn spawn(path: &str) -> i32 {
    let (buf, _len) = cstr(path);
    sc1(SYS_SPAWN, buf.as_ptr() as u64) as i32
}
pub fn key() -> Option<u8> {
    match sc0(SYS_KEY) { 0 => None, v => Some(v as u8) }
}

pub fn ipc_send(pid: u32, a0: u64, a1: u64) -> bool {
    sc3(SYS_IPC_SEND, pid as u64, a0, a1) != u64::MAX
}

pub struct Mail { pub from: u32, pub a0: u64, pub a1: u64 }

pub fn ipc_recv() -> Option<Mail> {
    let from: u64;
    let a0: u64;
    let a1: u64;
    unsafe {
        asm!("int 0x80", in("rax") SYS_IPC_RECV,
             lateout("rax") from, lateout("rdi") a0, lateout("rsi") a1);
    }
    if from == u64::MAX { None } else {
        Some(Mail { from: from as u32, a0, a1 })
    }
}

/* ------------------------------------------------------------------ */
/*  Pliki (ABI gotowe; jadro nie ma jeszcze tabeli fd)                  */
/* ------------------------------------------------------------------ */

pub fn open(path: &str) -> i32 {
    let (buf, _len) = cstr(path);
    sc1(SYS_OPEN, buf.as_ptr() as u64) as i32
}

pub fn read(fd: i32, buf: &mut [u8]) -> i32 {
    sc3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) as i32
}

pub fn write(fd: i32, buf: &[u8]) -> i32 {
    sc3(SYS_WRITE, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as i32
}

pub fn close(fd: i32) -> i32 {
    sc1(SYS_CLOSE, fd as u64) as i32
}

/// Czeka na zakonczenie dziecka. Zwraca `(pid, kod_wyjscia)` albo `None`.
pub fn wait() -> Option<(u32, i32)> {
    let pid: u64;
    let code: u64;
    unsafe {
        asm!("int 0x80", in("rax") SYS_WAIT,
             lateout("rax") pid, lateout("rdi") code);
    }
    if pid == u64::MAX { None } else { Some((pid as u32, code as i32)) }
}

/// Wpis katalogu: `Some(typ)` (1 = plik, 2 = katalog), nazwa trafia do `name`.
pub fn readdir(idx: u64, name: &mut [u8]) -> Option<u8> {
    let t = sc3(SYS_READDIR, idx, name.as_mut_ptr() as u64, name.len() as u64);
    if t == 0 || t == u64::MAX { None } else { Some(t as u8) }
}

/// Uruchamia program w jezyku Trangorge (core-lang) z podanej sciezki.
pub fn runcl(path: &str) -> i32 {
    let (buf, _len) = cstr(path);
    sc1(SYS_RUNCL, buf.as_ptr() as u64) as i32
}

/* ------------------------------------------------------------------ */
/*  UI — bezposredni zapis do framebuffera zmapowanego przez jadro     */
/* ------------------------------------------------------------------ */

/// Adres wirtualny framebuffera w przestrzeni uzytkownika (patrz SYS_UI_OPEN).
pub const UI_FB_VA: usize = 0x5000_0000;
/// Adres wirtualny tablicy czcionki 8x8 w przestrzeni uzytkownika.
pub const UI_FONT_VA: usize = 0x6000_0000;

/// Mapuje framebuffer i czcionke do przestrzeni uzytkownika.
/// Zwraca `(szerokosc, wysokosc, stride_w_bajtach)`.
pub fn ui_open() -> Option<(u32, u32, u32)> {
    let packed: u64;
    let stride: u64;
    unsafe {
        asm!("int 0x80", in("rax") SYS_UI_OPEN,
             lateout("rax") packed, lateout("rdi") stride);
    }
    if packed == u64::MAX {
        None
    } else {
        let w = ((packed >> 16) & 0xFFFF) as u32;
        let h = (packed & 0xFFFF) as u32;
        Some((w, h, stride as u32))
    }
}

/// Ustawia jeden piksel (32-bit, format framebuffera).
pub fn ui_pixel(stride: u32, x: i32, y: i32, color: u32, w: u32, h: u32) {
    if x < 0 || y < 0 || x as u32 >= w || y as u32 >= h || stride < 4 {
        return;
    }

    let off = (y as usize) * (stride as usize / 4) + x as usize;
    unsafe {
        core::ptr::write_volatile((UI_FB_VA as *mut u32).add(off), color);
    }
}

/// Wypelnia caly ekran jednym kolorem.
pub fn ui_clear(stride: u32, w: u32, h: u32, color: u32) {
    let _ = w;
    if stride < 4 {
        return;
    }

    let pixels = (stride as usize / 4) * h as usize;

    for i in 0..pixels {
        unsafe {
            core::ptr::write_volatile((UI_FB_VA as *mut u32).add(i), color);
        }
    }
}

/// Rysuje tekst czcionka 8x8 (`font8x8` z jadra: glif `c` ma indeks `c - 32`).
pub fn ui_text(stride: u32, x: i32, y: i32, s: &str, color: u32, w: u32, h: u32) {
    let mut cx = x;

    for &b in s.as_bytes() {
        let idx = if b < 32 || b > 127 { 0 } else { (b - 32) as usize };

        for row in 0..8i32 {
            let bits = unsafe {
                core::ptr::read_volatile((UI_FONT_VA as *const u8).add(idx * 8 + row as usize))
            };

            for col in 0..8i32 {
                if bits & (0x80 >> col) != 0 {
                    ui_pixel(stride, cx + col, y + row, color, w, h);
                }
            }
        }

        cx += 8;
    }
}


/* bump heap */
static mut HEAP: [u8; 256 * 1024] = [0; 256 * 1024];
static mut HEAP_POS: usize = 0;

pub fn malloc(n: usize) -> *mut u8 {
    unsafe {
        let p = HEAP_POS;
        HEAP_POS = (HEAP_POS + n + 15) & !15;
        // `addr_of_mut!` zamiast `HEAP.as_mut_ptr()` — bez tworzenia
        // referencji do `static mut` (lint `static_mut_refs`).
        core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(p)
    }
}

/* liczby */
pub fn put_u32(v: u32) {
    let mut buf = [0u8; 10];
    let mut n = 0usize;
    let mut x = v;

    if x == 0 { buf[0] = b'0'; n = 1; }
    else {
        while x > 0 { buf[n] = b'0' + (x % 10) as u8; x /= 10; n += 1; }
        for i in 0..n / 2 { buf.swap(i, n - 1 - i); }
    }

    if let Ok(s) = core::str::from_utf8(&buf[..n]) {
        log(s);
    }
}