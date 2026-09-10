pub const SVC_SYS: u32 = 0;
pub const SVC_VIDEO: u32 = 1;
pub const SVC_AUDIO: u32 = 2;
pub const SVC_INPUT: u32 = 3;
pub const SVC_BLOCK: u32 = 4;
pub const SVC_NET: u32 = 5;

pub const fn svc_cmd(class: u32, op: u32) -> u32 {
    (class << 8) | (op & 0xFF)
}

pub const fn svc_class(cmd: u32) -> u32 {
    cmd >> 8
}

pub const fn svc_op(cmd: u32) -> u32 {
    cmd & 0xFF
}

pub const VID_FB_INFO: u32 = 1;
pub const VID_FB_TAKEOVER: u32 = 2;
pub const VID_FB_RELEASE: u32 = 3;
pub const IN_KEY_POLL: u32 = 1;
pub const AUD_PLAY: u32 = 1;
pub const AUD_STOP: u32 = 2;
pub const AUD_JACK: u32 = 3;
pub const AUD_AMP: u32 = 4;
pub const BLK_COUNT: u32 = 1;
pub const BLK_READ: u32 = 2;
pub const BLK_WRITE: u32 = 3;

// ---------------------------------------------------------------------
// Driver-space ABI: commands, messages and rings (mirrors
// kernel/src/driverspaceinit/abi/abi.rs and libs/dsabi.h).
// ---------------------------------------------------------------------

pub const DS_MAGIC: u64 = 0x4452_5653_5041_4345;
pub const DS_VERSION: u32 = 1;
pub const DS_RING_CAP: u64 = 16;

pub const DS_FLAG_RESPONSE: u32 = 1 << 0;

pub const DS_INIT_PARAMS_VA: u64 = 0x4000_0000;
pub const DS_K2D_VA: u64 = 0x4000_1000;
pub const DS_D2K_VA: u64 = 0x4000_2000;
pub const DS_SCRATCH_VA: u64 = 0x4000_3000;
pub const DS_SCRATCH_SIZE: usize = 4096;
pub const DS_SWITCH_VA: u64 = 0x4000_4000;

/// Commands understood by the kernel driver-space service.
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
    JackQuery = 40,
    JackSetAmp = 41,
    AudioPlay = 42,
    AudioStop = 43,
    AudioInfo = 48,
    PagePhys = 49,
}

/// Header of a driver-space ring.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DsRing {
    pub head: u64,
    pub tail: u64,
    pub cap: u64,
}

pub const DS_RING_HDR_SIZE: usize = core::mem::size_of::<DsRing>();

/// A single message exchanged between kernel and driver space.
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

/// Parameters handed to driver space on boot (at `DS_INIT_PARAMS_VA`).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DsInitParams {
    pub magic: u64,
    pub version: u32,
    pub pad: u32,
    pub k2d_va: u64,
    pub d2k_va: u64,
    pub ring_cap: u64,
    pub ds_va_base: u64,
    pub ds_va_size: u64,
}

/// Library-side error type for failed driver-space calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsError {
    Timeout,
    QueueFull,
    BadStatus(i32),
    NoAspace,
    NoMemory,
    NotPrepared,
}

impl From<i32> for DsError {
    fn from(v: i32) -> Self {
        DsError::BadStatus(v)
    }
}