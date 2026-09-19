pub trait Write {
    fn write_str(&mut self, s: &str) -> Result<(), ()>;
    
    fn write_u64(&mut self, mut val: u64, base: u8) -> Result<(), ()> {
        let mut buf = [0u8; 64];
        let mut i = 64;
        if val == 0 {
            i -= 1;
            buf[i] = b'0';
        } else {
            while val > 0 {
                i -= 1;
                let rem = (val % base as u64) as u8;
                buf[i] = if rem < 10 { b'0' + rem } else { b'a' + rem - 10 };
                val /= base as u64;
            }
        }
        self.write_str(unsafe { core::str::from_utf8_unchecked(&buf[i..]) })
    }
}

pub trait Read {
    fn read_byte(&mut self) -> Option<u8>;
}