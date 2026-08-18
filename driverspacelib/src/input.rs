use crate::abi::*;
use crate::svc;

pub fn key_req() -> u64 {
    svc::call(SVC_INPUT, IN_KEY_POLL, 0, 0, 0)
}

pub fn key_take(id: u64) -> Option<u8> {
    let r = svc::take(id)?;

    if r.arg0 == 0 { None } else { Some(r.arg0 as u8) }
}