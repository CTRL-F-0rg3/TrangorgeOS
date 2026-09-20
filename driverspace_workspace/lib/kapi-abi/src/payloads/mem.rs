#![no_std]

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DmaAllocPayload {
    pub size: u64,
    pub flags: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MmioMapPayload {
    pub phys_base: u64,
    pub size: u64,
}