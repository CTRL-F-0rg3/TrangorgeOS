use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum KernelError {
    Ok = 0,
    NoMemory = -1,
    InvalidArg = -2,
    NotFound = -3,
    AlreadyExists = -4,
    PermissionDenied = -5,
    Busy = -6,
    Timeout = -7,
    NotSupported = -8,
    IoError = -9,
    BadAddress = -10,
    Deadlock = -11,
    Interrupted = -12,
    WouldBlock = -13,
    NoSpace = -14,
    NotEmpty = -15,
    Overflow = -16,
    HardwareFault = -17,
    Unknown = -128,
}

impl KernelError {
    #[inline]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    #[inline]
    pub const fn code(self) -> i32 {
        self as i32
    }

    #[inline]
    pub const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            -1 => Self::NoMemory,
            -2 => Self::InvalidArg,
            -3 => Self::NotFound,
            -4 => Self::AlreadyExists,
            -5 => Self::PermissionDenied,
            -6 => Self::Busy,
            -7 => Self::Timeout,
            -8 => Self::NotSupported,
            -9 => Self::IoError,
            -10 => Self::BadAddress,
            -11 => Self::Deadlock,
            -12 => Self::Interrupted,
            -13 => Self::WouldBlock,
            -14 => Self::NoSpace,
            -15 => Self::NotEmpty,
            -16 => Self::Overflow,
            -17 => Self::HardwareFault,
            _ => Self::Unknown,
        }
    }
}

impl From<kstd_base::Status> for KernelError {
    fn from(s: kstd_base::Status) -> Self {
        match s {
            kstd_base::Status::Success => Self::Ok,
            kstd_base::Status::OutOfMemory => Self::NoMemory,
            kstd_base::Status::InvalidArgument => Self::InvalidArg,
            kstd_base::Status::NotFound => Self::NotFound,
            kstd_base::Status::AlreadyExists => Self::AlreadyExists,
            kstd_base::Status::PermissionDenied => Self::PermissionDenied,
            kstd_base::Status::Busy => Self::Busy,
            kstd_base::Status::TimedOut => Self::Timeout,
            kstd_base::Status::NotSupported => Self::NotSupported,
            kstd_base::Status::IOError => Self::IoError,
            kstd_base::Status::BadAddress => Self::BadAddress,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}