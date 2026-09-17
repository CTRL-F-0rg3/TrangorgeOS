//! 内存管理相关的 IPC 消息载荷。

use crate::primitives::PhysAddr;

/// 驱动请求映射 MMIO 区域 (`Opcode::MemMapMmio`)。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MmioMapPayload {
    /// 设备 MMIO 区域的物理基地址
    pub phys_addr: PhysAddr,

    /// 需要映射的字节数
    pub size: u64,

    /// 请求的缓存/权限标志 (bit 0 = cacheable, bit 1 = writable ...)
    pub flags: u32,

    /// 保留字段，用于对齐
    pub _pad: u32,
}

/// 驱动请求分配 DMA 缓冲区 (`Opcode::MemAllocDma`)。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DmaAllocPayload {
    /// 需要分配的字节数
    pub size: u64,

    /// 对齐要求（字节）
    pub align: u64,

    /// 附加标志（一致性、可缓存等）
    pub flags: u32,

    /// 保留字段，用于对齐
    pub _pad: u32,
}

