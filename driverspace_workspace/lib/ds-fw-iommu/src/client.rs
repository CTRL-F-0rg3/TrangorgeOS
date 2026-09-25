//! The IOMMU client: encodes framework calls as `DsCmd` IPC.
//!
//! Modelled on `ds-ipc::ManagerClient`: it only ships a request out and reads the
//! reply back, and holds no device state. Callers get a [`DsError`] they can
//! propagate directly.
//!
//! ## Payload lifetime
//!
//! Requests with a payload put the address of a stack struct in `arg0` and its
//! length in `arg1`. That relies on the synchronous semantics of
//! `kapi-syscall::sys_ipc_call`: the kernel has already read the payload by the
//! time the reply arrives.

use kapi_abi::{
    DsCmd, DsError, Handle,
    payloads::iommu::{
        IommuBindPayload, IommuControllerInfo, IommuFaultPayload, IommuInvalidatePayload,
        IommuInvalidateScope, IommuMapPayload, IommuReservedRegionPayload, IommuUnmapPayload,
    },
    wire::Decoder,
};
use kapi_syscall::sys_ipc_call;

use crate::{codec, types::*, MAX_PAYLOAD_LEN};

/// Handle of the IOMMU service endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IommuClient {
    service: Handle,
}

impl IommuClient {
    #[inline]
    pub const fn new(service: Handle) -> Self {
        Self { service }
    }

    #[inline]
    pub const fn endpoint(&self) -> Handle {
        self.service
    }

    // ── discovery ────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Total number of controllers.
    pub fn controller_count(&self) -> Result<u32, DsError> {
        let reply = sys_ipc_call(DsCmd::IommuEnumerate, 0, 0, 0);
        check(reply.status).map(|_| reply.arg0 as u32)
    }

