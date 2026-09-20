#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DsCmd {
    Init = 0x01,
    Log = 0x02,
    
    ReqMmio = 0x10,
    ReqIrq = 0x11,
    ReqDma = 0x12,
    
    PciFind = 0x20,
    PciRead = 0x21,
    PciWrite = 0x22,
    
    VideoFbInfo = 0x30,
    VideoTakeover = 0x31,
    
    PagePhys = 0x40,
}

impl DsCmd {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0x01 => Some(Self::Init),
            0x02 => Some(Self::Log),
            0x10 => Some(Self::ReqMmio),
            0x11 => Some(Self::ReqIrq),
            0x12 => Some(Self::ReqDma),
            0x20 => Some(Self::PciFind),
            0x21 => Some(Self::PciRead),
            0x22 => Some(Self::PciWrite),
            0x30 => Some(Self::VideoFbInfo),
            0x31 => Some(Self::VideoTakeover),
            0x40 => Some(Self::PagePhys),
            _ => None,
        }
    }
}