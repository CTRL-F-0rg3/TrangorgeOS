use core::sync::atomic::{AtomicU32, Ordering};

pub const VID_HDMI_INIT: u32 = 3;
pub const VID_HDMI_FILL: u32 = 4;
pub const VID_HDMI_POLL: u32 = 5;
pub const VID_HDMI_CAPS: u32 = 6;
pub const VID_MODE_GET: u32 = 7;
pub const VID_MODE_LIST: u32 = 8;
pub const VID_MODE_SET: u32 = 9;
pub const VID_GRANT_FB: u32 = 10;
pub const VID_REVOKE_FB: u32 = 11;

const BUDGET_MAX: u32 = 8;
static BUDGET: AtomicU32 = AtomicU32::new(BUDGET_MAX);

pub fn authorize(ring: u8, op: u32) -> bool {
    if op == VID_HDMI_INIT {
        return ring == 0;
    }

    if ring == 0 {
        return true;
    }

    if op == VID_MODE_SET && ring >= 3 {
        return false;
    }

    if op == VID_GRANT_FB && ring >= 3 {
        return false;
    }

    if op == VID_HDMI_FILL {
        return BUDGET
            .try_update(Ordering::AcqRel, Ordering::Acquire, |budget| {
                budget.checked_sub(1)
            })
            .is_ok();
    }

    ring < 4
}

pub fn tick_reset() {
    BUDGET.store(BUDGET_MAX, Ordering::Release);
}