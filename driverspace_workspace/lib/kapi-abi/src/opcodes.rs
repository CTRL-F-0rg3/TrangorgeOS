#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DsCmd {
    None = 0,
    Init = 1,
    Caps = 2,
    Ready = 3,
    RegisterDriver = 4,
    AttachDevice = 5,
    DetachDevice = 6,
    MapDeviceMemory = 7,
    BindIrq = 8,
    Shutdown = 9,
    Log = 10,
    
    MemAllocDma = 0x100,
    MemFreeDma = 0x101,
    MemMapMmio = 0x102,
    MemUnmapMmio = 0x103,
    PagePhys = 0x104,
    
    PciFind = 0x200,
    PciRead = 0x201,
    PciWrite = 0x202,
}

impl DsCmd {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::Init),
            2 => Some(Self::Caps),
            3 => Some(Self::Ready),
            4 => Some(Self::RegisterDriver),
            5 => Some(Self::AttachDevice),
            6 => Some(Self::DetachDevice),
            7 => Some(Self::MapDeviceMemory),
            8 => Some(Self::BindIrq),
            9 => Some(Self::Shutdown),
            10 => Some(Self::Log),
            0x100 => Some(Self::MemAllocDma),
            0x101 => Some(Self::MemFreeDma),
            0x102 => Some(Self::MemMapMmio),
            0x103 => Some(Self::MemUnmapMmio),
            0x104 => Some(Self::PagePhys),
            0x200 => Some(Self::PciFind),
            0x201 => Some(Self::PciRead),
            0x202 => Some(Self::PciWrite),
            _ => None,
        }
    }
}

// Alias dla kompatybilności z kodem używającym nazwy `Opcode`
pub use DsCmd as Opcode;