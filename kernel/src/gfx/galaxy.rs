use super::framebuffer::{Framebuffer, rgb};

/* Kąt pasma (Q16) */
const COS_A: i64 = 59636;
const SIN_A: i64 = 27525;

fn hash2(x: i64, y: i64) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

/* value noise 0..255, wejście Q8 */
fn vnoise(xq: i64, yq: i64) -> i64 {
    let xi = xq >> 8;
    let yi = yq >> 8;
    let fx = xq & 255;
    let fy = yq & 255;

    let sx = (fx * fx * (768 - 2 * fx)) >> 16;
    let sy = (fy * fy * (768 - 2 * fy)) >> 16;

    let h00 = (hash2(xi, yi) & 1023) as i64;
    let h10 = (hash2(xi + 1, yi) & 1023) as i64;
    let h01 = (hash2(xi, yi + 1) & 1023) as i64;
    let h11 = (hash2(xi + 1, yi + 1) & 1023) as i64;

    let top = h00 + ((h10 - h00) * sx >> 8);
    let bot = h01 + ((h11 - h01) * sx >> 8);

    (top + ((bot - top) * sy >> 8)) >> 2
}

/* 3 oktawy — rozbija kwadraty na organiczną strukturę */
fn fbm(xq: i64, yq: i64) -> i64 {
    let a = vnoise(xq, yq);
    let b = vnoise(xq * 2 + 12345, yq * 2 + 12345);
    let c = vnoise(xq * 4 + 67890, yq * 4 + 67890);

    (a * 144 + b * 72 + c * 40) >> 8
}

/* natężenie pasma 0..256 */
fn band_fall(x: i64, y: i64, w: i64, h: i64) -> i64 {
    let dx = x - w / 2;
    let dy = y - h / 2;

    let dn = (-dx * SIN_A + dy * COS_A) >> 16;
    let bw = h * 38 / 100;
    let ad = if dn < 0 { -dn } else { dn };

    if ad >= bw {
        return 0;
    }

    let q = bw * bw - ad * ad;

    (q / (bw * bw / 256)).min(256)
}

fn px_add(fb: &mut Framebuffer, x: i64, y: i64,
          r: i64, g: i64, b: i64, m: i64)
{
    if x < 0 || y < 0 || x >= fb.width as i64 || y >= fb.height as i64 {
        return;
    }

    fb.add(x as usize, y as usize,
           (r * m >> 8).min(255) as u32,
           (g * m >> 8).min(255) as u32,
           (b * m >> 8).min(255) as u32);
}

