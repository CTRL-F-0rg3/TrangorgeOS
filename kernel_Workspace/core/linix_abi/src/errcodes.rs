use kstd_base::Status;

// Standard Linux errno definitions (x86_64 subset)
pub const EPERM: u32 = 1;
pub const ENOENT: u32 = 2;
pub const ESRCH: u32 = 3;
pub const EINTR: u32 = 4;
pub const EIO: u32 = 5;
pub const ENXIO: u32 = 6;
pub const E2BIG: u32 = 7;
pub const ENOEXEC: u32 = 8;
pub const EBADF: u32 = 9;
pub const ECHILD: u32 = 10;
pub const EAGAIN: u32 = 11;
pub const ENOMEM: u32 = 12;
pub const EACCES: u32 = 13;
pub const EFAULT: u32 = 14;
pub const EBUSY: u32 = 16;
pub const EEXIST: u32 = 17;
pub const EINVAL: u32 = 22;
pub const ENOSYS: u32 = 38;

// Translate native kernel status to Linux errno for user-space compat
#[inline]
pub fn status_to_errno(s: Status) -> u32 {
    match s {
        Status::Success => 0,
        Status::OutOfMemory => ENOMEM,
        Status::InvalidArgument => EINVAL,
        Status::PermissionDenied => EACCES,
        Status::NotFound => ENOENT,
        Status::AlreadyExists => EEXIST,
        Status::Busy => EBUSY,
        Status::TimedOut => EAGAIN,
        Status::NotSupported => ENOSYS,
        Status::IOError => EIO,
        Status::BadAddress => EFAULT,
        _ => EPERM,
    }
}

// Translate Linux errno to native kernel status
#[inline]
pub fn errno_to_status(e: u32) -> Status {
    match e {
        0 => Status::Success,
        ENOMEM => Status::OutOfMemory,
        EINVAL => Status::InvalidArgument,
        EACCES | EPERM => Status::PermissionDenied,
        ENOENT => Status::NotFound,
        EEXIST => Status::AlreadyExists,
        EBUSY => Status::Busy,
        EAGAIN => Status::TimedOut,
        ENOSYS => Status::NotSupported,
        EIO => Status::IOError,
        EFAULT => Status::BadAddress,
        _ => Status::Unknown,
    }
}