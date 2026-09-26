//! IOMMU IPC message payloads.
//!
//! The IOMMU driver and `ds-manager` exchange controller descriptors,
//! address-space domains, requester bindings and mapping requests through
//! these structures. Every struct is strictly `#[repr(C)]`, because they are
//! interpreted by Rust, C and Odin alike in shared memory.
//!
//! All requests follow the same convention as `LogPayload`: `DsMsg::arg0`
//! carries the payload pointer and `DsMsg::arg1` the length in bytes.

use bitflags::bitflags;

/// IOMMU implementation family.
///
/// Mirrors `ControllerKind` in `drivers/iommu-driver`; both sides must be
/// extended together when a new architecture is added.
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

/// Translation stage: unit-level pass-through, stage 2 (nested
/// virtualization) or a nested domain.
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

/// Invalidation (cache/TLB flush) scope.
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
    /// Access permissions of an IOVA -> PA leaf mapping.
    ///
    /// This is the wire-level permission vocabulary; the driver translates it
    /// into the second-level PTE bitfields of the concrete IOMMU.
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
    /// Behaviour modifiers for a mapping request.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct IommuMapFlags: u32 {
        const NONE = 0;
        /// Treat `IommuMapPayload::phys_base` as a physical page the caller
        /// already owns (`remap` semantics); when unset the driver allocates
        /// the page and returns its base in `arg0`.
        const FIXED = 1 << 0;
        const COHERENT = 1 << 1;
        const PINNED = 1 << 2;
    }
}

/// Immutable description of one discovered IOMMU controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuControllerInfo {
    /// Driver-private controller index, `0..controller_count`.
    pub controller: u32,
    pub kind: IommuKind,
    /// Bit layout matches the driver's `CapabilityFlags`.
    pub capabilities: u64,
    pub stage: IommuStage,
    /// PCI segment; `0xFFFF` means the unit is not tied to a segment.
    pub segment: u32,
    pub mmio_base: u64,
    pub mmio_size: u64,
}

/// Request to establish an IOVA -> PA mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuMapPayload {
    /// Target address-space domain, as returned by `IommuDomainCreate`.
    pub domain: u32,
    pub flags: IommuMapFlags,
    pub permission: IommuPermission,
    pub _pad: u32,
    /// Start of the device-visible address (IOVA) range.
    pub iova: u64,
    /// Physical base when `FIXED` is set; ignored otherwise.
    pub phys_base: u64,
    /// Range length in bytes; must be a multiple of a granule the device supports.
    pub size: u64,
}

/// Request to tear down an IOVA mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuUnmapPayload {
    pub domain: u32,
    pub _pad: u32,
    pub iova: u64,
    pub size: u64,
}

/// Request to bind a PCI requester to an address-space domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuBindPayload {
    pub controller: u32,
    /// Packed requester id: `(segment << 16) | bdf`, matching the driver's
    /// `PciDevice` representation.
    pub requester: u32,
    pub domain: u32,
    /// Wire form of `BindingSelector`: 0 = Default, 1 = AddrSpace(id).
    pub selector: u32,
}

/// Explicit invalidation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuInvalidatePayload {
    pub scope: IommuInvalidateScope,
    pub controller: u32,
    pub domain: u32,
    /// Only meaningful for the `Device` / `DeviceLeaf` scopes.
    pub requester: u32,
    /// Invalidation granule in bytes; only for `Leaf` / `DeviceLeaf`.
    pub granule_bytes: u32,
    pub count_pages: u32,
    pub _pad: u32,
    /// Only meaningful for the `Leaf` / `DeviceLeaf` scopes.
    pub iova: u64,
}

/// A DMA reservation declared by firmware that the OS must preserve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuReservedRegionPayload {
    pub base: u64,
    /// Inclusive upper bound.
    pub limit: u64,
    /// Requester owning this reservation; `0xFFFFFFFF` means unowned.
    pub requester: u32,
    pub _pad: u32,
}

/// One reported IOMMU fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct IommuFaultPayload {
    pub controller: u32,
    /// Hardware reason code (raw bits of VT-d `FSTS` / IVRS `IOTL`, ...).
    pub reason: u32,
    pub requester: u32,
    pub _pad: u32,
    pub iova: u64,
    /// Faulting physical address; 0 when the hardware does not report it.
    pub faulting_phys: u64,
}
