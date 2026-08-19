use crate::driverspaceinit::abi::{svc_cmd, SVC_VIDEO};
// HDMI service commands
pub const VID_HDMI_FILL: u32 = 4;
pub const VID_HDMI_POLL: u32 = 5;
pub const VID_HDMI_CAPS: u32 = 6;
pub const VID_MODE_GET: u32 = 7;
pub const VID_MODE_LIST: u32 = 8;
pub const VID_MODE_SET: u32 = 9;
const BUDGET_MAX: u32 = 8;

static mut BUDGET: u32 = BUDGET_MAX;

pub fn authorize(ring: u8, op: u8) -> bool {
    if ring == 0 {
        return true;
    }

    if !crate::policy::bridge::check(ring, svc_cmd(SVC_VIDEO, op as u32), 0) {
        return false;
    }



    if op == VID_MODE_SET as u8 && ring >= 3 {
        return false;
    }
    if op == VID_HDMI_FILL as u8 {
        unsafe {
            if BUDGET == 0 {
                return false;
            }

            BUDGET -= 1;
        }
    }

    true
}

pub fn tick_reset() {
    unsafe { BUDGET = BUDGET_MAX; }
}

//crate::hdmi::aut::tick_reset();
