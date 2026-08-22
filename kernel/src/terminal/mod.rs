//! Interaktywny terminal + shell z obsługą plików na dysku (TFS).
//!
//! Prompt: `#$-=>` (biały) + `_` (czerwony kursor) + wpisywany tekst (niebieski).

use crate::fs::driver::block::BlockDevice;
use crate::vga_buffer::{Color, WRITER};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};

// Edytor jądra — `kernel/src/editor/editor.c`, skompilowany przez build.rs
// i dołączany do `libmm.a`. Znak 0 kończy ścieżkę C-stringa.
unsafe extern "C" {
    fn editor_run(path: *const u8) -> i32;
}

/* ------------------------------------------------------------------ */
/* Bufor klawiatury (SPSC ring buffer, IRQ-safe)                       */
/* ------------------------------------------------------------------ */

const KBUF_SIZE: usize = 256;
static KBUF: [AtomicU8; KBUF_SIZE] = [const { AtomicU8::new(0) }; KBUF_SIZE];
static KHEAD: AtomicUsize = AtomicUsize::new(0);
static KTAIL: AtomicUsize = AtomicUsize::new(0);
static SHIFT: AtomicBool = AtomicBool::new(false);

/* Bufor kodów edytora (k_input_keycode). Napełniany tylko podczas pracy
   edytora — wtedy scancode-y nie trafiają do bufora tekstu terminala. */
const KCODE_SIZE: usize = 64;
static KCODEBUF: [AtomicU32; KCODE_SIZE] = [const { AtomicU32::new(0) }; KCODE_SIZE];
static KCODE_HEAD: AtomicUsize = AtomicUsize::new(0);
static KCODE_TAIL: AtomicUsize = AtomicUsize::new(0);
static CAPTURE_KEYCODE: AtomicBool = AtomicBool::new(false);

fn kbuf_push(c: u8) {
    let tail = KTAIL.load(Ordering::Relaxed);
    let next = (tail + 1) % KBUF_SIZE;
    if next == KHEAD.load(Ordering::Acquire) {
        return; // pełny — porzuć
    }
    KBUF[tail].store(c, Ordering::Relaxed);
    KTAIL.store(next, Ordering::Release);
}

fn kbuf_pop() -> Option<u8> {
    let head = KHEAD.load(Ordering::Relaxed);
    if head == KTAIL.load(Ordering::Acquire) {
        return None; // pusty
    }
    let c = KBUF[head].load(Ordering::Relaxed);
    KHEAD.store((head + 1) % KBUF_SIZE, Ordering::Release);
    Some(c)
}

/* ------------------------------------------------------------------ */
/* Scancode set 1 → znak                                              */
/* ------------------------------------------------------------------ */

/// Zamienia scancode na znak (z obsługą Shift). Zwraca None dla klawiszy
/// specjalnych (np. samego Shift/Ctrl/Alt).
fn scancode_to_char(code: u8) -> Option<char> {
    let shift = SHIFT.load(Ordering::Relaxed);

    let c: char = match code {
        // Enter / Backspace / Space
        0x1C => '\n',
        0x0E => '\x08',
        0x39 => ' ',

        // rząd liczb
        0x02 => if shift { '!' } else { '1' },
        0x03 => if shift { '@' } else { '2' },
        0x04 => if shift { '#' } else { '3' },
        0x05 => if shift { '$' } else { '4' },
        0x06 => if shift { '%' } else { '5' },
        0x07 => if shift { '^' } else { '6' },
        0x08 => if shift { '&' } else { '7' },
        0x09 => if shift { '*' } else { '8' },
        0x0A => if shift { '(' } else { '9' },
        0x0B => if shift { ')' } else { '0' },

        // litery górny rząd q-p
        0x10 => if shift { 'Q' } else { 'q' },
        0x11 => if shift { 'W' } else { 'w' },
        0x12 => if shift { 'E' } else { 'e' },
        0x13 => if shift { 'R' } else { 'r' },
        0x14 => if shift { 'T' } else { 't' },
        0x15 => if shift { 'Y' } else { 'y' },
        0x16 => if shift { 'U' } else { 'u' },
        0x17 => if shift { 'I' } else { 'i' },
        0x18 => if shift { 'O' } else { 'o' },
        0x19 => if shift { 'P' } else { 'p' },

        // a-l
        0x1E => if shift { 'A' } else { 'a' },
        0x1F => if shift { 'S' } else { 's' },
        0x20 => if shift { 'D' } else { 'd' },
        0x21 => if shift { 'F' } else { 'f' },
        0x22 => if shift { 'G' } else { 'g' },
        0x23 => if shift { 'H' } else { 'h' },
        0x24 => if shift { 'J' } else { 'j' },
        0x25 => if shift { 'K' } else { 'k' },
        0x26 => if shift { 'L' } else { 'l' },

        // z-m
        0x2C => if shift { 'Z' } else { 'z' },
        0x2D => if shift { 'X' } else { 'x' },
        0x2E => if shift { 'C' } else { 'c' },
        0x2F => if shift { 'V' } else { 'v' },
        0x30 => if shift { 'B' } else { 'b' },
        0x31 => if shift { 'N' } else { 'n' },
        0x32 => if shift { 'M' } else { 'm' },

        // interpunkcja
        0x0C => if shift { '_' } else { '-' },
        0x0D => if shift { '+' } else { '=' },
        0x1A => if shift { '{' } else { '[' },
        0x1B => if shift { '}' } else { ']' },
        0x27 => if shift { ':' } else { ';' },
        0x28 => if shift { '"' } else { '\'' },
        0x29 => if shift { '~' } else { '`' },
        0x2B => if shift { '|' } else { '\\' },
        0x33 => if shift { '<' } else { ',' },
        0x34 => if shift { '>' } else { '.' },
        0x35 => if shift { '?' } else { '/' },

        _ => return None,
    };

    Some(c)
}

