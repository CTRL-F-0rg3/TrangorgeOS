// aut.rs
pub const BATT_STATUS: u32 = 1;
pub const BATT_THRESH: u32 = 2;

pub fn authorize(ring: u8, op: u8) -> bool {
    if op == BATT_THRESH as u8 && ring >= 3 {
        return false;
    }

    true
}