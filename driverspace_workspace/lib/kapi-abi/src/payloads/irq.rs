//! 中断绑定相关的 IPC 消息载荷。

/// 驱动请求绑定/解绑 IRQ (`Opcode::IrqBind` / `Opcode::IrqUnbind`)。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IrqBindPayload {
    /// IRQ / MSI vector 编号
    pub irq_number: u32,

    /// 触发方式与优先级标志
    pub flags: u32,

    /// 保留字段，用于对齐
    pub _pad: u32,
}

