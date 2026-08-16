use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, rgb};
use super::galaxy;
use crate::mm::ffi;

const VGA_TEXT_PHYS: u64 = 0xB8000;
const DIRECT_BASE: u64 = 0xFFFF888000000000;

pub const COLS: usize = 80;
pub const ROWS: usize = 25;

const VGA_PALETTE: [(u32, u32, u32); 16] = [
    (0,0,0),(0,0,170),(0,170,0),(0,170,170),
    (170,0,0),(170,0,170),(170,85,0),(170,170,170),
    (85,85,85),(85,85,255),(85,255,85),(85,255,255),
    (255,85,85),(255,85,255),(255,255,85),(255,255,255),
];

static mut FB: Option<Framebuffer> = None;
static mut CLEAN: *mut u32 = core::ptr::null_mut();

fn fb() -> &'static mut Framebuffer {
    unsafe { FB.as_mut().unwrap() }
}

fn delay() {
    for _ in 0..300_000 {
        core::hint::spin_loop();
    }
}

pub fn init(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
    let size = (stride * height * 4) as usize;

    let mut virt = 0u64;

    if !unsafe { ffi::vmm_map_device(fb_phys, size as usize, &mut virt) } {
        return false;
    }

    let fb = Framebuffer {
        ptr: virt as *mut u32,
        width: width as usize,
        height: height as usize,
        stride: stride as usize,
    };

    unsafe { FB = Some(fb) };

    for t in (0..=256).step_by(8) {
        galaxy::render(fb(), t);
        delay();
    }

    let mut buf = 0u64;

    if !unsafe { ffi::vmm_alloc(size, 1 << 0, &mut buf) } {
        return false;
    }

    unsafe {
        CLEAN = buf as *mut u32;

        core::ptr::copy_nonoverlapping(
            fb().ptr as *const u32,
            CLEAN,
            fb().stride * fb().height,
        );
    }

    refresh();

    true
}

fn mix(base: u32, c: (u32, u32, u32), a: u32) -> u32 {
    let br = (base >> 16) & 0xFF;
    let bg = (base >> 8) & 0xFF;
    let bb = base & 0xFF;

    let r = br + ((c.0.min(255).saturating_sub(br)) * a >> 8);
    let g = bg + ((c.1.min(255).saturating_sub(bg)) * a >> 8);
    let b = bb + ((c.2.min(255).saturating_sub(bb)) * a >> 8);

    rgb(r, g, b)
}

pub fn refresh() {
    let (w, h, s) = {
        let f = fb();
        (f.width, f.height, f.stride)
    };

    unsafe {
        core::ptr::copy_nonoverlapping(CLEAN, fb().ptr, s * h);
    }

    let vga = (DIRECT_BASE + VGA_TEXT_PHYS) as *const u16;

    for row in 0..ROWS {
        for col in 0..COLS {
            let cell = unsafe { *vga.add(row * COLS + col) };

            let ch = (cell & 0xFF) as u8;
            let attr = (cell >> 8) as u8;

            let fg = VGA_PALETTE[(attr & 0x0F) as usize];
            let bg = VGA_PALETTE[(attr >> 4) as usize];

            let glyph = if (0x20..=0x7E).contains(&ch) {
                FONT8X8[(ch - 0x20) as usize]
            } else {
                FONT8X8[('?' as u8 - 0x20) as usize]
            };

            for gy in 0..8 {
                let bits = glyph[gy];

                for gx in 0..8 {
                    let px = col * 8 + gx;
                    let py = row * 8 + gy;

                    if px >= w || py >= h {
                        continue;
                    }

                    let base = fb().get(px, py);

                    let c = if bits & (0x80 >> gx) != 0 {
                        rgb(fg.0, fg.1, fg.2)
                    } else {
                        mix(base, bg, 110)
                    };

                    fb().set(px, py, c);
                }
            }
        }
    }
}