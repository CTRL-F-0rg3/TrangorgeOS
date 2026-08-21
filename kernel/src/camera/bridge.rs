use super::aut;
use crate::driverspaceinit::abi::DsMsg;

extern "C" {
    fn camera_caps_get_w(w: *mut u32, h: *mut u32, fmt: *mut u32, fps: *mut u32) -> bool;
    fn camera_start() -> bool;
    fn camera_stop() -> bool;
    fn camera_frame_to_phys(phys: u64, cap: u32, fid: *mut u64) -> bool;
}

fn grant_phys(va: u64) -> Option<u64> {
    crate::driverspaceinit::init::service::grant_phys(va).map(|(p, _)| p)
}

pub fn cam_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op as u8) {
        return -1;
    }

    match op {
        x if x == aut::CAM_CAPS => {
            let mut w = 0u32;
            let mut h = 0u32;
            let mut fmt = 0u32;
            let mut fps = 0u32;

            let ok = unsafe { camera_caps_get_w(&mut w, &mut h, &mut fmt, &mut fps) };

            if !ok {
                return 1;
            }

            r.arg0 = ((w as u64) << 16) | h as u64;
            r.arg1 = fmt as u64;
            r.arg2 = fps as u64;
            0
        }

        x if x == aut::CAM_START => {
            if unsafe { camera_start() } { 0 } else { -1 }
        }

        x if x == aut::CAM_STOP => {
            if unsafe { camera_stop() } { 0 } else { -1 }
        }

        x if x == aut::CAM_FRAME => {
            let phys = match grant_phys(m.arg0) {
                Some(p) => p,
                None => return -1,
            };

            let mut fid = 0u64;

            let ok = unsafe { camera_frame_to_phys(phys, m.arg1 as u32, &mut fid) };

            if !ok {
                return 1;
            }

            r.arg0 = fid;
            0
        }

        _ => -1,
    }
}
