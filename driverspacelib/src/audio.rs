use crate::abi::*;
use crate::runtime::{request, take_resp};

pub struct AudioBars {
    pub nam_phys: u64,
    pub bm_phys: u64,
}

pub fn info_req() -> u64 {
    request(DsCmd::AudioInfo, 0, 0, 0)
}

pub fn info_take(id: u64) -> Option<AudioBars> {
    let r = take_resp(id)?;

    if r.status != 0 {
        return None;
    }

    Some(AudioBars { nam_phys: r.arg0, bm_phys: r.arg1 })
}

pub fn page_phys_req(va: u64) -> u64 {
    request(DsCmd::PagePhys, va, 0, 0)
}

pub fn page_phys_take(id: u64) -> Option<u64> {
    let r = take_resp(id)?;

    if r.status != 0 {
        return None;
    }

    Some(r.arg0)
}