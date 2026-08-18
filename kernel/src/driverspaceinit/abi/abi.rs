pub const DS_MAGIC: u64 = 0x4452_5653_5041_4345;
pub const DS_VERSION: u32 = 1;
pub const DS_RING_CAP: u64 = 16;

pub const DS_FLAG_RESPONSE: u32 = 1 << 0;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    AllocPages = 11,
    FreePages = 12,
    MapMmio = 13,
    GetDeviceCount = 14,
    BlockRead = 15,
    BlockWrite = 16,
    EventDeviceAdded = 30,
    AudioInfo = 48,
    PagePhys = 49,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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

pub const DS_MSG_SIZE: usize = core::mem::size_of::<DsMsg>();

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DsRing {
    pub head: u64,
    pub tail: u64,
    pub cap: u64,
}

pub const DS_RING_HDR_SIZE: usize = core::mem::size_of::<DsRing>();