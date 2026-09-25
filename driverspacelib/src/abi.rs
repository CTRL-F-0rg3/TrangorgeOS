//! Driver-space ABI on top of the independent `tg_comm` shared-memory protocol.
//!
//! The wire transport is the 64-byte `tg_comm::CommMsg` over a shared-memory
//! `tg_comm::SpscRing`, gated by the kernel's capability authorization. The
//! `DsCmd`/`DsMsg` types below are the *client-facing* convenience layer kept
//! for source compatibility with existing driver code.

// Re-export the protocol core so driver code can use it directly.
pub use tg_comm::{authorize, AuthorizeResult, CapId, CapTable, CommMsg, Layer,
                  MsgKind, OpClass, Rights, SpscRing, opcode};

pub const DS_MAGIC: u64 = 0x4452_5653_5041_4345;
pub const DS_VERSION: u32 = 1;
pub const DS_FLAG_RESPONSE: u32 = 1 << 0;

/// Fixed virtual addresses of the shared rings (mapped by the kernel).
pub const DS_INIT_PARAMS_VA: u64 = 0x4000_0000;
pub const DS_K2D_VA: u64 = 0x4000_1000; // kernel -> driver (replies)
pub const DS_D2K_VA: u64 = 0x4000_2000; // driver -> kernel (requests)
pub const DS_SCRATCH_VA: u64 = 0x4000_3000;
pub const DS_SCRATCH_SIZE: usize = 4096;
pub const DS_RING_SLOTS: u32 = 256;

/// Capability handles handed to the driver-space manager by the kernel.
pub const DS_CAP_DEVICE: u32 = 1; // CALL over a Device
pub const DS_CAP_MEMORY: u32 = 2; // CALL|MAP|WRITE|MANAGE over Memory

/// Commands understood by the kernel driver-space service (Sys class).
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

/// Client-facing response message (filled from a `CommMsg` reply).
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

impl DsMsg {
    pub const EMPTY: Self = Self {
        id: 0, cmd: 0, flags: 0, arg0: 0, arg1: 0, arg2: 0, status: 0, pad: 0,
    };
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
