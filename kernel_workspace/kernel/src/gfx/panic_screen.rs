use crate::vga_buffer::Color;
use crate::{print_colored, println};
use core::panic::PanicInfo;

const BG: (u32, u32, u32) = (26, 0, 0);

const RULE: &str =
    "================================================================================";


pub fn show(info: &PanicInfo) -> ! {

    crate::serial::print_args(format_args!(
        "\n\n================ KERNEL PANIC ================\n{}\n================================================\n",
        info
    ));

    unsafe {
        crate::vga_buffer::WRITER.force_unlock();
    }
    super::console::set_enabled(true);


    let painted =
        super::console::test_fill(BG.0, BG.1, BG.2) && super::console::resync_background();

    crate::vga_buffer::WRITER.lock().clear_screen();

    let cols = super::console::cols().max(1);
    let rule = &RULE[..RULE.len().min(cols)];

    print_colored!(Color::Yellow, "{}\n", rule);
    print_colored!(Color::LightRed, "  KERNEL PANIC - system halted\n");
    print_colored!(Color::Yellow, "{}\n", rule);
    println!();
    print_colored!(Color::White, "{}\n", info);
    println!();
    print_colored!(
        Color::DarkGray,
        "full transcript on COM1 (serial) - reset to continue\n"
    );

    if painted {
        super::refresh();
    }

    crate::hlt_loop();
}