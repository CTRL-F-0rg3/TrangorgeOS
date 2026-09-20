use crate::arch::x86_64::{outb, inb};

const COM1: u16 = 0x3F8;

pub fn init_serial() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB
        outb(COM1 + 0, 0x03); // 38400 baud
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

fn is_transmitter_empty() -> bool {
    unsafe { (inb(COM1 + 5) & 0x20) != 0 }
}

pub fn print_char(c: u8) {
    while !is_transmitter_empty() { core::hint::spin_loop(); }
    unsafe { outb(COM1, c); }
}

pub fn print_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' { print_char(b'\r'); }
        print_char(b);
    }
}