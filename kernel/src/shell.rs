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
        "help" => println!(
            "available commands: help, meminfo, diskinfo, format, ls, write <name> <text>, read <name>, echo <text>"
        ),
        "meminfo" => print_meminfo(),
        "diskinfo" => print_diskinfo(),
        "format" => cmd_format(),
        "ls" => cmd_ls(),
        cmd if cmd.starts_with("write ") => cmd_write(&cmd[6..]),
        cmd if cmd.starts_with("read ") => cmd_read(&cmd[5..]),
        cmd if cmd.starts_with("echo ") => println!("{}", &cmd[5..]),
        other => println!("unknown command: {}", other),
    }
}

fn cmd_format() {
    let mut registry = crate::fs::driver::REGISTRY.lock();
    let Some(device) = registry.get(0) else {
        println!("no block device registered");
        return;
    };

    if device.block_size() != 512 {
        println!("format: only 512-byte sector devices are supported right now");
        return;
    }

    let options = crate::fs::format::Fat32FormatOptions {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 1,
        num_fats: 1,
        total_sectors: device.block_count() as u32,
    };

    match crate::fs::format::format_fat32(device, &options) {
        Ok(()) => println!("formatted device 0 as FAT32"),
        Err(e) => println!("format failed: {:?}", e),
    }
}

fn cmd_ls() {
    let mut registry = crate::fs::driver::REGISTRY.lock();
    let Some(device) = registry.get(0) else {
        println!("no block device registered");
        return;
    };

    let bpb = match read_bpb(device) {
        Some(bpb) => bpb,
        None => return,
    };

    match crate::fs::fat32::list_directory(device, &bpb, bpb.root_cluster) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("(empty)");
            }
            for entry in entries {
                println!("{}  {} bytes", entry.name, entry.metadata.size_bytes);
            }
        }
        Err(e) => println!("ls failed: {:?}", e),
    }
}

fn cmd_write(args: &str) {
    let mut parts = args.splitn(2, ' ');
    let Some(name) = parts.next() else {
        println!("usage: write <name> <content>");
        return;
    };
    let content = parts.next().unwrap_or("");

    let mut registry = crate::fs::driver::REGISTRY.lock();
    let Some(device) = registry.get(0) else {
        println!("no block device registered");
        return;
    };

    let bpb = match read_bpb(device) {
        Some(bpb) => bpb,
        None => return,
    };

    match crate::fs::fat32::write_file(device, &bpb, bpb.root_cluster, name, content.as_bytes()) {
        Ok(()) => println!("wrote {} bytes to {}", content.len(), name),
        Err(e) => println!("write failed: {:?}", e),
    }
}

fn cmd_read(name: &str) {
    let name = name.trim();
    if name.is_empty() {
        println!("usage: read <name>");
        return;
    }

    let mut registry = crate::fs::driver::REGISTRY.lock();
    let Some(device) = registry.get(0) else {
        println!("no block device registered");
        return;
    };

    let bpb = match read_bpb(device) {
        Some(bpb) => bpb,
        None => return,
    };

    match crate::fs::fat32::find_entry(device, &bpb, bpb.root_cluster, name) {
        Ok(Some((cluster, metadata))) => {
            match crate::fs::fat32::read_file(device, &bpb, cluster, metadata.size_bytes) {
                Ok(data) => match core::str::from_utf8(&data) {
                    Ok(text) => println!("{}", text),
                    Err(_) => println!("(binary content, {} bytes)", data.len()),
                },
                Err(e) => println!("read failed: {:?}", e),
            }
        }
        Ok(None) => println!("file not found: {}", name),
        Err(e) => println!("error: {:?}", e),
    }
}

fn read_bpb(
    device: &mut (dyn crate::fs::disc::BlockDevice + Send),
) -> Option<crate::fs::fat32::Fat32BootSector> {
    let mut boot_sector_buf = [0u8; 512];
    if device.read_block(0, &mut boot_sector_buf).is_err() {
        println!("failed to read boot sector");
        return None;
    }
    match crate::fs::fat32::parse_boot_sector(&boot_sector_buf) {
        Ok(bpb) => Some(bpb),
        Err(_) => {
            println!("device 0 is not formatted as FAT32 - run 'format' first");
            None
        }
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
