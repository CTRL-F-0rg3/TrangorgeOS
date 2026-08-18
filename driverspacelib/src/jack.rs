use crate::abi::*;
use crate::runtime::{request, take_resp};

pub struct JackInfo {
    pub present: bool,
    pub amp_on: bool,
}

pub fn query_req() -> u64 {
    request(DsCmd::JackQuery, 0, 0, 0)
}

pub fn query_take(id: u64) -> Option<JackInfo> {
    let r = take_resp(id)?;

    Some(JackInfo {
        present: r.arg0 & 1 != 0,
        amp_on: r.arg0 & 2 != 0,
    })
}

pub fn set_amp(on: bool) {
    request(DsCmd::JackSetAmp, on as u64, 0, 0);
}

pub fn play(va: u64, len: u32) -> u64 {
    request(DsCmd::AudioPlay, va, len as u64, 0)
}

pub fn stop() {
    request(DsCmd::AudioStop, 0, 0, 0);
}