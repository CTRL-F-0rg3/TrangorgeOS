#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Status {
    Success = 0,
    OutOfMemory = 1,
    InvalidArgument = 2,
    PermissionDenied = 3,
    NotFound = 4,
    AlreadyExists = 5,
    Busy = 6,
    TimedOut = 7,
    NotSupported = 8,
    IOError = 9,
    BadAddress = 10,
    Unknown = 0xFFFF_FFFF,
}

impl From<u32> for Status {
    #[inline]
    fn from(val: u32) -> Self {
        match val {
            0 => Self::Success,
            1 => Self::OutOfMemory,
            2 => Self::InvalidArgument,
            3 => Self::PermissionDenied,
            4 => Self::NotFound,
            5 => Self::AlreadyExists,
            6 => Self::Busy,
            7 => Self::TimedOut,
            8 => Self::NotSupported,
            9 => Self::IOError,
            10 => Self::BadAddress,
            _ => Self::Unknown,
        }
    }
}

impl From<Status> for u32 {
    #[inline]
    fn from(val: Status) -> Self {
        val as u32
    }
}

impl Status {
    #[inline]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Success)
    }
    
    #[inline]
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }
}