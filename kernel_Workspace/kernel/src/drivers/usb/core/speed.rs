pub const SPEED_FULL: u32 = 1;
pub const SPEED_LOW: u32 = 2;
pub const SPEED_HIGH: u32 = 3;
pub const SPEED_SUPER: u32 = 4;

pub fn default_ep0_mps(speed: u32) -> u16 {
    match speed {
        SPEED_HIGH => 64,
        SPEED_SUPER => 512,
        _ => 8,
    }
}

pub const EP_CONTROL: u8 = 0;
pub const EP_ISO: u8 = 1;
pub const EP_BULK: u8 = 2;
pub const EP_INTERRUPT: u8 = 3;