pub fn render(fb: &mut Framebuffer, t: u32) {
    let m = t.min(256) as i64;
    let w = fb.width as i64;
    let h = fb.height as i64;

    /* ---------- 1. tło + mgławica (fbm + dither = bez kwadratów) ---------- */

    for y in 0..fb.height {
        for x in 0..fb.width {
            let xi = x as i64;
            let yi = y as i64;

            let mut r = 3 + yi * 4 / h;
            let mut g = 3 + yi * 4 / h;
            let mut b = 8 + yi * 8 / h;

            let fall = band_fall(xi, yi, w, h);

            if fall > 0 {
                let n_f = fbm((xi << 8) / 26, (yi << 8) / 26);
                let n_b = vnoise((xi << 8) / 110 + 7777,
                                 (yi << 8) / 110 + 7777);

                let cloud = (n_f * 3 + n_b * 2) / 5;

                /* dither ±8 rozbija banding i resztki siatki */
                let dith = ((hash2(xi, yi) & 15) as i64) - 8;

                let mut i = (fall * cloud >> 8) + dith;

                if i < 0 { i = 0; }
                if i > 255 { i = 255; }

                /* kolor: duża skala blendowana z drobną -> miękkie strefy */
                let mix = (n_b * 3 + n_f * 2) / 5;

                let cr = 35 + (mix * 95 >> 8) + (n_f * 25 >> 8);
                let cg = 25 + (mix * 45 >> 8);
                let cb = 120 + (mix * 100 >> 8);

                r += i * cr >> 8;
                g += i * cg >> 8;
                b += i * cb >> 8;
            }

            let dvx = xi - w / 2;
            let dvy = yi - h / 2;
            let dv2 = dvx * dvx + dvy * dvy;
            let vr2 = (w * w / 4 + h * h / 4).max(1);
            let vign = 256 - (dv2 * 70 / vr2).min(70);

            r = r * vign >> 8;
            g = g * vign >> 8;
            b = b * vign >> 8;

            fb.set(x, y, rgb(
                (r * m >> 8).min(255) as u32,
                (g * m >> 8).min(255) as u32,
                (b * m >> 8).min(255) as u32,
            ));
        }
    }

    /* ---------- 2. gwiazdy główne (gęstsze w paśmie) ---------- */

    let cell: i64 = 6;
    let cw = (w + cell - 1) / cell;
    let ch = (h + cell - 1) / cell;

    for cy in 0..ch {
        for cx in 0..cw {
            let hh = hash2(cx * 31 + 7, cy * 17 + 3);

            let midx = cx * cell + cell / 2;
            let midy = cy * cell + cell / 2;
            let fall = band_fall(midx, midy, w, h);

            /* mocniejszy boost w paśmie niż wcześniej */
            let thresh = 1600 + fall * 14;

            if i64::from(hh & 0xFFFF) > thresh {
                continue;
            }

            let jx = ((hh >> 4) % cell as u32) as i64;
            let jy = ((hh >> 12) % cell as u32) as i64;
            let sx = cx * cell + jx;
            let sy = cy * cell + jy;

            let br = 70 + ((hh >> 20) % 186) as i64;

            let (cr, cg, cb) = match (hh >> 8) & 3 {
                0 => (170, 200, 255),
                1 => (255, 220, 180),
                _ => (255, 255, 255),
            };

            px_add(fb, sx, sy,
                   cr * br / 255, cg * br / 255, cb * br / 255, m);

            if br > 230 {
                px_add(fb, sx + 1, sy, cr / 3, cg / 3, cb / 3, m);
                px_add(fb, sx, sy + 1, cr / 3, cg / 3, cb / 3, m);
                px_add(fb, sx - 1, sy, cr / 4, cg / 4, cb / 4, m);
            }
        }
    }

    /* ---------- 3. DROBNA warstwa gwiazd TYLKO w paśmie ---------- */

    let cell2: i64 = 3;
    let cw2 = (w + cell2 - 1) / cell2;
    let ch2 = (h + cell2 - 1) / cell2;

    for cy in 0..ch2 {
        for cx in 0..cw2 {
            let hh = hash2(cx * 57 + 11, cy * 43 + 5);

            let midx = cx * cell2 + cell2 / 2;
            let midy = cy * cell2 + cell2 / 2;
            let fall = band_fall(midx, midy, w, h);

            /* sieje tylko tam, gdzie jest mgła */
            if fall < 40 {
                continue;
            }

            let t2 = (fall * 6) as u32;   /* 240..1536 z 65536 */

            if (hh & 0xFFFF) > t2 {
                continue;
            }

            let jx = ((hh >> 3) % cell2 as u32) as i64;
            let jy = ((hh >> 11) % cell2 as u32) as i64;

            let br = 30 + ((hh >> 19) % 110) as i64;

            let (cr, cg, cb) = if (hh >> 7) & 1 == 0 {
                (200, 210, 255)
            } else {
                (255, 255, 255)
            };

            px_add(fb, cx * cell2 + jx, cy * cell2 + jy,
                   cr * br / 255, cg * br / 255, cb * br / 255, m);
        }
    }

    /* ---------- 4. spadająca gwiazda ---------- */

    let x0 = w * 28 / 100;
    let y0 = h * 35 / 100;
    let x1 = w * 345 / 1000;
    let y1 = h * 255 / 1000;

    for s in 0..70i64 {
        let x = x0 + (x1 - x0) * s / 70;
        let y = y0 + (y1 - y0) * s / 70;
        let br = 230 - s * 2;

        px_add(fb, x, y, br, br, br.min(255), m);
    }
}