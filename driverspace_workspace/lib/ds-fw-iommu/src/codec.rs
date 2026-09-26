//! Wire encoding and decoding for IOMMU payloads.
//!
//! Every IOMMU request and reply uses one convention (the same one as
//! `kapi-abi::payloads::sys::LogPayload`): `DsMsg::arg0` carries the payload
//! pointer and `DsMsg::arg1` the length.
//!
//! Serialization goes through the `Encoder` / `Decoder` provided by
//! `kapi-abi::wire`, so byte order and bounds checking match the rest of the
//! system and the framework and the driver never grow a second implementation.

use kapi_abi::{
    payloads::iommu::{
        IommuBindPayload, IommuControllerInfo, IommuFaultPayload, IommuInvalidatePayload,
        IommuMapPayload, IommuReservedRegionPayload, IommuUnmapPayload,
    },
    wire::{Decoder, Encoder},
};

/// Worst-case encoded length of a single IOMMU payload, in bytes.
///
/// Derived from the field sum of the largest payload, `IommuInvalidatePayload`,
/// rounded up; used by the driver to size stack scratch buffers.
pub const MAX_PAYLOAD_LEN: usize = 48;

// ── encode ──────────────────────────────────────────────────────────

/// Encode an `IommuControllerInfo`.
pub fn encode_controller_info(enc: &mut Encoder<'_>, value: &IommuControllerInfo) -> bool {
    enc.write_u32(value.controller)
        && enc.write_u32(value.kind as u32)
        && enc.write_u64(value.capabilities)
        && enc.write_u32(value.stage as u32)
        && enc.write_u32(value.segment)
        && enc.write_u64(value.mmio_base)
        && enc.write_u64(value.mmio_size)
}
/// Encode an `IommuMapPayload`.
pub fn encode_map(enc: &mut Encoder<'_>, value: &IommuMapPayload) -> bool {
    enc.write_u32(value.domain)
        && enc.write_u32(value.flags.bits())
        && enc.write_u32(value.permission.bits())
        && enc.write_u32(0)
        && enc.write_u64(value.iova)
        && enc.write_u64(value.phys_base)
        && enc.write_u64(value.size)
}
/// Encode an `IommuUnmapPayload`.
pub fn encode_unmap(enc: &mut Encoder<'_>, value: &IommuUnmapPayload) -> bool {
    enc.write_u32(value.domain)
        && enc.write_u32(0)
        && enc.write_u64(value.iova)
        && enc.write_u64(value.size)
}
/// Encode an `IommuBindPayload`.
pub fn encode_bind(enc: &mut Encoder<'_>, value: &IommuBindPayload) -> bool {
    enc.write_u32(value.controller)
        && enc.write_u32(value.requester)
        && enc.write_u32(value.domain)
        && enc.write_u32(value.selector)
}
/// Encode an `IommuInvalidatePayload`.
pub fn encode_invalidate(enc: &mut Encoder<'_>, value: &IommuInvalidatePayload) -> bool {
    enc.write_u32(value.scope as u32)
        && enc.write_u32(value.controller)
        && enc.write_u32(value.domain)
        && enc.write_u32(value.requester)
        && enc.write_u32(value.granule_bytes)
        && enc.write_u32(value.count_pages)
        && enc.write_u32(0)
        && enc.write_u64(value.iova)
}
/// Encode an `IommuReservedRegionPayload`.
pub fn encode_reserved_region(enc: &mut Encoder<'_>, value: &IommuReservedRegionPayload) -> bool {
    enc.write_u64(value.base)
        && enc.write_u64(value.limit)
        && enc.write_u32(value.requester)
        && enc.write_u32(0)
}
/// Encode an `IommuFaultPayload`.
pub fn encode_fault(enc: &mut Encoder<'_>, value: &IommuFaultPayload) -> bool {
    enc.write_u32(value.controller)
        && enc.write_u32(value.reason)
        && enc.write_u32(value.requester)
        && enc.write_u32(0)
        && enc.write_u64(value.iova)
        && enc.write_u64(value.faulting_phys)
}

// ── decode ──────────────────────────────────────────────────────────

