use super::aut;
use crate::driverspaceinit::abi::abi::DsMsg;

extern "C" {
    fn hdmi_iface_acquire(owner: u32) -> bool;
    fn hdmi_iface_release(owner: u32) -> bool;

    fn hdmi_iface_init(owner: u32, fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
    fn hdmi_iface_ready() -> bool;

    fn hdmi_iface_mode_set(owner: u32, id: u32) -> bool;
    fn hdmi_iface_mode_current(id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
    fn hdmi_iface_mode_at(i: u32, id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;

    fn hdmi_iface_submit_fill(owner: u32, color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
    fn hdmi_iface_poll(owner: u32, out: *mut u64) -> bool;

    fn hdmi_iface_caps(w: *mut u32, h: *mut u32, s: *mut u32, phys: *mut u64);

    fn hdmi_iface_fb_grant(owner: u32, phys: *mut u64, w: *mut u32, h: *mut u32, s: *mut u32) -> bool;
    fn hdmi_iface_fb_revoke(owner: u32) -> bool;
}

pub fn hdmi_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8, owner: u32) -> i32 {
    if !aut::authorize(ring, op) {
        return -1;
    }

    if op == aut::VID_HDMI_INIT {
        let (w, h, stride, phys) = crate::gfx::console::fb_info();
        return if unsafe { hdmi_iface_init(0, phys, w, h, stride) } { 0 } else { -2 };
    }

    if op == aut::VID_HDMI_ACQUIRE {
        return if owner != 0 && unsafe { hdmi_iface_acquire(owner) } { 0 } else { -1 };
    }

    if op == aut::VID_HDMI_RELEASE {
        return if owner != 0 && unsafe { hdmi_iface_release(owner) } { 0 } else { -1 };
    }

    if !unsafe { hdmi_iface_ready() } {
        return -2;
    }

    match op {
        aut::VID_HDMI_FILL => {
            let sequence = unsafe {
                hdmi_iface_submit_fill(
                    owner,
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
            if !unsafe { hdmi_iface_poll(owner, &mut sequence) } {
                return 1;
            }
            r.arg0 = sequence;
            0
        }
        aut::VID_HDMI_CAPS => {
            let mut w = 0u32;
            let mut h = 0u32;
            let mut s = 0u32;
            let mut phys = 0u64;
            unsafe { hdmi_iface_caps(&mut w, &mut h, &mut s, &mut phys) };
            r.arg0 = ((w as u64) << 32) | h as u64;
            r.arg1 = s as u64;
            r.arg2 = phys;
            0
        }
        aut::VID_MODE_GET => {
            let mut id = 0u32;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut refresh = 0u32;
            if !unsafe { hdmi_iface_mode_current(&mut id, &mut w, &mut h, &mut refresh) } {
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
            if !unsafe { hdmi_iface_mode_at(m.arg0 as u32, &mut id, &mut w, &mut h, &mut refresh) } {
                return 1;
            }
            r.arg0 = id as u64;
            r.arg1 = ((w as u64) << 32) | h as u64;
            r.arg2 = refresh as u64;
            0
        }
        aut::VID_MODE_SET => {
            if unsafe { hdmi_iface_mode_set(owner, m.arg0 as u32) } { 0 } else { -1 }
        }
        aut::VID_GRANT_FB => {
            let mut phys = 0u64;
            let mut w = 0u32;
            let mut h = 0u32;
            let mut s = 0u32;
            if !unsafe { hdmi_iface_fb_grant(owner, &mut phys, &mut w, &mut h, &mut s) } {
                return -1;
            }
            r.arg0 = phys;
            r.arg1 = ((w as u64) << 32) | h as u64;
            r.arg2 = s as u64;
            0
        }
        aut::VID_REVOKE_FB => {
            if unsafe { hdmi_iface_fb_revoke(owner) } { 0 } else { -1 }
        }
        _ => -1,
    }
}