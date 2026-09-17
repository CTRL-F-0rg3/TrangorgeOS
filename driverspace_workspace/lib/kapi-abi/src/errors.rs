//! 统一的 Driverspace 错误码。

/// 核心错误码枚举。
/// 注意：必须保持 #[repr(u32)] 以便在 IPC 消息中直接传输。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DsError {
    /// 操作成功
    Success = 0,
    
    /// 通用/未知错误
    Unknown = 1,
    
    /// 提供的句柄无效或已释放
    InvalidHandle = 2,
    
    /// 内存不足（物理或虚拟）
    OutOfMemory = 3,
    
    /// 权限不足（缺少对应的 Capability）
    PermissionDenied = 4,
    
    /// 设备不存在或未找到
    DeviceNotFound = 5,
    
    /// 设备正忙（例如 IRQ 已被占用）
    DeviceBusy = 6,
    
    /// IPC 消息格式错误或 Opcode 不支持
    InvalidMessage = 7,
    
    /// 缓冲区太小，无法容纳数据
    BufferTooSmall = 8,
    
    /// 硬件/设备报告了致命故障
    DeviceFault = 9,
    
    /// 操作超时
    Timeout = 10,
}

impl DsError {
    /// 将 u32 转换为 DsError，如果值超出范围则返回 Unknown。
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => Self::Success,
            1 => Self::Unknown,
            2 => Self::InvalidHandle,
            3 => Self::OutOfMemory,
            4 => Self::PermissionDenied,
            5 => Self::DeviceNotFound,
            6 => Self::DeviceBusy,
            7 => Self::InvalidMessage,
            8 => Self::BufferTooSmall,
            9 => Self::DeviceFault,
            10 => Self::Timeout,
            _ => Self::Unknown,
        }
    }
}