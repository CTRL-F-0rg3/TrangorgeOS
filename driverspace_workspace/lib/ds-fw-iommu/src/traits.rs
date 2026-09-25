//! IOMMU 设备类契约。
//!
//! 驱动（`drivers/iommu-driver`）实现本 trait；`IommuService` 负责把 IPC 请求
//! 路由到这些方法上。框架本身不含任何硬件知识。
//!
//! ## 域与映射的归属关系
//!
//! 域由驱动分配（[`IommuDevice::domain_create`]），请求方持有 [`DomainId`]。
//! 映射请求**总是带域**——这保证了 IOVA 空间不会在不同设备之间被无意共享。
//! 如果调用方需要“透传”，它应当创建一个域并使用
//! `IommuPermission::RWE` 做全量映射，而不是引入一个无域的旁路。

use kapi_abi::{
    DsError,
    payloads::iommu::{
        IommuBindPayload, IommuControllerInfo, IommuFaultPayload, IommuInvalidatePayload,
        IommuInvalidateScope, IommuMapPayload, IommuReservedRegionPayload, IommuUnmapPayload,
    },
};

use crate::types::{ControllerId, DomainId, RequesterId};

/// 一个可被 `ds-manager` 调度的 IOMMU 设备。
///
/// 实现方是 `no_std` 且自包含的：所有宿主资源（MMIO、DMA 内存、ACPI 表、
/// PCI 配置访问）都必须通过 `DsCmd` 向 `ds-manager` 申请，不能直接触碰物理地址。
pub trait IommuDevice {
    // ── 发现 ──────────────────────────────────────────────────────────

    /// 驱动暴露的控制器数量。
    fn controller_count(&self) -> usize;

    /// 读取 `index` 号控制器的描述。
    ///
    /// 索引越界时返回 `false` 且不修改 `out`。
    fn controller_info(&self, index: usize, out: &mut IommuControllerInfo) -> bool;

    // ── 域生命周期 ────────────────────────────────────────────────────

    /// 在 `controller` 上创建一个地址空间域。
    ///
    /// `hint` 是调用方偏好的域号（通常为 0，表示“由驱动分配”）。返回
    /// `Err(DsError::OutOfMemory)` 表示硬件域耗尽。
    fn domain_create(
        &mut self,
        controller: ControllerId,
        hint: u32,
    ) -> Result<DomainId, DsError>;

    /// 销毁域。销毁前实现必须先撤销其上的所有映射。
    fn domain_destroy(&mut self, domain: DomainId) -> Result<(), DsError>;

    // ── 绑定 ──────────────────────────────────────────────────────────

    /// 把 `IommuBindPayload::requester` 绑定到 `domain`。
    fn bind(&mut self, request: &IommuBindPayload) -> Result<(), DsError>;

    /// 解绑 requester，使其回到透传/无翻译状态。
    fn unbind(&mut self, controller: ControllerId, requester: RequesterId)
        -> Result<(), DsError>;

    // ── 映射 ──────────────────────────────────────────────────────────

    /// 建立 IOVA 映射。
    ///
    /// 返回实际使用的物理基址：请求带 `FIXED` 时它等于请求中的
    /// `phys_base`，否则由驱动分配。
    fn map(&mut self, request: &IommuMapPayload) -> Result<u64, DsError>;

    /// 撤销 IOVA 映射。区间必须与先前 `map` 的区间对齐。
    fn unmap(&mut self, request: &IommuUnmapPayload) -> Result<(), DsError>;

    /// 显式失效。返回硬件实际完成的（可能比请求更大的）作用域。
    fn invalidate(
        &mut self,
        request: &IommuInvalidatePayload,
    ) -> Result<IommuInvalidateScope, DsError>;

    // ── 固件保留区 ────────────────────────────────────────────────────

    /// 固件声明、必须原样保留的 DMA 区域数量。
    fn reserved_region_count(&self) -> usize;

    /// 读取第 `index` 个保留区。索引越界时返回 `false`。
    fn reserved_region(&self, index: usize, out: &mut IommuReservedRegionPayload) -> bool;

    // ── 故障上报 ──────────────────────────────────────────────────────

    /// 尚未被读取的故障数量。
    fn pending_faults(&self) -> usize;

    /// 读取第 `index` 个故障。索引越界时返回 `false`。
    fn read_fault(&mut self, index: usize, out: &mut IommuFaultPayload) -> bool;
}
