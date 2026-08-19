// aut.rs
pub const BT_INFO: u32 = 1;
pub const BT_CMD: u32 = 2;
pub const BT_EVT: u32 = 3;
pub const BT_ACL_OUT: u32 = 4;
pub const BT_ACL_IN: u32 = 5;

pub fn authorize(ring: u8, op: u8) -> bool {
    if ring == 0 {
        return true;
    }

    if op == BT_CMD as u8 || op == BT_ACL_OUT as u8 {
        if ring >= 3 {
            return false;
        }
    }

    true
}