    /// Read the descriptor of controller `index`.
    pub fn controller_info(&self, index: u32) -> Result<IommuControllerInfo, DsError> {
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let reply = sys_ipc_call(DsCmd::IommuQueryController, index as u64, 0, 0);
        check(reply.status)?;

        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_controller_info(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    // ── domains ────────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Create a domain on `controller`; the driver picks the domain number.
    pub fn create_domain(&self, controller: ControllerId) -> Result<DomainId, DsError> {
        self.create_domain_hinted(controller, 0)
    }

    /// [`Self::create_domain`] with a preferred domain number.
    pub fn create_domain_hinted(
        &self,
        controller: ControllerId,
        hint: u32,
    ) -> Result<DomainId, DsError> {
        let reply =
            sys_ipc_call(DsCmd::IommuDomainCreate, controller.index() as u64, hint as u64, 0);
        let domain = check(reply.status).map(|_| DomainId(reply.arg0 as u32))?;
        if !domain.is_valid() {
            return Err(DsError::DeviceFault);
        }
        Ok(domain)
    }

    /// Destroy a domain.
    pub fn destroy_domain(&self, domain: DomainId) -> Result<(), DsError> {
        if !domain.is_valid() {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(DsCmd::IommuDomainDestroy, domain.raw() as u64, 0, 0);
        check(reply.status)
    }

    // ── binding ──────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Bind `requester` to `domain`.
    pub fn bind(
        &self,
        controller: ControllerId,
        requester: RequesterId,
        domain: DomainId,
    ) -> Result<(), DsError> {
        if !requester.is_valid() || !domain.is_valid() {
            return Err(DsError::InvalidMessage);
        }
        let payload = IommuBindPayload {
            controller: controller.index(),
            requester: requester.raw(),
            domain: domain.raw(),
            selector: 0,
        };
        let reply = sys_ipc_call(
            DsCmd::IommuBind,
            &payload as *const _ as u64,
            core::mem::size_of::<IommuBindPayload>() as u64,
            0,
        );
        check(reply.status)
    }

    /// Unbind a requester.
    pub fn unbind(&self, controller: ControllerId, requester: RequesterId) -> Result<(), DsError> {
        if !requester.is_valid() {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(
            DsCmd::IommuUnbind,
            controller.index() as u64,
            requester.raw() as u64,
            0,
        );
        check(reply.status)
    }
}

impl IommuClient {
    // ── mapping ──────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Establish an IOVA mapping, returning the physical base actually used.
    pub fn map(&self, request: &IommuMapPayload) -> Result<u64, DsError> {
        if request.size == 0 || request.size & 0xFFF != 0 {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(
            DsCmd::IommuMap,
            request as *const _ as u64,
            core::mem::size_of::<IommuMapPayload>() as u64,
            0,
        );
        check(reply.status).map(|_| reply.arg0)
    }

    /// Tear down an IOVA mapping.
    pub fn unmap(&self, request: &IommuUnmapPayload) -> Result<(), DsError> {
        if request.size == 0 || request.size & 0xFFF != 0 {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(
            DsCmd::IommuUnmap,
            request as *const _ as u64,
            core::mem::size_of::<IommuUnmapPayload>() as u64,
            0,
        );
        check(reply.status)
    }

    /// Explicit invalidation, returning the scope the hardware completed.
    pub fn invalidate(
        &self,
        request: &IommuInvalidatePayload,
    ) -> Result<IommuInvalidateScope, DsError> {
        let reply = sys_ipc_call(
            DsCmd::IommuInvalidate,
            request as *const _ as u64,
            core::mem::size_of::<IommuInvalidatePayload>() as u64,
            0,
        );
        check(reply.status).map(|_| IommuInvalidateScope::from_u32(reply.arg0 as u32))
    }

    // ── firmware reservations ──────────────────────────────────────────────────────────────────────────────────────

    /// Total number of reserved regions.
    pub fn reserved_region_count(&self) -> Result<u32, DsError> {
        let reply = sys_ipc_call(DsCmd::IommuReservedRegions, u32::MAX as u64, 0, 0);
        check(reply.status).map(|_| reply.arg0 as u32)
    }

    /// Read reserved region `index`.
    pub fn reserved_region(&self, index: u32) -> Result<IommuReservedRegionPayload, DsError> {
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let reply = sys_ipc_call(DsCmd::IommuReservedRegions, index as u64, 0, 0);
        check(reply.status)?;
        if reply.arg2 == 0 {
            return Err(DsError::DeviceNotFound);
        }
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_reserved_region(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    // ── faults ───────────────────────────────────────────────────────────────────────────────────────────────────────────

    /// Number of pending faults.
    pub fn pending_faults(&self) -> Result<u32, DsError> {
        let reply = sys_ipc_call(DsCmd::IommuFaultRead, u32::MAX as u64, 0, 0);
        check(reply.status).map(|_| reply.arg0 as u32)
    }

    /// Read fault `index`.
    pub fn read_fault(&self, index: u32) -> Result<IommuFaultPayload, DsError> {
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let reply = sys_ipc_call(DsCmd::IommuFaultRead, index as u64, 0, 0);
        check(reply.status)?;
        if reply.arg2 == 0 {
            return Err(DsError::DeviceNotFound);
        }
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_fault(&mut decoder).ok_or(DsError::BufferTooSmall)
    }
}

/// `Ok` when the reply status is 0, otherwise translated into a [`DsError`].
#[inline]
fn check(status: i32) -> Result<(), DsError> {
    if status == 0 {
        Ok(())
    } else {
        Err(DsError::from_u32(status as u32))
    }
}

/// Extract the payload length from a reply, clamped to the local buffer size.
#[inline]
fn payload_len(reply: &kapi_abi::DsMsg) -> usize {
    let len = reply.arg2 as usize;
    if len > MAX_PAYLOAD_LEN {
        MAX_PAYLOAD_LEN
    } else {
        len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_arguments_are_rejected_before_the_ipc_round_trip() {
        let client = IommuClient::new(Handle::MANAGER);

        assert_eq!(
            client.destroy_domain(DomainId::INVALID).unwrap_err(),
            DsError::InvalidMessage
        );
        assert_eq!(
            client.bind(ControllerId(0), RequesterId::NONE, DomainId(1)).unwrap_err(),
            DsError::InvalidMessage
        );
        assert_eq!(
            client.bind(ControllerId(0), RequesterId(1), DomainId::INVALID).unwrap_err(),
            DsError::InvalidMessage
        );
        assert_eq!(
            client.unbind(ControllerId(0), RequesterId::NONE).unwrap_err(),
            DsError::InvalidMessage
        );
    }

    #[test]
    fn misaligned_map_requests_are_rejected() {
        let client = IommuClient::new(Handle::MANAGER);
        let request = IommuMapPayload {
            domain: 1,
            flags: kapi_abi::payloads::iommu::IommuMapFlags::FIXED,
            permission: kapi_abi::payloads::iommu::IommuPermission::RW,
            _pad: 0,
            iova: 0x1000,
            phys_base: 0,
            size: 0x1800, // not page-aligned
        };
        assert_eq!(client.map(&request).unwrap_err(), DsError::InvalidMessage);
    }

    #[test]
    fn status_translation_covers_the_wire_vocabulary() {
        assert!(check(0).is_ok());
        assert_eq!(
            check(DsError::PermissionDenied as i32).unwrap_err(),
            DsError::PermissionDenied
        );
        assert_eq!(
            check(DsError::InvalidMessage as i32).unwrap_err(),
            DsError::InvalidMessage
        );
    }

    #[test]
    fn payload_length_is_clamped_to_the_local_buffer() {
        let mut reply = kapi_abi::DsMsg::new(DsCmd::IommuQueryController as u32);
        reply.arg2 = MAX_PAYLOAD_LEN as u64;
        assert_eq!(payload_len(&reply), MAX_PAYLOAD_LEN);

        reply.arg2 = u64::MAX;
        assert_eq!(payload_len(&reply), MAX_PAYLOAD_LEN);
    }
}

