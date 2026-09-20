#![no_std]

use crate::primitives::Status;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DsMsg {
    pub cmd: u32,
    pub status: i32,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
}

impl DsMsg {
    pub const fn new(cmd: u32) -> Self {
        Self { cmd, status: 0, arg0: 0, arg1: 0, arg2: 0, arg3: 0 }
    }

    pub fn is_ok(&self) -> bool {
        self.status == 0
    }

    pub fn error_status(&self) -> Status {
        Status::from(self.status)
    }
}