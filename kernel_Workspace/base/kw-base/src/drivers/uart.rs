use crate::arch::x86_64::{inb, outb};
use kstd_io::traits::Write;

pub struct Uart16550 {
    port: u16,
}

impl Uart16550 {
    pub const COM1: u16 = 0x3F8;

    pub fn new(port: u16) -> Self {
        let mut uart = Self { port };
        uart.init();
        uart
    }

    fn init(&mut self) {
        unsafe {
            outb(self.port + 1, 0x00); // Disable interrupts
            outb(self.port + 3, 0x80); // Enable DLAB
            outb(self.port + 0, 0x03); // 38400 baud
            outb(self.port + 1, 0x00);
            outb(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO
            outb(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    fn is_transmitter_empty(&self) -> bool {
        unsafe { (inb(self.port + 5) & 0x20) != 0 }
    }

    pub fn write_byte(&mut self, b: u8) {
        while !self.is_transmitter_empty() { core::hint::spin_loop(); }
        unsafe { outb(self.port, b); }
    }
}

impl Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        for b in s.bytes() {
            if b == b'\n' { self.write_byte(b'\r'); }
            self.write_byte(b);
        }
        Ok(())
    }
}