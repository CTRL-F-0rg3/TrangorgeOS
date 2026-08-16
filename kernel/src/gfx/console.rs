use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, rgb};
use super::galaxy;
use crate::mm::ffi;

pub const COLS: usize = 80;
pub const ROWS: usize = 25;

const VGA_PALETTE: [(u32, u32, u32); 16] = [
    (0,0,0),(0,0,170),(0,170,0),(0,170,170),
    (170,0,0),(170,0,170),(170,85,0),(170,170,170),
    (85,85,85),(85,85,255),(85,255,85),(85,255,255),
    (255,85,85),(255,85,255),(255,255,85),(255,255,255),
];

static mut FB: Option<Framebuffer> = None;
static mut CLEAN: *mut u8 = core::ptr::null_mut();
static mut READY: bool = false;

fn fb() -> &'static mut Framebuffer {
    unsafe { FB.as_mut().unwrap() }
}

fn set_palette_rgb332() {
    use x86_64::instructions::port::Port;

    let mut idx_port = Port::<u8>::new(0x3C8);
    let mut data_port = Port::<u8>::new(0x3C9);

    unsafe {
        idx_port.write(0);

        for i in 0..256u32 {
            let r = ((i >> 5) & 0x7) * 9;
            let g = ((i >> 2) & 0x7) * 9;
            let b = (i & 0x3) * 21;

            data_port.write(r as u8);
            data_port.write(g as u8);
            data_port.write(b as u8);
        }
    }
}

fn delay() {
    for _ in 0..50_000 {
        core::hint::spin_loop();
    }
}

pub fn init(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
    if unsafe { READY } {
        return true;
    }

    // Zaokrąglamy do strony — vmm_map_device wymaga wyrównania, a okno
    // pamięci VGA w trybie 13h to 64 KiB (0xA0000..0xB0000).
    let size = ((stride * height) as usize + 0xFFF) & !0xFFF;

    let mut virt = 0u64;

    if !unsafe { ffi::vmm_map_device(fb_phys, size, &mut virt) } {
        return false;
    }

    set_palette_rgb332();

    let framebuffer = Framebuffer {
        ptr: virt as *mut u8,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    };

    unsafe { FB = Some(framebuffer) };

    for t in (0..=256).step_by(32) {
        galaxy::render(fb(), t);
        delay();
    }

    let mut buf = 0u64;

    if !unsafe { ffi::vmm_alloc(size, 1 << 0, &mut buf) } {
        return false;
    }

    unsafe {
        CLEAN = buf as *mut u8;

        core::ptr::copy_nonoverlapping(
            fb().ptr as *const u8,
            CLEAN,
            fb().stride * fb().height,
        );
    }

    unsafe { READY = true };

    refresh();

    true
}

pub fn refresh() {
    if !unsafe { READY } {
        return;
    }

    let (w, h, s) = {
        let f = fb();
        (f.width, f.height, f.stride)
    };

    unsafe {
        core::ptr::copy_nonoverlapping(CLEAN, fb().ptr, s * h);
    }

    for row in 0..ROWS {
        for col in 0..COLS {
            let (ch, attr) = crate::vga_buffer::text_cell(row, col);

            let fg = VGA_PALETTE[(attr & 0x0F) as usize];
            let bg = VGA_PALETTE[(attr >> 4) as usize];

            let glyph = if (0x20..=0x7E).contains(&ch) {
                FONT8X8[(ch - 0x20) as usize]
            } else {
                FONT8X8[('?' as u8 - 0x20) as usize]
            };

            // Font 8x8 renderujemy jako 4x8 (downscale 2:1 w poziomie),
            // żeby całe 80 kolumn zmieściło się w 320 px.
            for gy in 0..8 {
                let bits = glyph[gy];

                for gx in 0..4 {
                    let px = col * 4 + gx;
                    let py = row * 8 + gy;

                    if px >= w || py >= h {
                        continue;
                    }

                    let c = if bits & (0xC0 >> (gx * 2)) != 0 {
                        rgb(fg.0, fg.1, fg.2)
                    } else {
                        rgb(bg.0, bg.1, bg.2)
                    };

                    fb().set(px, py, c);
                }
            }
        }
    }
}

/// Debug: zrzut framebuffera jako ASCII (do diagnostyki orientacji tekstu).
pub fn debug_dump() {
    crate::serial::write_str("--- gfx framebuffer dump ---\n");

    for row in 0..ROWS {
        for col in 0..COLS {
            let mut bright = false;

            for gy in 0..8 {
                for gx in 0..4 {
                    let c = fb().get(col * 4 + gx, row * 8 + gy);

                    if (c & 0xFFFFFF) > 0x404040 {
                        bright = true;
                    }
                }
            }

            crate::serial::write_byte(if bright { b'#' } else { b'.' });
        }

        crate::serial::write_byte(b'\n');
    }

    crate::serial::write_str("--- end dump ---\n");
}