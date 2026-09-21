#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SysOpcode {
    MemObjectCreate = 0x0100,
    MemObjectDestroy = 0x0101,
    MemMap = 0x0102,
    MemUnmap = 0x0103,
    MemGrant = 0x0104,

    CapCreate = 0x0200,
    CapDerive = 0x0201,
    CapTransfer = 0x0202,
    CapRevoke = 0x0203,

    ChannelCreate = 0x0300,
    ChannelDestroy = 0x0301,
    ChannelBind = 0x0302,

    IpcSend = 0x0400,
    IpcRecv = 0x0401,
    IpcCall = 0x0402,
    IpcReply = 0x0403,
    IpcNotify = 0x0404,
    IpcWait = 0x0405,

    ShmemCreate = 0x0500,
    ShmemAttach = 0x0501,
    ShmemDetach = 0x0502,
}