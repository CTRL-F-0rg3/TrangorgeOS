use core::panic::PanicInfo;
use core::fmt::Write as FmtWrite;
use kstd_io::traits::Write;

fn write_all(serial: &mut kstd_io::Serial, vga: &mut kstd_io::VgaConsole, s: &str) {
    let _ = serial.write_str(s);
    let _ = vga.write_str(s);
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    let mut serial = kstd_io::Serial;
    let mut vga = kstd_io::VgaConsole;

    write_all(&mut serial, &mut vga, "\n!!! KERNEL PANIC !!!\n");

    if let Some(location) = info.location() {
        let _ = core::write!(FmtWriter(&mut serial), "At {}:{}:{}\n", location.file(), location.line(), location.column());
        let _ = core::write!(FmtWriter(&mut vga), "At {}:{}:{}\n", location.file(), location.line(), location.column());
    }

    {
        let message = info.message();
        let msg = heapless_msg(message);
        write_all(&mut serial, &mut vga, msg.as_str());
    }


    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

struct FmtWriter<'a, W: Write>(&'a mut W);

impl<'a, W: Write> core::fmt::Write for FmtWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = self.0.write_str(s);
        Ok(())
    }
}

// Render the panic message into a fixed buffer without using alloc.
// Note: kstd_io::Serial already truncates to 4096 bytes internally, we pass &str directly.
fn heapless_msg(message: core::panic::PanicMessage) -> kstd_data::string::String {
    use core::fmt::Write as _;
    let mut s = kstd_data::string::String::new();
    let _ = core::write!(FmtMsg(&mut s), "{}", message);
    s
}

struct FmtMsg<'a>(&'a mut kstd_data::string::String);

impl<'a> core::fmt::Write for FmtMsg<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.push_str(s);
        Ok(())
    }
}
