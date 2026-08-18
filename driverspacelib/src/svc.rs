use crate::abi::*;
use crate::runtime::{request, take_resp};

pub fn call(class: u32, op: u32, a0: u64, a1: u64, a2: u64) -> u64 {
    request_raw(svc_cmd(class, op), a0, a1, a2)
}

pub fn take(id: u64) -> Option<DsMsg> {
    take_resp(id)
}