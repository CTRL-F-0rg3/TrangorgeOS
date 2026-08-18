use crate::abi::*;
use crate::svc;

pub struct FbInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub phys: u64,
}

pub fn fb_info_req() -> u64 {
    svc::call(SVC_VIDEO, VID_FB_INFO, 0, 0, 0)
}

pub fn fb_info_take(id: u64) -> Option<FbInfo> {
    let r = svc::take(id)?;

    Some(FbInfo {
        width: (r.arg0 >> 16) as u32,
        height: (r.arg0 & 0xFFFF) as u32,
        stride: r.arg1 as u32,
        phys: r.arg2,
    })
}

pub fn takeover() -> u64 {
    svc::call(SVC_VIDEO, VID_FB_TAKEOVER, 0, 0, 0)
}

pub fn release() -> u64 {
    svc::call(SVC_VIDEO, VID_FB_RELEASE, 0, 0, 0)
}