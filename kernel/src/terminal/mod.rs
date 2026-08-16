//! Interaktywny terminal + shell z obsługą plików na dysku (TFS).
//!
//! Prompt: `#$-=>` (biały) + `_` (czerwony kursor) + wpisywany tekst (niebieski).

use crate::fs::driver::block::BlockDevice;
use crate::vga_buffer::{Color, WRITER};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};

/* ------------------------------------------------------------------ */
/* Bufor klawiatury (SPSC ring buffer, IRQ-safe)                       */
/* ------------------------------------------------------------------ */

const KBUF_SIZE: usize = 256;
static KBUF: [AtomicU8; KBUF_SIZE] = [const { AtomicU8::new(0) }; KBUF_SIZE];
static KHEAD: AtomicUsize = AtomicUsize::new(0);
static KTAIL: AtomicUsize = AtomicUsize::new(0);
static SHIFT: AtomicBool = AtomicBool::new(false);

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
pub fn push_scancode(code: u8) {
    match code {
        0x2A | 0x36 => SHIFT.store(true, Ordering::Relaxed),
        0xAA | 0xB6 => SHIFT.store(false, Ordering::Relaxed),
        _ => {
            if let Some(c) = scancode_to_char(code) {
                kbuf_push(c as u8);
            }
        }
    }
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

fn dev() -> Option<&'static dyn BlockDevice> {
    crate::fs::root_device()
}

fn execute(line: &str) {
    let (cmd, rest) = split_once_space(line);

    match cmd {
        "help" => {
            crate::println!("commands: help clear echo info ls cd mkdir format write read rm res");
            crate::println!("  write <name> <text...>  write a text file to disk");
            crate::println!("  read  <name>            read a file from disk");
            crate::println!("  rm    <name>            remove a file or empty folder");
            crate::println!("  ls                      list the current folder");
            crate::println!("  mkdir <name>            create a folder");
            crate::println!("  cd    <name|/>          change folder ( / = root )");
            crate::println!("  res   <320|640>         change resolution");
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
                _ => false,
            };
            if ok {
                crate::println!("resolution: {}", crate::gfx::current_resolution());
            } else {
                crate::println!("usage: res <320|640>");
            }
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

