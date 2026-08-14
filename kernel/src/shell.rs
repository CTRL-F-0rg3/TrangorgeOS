use crate::keyboard::KeyEvent;
use crate::vga_buffer::Color;
use crate::{print, print_colored, println};
use spin::Mutex;

const LINE_CAPACITY: usize = 128;

struct LineBuffer {
    buf: [u8; LINE_CAPACITY],
    len: usize,
}

impl LineBuffer {
    const fn new() -> Self {
        LineBuffer {
            buf: [0; LINE_CAPACITY],
            len: 0,
        }
    }

    fn push(&mut self, c: char) -> bool {
        if self.len >= LINE_CAPACITY || !c.is_ascii() {
            return false;
        }
        self.buf[self.len] = c as u8;
        self.len += 1;
        true
    }

    fn pop(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }
        self.len -= 1;
        true
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

static LINE: Mutex<LineBuffer> = Mutex::new(LineBuffer::new());

crate::test_module!({
    for c in "hi".chars() {
        handle_key_event(KeyEvent::Char(c));
    }
    if LINE.lock().len != 2 {
        return Err("line buffer did not accumulate typed characters");
    }

    handle_key_event(KeyEvent::Backspace);
    if LINE.lock().len != 1 {
        return Err("backspace did not shrink the line buffer");
    }

    handle_key_event(KeyEvent::Enter);
    if LINE.lock().len != 0 {
        return Err("line buffer was not cleared after Enter");
    }

    Ok("shell line buffer accumulate/backspace/clear verified")
});

pub fn init() {
    println!();
    print_prompt();
}

fn print_prompt() {
    print_colored!(Color::White, "_-#@");
    print_colored!(Color::Magenta, ">");
    print_colored!(Color::Red, "_");
}

pub fn handle_key_event(event: KeyEvent) {
    match event {
        KeyEvent::Char(c) => {
            let mut line = LINE.lock();
            if line.push(c) {
                print_colored!(Color::Brown, "{}", c);
            }
        }
        KeyEvent::Backspace => {
            let mut line = LINE.lock();
            if line.pop() {
                crate::vga_buffer::backspace();
            }
        }
        KeyEvent::Enter => {
            println!();
            let mut line = LINE.lock();
            run_command(line.as_str());
            line.clear();
            print_prompt();
        }
        KeyEvent::Tab | KeyEvent::None => {}
    }
}

fn run_command(input: &str) {
    let trimmed = input.trim();
    match trimmed {
        "" => {}
        "help" => println!("available commands: help, meminfo, diskinfo, echo <text>"),
        "meminfo" => print_meminfo(),
        "diskinfo" => print_diskinfo(),
        cmd if cmd.starts_with("echo ") => println!("{}", &cmd[5..]),
        other => println!("unknown command: {}", other),
    }
}

fn print_meminfo() {
    use core::sync::atomic::Ordering;

    let in_use = crate::allocator::stats::bytes_in_use();
    let peak = crate::allocator::stats::PEAK_BYTES.load(Ordering::Relaxed);
    let allocations = crate::allocator::stats::ALLOCATIONS.load(Ordering::Relaxed);
    println!(
        "heap: {} B in use, {} B peak, {} allocations",
        in_use, peak, allocations
    );

    match crate::allocator::physical::stats() {
        Some((total, used, free)) => {
            let page = crate::allocator::config::PAGE_SIZE;
            println!(
                "physical RAM: {} KB used / {} KB total ({} KB free)",
                used * page / 1024,
                total * page / 1024,
                free * page / 1024
            );
        }
        None => println!("physical RAM: allocator not initialized"),
    }
}

fn print_diskinfo() {
    let mut registry = crate::fs::driver::REGISTRY.lock();
    let count = registry.len();
    if count == 0 {
        println!("no block devices registered");
        return;
    }
    for index in 0..count {
        if let Some(device) = registry.get(index) {
            let block_size = device.block_size();
            let block_count = device.block_count();
            let total_kb = block_size * block_count / 1024;
            println!(
                "device {}: {} B/block x {} blocks = {} KB",
                index, block_size, block_count, total_kb
            );
        }
    }
}