/// Decode an `IommuControllerInfo`.
pub fn decode_controller_info(dec: &mut Decoder<'_>) -> Option<IommuControllerInfo> {
    use kapi_abi::payloads::iommu::{IommuKind, IommuStage};
    Some(IommuControllerInfo {
        controller: dec.read_u32()?,
        kind: IommuKind::from_u32(dec.read_u32()?),
        capabilities: dec.read_u64()?,
        stage: IommuStage::from_u32(dec.read_u32()?),
        segment: dec.read_u32()?,
        mmio_base: dec.read_u64()?,
        mmio_size: dec.read_u64()?,
    })
}
/// Decode an `IommuMapPayload`.
pub fn decode_map(dec: &mut Decoder<'_>) -> Option<IommuMapPayload> {
    use kapi_abi::payloads::iommu::{IommuMapFlags, IommuPermission};
    Some(IommuMapPayload {
        domain: dec.read_u32()?,
        flags: IommuMapFlags::from_bits_truncate(dec.read_u32()?),
        permission: IommuPermission::from_bits_truncate(dec.read_u32()?),
        _pad: dec.read_u32()?,
        iova: dec.read_u64()?,
        phys_base: dec.read_u64()?,
        size: dec.read_u64()?,
    })
}
/// Decode an `IommuUnmapPayload`.
pub fn decode_unmap(dec: &mut Decoder<'_>) -> Option<IommuUnmapPayload> {
    Some(IommuUnmapPayload {
        domain: dec.read_u32()?,
        _pad: dec.read_u32()?,
        iova: dec.read_u64()?,
        size: dec.read_u64()?,
    })
}
/// Decode an `IommuBindPayload`.
pub fn decode_bind(dec: &mut Decoder<'_>) -> Option<IommuBindPayload> {
    Some(IommuBindPayload {
        controller: dec.read_u32()?,
        requester: dec.read_u32()?,
        domain: dec.read_u32()?,
        selector: dec.read_u32()?,
    })
}
/// Decode an `IommuInvalidatePayload`.
pub fn decode_invalidate(dec: &mut Decoder<'_>) -> Option<IommuInvalidatePayload> {
    use kapi_abi::payloads::iommu::IommuInvalidateScope;
    Some(IommuInvalidatePayload {
        scope: IommuInvalidateScope::from_u32(dec.read_u32()?),
        controller: dec.read_u32()?,
        domain: dec.read_u32()?,
        requester: dec.read_u32()?,
        granule_bytes: dec.read_u32()?,
        count_pages: dec.read_u32()?,
        _pad: dec.read_u32()?,
        iova: dec.read_u64()?,
    })
}
/// Decode an `IommuReservedRegionPayload`.
pub fn decode_reserved_region(dec: &mut Decoder<'_>) -> Option<IommuReservedRegionPayload> {
    Some(IommuReservedRegionPayload {
        base: dec.read_u64()?,
        limit: dec.read_u64()?,
        requester: dec.read_u32()?,
        _pad: dec.read_u32()?,
    })
}
/// Decode an `IommuFaultPayload`.
pub fn decode_fault(dec: &mut Decoder<'_>) -> Option<IommuFaultPayload> {
    Some(IommuFaultPayload {
        controller: dec.read_u32()?,
        reason: dec.read_u32()?,
        requester: dec.read_u32()?,
        _pad: dec.read_u32()?,
        iova: dec.read_u64()?,
        faulting_phys: dec.read_u64()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kapi_abi::payloads::iommu::{
        IommuInvalidateScope, IommuKind, IommuMapFlags, IommuPermission, IommuStage,
    };
    #[test]
    fn map_payload_round_trips() {
        let original = IommuMapPayload {
            domain: 3,
            flags: IommuMapFlags::FIXED | IommuMapFlags::COHERENT,
            permission: IommuPermission::RWE,
            _pad: 0,
            iova: 0x1_0000_0000,
            phys_base: 0x8000_0000,
            size: 0x20_0000,
        };
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut buf);
        assert!(encode_map(&mut enc, &original));
        let mut dec = Decoder::new(&buf);
        let decoded = decode_map(&mut dec).expect("decodes");
        assert_eq!(decoded.domain, original.domain);
        assert_eq!(decoded.flags, original.flags);
        assert_eq!(decoded.permission, original.permission);
        assert_eq!(decoded.iova, original.iova);
        assert_eq!(decoded.phys_base, original.phys_base);
        assert_eq!(decoded.size, original.size);
    }
    #[test]
    fn every_payload_fits_the_budget() {
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut buf);
        assert!(encode_invalidate(
            &mut enc,
            &IommuInvalidatePayload {
                scope: IommuInvalidateScope::DeviceLeaf,
                controller: 1,
                domain: 2,
                requester: 3,
                granule_bytes: 0x1000,
                count_pages: 4,
                _pad: 0,
                iova: 0x1000,
            }
        ));
        assert!(enc.pos() <= MAX_PAYLOAD_LEN);
        let mut enc = Encoder::new(&mut buf);
        assert!(encode_controller_info(
            &mut enc,
            &IommuControllerInfo {
                controller: 0,
                kind: IommuKind::IntelVtd,
                capabilities: 0x1f,
                stage: IommuStage::Stage1,
                segment: 0,
                mmio_base: 0xfed0_0000,
                mmio_size: 0x1000,
            }
        ));
        assert!(enc.pos() <= MAX_PAYLOAD_LEN);
    }
    #[test]
    fn truncated_payload_is_rejected() {
        let buf = [0u8; 4];
        let mut dec = Decoder::new(&buf);
        assert!(decode_map(&mut dec).is_none());
    }

    #[test]
    fn every_payload_round_trips() {
        use kapi_abi::payloads::iommu::{
            IommuBindPayload, IommuFaultPayload, IommuReservedRegionPayload, IommuUnmapPayload,
        };

        let mut buf = [0u8; MAX_PAYLOAD_LEN];

        // controller info
        let info = IommuControllerInfo {
            controller: 3,
            kind: IommuKind::RiscvIommu,
            capabilities: 0xdead_beef_0000_0001,
            stage: IommuStage::Nested,
            segment: 2,
            mmio_base: 0x9000_0000,
            mmio_size: 0x2000,
        };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_controller_info(&mut enc, &info));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_controller_info(&mut dec).unwrap();
        assert_eq!(back.controller, info.controller);
        assert_eq!(back.kind, info.kind);
        assert_eq!(back.capabilities, info.capabilities);
        assert_eq!(back.stage, info.stage);
        assert_eq!(back.segment, info.segment);
        assert_eq!(back.mmio_base, info.mmio_base);
        assert_eq!(back.mmio_size, info.mmio_size);

        // bind
        let bind = IommuBindPayload {
            controller: 1,
            requester: 0x0002_2a28,
            domain: 4,
            selector: 9,
        };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_bind(&mut enc, &bind));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_bind(&mut dec).unwrap();
        assert_eq!(back.requester, bind.requester);
        assert_eq!(back.domain, bind.domain);
        assert_eq!(back.selector, bind.selector);

        // unmap
        let unmap = IommuUnmapPayload { domain: 5, _pad: 0, iova: 0x1_0000, size: 0x2_0000 };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_unmap(&mut enc, &unmap));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_unmap(&mut dec).unwrap();
        assert_eq!(back.domain, unmap.domain);
        assert_eq!(back.iova, unmap.iova);
        assert_eq!(back.size, unmap.size);

        // invalidate
        let inv = IommuInvalidatePayload {
            scope: IommuInvalidateScope::DeviceLeaf,
            controller: 6,
            domain: 7,
            requester: 8,
            granule_bytes: 0x2000,
            count_pages: 0x10,
            _pad: 0,
            iova: 0x2000,
        };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_invalidate(&mut enc, &inv));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_invalidate(&mut dec).unwrap();
        assert_eq!(back.scope, inv.scope);
        assert_eq!(back.granule_bytes, inv.granule_bytes);
        assert_eq!(back.count_pages, inv.count_pages);
        assert_eq!(back.iova, inv.iova);

        // reserved region
        let region = IommuReservedRegionPayload {
            base: 0x1000,
            limit: 0x1fff,
            requester: 0x1234,
            _pad: 0,
        };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_reserved_region(&mut enc, &region));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_reserved_region(&mut dec).unwrap();
        assert_eq!(back.base, region.base);
        assert_eq!(back.limit, region.limit);
        assert_eq!(back.requester, region.requester);

        // fault
        let fault = IommuFaultPayload {
            controller: 1,
            reason: 0xdead,
            requester: 0x4321,
            _pad: 0,
            iova: 0x3000,
            faulting_phys: 0x4000,
        };
        let len = {
            let mut enc = Encoder::new(&mut buf);
            assert!(encode_fault(&mut enc, &fault));
            enc.pos()
        };
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_fault(&mut dec).unwrap();
        assert_eq!(back.reason, fault.reason);
        assert_eq!(back.requester, fault.requester);
        assert_eq!(back.iova, fault.iova);
        assert_eq!(back.faulting_phys, fault.faulting_phys);
    }
}
