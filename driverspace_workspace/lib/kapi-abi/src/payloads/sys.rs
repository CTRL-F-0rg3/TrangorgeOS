//! 系统级消息载荷（日志等）。

/// 驱动通过 Manager 转发日志消息 (`Opcode::SysLog`)。
///
/// 结构体本身只描述元数据；`module` 与 `msg` 的字节内容紧跟在载荷之后
/// 放在同一个 IPC 缓冲区里。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LogPayload {
    /// 日志级别（对应 `ds-log` 的 `LogLevel`）
    pub level: u8,

    /// 模块名长度（字节）
    pub module_len: u16,

    /// 消息正文长度（字节）
    pub msg_len: u16,

    /// 保留字段，用于对齐
    pub _pad: u32,
}

/// 驱动向内核索取 ACPI SDT 表的请求（`DsCmd::SysAcpiTable`）。
///
/// 驱动不能自己遍历 RSDT——那属于内核的固件职责。驱动只描述自己需要哪张表，
/// 由内核负责定位、校验并把表映射到调用方的地址空间。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AcpiTableRequest {
    /// 四字符签名的 LE `u32`，例如 `u32::from_le_bytes(*b"DMAR")`。
    pub signature: u32,
    /// 同签名多实例时的序号，`0` 表示第一个。
    pub instance: u32,
    pub _pad: u64,
}

impl AcpiTableRequest {
    /// 由四字符签名构造请求。
    #[inline]
    pub const fn new(signature: [u8; 4], instance: u32) -> Self {
        Self {
            signature: u32::from_le_bytes(signature),
            instance,
            _pad: 0,
        }
    }

    /// 请求中的四字符签名。
    #[inline]
    pub const fn signature_bytes(self) -> [u8; 4] {
        self.signature.to_le_bytes()
    }
}

/// 内核回复 `DsCmd::SysAcpiTable` 时使用的寄存器约定。
///
/// 回复里 `DsMsg::arg0` = 表在调用方地址空间中的虚拟基址，
/// `DsMsg::arg1` = 表长度（字节）。表在调用方释放前必须保持可读。
pub mod acpi_table_reply {
    /// `DsMsg::arg0`：映射后的虚拟基址。
    pub const VIRT_BASE: usize = 0;
    /// `DsMsg::arg1`：表长度（字节）。
    pub const LENGTH: usize = 1;
}
