use super::framebuffer::Framebuffer;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }
}

const COS: i64 = 65214;
const SIN: i64 = 6540;
const GROW: i64 = 66847;

fn plot_soft(fb: &mut Framebuffer, x: i64, y: i64, t: u32, r: u32, g: u32, b: u32) {
    if x < 0 || y < 0 || x >= fb.width as i64 || y >= fb.height as i64 {
        return;
    }

    let m = t.min(256);

    fb.add(x as usize, y as usize,
           (r as u64 * m as u64 >> 8) as u32,
           (g as u64 * m as u64 >> 8) as u32,
           (b as u64 * m as u64 >> 8) as u32);
}

pub fn render(fb: &mut Framebuffer, t: u32) {
    for y in 0..fb.height {
        for x in 0..fb.width {
            fb.set(x, y, 0xFF000000);
        }
    }

    let cx = fb.width as i64 / 2;
    let cy = fb.height as i64 / 2;

    let core_r = (fb.height.min(fb.width) as i64) / 5;

    let mut y = -core_r;
    while y <= core_r {
        let mut x = -core_r;
        while x <= core_r {
            let d2 = x * x + y * y;
            let r2 = core_r * core_r;
            let i = (255 * r2 / (d2 + r2 / 3)).min(255) as u32;

            plot_soft(fb, cx + x, cy + y, t,
                      i, (i as u64 * 220 / 255) as u32, (i as u64 * 160 / 255) as u32);

            x += 2;
        }
        y += 2;
    }

    for arm in 0..2u64 {
        let mut px: i64 = 1 << 16;
        let mut py: i64 = 0;

        if arm == 1 {
            px = -px;
        }

        let max_r = (fb.height.min(fb.width) as i64) * 46 / 100;

        for step in 0..240 {
            let nx = (px * COS - py * SIN) >> 16;
            let ny = (px * SIN + py * COS) >> 16;

            px = (nx * GROW) >> 16;
            py = (ny * GROW) >> 16;

            let sx = cx + (px * max_r >> 16);
            let sy = cy + (py * max_r >> 16);

            let warm = step < 60;

            let (r, g, b) = if warm {
                (255u32, 200, 150)
            } else {
                (90, 140, 255)
            };

            let mut rng = Rng(0x9E3779B97F4A7C15 ^ step as u64 ^ arm);

            for _ in 0..10 {
                let ox = rng.range(7) as i64 - 3;
                let oy = rng.range(7) as i64 - 3;
                let dim = 40 + rng.range(120) as u32;

                plot_soft(fb, sx + ox, sy + oy, t,
                          r * dim / 255, g * dim / 255, b * dim / 255);
            }
        }
    }

    let mut rng = Rng(0x123456789ABCDEF);

    for _ in 0..700 {
        let x = rng.range(fb.width) as i64;
        let y = rng.range(fb.height) as i64;
        let kind = rng.range(10);
        let br = 100 + rng.range(155) as u32;

        let (r, g, b) = match kind {
            0..=1 => (170, 200, 255),
            2..=3 => (255, 220, 180),
            _ => (255, 255, 255),
        };

        plot_soft(fb, x, y, t, r * br / 255, g * br / 255, b * br / 255);

        if br > 220 {
            plot_soft(fb, x + 1, y, t, r / 3, g / 3, b / 3);
            plot_soft(fb, x, y + 1, t, r / 3, g / 3, b / 3);
        }
    }
}