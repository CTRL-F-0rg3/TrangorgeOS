//! 设备管理相关的 IPC 消息载荷。
//! 所有结构体必须严格 #[repr(C)]，因为内核（可能是C/Rust）和驱动（Rust/C/Odin）共享这些内存。

use crate::primitives::PhysAddr;
use bitflags::bitflags;

/// 设备总线类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BusType {
    Pci     = 0,
    Usb     = 1,
    Acpi    = 2,
    Platform= 3, // 如 UART, 早期 Framebuffer
    Unknown = 0xFF,
}

bitflags! {
    /// 设备能力/属性标志
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct DeviceFlags: u32 {
        /// 设备支持 DMA (Direct Memory Access)
        const DMA_CAPABLE   = 1 << 0;
        /// 设备支持总线主控 (Bus Mastering)
        const BUS_MASTER    = 1 << 1;
        /// 设备是可热插拔的
        const HOTPLUGGABLE  = 1 << 2;
        /// 设备当前处于低功耗状态
        const LOW_POWER     = 1 << 3;
    }
}

/// 内核发送给 Manager 的 "设备接入" (DevAttach) 消息载荷。
/// 当内核枚举到一个新硬件时，会填充此结构并发送给 ds-manager。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DevAttachPayload {
    /// 内核分配的设备唯一标识符（在系统重启前有效）
    pub device_id: u64,
    
    /// 设备所在的总线类型
    pub bus_type: BusType,
    
    /// 设备标志（DMA, 热插拔等）
    pub flags: DeviceFlags,
    
    /// 供应商 ID (Vendor ID) - 例如 PCI 的 0x1002 (AMD)
    pub vendor_id: u32,
    
    /// 设备 ID (Device ID) - 例如 PCI 的具体型号
    pub device_id_hw: u32,
    
    /// 子类/接口类型 (Class Code / Subclass)
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    
    /// 设备所需的 MMIO 物理基地址（如果有）
    pub mmio_base: PhysAddr,
    
    /// MMIO 区域大小
    pub mmio_size: u64,
    
    /// 设备使用的 IRQ 号（如果是 PCI，可能是 MSI/MSI-X 的 vector）
    pub irq_number: u32,
    
    /// 保留字段，用于对齐和 future-proofing
    _reserved: [u64; 4], 
}