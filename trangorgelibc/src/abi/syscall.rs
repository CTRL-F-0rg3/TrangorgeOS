
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syscall {
    Yield = 0,
    Reboot = 1,
    Shutdown = 2,
    GetSystemInfo = 3,

    Malloc = 10,
    Free = 11,

    ConsoleWrite = 20,
    ConsoleRead = 21,

    FsOpen = 30,
    FsRead = 31,
    FsWrite = 32,
    FsClose = 33,
    DeviceRegister = 40,
    DeviceIo = 41,

    IpcSend = 50,
    IpcRecv = 51,
}