/// Wywoływane z przerwania klawiatury. Śledzi Shift i wrzuca znaki do bufora.
///
/// W trybie edytora (`set_keycode_capture(true)`) scancode-y trafiają do bufora
/// kodów edytora zamiast do bufora tekstu terminala.
pub fn push_scancode(code: u8) {
    match code {
        0x2A | 0x36 => SHIFT.store(true, Ordering::Relaxed),
        0xAA | 0xB6 => SHIFT.store(false, Ordering::Relaxed),
        _ => {
            if CAPTURE_KEYCODE.load(Ordering::Relaxed) {
                if let Some(k) = scancode_to_keycode(code) {
                    keycode_push(k);
                }
            } else if let Some(c) = scancode_to_char(code) {
                kbuf_push(c as u8);
            }
        }
    }
}

/// Scancode set 1 → kod edytora (kernel/src/editor/editor.h, EDK_* = 0x100+).
fn scancode_to_keycode(code: u8) -> Option<u32> {
    match code {
        0x1C => Some(0x100), // EDK_ENTER
        0x0E => Some(0x101), // EDK_BACKSPACE
        0x01 => Some(0x102), // EDK_ESC
        0x4D => Some(0x103), // EDK_RIGHT
        0x4B => Some(0x104), // EDK_LEFT
        0x50 => Some(0x105), // EDK_DOWN
        0x48 => Some(0x106), // EDK_UP
        0x47 => Some(0x107), // EDK_HOME
        0x4F => Some(0x108), // EDK_END
        0x53 => Some(0x109), // EDK_DELETE
        0x0F => Some(0x10A), // EDK_TAB
        0x3F => Some(0x110), // EDK_F5
        0x42 => Some(0x111), // EDK_F8
        _ => scancode_to_char(code).map(|c| c as u32),
    }
}

fn keycode_push(k: u32) {
    let tail = KCODE_TAIL.load(Ordering::Relaxed);
    let next = (tail + 1) % KCODE_SIZE;

    if next == KCODE_HEAD.load(Ordering::Acquire) {
        return; // pełny — porzuć
    }

    KCODEBUF[tail].store(k, Ordering::Relaxed);
    KCODE_TAIL.store(next, Ordering::Release);
}

/// Odczyt kodu edytora — używany przez `k_input_keycode()` (kstd_glue.rs).
pub fn pop_keycode() -> Option<u32> {
    let head = KCODE_HEAD.load(Ordering::Relaxed);

    if head == KCODE_TAIL.load(Ordering::Acquire) {
        return None;
    }

    let k = KCODEBUF[head].load(Ordering::Relaxed);
    KCODE_HEAD.store((head + 1) % KCODE_SIZE, Ordering::Release);
    Some(k)
}

/// Przekierowuje scancode-y PS/2 do bufora edytora (true = edytor aktywny).
pub fn set_keycode_capture(on: bool) {
    CAPTURE_KEYCODE.store(on, Ordering::Relaxed);
}

/* ------------------------------------------------------------------ */
/* Stan terminala                                                     */
/* ------------------------------------------------------------------ */

const MAX_LINE: usize = 120;
const PROMPT: &str = "#$-=>";
const CURSOR: &str = "_";
const CONSOLE_COLOR: Color = Color::Green;
const INPUT_COLOR: Color = Color::LightBlue;

