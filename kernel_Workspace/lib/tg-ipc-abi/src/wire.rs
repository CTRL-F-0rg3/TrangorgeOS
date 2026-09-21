#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgKind {
    Data = 0,
    Request = 1,
    Reply = 2,
    CapTransfer = 3,
    Notification = 4,
    Control = 5,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MsgHeader {
    pub kind: MsgKind,
    pub flags: u8,
    pub payload_len: u16,
    pub seq: u32,
    pub source_handle: u32,
    pub target_handle: u32,
}

impl MsgHeader {
    pub const SIZE: usize = core::mem::size_of::<Self>();

    pub const FLAG_URGENT: u8 = 1 << 0;
    pub const FLAG_NO_REPLY: u8 = 1 << 1;
    pub const FLAG_CAP_ATTACHED: u8 = 1 << 2;

    #[inline]
    pub fn new(kind: MsgKind, payload_len: u16, seq: u32) -> Self {
        Self {
            kind,
            flags: 0,
            payload_len,
            seq,
            source_handle: 0,
            target_handle: 0,
        }
    }

    #[inline]
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags |= flags;
        self
    }

    #[inline]
    pub fn total_len(&self) -> usize {
        Self::SIZE + self.payload_len as usize
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CapTransferBody {
    pub obj_id: u64,
    pub rights: u32,
    pub offset_pages: u32,
    pub length_pages: u32,
    pub _pad: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NotificationBody {
    pub event_id: u64,
    pub data: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IpcError {
    Ok = 0,
    InvalidHandle = 1,
    NoSpace = 2,
    NoData = 3,
    PermissionDenied = 4,
    Timeout = 5,
    InvalidMsg = 6,
    ChannelClosed = 7,
}