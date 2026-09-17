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