static LINE: [AtomicU8; MAX_LINE] = [const { AtomicU8::new(0) }; MAX_LINE];
static LINE_LEN: AtomicUsize = AtomicUsize::new(0);
static LINE_START_COL: AtomicUsize = AtomicUsize::new(0);

// Current working directory (first block of its entry table).
static CURRENT_DIR: AtomicU32 = AtomicU32::new(crate::fs::tfs::ROOT_DIR);
static NET_TIME_MS: AtomicU64 = AtomicU64::new(0);

fn line_as_str(buf: &mut [u8]) -> &str {
    let len = LINE_LEN.load(Ordering::Relaxed);
    for i in 0..len {
        buf[i] = LINE[i].load(Ordering::Relaxed);
    }
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

fn line_clear() {
    LINE_LEN.store(0, Ordering::Relaxed);
}

fn draw_prompt() {
    let start_col = {
        let mut w = WRITER.lock();
        let col = w.column();
        w.set_color(Color::White);
        w.write_string(PROMPT);
        w.set_color(Color::Red);
        w.write_string(CURSOR);
        w.set_color(CONSOLE_COLOR);
        col
    };
    LINE_START_COL.store(start_col, Ordering::Relaxed);

    crate::serial::write_str(PROMPT);
    crate::serial::write_str(CURSOR);
    crate::gfx::refresh();
}

/// Rysuje prompt + wpisany tekst (niebieski) + kursor (czerwony).
fn redraw_line() {
    let mut line = [0u8; MAX_LINE];
    let text = line_as_str(&mut line);
    let start_col = LINE_START_COL.load(Ordering::Relaxed);

    {
        let mut w = WRITER.lock();
        w.set_column(start_col);
        w.clear_to_end();
        w.set_color(Color::White);
        w.write_string(PROMPT);
        w.set_color(INPUT_COLOR);
        w.write_string(text);
        w.set_color(Color::Red);
        w.write_string(CURSOR);
        w.set_color(CONSOLE_COLOR);
    }

    crate::gfx::refresh();
}

fn handle_char(c: char) {
    match c {
        '\n' => {
            let mut line = [0u8; MAX_LINE];
            let text = line_as_str(&mut line);
            let owned = alloc::string::String::from(text);
            let start_col = LINE_START_COL.load(Ordering::Relaxed);

            // usuń kursor, zostaw prompt + tekst, potem nowa linia
            {
                let mut w = WRITER.lock();
                w.set_column(start_col);
                w.clear_to_end();
                w.set_color(Color::White);
                w.write_string(PROMPT);
                w.set_color(INPUT_COLOR);
                w.write_string(text);
                w.set_color(CONSOLE_COLOR);
                w.write_byte(b'\n');
            }
            crate::serial::write_str("\n");
            crate::gfx::refresh();

            execute(&owned);
            line_clear();
            draw_prompt();
        }
        '\x08' => {
            if LINE_LEN.load(Ordering::Relaxed) > 0 {
                LINE_LEN.fetch_sub(1, Ordering::Relaxed);
                redraw_line();
                crate::serial::write_str("\x08 \x08");
            }
        }
        c if (c as u32) >= 0x20 && (c as u32) < 0x7F => {
            let len = LINE_LEN.load(Ordering::Relaxed);
            if len < MAX_LINE {
                LINE[len].store(c as u8, Ordering::Relaxed);
                LINE_LEN.store(len + 1, Ordering::Relaxed);
                redraw_line();
                let mut b = [0u8; 4];
                crate::serial::write_str(c.encode_utf8(&mut b));
            }
        }
        _ => {}
    }
}

/* ------------------------------------------------------------------ */
/* Komendy                                                            */
/* ------------------------------------------------------------------ */

fn split_once_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

/// Parses a resolution specifier of the form `WxH` or `W:H` (e.g. `1920:1080`).
fn parse_resolution(s: &str) -> Option<(u32, u32)> {
    let sep = s.find(|c| c == 'x' || c == 'X' || c == ':')?;
    let (w, h) = (&s[..sep], &s[sep + 1..]);
    let w: u32 = w.trim().parse().ok()?;
    let h: u32 = h.trim().parse().ok()?;
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return None;
    }
    Some((w, h))
}

fn dev() -> Option<&'static dyn BlockDevice> {
    crate::fs::root_device()
}

