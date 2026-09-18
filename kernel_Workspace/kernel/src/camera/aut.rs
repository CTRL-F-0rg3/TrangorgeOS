pub const CAM_CAPS: u32 = 1;
pub const CAM_START: u32 = 2;
pub const CAM_STOP: u32 = 3;
pub const CAM_FRAME: u32 = 4;

pub fn authorize(ring: u8, op: u8) -> bool {
    if ring == 0 {
        return true;
    }

    if op == CAM_START as u8 || op == CAM_STOP as u8 || op == CAM_FRAME as u8 {
        if ring >= 3 {
            return false;
        }
    }

    true
}
