use super::framebuffer::{Framebuffer, rgb};

const P_COS: i64 = 59636;
const P_SIN: i64 = 27525;

fn hash2(x: i64, y: i64) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

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

fn fbm(xq: i64, yq: i64) -> i64 {
    let a = vnoise(xq, yq);
    let b = vnoise(xq * 2 + 12345, yq * 2 + 12345);
    let c = vnoise(xq * 4 + 67890, yq * 4 + 67890);

    (a * 144 + b * 72 + c * 40) >> 8
}

fn band_fall(x: i64, y: i64, w: i64, h: i64) -> i64 {
    let dx = x - w / 2;
    let dy = y - h / 2;

    let dn = (-dx * P_SIN + dy * P_COS) >> 16;
    let bw = h * 38 / 100;
    let ad = if dn < 0 { -dn } else { dn };

    if ad >= bw {
        return 0;
    }

    let q = bw * bw - ad * ad;

    (q / (bw * bw / 256)).min(256)
}


fn green_fall(x: i64, y: i64, w: i64, _h: i64) -> i64 {
    let n1 = vnoise((y << 8) / 160 + 31337, 4242);   
    let n2 = vnoise((y << 8) / 55 + 777, 1234);      

    let wave = ((n1 - 128) * (w / 10) + (n2 - 128) * (w / 26)) >> 8;

    let xc = w * 20 / 100 + wave;

    let bw = w * 11 / 100 + ((n2 * (w / 14)) >> 8);

    let dx = x - xc;
    let ad = if dx < 0 { -dx } else { dx };

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


    for y in 0..fb.height {
        for x in 0..fb.width {
            let xi = x as i64;
            let yi = y as i64;

            let mut r = 3 + yi * 4 / h;
            let mut g = 3 + yi * 4 / h;
            let mut b = 8 + yi * 8 / h;

            let dith = ((hash2(xi, yi) & 15) as i64) - 8;

            let fp = band_fall(xi, yi, w, h);
            let fg = green_fall(xi, yi, w, h);

            if fp > 0 {
                let n_f = fbm((xi << 8) / 26, (yi << 8) / 26);
                let n_b = vnoise((xi << 8) / 110 + 7777,
                                 (yi << 8) / 110 + 7777);

                let cloud = (n_f * 3 + n_b * 2) / 5;

                let mut i = (fp * cloud >> 8) + dith;
                if i < 0 { i = 0; }
                if i > 255 { i = 255; }

                let mix = (n_b * 3 + n_f * 2) / 5;

                let cr = 35 + (mix * 95 >> 8) + (n_f * 25 >> 8);
                let cg = 25 + (mix * 45 >> 8);
                let cb = 120 + (mix * 100 >> 8);

                r += i * cr >> 8;
                g += i * cg >> 8;
                b += i * cb >> 8;
            }

            if fg > 0 {
                let n_f = fbm((xi << 8) / 30 + 4444, (yi << 8) / 30 + 4444);
                let n_b = vnoise((xi << 8) / 120 + 9999,
                                 (yi << 8) / 120 + 9999);

                let cloud = (n_f * 3 + n_b * 2) / 5;

                let mut i = (fg * cloud >> 8) + dith;
                if i < 0 { i = 0; }
                if i > 255 { i = 255; }

                let cr = 15 + (n_f * 45 >> 8);
                let cg = 90 + (n_b * 110 >> 8);
                let cb = 60 + (n_f * 90 >> 8);

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


    let cell: i64 = 6;
    let cw = (w + cell - 1) / cell;
    let ch = (h + cell - 1) / cell;

    for cy in 0..ch {
        for cx in 0..cw {
            let hh = hash2(cx * 31 + 7, cy * 17 + 3);

            let midx = cx * cell + cell / 2;
            let midy = cy * cell + cell / 2;

            let fp = band_fall(midx, midy, w, h);
            let fg = green_fall(midx, midy, w, h);
            let fall = if fp > fg { fp } else { fg };

            let thresh = (3200 + fall * 16) as u32;

            if (hh & 0xFFFF) > thresh {
                continue;
            }

            let jx = ((hh >> 4) % cell as u32) as i64;
            let jy = ((hh >> 12) % cell as u32) as i64;

            let br = 70 + ((hh >> 20) % 186) as i64;

            let (cr, cg, cb) = match (hh >> 8) & 3 {
                0 => (170, 200, 255),
                1 => (255, 220, 180),
                _ => (255, 255, 255),
            };

            px_add(fb, cx * cell + jx, cy * cell + jy,
                   cr * br / 255, cg * br / 255, cb * br / 255, m);

            if br > 230 {
                px_add(fb, cx * cell + jx + 1, cy * cell + jy,
                       cr / 3, cg / 3, cb / 3, m);
                px_add(fb, cx * cell + jx, cy * cell + jy + 1,
                       cr / 3, cg / 3, cb / 3, m);
            }
        }
    }


    let cell4: i64 = 3;
    let cw4 = (w + cell4 - 1) / cell4;
    let ch4 = (h + cell4 - 1) / cell4;

    for cy in 0..ch4 {
        for cx in 0..cw4 {
            let hh = hash2(cx * 131 + 1, cy * 107 + 9);

            if (hh & 0xFFFF) > 950 {
                continue;
            }

            let jx = ((hh >> 3) % cell4 as u32) as i64;
            let jy = ((hh >> 11) % cell4 as u32) as i64;

            let br = 40 + ((hh >> 19) % 216) as i64;

            let (cr, cg, cb) = match (hh >> 7) & 3 {
                0 => (180, 205, 255),
                1 => (255, 230, 190),
                _ => (255, 255, 255),
            };

            px_add(fb, cx * cell4 + jx, cy * cell4 + jy,
                   cr * br / 255, cg * br / 255, cb * br / 255, m);

            if br > 240 {
                px_add(fb, cx * cell4 + jx + 1, cy * cell4 + jy,
                       cr / 4, cg / 4, cb / 4, m);
                px_add(fb, cx * cell4 + jx - 1, cy * cell4 + jy,
                       cr / 4, cg / 4, cb / 4, m);
                px_add(fb, cx * cell4 + jx, cy * cell4 + jy + 1,
                       cr / 4, cg / 4, cb / 4, m);
                px_add(fb, cx * cell4 + jx, cy * cell4 + jy - 1,
                       cr / 4, cg / 4, cb / 4, m);
            }
        }
    }


    for cy in 0..ch4 {
        for cx in 0..cw4 {
            let hh = hash2(cx * 57 + 11, cy * 43 + 5);

            let midx = cx * cell4 + cell4 / 2;
            let midy = cy * cell4 + cell4 / 2;

            let fp = band_fall(midx, midy, w, h);
            let fg = green_fall(midx, midy, w, h);
            let fall = if fp > fg { fp } else { fg };

            if fall < 40 {
                continue;
            }

            let t2 = (fall * 8) as u32;

            if (hh & 0xFFFF) > t2 {
                continue;
            }

            let br = 30 + ((hh >> 19) % 110) as i64;

            let (cr, cg, cb) = if fg > fp {
                (180, 255, 220)
            } else {
                (210, 220, 255)
            };

            px_add(fb, cx * cell4 + ((hh >> 3) % cell4 as u32) as i64,
                   cy * cell4 + ((hh >> 11) % cell4 as u32) as i64,
                   cr * br / 255, cg * br / 255, cb * br / 255, m);
        }
    }


    let cell3: i64 = 4;
    let cw3 = (w + cell3 - 1) / cell3;
    let ch3 = (h + cell3 - 1) / cell3;

    for cy in 0..ch3 {
        for cx in 0..cw3 {
            let hh = hash2(cx * 91 + 3, cy * 71 + 13);

            let midx = cx * cell3 + cell3 / 2;
            let midy = cy * cell3 + cell3 / 2;

            let fp = band_fall(midx, midy, w, h);
            let fg = green_fall(midx, midy, w, h);
            let fall = if fp > fg { fp } else { fg };

            if fall <= 140 {
                continue;
            }

            let t3 = ((fall - 140) * 10) as u32;

            if (hh & 0xFFFF) > t3 {
                continue;
            }

            let bx = cx * cell3 + ((hh >> 5) % cell3 as u32) as i64;
            let by = cy * cell3 + ((hh >> 13) % cell3 as u32) as i64;

            let br = 50 + ((hh >> 21) % 160) as i64;

            let (cr, cg, cb) = if fg > fp {
                (200, 255, 230)
            } else {
                (230, 230, 255)
            };

            let r0 = cr * br / 255;
            let g0 = cg * br / 255;
            let b0 = cb * br / 255;

            match (hh >> 16) & 7 {
                0 => { px_add(fb, bx, by, r0, g0, b0, m); }
                1 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx + 1, by, r0 * 2 / 3, g0 * 2 / 3, b0 * 2 / 3, m); }
                2 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx, by + 1, r0 * 2 / 3, g0 * 2 / 3, b0 * 2 / 3, m); }
                3 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx + 1, by + 1, r0 / 2, g0 / 2, b0 / 2, m); }
                4 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx - 1, by + 1, r0 / 2, g0 / 2, b0 / 2, m); }
                5 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx + 1, by, r0 / 2, g0 / 2, b0 / 2, m);
                       px_add(fb, bx, by + 1, r0 / 2, g0 / 2, b0 / 2, m); }
                6 => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx + 1, by, r0 / 2, g0 / 2, b0 / 2, m);
                       px_add(fb, bx - 1, by, r0 / 2, g0 / 2, b0 / 2, m);
                       px_add(fb, bx, by + 1, r0 / 2, g0 / 2, b0 / 2, m); }
                _ => { px_add(fb, bx, by, r0, g0, b0, m);
                       px_add(fb, bx + 2, by + 1, r0 / 3, g0 / 3, b0 / 3, m); }
            }
        }
    }


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