fn execute(line: &str) {
    let (cmd, rest) = split_once_space(line);

    match cmd {
        "help" => {
            crate::println!("commands: help clear echo info ping ls cd mkdir format write read rm edit res poweroff reboot");
            crate::println!("  write <name> <text...>  write a text file to disk");
            crate::println!("  read  <name>            read a file from disk");
            crate::println!("  rm    <name>            remove a file or empty folder");
            crate::println!("  edit  <file>            open a file in the kernel editor (ESC quits)");
            crate::println!("  ls                      list the current folder");
            crate::println!("  mkdir <name>            create a folder");
            crate::println!("  cd    <name|/>          change folder ( / = root )");
            crate::println!("  res   <WxH|W:H>         change resolution (e.g. res 1920:1080)");
            crate::println!("  ping  <IPv4>            send one ICMP Echo Request");
            crate::println!("  poweroff                power off the machine");
            crate::println!("  reboot                  reboot the machine");
            crate::println!("  format                  format the disk (TFS)");
        }
        "clear" => {
            WRITER.lock().clear_screen();
            crate::serial::write_str("\x1b[2J\x1b[H");
        }
        "echo" => {
            crate::println!("{}", rest);
        }
        "info" => {
            crate::println!("TrangorgeOS terminal");
            crate::println!(
                "  RAM total: {} MiB, free: {} MiB",
                unsafe { crate::mm::ffi::mm_total_ram() } / 1024 / 1024,
                unsafe { crate::mm::ffi::mm_free_ram() } / 1024 / 1024
            );
            crate::println!("  CPUs: {}", crate::cpu::total_cpus());
        }
        "ping" => {
            let address = match crate::nic::parse_ipv4(rest.trim()) {
                Some(value) => value,
                None => {
                    crate::println!("usage: ping <IPv4>");
                    crate::gfx::refresh();
                    return;
                }
            };
            let now_ms = NET_TIME_MS.load(Ordering::Relaxed);
            match crate::nic::runtime::start_ping(address, now_ms) {
                Ok(result) => print_ping_result(result),
                Err(error) => crate::println!("ping failed: {:?}", error),
            }
        }
        "ls" => {
            let dir = CURRENT_DIR.load(Ordering::Relaxed);
            match dev() {
                Some(d) => {
                    let mut sink = Sink;
                    let _ = crate::fs::tfs::list_dir(d, dir, &mut sink);
                }
                None => crate::println!("no disk"),
            }
        }
        "cd" => {
            let (name, _) = split_once_space(rest);
            if name.is_empty() {
                crate::println!("usage: cd <name|/>");
            } else if name == "/" {
                CURRENT_DIR.store(crate::fs::tfs::ROOT_DIR, Ordering::Relaxed);
            } else {
                let dir = CURRENT_DIR.load(Ordering::Relaxed);
                match dev() {
                    Some(d) => match crate::fs::tfs::find_dir(d, dir, name) {
                        Ok(next) => CURRENT_DIR.store(next, Ordering::Relaxed),
                        Err(e) => crate::println!("cd failed: {:?}", e),
                    },
                    None => crate::println!("no disk"),
                }
            }
        }
        "mkdir" => {
            let (name, _) = split_once_space(rest);
            if name.is_empty() {
                crate::println!("usage: mkdir <name>");
            } else {
                let dir = CURRENT_DIR.load(Ordering::Relaxed);
                match dev() {
                    Some(d) => match crate::fs::tfs::mkdir(d, dir, name) {
                        Ok(()) => crate::println!("created folder {}", name),
                        Err(e) => crate::println!("mkdir failed: {:?}", e),
                    },
                    None => crate::println!("no disk"),
                }
            }
        }
        "res" => {
            let (arg, _) = split_once_space(rest);
            let ok = match arg {
                "640" => crate::gfx::set_resolution(crate::gfx::vga::VideoMode::Mode12h),
                "320" => crate::gfx::set_resolution(crate::gfx::vga::VideoMode::Mode13h),
                _ => match parse_resolution(arg) {
                    Some((w, h)) => crate::gfx::set_resolution_w_h(w, h),
                    None => false,
                },
            };
            if ok {
                let (w, h) = crate::gfx::current_resolution();
                crate::println!("resolution: {}x{}", w, h);
            } else {
                crate::println!("usage: res <320|640|WxH|W:H>  (e.g. res 1920:1080)");
            }
        }
        "poweroff" => {
            crate::println!("powering off...");
            if !crate::cpu::poweroff() {
                crate::println!("poweroff failed (ACPI not available)");
            }
        }
        "reboot" => {
            crate::println!("rebooting...");
            crate::cpu::reboot();
        }
        "format" => match dev() {
            Some(d) => match crate::fs::tfs::format(d) {
                Ok(()) => crate::println!("disk formatted (TFS)"),
                Err(e) => crate::println!("format failed: {:?}", e),
            },
            None => crate::println!("no disk"),
        },
        "write" => {
            let (name, text) = split_once_space(rest);
            if name.is_empty() {
                crate::println!("usage: write <name> <text...>");
            } else {
                let dir = CURRENT_DIR.load(Ordering::Relaxed);
                match dev() {
                    Some(d) => match crate::fs::tfs::write_file(d, dir, name, text.as_bytes()) {
                        Ok(()) => crate::println!("wrote {} ({} bytes)", name, text.len()),
                        Err(e) => crate::println!("write failed: {:?}", e),
                    },
                    None => crate::println!("no disk"),
                }
            }
        }
        "read" => {
            let (name, _) = split_once_space(rest);
            if name.is_empty() {
                crate::println!("usage: read <name>");
            } else {
                let dir = CURRENT_DIR.load(Ordering::Relaxed);
                match dev() {
                    Some(d) => match crate::fs::tfs::read_file(d, dir, name) {
                        Ok(data) => {
                            let s = core::str::from_utf8(&data).unwrap_or("(binary)");
                            crate::println!("{}", s);
                        }
                        Err(e) => crate::println!("read failed: {:?}", e),
                    },
                    None => crate::println!("no disk"),
                }
            }
        }
        "edit" => {
            let (name, _) = split_once_space(rest);

            if name.is_empty() {
                crate::println!("usage: edit <file>  (ESC wyjdzie z edytora)");
            } else {
                let mut path = [0u8; 128];
                let n = name.len().min(path.len() - 1);
                path[..n].copy_from_slice(&name.as_bytes()[..n]);
                path[n] = 0;

                set_keycode_capture(true);

                let rc = unsafe { editor_run(path.as_ptr()) };

                set_keycode_capture(false);
                WRITER.lock().clear_screen();
                crate::gfx::refresh();

                match rc {
                    0 => crate::println!("edytor: powrót do terminala"),
                    -1 => crate::println!("edytor: brak framebuffera HDMI (gfx nieaktywne)"),
                    other => crate::println!("edytor: kod wyjścia {}", other),
                }
            }
        }
        "rm" => {
            let (name, _) = split_once_space(rest);
            if name.is_empty() {
                crate::println!("usage: rm <name>");
            } else {
                let dir = CURRENT_DIR.load(Ordering::Relaxed);
                match dev() {
                    Some(d) => match crate::fs::tfs::remove(d, dir, name) {
                        Ok(()) => crate::println!("removed {}", name),
                        Err(e) => crate::println!("rm failed: {:?}", e),
                    },
                    None => crate::println!("no disk"),
                }
            }
        }
        "" => {}
        other => {
            crate::println!("unknown command: {} (try 'help')", other);
        }
    }

    crate::gfx::refresh();
}

