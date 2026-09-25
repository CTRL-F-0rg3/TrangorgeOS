//! IOMMU 相关的 IPC 消息载荷。
//!
//! IOMMU 驱动与 `ds-manager` 之间通过这些结构交换控制器描述符、地址空间域、
//! 绑定关系与映射请求。所有结构体严格 `#[repr(C)]`，因为它们会在共享内存中
//! 被 Rust / C / Odin 共同解释。
//!
//! 所有请求都遵循与 `LogPayload` 相同的约定：`DsMsg::arg0` 携带载荷指针，
//! `DsMsg::arg1` 携带长度（字节）。

use bitflags::bitflags;

/// IOMMU 实现族。
///
/// 与 `drivers/iommu-driver` 的 `ControllerKind` 一一对应；新增架构时两边
/// 必须同步扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IommuKind {
    IntelVtd = 0,
    AmdVi = 1,
    ArmSmmuV2 = 2,
    ArmSmmuV3 = 3,
    RiscvIommu = 4,
    Unknown = 0xFFFF,
}

impl IommuKind {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::IntelVtd,
            1 => Self::AmdVi,
            2 => Self::ArmSmmuV2,
            3 => Self::ArmSmmuV3,
            4 => Self::RiscvIommu,
            _ => Self::Unknown,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IntelVtd => "intel-vtd",
            Self::AmdVi => "amd-vi",
            Self::ArmSmmuV2 => "arm-smmu-v2",
            Self::ArmSmmuV3 => "arm-smmu-v3",
            Self::RiscvIommu => "riscv-iommu",
            Self::Unknown => "unknown",
        }
    }
}

/// 翻译阶段：单元级直通、阶段二（嵌套虚拟化）或嵌套域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IommuStage {
    Stage1 = 0,
    Stage2 = 1,
    Nested = 2,
}

impl IommuStage {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Stage1,
            1 => Self::Stage2,
            2 => Self::Nested,
            _ => Self::Stage1,
        }
    }
}

/// 失效（cache/TLB invalidation）作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IommuInvalidateScope {
    Global = 0,
    Domain = 1,
    Leaf = 2,
    Device = 3,
    DeviceLeaf = 4,
}

impl IommuInvalidateScope {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Global,
            1 => Self::Domain,
            2 => Self::Leaf,
            3 => Self::Device,
            4 => Self::DeviceLeaf,
            _ => Self::Global,
        }
    }
}

bitflags! {
    /// IOVA -> PA 叶子映射的访问权限。
    ///
    /// 这是 wire 层的权限词汇；驱动负责把它翻译成具体 IOMMU 的
    /// 第二级 PTE 位域。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct IommuPermission: u32 {
        const NONE = 0;
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const PRIVILEGED = 1 << 3;
        const RW = (1 << 0) | (1 << 1);
        const RWE = (1 << 0) | (1 << 1) | (1 << 2);
    }
}

bitflags! {
    /// 映射请求的行为修饰位。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct IommuMapFlags: u32 {
        const NONE = 0;
        /// 把 `IommuMapPayload::phys_base` 视为调用方已拥有的物理页
        /// （`remap` 语义）；未设置时由驱动分配物理页并在 `arg0` 返回基址。
        const FIXED = 1 << 0;
        const COHERENT = 1 << 1;
        const PINNED = 1 << 2;
    }
}

/// 一个已发现的 IOMMU 控制器的不可变描述。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuControllerInfo {
    /// 驱动私有的控制器索引，`0..controller_count`。
    pub controller: u32,
    pub kind: IommuKind,
    /// 与驱动 `CapabilityFlags` 的位定义一致。
    pub capabilities: u64,
    pub stage: IommuStage,
    /// PCI segment；`0xFFFF` 表示该单元不属于某个 segment。
    pub segment: u32,
    pub mmio_base: u64,
    pub mmio_size: u64,
}

/// 建立 IOVA -> PA 映射的请求。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuMapPayload {
    /// 目标地址空间域，由 `IommuDomainCreate` 返回。
    pub domain: u32,
    pub flags: IommuMapFlags,
    pub permission: IommuPermission,
    pub _pad: u32,
    /// 设备可见地址（IOVA）区间起点。
    pub iova: u64,
    /// `FIXED` 时为物理基址，否则驱动忽略该字段。
    pub phys_base: u64,
    /// 区间长度（字节），必须是设备支持的粒度的整数倍。
    pub size: u64,
}

/// 撤销 IOVA 映射的请求。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuUnmapPayload {
    pub domain: u32,
    pub _pad: u32,
    pub iova: u64,
    pub size: u64,
}

/// 把一个 PCI requester 绑定到地址空间域的请求。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuBindPayload {
    pub controller: u32,
    /// 打包后的 requester id：`(segment << 16) | bdf`，与驱动
    /// `PciDevice` 的表示一致。
    pub requester: u32,
    pub domain: u32,
    /// `BindingSelector` 的线缆化表示：0 = Default，1 = AddrSpace(id)。
    pub selector: u32,
}

/// 显式失效请求。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuInvalidatePayload {
    pub scope: IommuInvalidateScope,
    pub controller: u32,
    pub domain: u32,
    /// `Device` / `DeviceLeaf` 作用域下有效。
    pub requester: u32,
    /// 失效粒度（字节），`Leaf` / `DeviceLeaf` 作用域下有效。
    pub granule_bytes: u32,
    pub count_pages: u32,
    pub _pad: u32,
    /// `Leaf` / `DeviceLeaf` 作用域下有效。
    pub iova: u64,
}

/// 固件声明、必须原样保留的 DMA 保留区。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuReservedRegionPayload {
    pub base: u64,
    /// 闭区间上界。
    pub limit: u64,
    /// 拥有该保留区的 requester；`0xFFFFFFFF` 表示无归属。
    pub requester: u32,
    pub _pad: u32,
}

/// 一次已上报的 IOMMU 故障。
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IommuFaultPayload {
    pub controller: u32,
    /// 硬件原因码（VT-d `FSTS` / IVRS `IOTL` 等的原始位域）。
    pub reason: u32,
    pub requester: u32,
    pub _pad: u32,
    pub iova: u64,
    /// 触发故障的物理地址，硬件不提供时为 0。
    pub faulting_phys: u64,
}
