use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct MessageInfo {
    pub label: u32,
    pub caps_unwrapped: u8,
    pub extra_caps: u8,
    pub length: u8,
    pub _padding: u8,
}

impl MessageInfo {
    pub const fn new(label: u32, length: u8) -> Self {
        Self {
            label,
            caps_unwrapped: 0,
            extra_caps: 0,
            length,
            _padding: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MessageRegisters {
    pub mr: [u64; 8],
}

impl MessageRegisters {
    pub const fn empty() -> Self {
        Self { mr: [0; 8] }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcMessage {
    pub info: MessageInfo,
    pub regs: MessageRegisters,
}