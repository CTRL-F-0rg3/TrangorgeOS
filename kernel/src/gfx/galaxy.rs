use super::framebuffer::{Framebuffer, rgb};

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

fn hash2(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

struct Blob {
    x: i64,
    y: i64,
    r2: i64,
    cr: i64,
    cg: i64,
    cb: i64,
    inten: i64,
}

// A more saturated, higher-contrast palette than before — magenta/teal/
// orange as a warm counterpoint to the rest of the cool colors, so the
// nebula does not look flat-purple but has distinct color zones.
const NEBULA_COLORS: [(i64, i64, i64); 7] = [
    (168, 32, 176),  // magenta
    (32, 88, 214),   // deep blue
    (24, 168, 176),  // teal
    (214, 64, 96),   // pink/red
    (48, 32, 150),   // indigo
    (128, 24, 190),  // violet
    (232, 128, 40),  // orange (warm counterpoint)
];

fn plot_soft(fb: &mut Framebuffer, x: i64, y: i64, t: u32, r: u32, g: u32, b: u32) {
    if x < 0 || y < 0 || x >= fb.width as i64 || y >= fb.height as i64 {
        return;
    }

    let m = t.min(256) as u64;

    fb.add(x as usize, y as usize,
           (r as u64 * m >> 8) as u32,
           (g as u64 * m >> 8) as u32,
           (b as u64 * m >> 8) as u32);
}

pub fn render(fb: &mut Framebuffer, t: u32) {
    let m = t.min(256) as i64;
    let h = fb.height as i64;
    let w = fb.width as i64;

    let mut rng = Rng(0x5EED_C0DE);

    let mut blobs = [
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
        Blob { x: 0, y: 0, r2: 1, cr: 0, cg: 0, cb: 0, inten: 0 },
    ];

    for i in 0..7 {
        let rad = h / 4 + rng.range(h as usize / 3) as i64;
        let c = NEBULA_COLORS[i];

        blobs[i] = Blob {
            x: rng.range(fb.width) as i64,
            y: rng.range(fb.height) as i64,
            r2: rad * rad,
            cr: c.0,
            cg: c.1,
            cb: c.2,
            inten: 70 + rng.range(80) as i64,
        };
    }

    let cx = w / 2;
    let cy = h / 2;
    // Promień do winiety (dla lekkiego przyciemnienia krawędzi, żeby
    // środek — z jądrem galaktyki — wybijał się głębią).
    let vign_r2 = (cx * cx + cy * cy).max(1);

    for y in 0..fb.height {
        for x in 0..fb.width {
            let mut ar = 0i64;
            let mut ag = 0i64;
            let mut ab = 0i64;

            for b in blobs.iter() {
                let dx = x as i64 - b.x;
                let dy = y as i64 - b.y;
                let d2 = dx * dx + dy * dy;

                if d2 >= b.r2 {
                    continue;
                }

                let f = b.r2 - d2;
                let f = (f * f) / (b.r2 * b.r2 / 256);

                let n = ((hash2(x as u32 >> 1, y as u32 >> 1) & 127) + 128) as i64;

                let c = (f * n >> 8) * b.inten >> 8;

                ar += c * b.cr >> 8;
                ag += c * b.cg >> 8;
                ab += c * b.cb >> 8;
            }

            // Winieta: lekkie przyciemnienie proporcjonalne do odległości
            // od środka, żeby krawędzie ekranu nie konkurowały z jądrem.
            let dvx = x as i64 - cx;
            let dvy = y as i64 - cy;
            let dv2 = dvx * dvx + dvy * dvy;
            let vign = (256 - (dv2 * 96 / vign_r2).min(96)).max(160);

            ar = ar * vign >> 8;
            ag = ag * vign >> 8;
            ab = ab * vign >> 8;

            let r = (ar * m >> 8).min(255) as u32;
            let g = (ag * m >> 8).min(255) as u32;
            let bch = (ab * m >> 8).min(255) as u32;

            fb.set(x, y, rgb(r, g, bch));
        }
    }

    let core_r = (fb.height.min(fb.width) as i64) / 5;

    let mut y = -core_r;
    while y <= core_r {
        let mut x = -core_r;
        while x <= core_r {
            let d2 = x * x + y * y;
            let r2 = core_r * core_r;
            let i = (255 * r2 / (d2 + r2 / 3)).min(255) as u32;

            // Jądro cieplejsze niż poprzednio (bardziej złociste niż białe).
            plot_soft(fb, cx + x, cy + y, t,
                      i, i * 200 / 255, i * 120 / 255);

            x += 2;
        }
        y += 2;
    }

    // Dwa ramiona spiralne jak wcześniej, ale gęstsze (więcej ziaren na
    // krok), żeby spirala była wyraźniej widoczna na tle mgławicy.
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

            let (r, g, b) = if step < 60 {
                (255u32, 200, 150)
            } else {
                (110, 160, 255)
            };

            let mut srng = Rng(0x9E3779B97F4A7C15 ^ step as u64 ^ arm);

            for _ in 0..14 {
                let ox = srng.range(7) as i64 - 3;
                let oy = srng.range(7) as i64 - 3;
                let dim = 40 + srng.range(140) as u32;

                plot_soft(fb, sx + ox, sy + oy, t,
                          r * dim / 255, g * dim / 255, b * dim / 255);
            }
        }
    }

    let mut srng = Rng(0x123456789ABCDEF);

    for _ in 0..700 {
        let x = srng.range(fb.width) as i64;
        let y = srng.range(fb.height) as i64;
        let kind = srng.range(10);
        let br = 100 + srng.range(155) as u32;

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