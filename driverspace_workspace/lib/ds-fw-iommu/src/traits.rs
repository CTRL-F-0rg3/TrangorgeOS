//! The IOMMU device-class contract.
//!
//! Drivers (`drivers/iommu-driver`) implement this trait; `IommuService`
//! routes IPC requests onto these methods. The framework itself contains no
//! hardware knowledge.
//!
//! ## Domain and mapping ownership
//!
//! Domains are allocated by the driver ([`IommuDevice::domain_create`]) and the
//! caller holds the resulting [`DomainId`]. Mapping requests **always carry a
//! domain**, which is what keeps one device's IOVA space from being shared by
//! accident with another's. A caller that wants pass-through semantics should
//! create a domain and map everything with `IommuPermission::RWE` instead of
//! reaching for a domain-less side path.

use kapi_abi::{
    DsError,
    payloads::iommu::{
        IommuBindPayload, IommuControllerInfo, IommuFaultPayload, IommuInvalidatePayload,
        IommuInvalidateScope, IommuMapPayload, IommuReservedRegionPayload, IommuUnmapPayload,
    },
};

use crate::types::{ControllerId, DomainId, RequesterId};

/// An IOMMU device that `ds-manager` can schedule.
///
/// Implementations are `no_std` and self-contained: every host resource (MMIO,
/// DMA memory, ACPI tables, PCI config access) must be requested from
/// `ds-manager` with a `DsCmd`; touching physical addresses directly is not
/// allowed.
pub trait IommuDevice {
    // ── discovery ────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Number of controllers the driver exposes.
    fn controller_count(&self) -> usize;

    /// Read the descriptor of controller `index`.
    ///
    /// Returns `false` and leaves `out` untouched when out of range.
    fn controller_info(&self, index: usize, out: &mut IommuControllerInfo) -> bool;

    // ── domain lifecycle ───────────────────────────────────────────────────────────────────────────────────────────

    /// Create an address-space domain on `controller`.
    ///
    /// `hint` is the domain number the caller would prefer (usually 0, meaning
    /// "let the driver pick"). `Err(DsError::OutOfMemory)` means the hardware
    /// ran out of domains.
    fn domain_create(
        &mut self,
        controller: ControllerId,
        hint: u32,
    ) -> Result<DomainId, DsError>;

    /// Destroy a domain. Implementations must have dropped all of its mappings first.
    fn domain_destroy(&mut self, domain: DomainId) -> Result<(), DsError>;

    // ── binding ──────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Bind `IommuBindPayload::requester` to `domain`.
    fn bind(&mut self, request: &IommuBindPayload) -> Result<(), DsError>;

    /// Unbind a requester, returning it to pass-through / untranslated state.
    fn unbind(&mut self, controller: ControllerId, requester: RequesterId)
        -> Result<(), DsError>;

    // ── mapping ──────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Establish an IOVA mapping.
    ///
    /// Returns the physical base actually used: with `FIXED` it equals the
    /// requested `phys_base`, otherwise the driver allocated it.
    fn map(&mut self, request: &IommuMapPayload) -> Result<u64, DsError>;

    /// Tear down an IOVA mapping. The range must match a previous `map`.
    fn unmap(&mut self, request: &IommuUnmapPayload) -> Result<(), DsError>;

    /// Explicit invalidation, returning the scope the hardware completed (possibly wider than requested).
    fn invalidate(
        &mut self,
        request: &IommuInvalidatePayload,
    ) -> Result<IommuInvalidateScope, DsError>;

    // ── firmware reservations ──────────────────────────────────────────────────────────────────────────────────────

    /// Number of firmware-declared DMA regions that must be preserved.
    fn reserved_region_count(&self) -> usize;

    /// Read reservation `index`; returns `false` when out of range.
    fn reserved_region(&self, index: usize, out: &mut IommuReservedRegionPayload) -> bool;

    // ── fault reporting ──────────────────────────────────────────────────────────────────────────────────────────────

    /// Number of faults that have not been read yet.
    fn pending_faults(&self) -> usize;

    /// Read fault `index`; returns `false` when out of range.
    fn read_fault(&mut self, index: usize, out: &mut IommuFaultPayload) -> bool;
}
