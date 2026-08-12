

#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Errno {
    Ok = 0,
    InvalidArg = 1,
    NotFound = 2,
    NoMemory = 3,
    IoError = 4,
    NotSupported = 5,
    PermissionDenied = 6,
    Busy = 7,
    BufferTooSmall = 8,
    BadHandle = 9,
    EndOfFile = 10,
}

pub type TResult<T> = Result<T, Errno>;