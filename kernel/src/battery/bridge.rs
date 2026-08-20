// bridge.rs
use super::aut;
use crate::driverspaceinit::abi::DsMsg;

extern "C" {
    fn battery_status_packed(a0: *mut u64, a1: *mut u64, a2: *mut u64) -> bool;
    fn battery_set_threshold(pct: u32) -> bool;
}

pub fn batt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op as u8) {
        return -1;
    }

    match op {
        x if x == aut::BATT_STATUS => {
            let mut a0 = 0u64;
            let mut a1 = 0u64;
            let mut a2 = 0u64;

            let ok = unsafe { battery_status_packed(&mut a0, &mut a1, &mut a2) };

            if !ok {
                return 1;
            }

            r.arg0 = a0;
            r.arg1 = a1;
            r.arg2 = a2;
            0
        }

        x if x == aut::BATT_THRESH => {
            if unsafe { battery_set_threshold(m.arg0 as u32) } { 0 } else { -1 }
        }

        _ => -1,
    }
}