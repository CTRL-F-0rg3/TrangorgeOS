//! IPC 消息操作码定义。
//! Manager 根据 Opcode 决定将消息转发给哪个子系统或驱动。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Opcode {
    // === 系统与控制 (0x0000 - 0x00FF) ===
    /// 驱动向 Manager 注册自己
    DriverRegister      = 0x0001,
    /// 驱动请求退出/注销
    DriverUnregister    = 0x0002,
    /// 心跳/保活消息
    Heartbeat           = 0x0003,

    // === 设备管理 (0x0100 - 0x01FF) ===
    /// 内核通知 Manager：检测到新设备
    DevAttach           = 0x0100,
    /// 内核通知 Manager：设备移除
    DevDetach           = 0x0101,
    /// Manager 通知驱动：接管该设备
    DevBind             = 0x0102,
    /// 驱动向 Manager 报告设备状态变化
    DevStatusChange     = 0x0103,

    // === 资源请求 (0x0200 - 0x02FF) ===
    /// 驱动请求映射 MMIO 内存
    MemMapMmio          = 0x0200,
    /// 驱动请求分配 DMA 缓冲区
    MemAllocDma         = 0x0201,
    /// 驱动请求绑定 IRQ 中断线
    IrqBind             = 0x0210,
    /// 驱动请求解除 IRQ 绑定
    IrqUnbind           = 0x0211,

    // === IPC 与同步 (0x0300 - 0x03FF) ===
    /// 请求创建新的 IPC 通道
    IpcCreateChannel    = 0x0300,
    /// 请求连接到某个 Port
    IpcConnectPort      = 0x0301,
}

impl Opcode {
    pub fn from_u32(val: u32) -> Option<Self> {
        // 在实际生产中，这里可以使用宏或 match 来安全转换
        match val {
            0x0001 => Some(Self::DriverRegister),
            0x0002 => Some(Self::DriverUnregister),
            0x0003 => Some(Self::Heartbeat),
            0x0100 => Some(Self::DevAttach),
            0x0101 => Some(Self::DevDetach),
            0x0102 => Some(Self::DevBind),
            0x0103 => Some(Self::DevStatusChange),
            0x0200 => Some(Self::MemMapMmio),
            0x0201 => Some(Self::MemAllocDma),
            0x0210 => Some(Self::IrqBind),
            0x0211 => Some(Self::IrqUnbind),
            0x0300 => Some(Self::IpcCreateChannel),
            0x0301 => Some(Self::IpcConnectPort),
            _ => None,
        }
    }
}