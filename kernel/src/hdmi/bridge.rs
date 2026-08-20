use super::aut;
use crate::driverspaceinit::abi::abi::DsMsg;

extern "C" {
    fn hdmi_init_with(fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
    fn hdmi_ready() -> bool;
    fn hdmi_submit_fill(color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
    fn hdmi_poll(out: *mut u64) -> bool;
    fn hdmi_mode_set_by_id(id: u32) -> bool;
    fn hdmi_mode_current_raw(id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
    fn hdmi_mode_at_raw(i: u32, id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
    fn hdmi_caps_raw(w: *mut u32, h: *mut u32, s: *mut u32, phys: *mut u64);
}

pub fn hdmi_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op) {
        return -1;
    }

    if op == aut::VID_HDMI_INIT {
        let (w, h, stride, phys) = crate::gfx::console::fb_info();
        return if unsafe { hdmi_init_with(phys, w, h, stride) } { 0 } else { -2 };
    }

    if !unsafe { hdmi_ready() } {
        return -2;
    }

    match op {
        aut::VID_HDMI_FILL => {
            let sequence = unsafe {
                hdmi_submit_fill(
                    m.arg0 as u32,
                    (m.arg1 & 0xffff) as u32,
                    (m.arg1 >> 16) as u32,
                    (m.arg2 & 0xffff) as u32,
                    (m.arg2 >> 16) as u32,
                )
            };
            if sequence == 0 {
                -1
            } else {
                r.arg0 = sequence;
                0
            }
        }
        aut::VID_HDMI_POLL => {
            let mut sequence = 0u64;
            if !unsafe { hdmi_poll(&mut sequence) } {
                return 1;
            }
            r.arg0 = sequence;
            0
        }
        aut::VID_HDMI_CAPS => {
            let (w, h, stride, phys) = crate::gfx::console::fb_info();
            r.arg0 = ((w as u64) << 32) | h as u64;
            r.arg1 = stride as u64;
            r.arg2 = phys;
            0
        }
        aut::VID_MODE_GET => {
            let mut id = 0u32;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut refresh = 0u32;
            if !unsafe { hdmi_mode_current_raw(&mut id, &mut w, &mut h, &mut refresh) } {
                return -1;
            }
            r.arg0 = id as u64;
            r.arg1 = ((w as u64) << 32) | h as u64;
            r.arg2 = refresh as u64;
            0
        }
        aut::VID_MODE_LIST => {
            let mut id = 0u32;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut refresh = 0u32;
            if !unsafe { hdmi_mode_at_raw(m.arg0 as u32, &mut id, &mut w, &mut h, &mut refresh) } {
                return 1;
            }
            r.arg0 = id as u64;
            r.arg1 = ((w as u64) << 32) | h as u64;
            r.arg2 = refresh as u64;
            0
        }
        aut::VID_MODE_SET => {
            if unsafe { hdmi_mode_set_by_id(m.arg0 as u32) } { 0 } else { -1 }
        }

        x if x == aut::VID_GRANT_FB => {
            extern "C" { fn hdmi_fb_grant() -> bool; }

            let mut w = 0u32; let mut h = 0u32; let mut s = 0u32; let mut phys = 0u64;
            unsafe { hdmi_caps_raw(&mut w, &mut h, &mut s, &mut phys) };

            if phys == 0 || !unsafe { hdmi_fb_grant() } {
                return -1;
            }

            r.arg0 = phys;
            r.arg1 = ((w as u64) << 32) | h as u64;
            r.arg2 = s as u64;
            0
        }

        x if x == aut::VID_REVOKE_FB => {
            extern "C" { fn hdmi_fb_revoke(); }
            unsafe { hdmi_fb_revoke() };
            0
        }

        _ => -1,
    }
}
