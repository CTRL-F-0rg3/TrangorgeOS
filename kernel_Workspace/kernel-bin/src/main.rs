#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);

    loop {}
}