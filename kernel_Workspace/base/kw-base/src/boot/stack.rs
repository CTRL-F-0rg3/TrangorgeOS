#[repr(C, align(16))]
pub struct BootStack {
    pub data: [u8; 65536],
}

impl BootStack {
    pub const fn new() -> Self {
        Self { data: [0; 65536] }
    }

    pub fn top(&self) -> u64 {
        (self.data.as_ptr() as u64) + (self.data.len() as u64)
    }
}