#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Handle(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CapId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    InvalidArg = -1,
    PermissionDenied = -2,
    NotFound = -3,
    OutOfMemory = -4,
    Busy = -5,
    IoError = -6,
}

impl From<i32> for Status {
    fn from(val: i32) -> Self {
        match val {
            0 => Self::Ok,
            -1 => Self::InvalidArg,
            -2 => Self::PermissionDenied,
            -3 => Self::NotFound,
            -4 => Self::OutOfMemory,
            -5 => Self::Busy,
            -6 => Self::IoError,
            _ => Self::IoError,
        }
    }
}