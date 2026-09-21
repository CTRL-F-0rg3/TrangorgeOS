pub const LINUX_EPERM: u32 = 1;
pub const LINUX_ENOENT: u32 = 2;
pub const LINUX_ESRCH: u32 = 3;
pub const LINUX_EINTR: u32 = 4;
pub const LINUX_EIO: u32 = 5;
pub const LINUX_ENOMEM: u32 = 12;
pub const LINUX_EACCES: u32 = 13;
pub const LINUX_EFAULT: u32 = 14;
pub const LINUX_EBUSY: u32 = 16;
pub const LINUX_EINVAL: u32 = 22;

pub const TG_OK: u32 = 0;
pub const TG_ERR_INVALID_HANDLE: u32 = 1;
pub const TG_ERR_NO_SPACE: u32 = 2;
pub const TG_ERR_NO_DATA: u32 = 3;
pub const TG_ERR_PERMISSION_DENIED: u32 = 4;
pub const TG_ERR_TIMEOUT: u32 = 5;
pub const TG_ERR_INVALID_MSG: u32 = 6;

pub fn to_linux_errno(tg_status: u32) -> u32 {
    match tg_status {
        TG_OK => 0,
        TG_ERR_PERMISSION_DENIED => LINUX_EACCES,
        TG_ERR_INVALID_HANDLE | TG_ERR_INVALID_MSG => LINUX_EINVAL,
        TG_ERR_NO_SPACE => LINUX_ENOMEM,
        TG_ERR_TIMEOUT => LINUX_EBUSY,
        _ => LINUX_EIO,
    }
}