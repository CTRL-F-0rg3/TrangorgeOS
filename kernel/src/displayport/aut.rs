pub const DP_STATUS: u32 = 1;
pub const DP_LINK: u32 = 2;
pub const DP_MODES: u32 = 3;
pub const DP_MODE_SET: u32 = 4;
pub const DP_FILL: u32 = 5;

pub fn authorize(ring: u8, op: u8) -> bool {
    if ring == 0 {
        return true;
    }

    if op == DP_MODE_SET as u8 && ring >= 3 {
        return false;
    }

    true
}
// Todo remake a autorisation system for displayport, this is a temporary solution to avoid the ring 0 to be used by other processes.