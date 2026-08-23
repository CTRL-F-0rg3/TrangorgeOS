#![no_std]
#![no_main]

use trangorgelibc as tr;

const BG: u32 = 0xFF0A0A12;
const FG: u32 = 0xFFD0D0D0;
const ACC: u32 = 0xFF4EC9B0;

static mut HIST: [[u8; 64]; 24] = [[0; 64]; 24];
static mut HN: usize = 0;

fn push_hist(line: &[u8]) {
    unsafe {
        for i in 0..23 {
            HIST[i] = HIST[i + 1];
        }

        HIST[23] = [0; 64];

        for i in 0..line.len().min(63) {
            HIST[23][i] = line[i];
        }

        HN = HN.min(23) + 1;
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let (w, h, stride) = match tr::ui_open() {
        Some(v) => v,
        None => {
            tr::log("terminal: ui_open failed");
            tr::exit(1);
        }
    };

    let mut line = [0u8; 64];
    let mut len = 0usize;

    loop {
        /* redraw */
        tr::ui_clear(stride, w, h, BG);

        tr::ui_text(stride, 8, 8, "TrangorgeOS terminal", ACC, w, h);
        tr::ui_text(stride, 8, 20, "-------------------", ACC, w, h);

        unsafe {
            let start = if HN > 20 { HN - 20 } else { 0 };

            for i in start..HN {
                let s = core::str::from_utf8(&HIST[i])
                    .unwrap_or("")
                    .split('\0')
                    .next()
                    .unwrap_or("");

                tr::ui_text(stride, 8, 36 + ((i - start) as i32) * 12,
                            s, FG, w, h);
            }
        }

        let y = h as i32 - 24;

        tr::ui_text(stride, 8, y, ">", ACC, w, h);

        let cur = core::str::from_utf8(&line[..len]).unwrap_or("");
        tr::ui_text(stride, 20, y, cur, FG, w, h);

        /* kursor */
        let cx = 20 + (len as i32) * 8;
        tr::ui_pixel(stride, cx, y, FG, w, h);
        tr::ui_pixel(stride, cx, y + 1, FG, w, h);
        tr::ui_pixel(stride, cx, y + 2, FG, w, h);

        /* input */
        if let Some(k) = tr::key() {
            match k {
                b'\n' => {
                    push_hist(&line[..len]);
                    line = [0; 64];
                    len = 0;
                }
                8 => {
                    if len > 0 {
                        len -= 1;
                    }
                }
                _ => {
                    if len < 63 && k >= 32 {
                        line[len] = k;
                        len += 1;
                    }
                }
            }
        }

        tr::yield_cpu();
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! {
    loop {}
}