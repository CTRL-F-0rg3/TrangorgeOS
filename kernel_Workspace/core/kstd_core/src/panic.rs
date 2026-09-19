use core::panic::PanicInfo;
use kstd_io::traits::Write;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = kstd_io::Serial;
    let mut vga = kstd_io::VgaConsole;

    let _ = serial.write_str("\n!!! KERNEL PANIC !!!\n");
    let _ = vga.write_str("\n!!! KERNEL PANIC !!!\n");

    if let Some(location) = info.location() {
        let _ = core::writeln!(serial, "At {}:{}:{}", location.file(), location.line(), location.column());
        let _ = core::writeln!(vga, "At {}:{}:{}", location.file(), location.line(), location.column());
    }

    if let Some(message) = info.message() {
        let _ = core::writeln!(serial, "{}", message);
        let _ = core::writeln!(vga, "{}", message);
    }

    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}