use super::aut;
use crate::driverspaceinit::abi::DsMsg;

extern "C" {
    fn hdmi_submit_fill(color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
    fn hdmi_poll(out: *mut u64) -> bool;
    fn hdmi_caps_raw(out_w: *mut u32, out_h: *mut u32, out_stride: *mut u32,
                     out_phys: *mut u64);
}

// Calls the HDMI service with the given operation and message, returning a result code.

pub fn hdmi_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op as u8) {
        return -1;
    }

    match op {
        x if x == aut::VID_HDMI_FILL => {
            let seq = unsafe {
                hdmi_submit_fill(m.arg0 as u32,
                                 (m.arg1 & 0xFFFF) as u32,
                                 (m.arg1 >> 16) as u32,
                                 (m.arg2 & 0xFFFF) as u32,
                                 (m.arg2 >> 16) as u32)
            };

            if seq == 0 { -1 } else { r.arg0 = seq; 0 }
        }

        
        x if x == aut::VID_HDMI_POLL => {
            let mut seq = 0u64;
            let ok = unsafe { hdmi_poll(&mut seq) };

            r.arg0 = seq;
            if ok { 0 } else { 1 }
        }

        x if x == aut::VID_HDMI_CAPS => {
            let (w, h, s, phys) = crate::gfx::console::fb_info();

            r.arg0 = ((w as u64) << 16) | h as u64;
            r.arg1 = s as u64;
            r.arg2 = phys;
            0
        }
        extern "C" {
    fn hdmi_mode_set_by_id(id: u32) -> bool;
}

        x if x == aut::VID_MODE_GET => {
            let (w, h, _, _) = crate::gfx::console::fb_info();

            r.arg0 = ((w as u64) << 16) | h as u64;
            0
        }

        x if x == aut::VID_MODE_LIST => {
            extern "C" {
                fn hdmi_mode_at_raw(i: u32,
                                    out_id: *mut u32,
                                    out_w: *mut u32,
                                    out_h: *mut u32,
                                    out_r: *mut u32) -> bool;
            }

            let mut id = 0u32;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut rr = 0u32;

            let ok = unsafe {
                hdmi_mode_at_raw(m.arg0 as u32, &mut id, &mut w, &mut h, &mut rr)
            };

            if !ok {
                return 1;
            }

            r.arg0 = id as u64;
            r.arg1 = ((w as u64) << 16) | h as u64;
            r.arg2 = rr as u64;
            0
        }

        x if x == aut::VID_MODE_SET => {
            if unsafe { hdmi_mode_set_by_id(m.arg0 as u32) } { 0 } else { -1 }
        }

        _ => -1,
    }
}

