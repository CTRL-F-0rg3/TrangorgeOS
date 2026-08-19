// bridge.rs
use super::aut;
use crate::driverspaceinit::abi::DsMsg;

extern "C" {
    fn dp_ready() -> bool;
    fn dp_link_info(rate: *mut u32, lanes: *mut u32);
    fn dp_mode_at(i: u32, id: *mut u32, w: *mut u32,
                  h: *mut u32, r: *mut u32) -> bool;
    fn dp_mode_set_by_id(id: u32) -> bool;
    fn dp_submit_fill(color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
}

pub fn dp_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op as u8) {
        return -1;
    }

    match op {
        x if x == aut::DP_STATUS => {
            r.arg0 = unsafe { dp_ready() } as u64;
            0
        }

        x if x == aut::DP_LINK => {
            let mut rate = 0u32;
            let mut lanes = 0u32;

            unsafe { dp_link_info(&mut rate, &mut lanes) };

            r.arg0 = rate as u64;
            r.arg1 = lanes as u64;
            0
        }

        x if x == aut::DP_MODES => {
            let mut id = 0u32;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut rr = 0u32;

            let ok = unsafe {
                dp_mode_at(m.arg0 as u32, &mut id, &mut w, &mut h, &mut rr)
            };

            if !ok {
                return 1;
            }

            r.arg0 = id as u64;
            r.arg1 = ((w as u64) << 16) | h as u64;
            r.arg2 = rr as u64;
            0
        }

        x if x == aut::DP_MODE_SET => {
            if unsafe { dp_mode_set_by_id(m.arg0 as u32) } { 0 } else { -1 }
        }

        x if x == aut::DP_FILL => {
            let seq = unsafe {
                dp_submit_fill(m.arg0 as u32,
                               (m.arg1 & 0xFFFF) as u32,
                               (m.arg1 >> 16) as u32,
                               (m.arg2 & 0xFFFF) as u32,
                               (m.arg2 >> 16) as u32)
            };

            r.arg0 = seq;
            0
        }

        _ => -1,
    }
}