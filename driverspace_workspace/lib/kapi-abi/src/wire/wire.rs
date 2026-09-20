#![no_std]
// driverspace_workspace/lib/kapi-abi/src/wire.rs (lub gdziekolwiek jest DsMsg)

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DsMsg {
    pub id: u64,
    pub cmd: u32,
    pub flags: u32,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub status: i32,
    pub pad: u32,
}

impl DsMsg {
    pub const fn new(cmd: u32) -> Self {
        Self {
            id: 0,
            cmd,
            flags: 0,
            arg0: 0,
            arg1: 0,
            arg2: 0,
            status: 0,
            pad: 0,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.status == 0
    }

    pub fn error_status(&self) -> crate::DsError {
        crate::DsError::from_u32(self.status as u32)
    }
}