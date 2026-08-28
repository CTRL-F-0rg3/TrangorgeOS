//! Full-screen kernel panic display.
//!
//! Before this module existed, the panic handler only did
//! `println!("{}", info)` — which writes into the (invisible) VGA text
//! buffer but never tells the graphical console to actually redraw the
//! framebuffer. Visually a panic looked exactly like a hang: whatever was
//! last on screen (e.g. "[mm] allocator initialized OK" over the galaxy)
//! just stayed there forever, with zero indication anything had gone
//! wrong or why. This module paints an unmistakable full-screen banner
//! and forces it to actually reach the framebuffer.
//!
//! Kept deliberately allocation-free (only `core::fmt::Write` into the
//! existing fixed-size VGA text buffer, no `Vec`/`String`) — a panic
//! triggered by a broken allocator must not itself depend on the
//! allocator to be able to report itself.

use crate::vga_buffer::Color;
use crate::{print_colored, println};
use core::panic::PanicInfo;

// Dark red backdrop: the whole screen reads as "something is wrong" on
// sight, before a single line of text has been read.
const BG: (u32, u32, u32) = (26, 0, 0);

const RULE: &str =
    "================================================================================";

/// Renders the panic screen and mirrors a clearly-delimited copy to the
/// serial port (COM1). Never returns — halts the CPU.
pub fn show(info: &PanicInfo) -> ! {
    // Always reaches the log, independent of VGA/gfx state, and independent
    // of anything below deadlocking or failing.
    crate::serial::print_args(format_args!(
        "\n\n================ KERNEL PANIC ================\n{}\n================================================\n",
        info
    ));

    // The code that panicked may have crashed while holding the VGA writer
    // lock, and the framebuffer console may currently be disabled by
    // whatever was running (e.g. the in-kernel editor toggles it off).
    // Force both back into a known state instead of risking a deadlock or
    // drawing to a console nobody will see.
    unsafe {
        crate::vga_buffer::WRITER.force_unlock();
    }
    super::console::set_enabled(true);

    // Only meaningful once gfx has an active framebuffer; both no-op safely
    // (return false) if a panic happens before/without gfx::init() ever
    // succeeding, so this can never itself panic.
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