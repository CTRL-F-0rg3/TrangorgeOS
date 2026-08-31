use core::fmt;

#[cfg(target_arch = "x86_64")]
mod x86_impl {
    use x86_64::instructions::port::Port;

    const COM1: u16 = 0x3F8;

    pub fn init() {
        let mut d0 = Port::<u8>::new(COM1);
        let mut d1 = Port::<u8>::new(COM1 + 1);
        let mut d2 = Port::<u8>::new(COM1 + 2);
        let mut d3 = Port::<u8>::new(COM1 + 3);
        let mut d4 = Port::<u8>::new(COM1 + 4);

        unsafe {
            d1.write(0x00);
            d3.write(0x80); 
            d0.write(0x03);
            d1.write(0x00);
            d3.write(0x03); 
            d2.write(0xC7);
            d4.write(0x0B);
        }
    }

    fn tx_empty() -> bool {
        let mut lsr = Port::<u8>::new(COM1 + 5);
        unsafe { lsr.read() & 0x20 != 0 }
    }

    pub fn write_byte(byte: u8) {
        let mut data = Port::<u8>::new(COM1);
        while !tx_empty() {}
        unsafe {
            data.write(byte);
        }
    }
}

#[cfg(target_arch = "riscv64")]
mod rv_impl {
    const BASE: usize = 0x1000_0000; 
    const THR: usize = 0; 
    const LSR: usize = 5; 

    pub fn init() {
    }

    fn tx_empty() -> bool {
        unsafe { core::ptr::read_volatile((BASE + LSR) as *const u8) & 0x20 != 0 }
    }

    pub fn write_byte(byte: u8) {
        while !tx_empty() {}
        unsafe { core::ptr::write_volatile((BASE + THR) as *mut u8, byte) }
    }
}

pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86_impl::init();
    #[cfg(target_arch = "riscv64")]
    rv_impl::init();
}

pub fn write_byte(byte: u8) {
    #[cfg(target_arch = "x86_64")]
    x86_impl::write_byte(byte);
    #[cfg(target_arch = "riscv64")]
    rv_impl::write_byte(byte);
}

pub fn write_str(s: &str) {
    for b in s.bytes() {
        write_byte(b);
    }
}

struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

pub fn print_args(args: fmt::Arguments) {
    use core::fmt::Write;
    let _ = SerialWriter.write_fmt(args);
}
