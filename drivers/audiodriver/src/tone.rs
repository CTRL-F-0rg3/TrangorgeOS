pub fn fill_square(buf: &mut [u8], periods: u32) {
    let half = buf.len() as u32 / (periods * 2);

    for i in 0..buf.len() as u32 {
        let v = if (i / half.max(1)) % 2 == 0 { 0xC0 } else { 0x40 };
        buf[i as usize] = v;
    }
}