pub const BT_INIT: u32 = 0;
pub const BT_INFO: u32 = 1;
pub const BT_CMD: u32 = 2;
pub const BT_EVT: u32 = 3;
pub const BT_ACL_OUT: u32 = 4;
pub const BT_ACL_IN: u32 = 5;

pub fn authorize(ring: u8, op: u32) -> bool {
    if op == BT_INIT {
        return ring == 0;
    }

    if ring == 0 {
        return true;
    }

    if op == BT_CMD || op == BT_ACL_OUT {
        return ring < 3;
    }

    ring < 4
}