fn print_ping_result(result: crate::nic::PingResult) {
    match result {
        crate::nic::PingResult::ArpRequestSent => crate::println!("ping: ARP request sent"),
        crate::nic::PingResult::EchoRequestSent => crate::println!("ping: ICMP Echo Request sent"),
        crate::nic::PingResult::EchoReply { source, sequence } => crate::println!(
            "ping: ICMP Echo Reply from {}.{}.{}.{} seq={}",
            source.0[0],
            source.0[1],
            source.0[2],
            source.0[3],
            sequence
        ),
        crate::nic::PingResult::Waiting => {}
    }
}

fn poll_network() {
    let now_ms = NET_TIME_MS.fetch_add(10, Ordering::Relaxed).wrapping_add(10);
    match crate::nic::runtime::poll(now_ms) {
        Ok(Some(result)) => print_ping_result(result),
        Ok(None) => {}
        Err(error) => crate::println!("ping failed: {:?}", error),
    }
}

/// Sink for `tfs::list_dir` — prints via `println!`.
struct Sink;

impl core::fmt::Write for Sink {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::print!("{}", s);
        Ok(())
    }
}

/* ------------------------------------------------------------------ */
/* Main loop                                                          */
/* ------------------------------------------------------------------ */

pub fn init() {
    WRITER.lock().set_color(CONSOLE_COLOR);
    draw_prompt();
}

pub fn run() -> ! {
    loop {
        poll_network();
        let mut handled = false;
        while let Some(b) = kbuf_pop() {
            handle_char(b as char);
            handled = true;
        }
        if !handled {
            x86_64::instructions::hlt();
        }
    }
}

