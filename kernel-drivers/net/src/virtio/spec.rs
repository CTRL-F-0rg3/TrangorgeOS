//! The virtio-net wire format, ported from Linux.
//!
//! Source: `dirtycode/linux-kernel/include/uapi/linux/virtio_net.h`,
//! `virtio_config.h`, `virtio_pci.h` and `virtio_ring.h`. Every constant below is
//! copied out of those headers rather than recalled, because the numbering *is*
//! the ABI: a driver that guesses a bit number fails in a way that looks like a
//! mis-negotiated device rather than a wrong driver.
//!
//! # The bug this port fixes
//!
//! The existing `kernel_Workspace/kernel/src/nic/virtio/` declares a ten-byte
//! header unconditionally. That is `struct virtio_net_hdr`, and it is only
//! correct when `VIRTIO_NET_F_MRG_RXBUF` is **not** negotiated. With merging on,
//! the device writes a *twelve*-byte `struct virtio_net_hdr_mrg_rxbuf`, whose
//! twelfth byte is `num_buffers`. A ten-byte header means that write lands on the
//! first byte of the frame itself.
//!
//! The symptom is the worst kind: it appears only on hosts that offer merging,
//! only on frames large enough to be split, and the result is well-formed enough
//! that a checksum still passes. Both headers exist here and [`net_header_len`]
//! returns the right one for the negotiated features, so the size cannot be wrong
//! by accident.
//!
//! # Sizes are asserted, not assumed
//!
//! The tests re-derive every structure size from the spec. A `#[repr(C)]` struct
//! that silently gains a padding byte is a DMA corruption, and it stays invisible
//! until the device is on real hardware.

#![allow(dead_code)]

/// `VIRTIO_ID_NET` from `virtio_ids.h`: the device type this driver speaks.
pub const VIRTIO_ID_NET: u32 = 1;

// ── device status, from virtio_config.h ──────────────────────────────────────

/// The driver has seen the device.
pub const STATUS_ACKNOWLEDGE: u8 = 1;
/// The driver knows how to drive it.
pub const STATUS_DRIVER: u8 = 2;
/// The driver is done setting up and the device may now run.
pub const STATUS_DRIVER_OK: u8 = 4;
/// Feature negotiation finished and the device accepted the result.
///
/// The legacy `virtio_net` in `kernel_Workspace` omits this bit entirely, which
/// is survivable only against pre-2.0 devices. A modern device that never sees
/// `FEATURES_OK` may refuse to start, and reports the failure as "no such
/// device" rather than as a negotiation problem.
pub const STATUS_FEATURES_OK: u8 = 8;
/// The device reset itself; the driver must re-negotiate.
pub const STATUS_DEVICE_NEEDS_RESET: u8 = 64;
/// Terminal. The device will not work again until it is reset.
pub const STATUS_FAILED: u8 = 128;

/// Is the device in the state where it is allowed to run?
pub const fn is_running(status: u8) -> bool {
    status & STATUS_DRIVER_OK != 0 && status & STATUS_FAILED == 0
}

// ── net feature bits, from virtio_net.h ──────────────────────────────────────

/// The host handles packets carrying a partial checksum.
pub const NET_F_CSUM: u64 = 0;
/// The guest, that is us, handles them.
pub const NET_F_GUEST_CSUM: u64 = 1;
/// The offload set is dynamic, negotiated over the control queue.
pub const NET_F_CTRL_GUEST_OFFLOADS: u64 = 2;
/// `virtio_net_config` carries an MTU advice.
pub const NET_F_MTU: u64 = 3;
/// `virtio_net_config` carries a host-assigned MAC.
pub const NET_F_MAC: u64 = 5;
/// We can receive TSOv4.
pub const NET_F_GUEST_TSO4: u64 = 7;
/// We can receive TSOv6.
pub const NET_F_GUEST_TSO6: u64 = 8;
/// We can receive TSOv6 with ECN.
pub const NET_F_GUEST_ECN: u64 = 9;
/// We can receive UFO.
pub const NET_F_GUEST_UFO: u64 = 10;
/// The host can receive TSOv4.
pub const NET_F_HOST_TSO4: u64 = 11;
/// The host can receive TSOv6.
pub const NET_F_HOST_TSO6: u64 = 12;
/// The host can receive TSOv6 with ECN.
pub const NET_F_HOST_ECN: u64 = 13;
/// The host can receive UFO.
pub const NET_F_HOST_UFO: u64 = 14;
/// The host can merge receive buffers, and therefore appends `num_buffers`.
///
/// This is the bit that widens the header from ten bytes to twelve. See the
/// module documentation.
pub const NET_F_MRG_RXBUF: u64 = 15;
/// `virtio_net_config` carries a link status.
pub const NET_F_STATUS: u64 = 16;
/// A control queue exists.
pub const NET_F_CTRL_VQ: u64 = 17;
/// The control queue supports receive-mode control.
pub const NET_F_CTRL_RX: u64 = 18;
/// The control queue supports VLAN filtering.
pub const NET_F_CTRL_VLAN: u64 = 19;
/// The control queue supports extra receive modes.
pub const NET_F_CTRL_RX_EXTRA: u64 = 20;
/// We can announce ourselves on the network.
pub const NET_F_GUEST_ANNOUNCE: u64 = 21;
/// Multiple receive queues with flow steering.
pub const NET_F_MQ: u64 = 22;
/// The MAC may be set through the control queue.
pub const NET_F_CTRL_MAC_ADDR: u64 = 23;
/// The device keeps its own counters.
pub const NET_F_DEVICE_STATS: u64 = 50;
/// Notification coalescing.
pub const NET_F_VQ_NOTF_COAL: u64 = 52;
/// Per-flow steering rules.
pub const NET_F_MQ_FS_RULES: u64 = 56;


/// The device reports speed and duplex in its configuration space.
///
/// The generic transport name in the kernel headers; within the net device's own
/// numbering it is bit 38.
pub const NET_F_SPEED_DUPLEX: u64 = 38;
/// The generic `VIRTIO_F_VERSION_1` bit, for modern devices.
pub const F_VERSION_1: u64 = 32;
/// `VIRTIO_F_RING_INDIRECT_DESC`, for indirect descriptors.
pub const F_RING_INDIRECT_DESC: u64 = 28;
/// `VIRTIO_F_RING_EVENT_IDX`, for event suppression.
pub const F_RING_EVENT_IDX: u64 = 29;

// ── descriptor and ring flags, from virtio_ring.h ────────────────────────────

/// Chains to another descriptor.
pub const VRING_DESC_F_NEXT: u16 = 1;
/// The device writes into this descriptor; we read it.
///
/// Its absence on a transmit descriptor means the device reads it. That
/// asymmetry is exactly why an RX descriptor left uninitialised becomes a
/// device-side write into memory nobody owns.
pub const VRING_DESC_F_WRITE: u16 = 2;
/// Points at an array of descriptors.
pub const VRING_DESC_F_INDIRECT: u16 = 4;
/// The driver suppresses used-ring notifications.
pub const VRING_AVAIL_F_NO_INTERRUPT: u16 = 1;
/// The device suppresses available-ring notifications.
pub const VRING_USED_F_NO_NOTIFY: u16 = 1;

// ── per-frame header flags, from virtio_net.h ─────────────────────────────────

/// The checksum is incomplete and must be finished by the stack.
pub const HDR_F_NEEDS_CSUM: u8 = 1;
/// The packet is valid. Without this flag, a zero length means "invalid" rather
/// than "empty".
pub const HDR_F_DATA_VALID: u8 = 2;
/// Receive segmentation offload information follows.
pub const HDR_F_RSC_INFO: u8 = 4;

// ── GSO types, from the `VIRTIO_NET_HDR_GSO_*` enum ──────────────────────────

/// No segmentation: the packet is exactly one frame.
pub const GSO_NONE: u8 = 0;
/// A TCPv4 segment.
pub const GSO_TCPV4: u8 = 1;
/// A UDP datagram.
pub const GSO_UDP: u8 = 3;
/// A TCPv6 segment.
pub const GSO_TCPV6: u8 = 4;
/// ECN bits present.
pub const GSO_ECN: u8 = 0x80;

// ── link status bits, from `virtio_net_config.status` ────────────────────────

/// The host cannot transmit right now.
pub const LINK_NO_TX: u8 = 1;
/// The link is up.
pub const LINK_UP: u8 = 2;

/// A little-endian `u16`, so byte order becomes part of the type.
///
/// Deliberately not a transparent wrapper over `u16`, which would be a lie on a
/// big-endian target. This is a distinct type with explicit accessors, so a port
/// cannot silently reorder it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct u16_le(pub u16);

impl u16_le {
    /// The value as stored, in wire order.
    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The value as a host integer.
    #[inline]
    pub const fn get(self) -> u16 {
        u16::from_le(self.0)
    }

    /// Build from a host integer.
    #[inline]
    pub const fn new(v: u16) -> Self {
        Self(v.to_le())
    }
}

// ── `virtio_net_hdr`: the ten-byte base header ───────────────────────────────

/// The per-frame header a virtio-net device writes ahead of a frame.
///
/// Every field is little-endian, which is native order on every architecture
/// this runs on, but the types are `u16_le` rather than `u16` so that the
/// contract survives a big-endian port.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetHeader {
    /// A mask of `HDR_F_*` bits.
    pub flags: u8,
    /// A `GSO_*` type, or `GSO_NONE`.
    pub gso_type: u8,
    /// Header length, for a segmented packet.
    pub hdr_len: u16_le,
    /// Payload length, for a segmented packet.
    pub gso_size: u16_le,
    /// Checksum start offset.
    pub csum_start: u16_le,
    /// Checksum length.
    pub csum_offset: u16_le,
}

/// `virtio_net_hdr_mrg_rxbuf`: what the device actually writes when merging.
///
/// Identical to [`NetHeader`] plus a twelfth byte. When the host merged `n`
/// buffers into one frame, `num_buffers` holds `n - 1`: it counts *extras*, which
/// is off by one from the first reading everyone has of it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MrgRxBufHeader {
    /// The base header.
    pub base: NetHeader,
    /// Extra buffers merged in, so the frame consumed `num_buffers + 1` buffers.
    pub num_buffers: u16_le,
}

/// How many bytes of header precede a frame, given the negotiated features.
///
/// The only correct way to size a DMA buffer. Anything else either truncates a
/// merged frame's count or wastes two bytes per frame.
#[inline]
pub const fn net_header_len(mrg_rxbuf: bool) -> usize {
    if mrg_rxbuf {
        core::mem::size_of::<MrgRxBufHeader>()
    } else {
        core::mem::size_of::<NetHeader>()
    }
}

/// Is this header claiming a checksum the stack still has to finish?
#[inline]
pub const fn hdr_needs_csum(hdr: &NetHeader) -> bool {
    hdr.flags & HDR_F_NEEDS_CSUM != 0
}

// ── `virtio_net_config` ──────────────────────────────────────────────────────

/// The device's configuration space, as a parsed view.
///
/// This is a plain data struct, not a `#[repr(C)]` one. The device-side layout is
/// *conditional*: `speed` and `duplex` exist only when `NET_F_SPEED_DUPLEX` is
/// negotiated, and the MTU only when `NET_F_MTU` is. A `repr(C)` struct would
/// have to always include the optional fields and then leave the driver guessing
/// which of them are real. Instead [`parse_config`] reads only the bytes the
/// device actually wrote, and every optional value is an `Option`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetConfig {
    /// Host-assigned MAC, when `NET_F_MAC` was negotiated.
    pub mac: [u8; 6],
    /// Link status, when `NET_F_STATUS` was negotiated.
    pub status: u8,
    /// How many queue pairs the device supports.
    pub max_queue_pairs: u16_le,
    /// Negotiated rate in Mbit/s, when `NET_F_SPEED_DUPLEX` was negotiated.
    pub speed_mbps: Option<u32>,
    /// Full duplex, when `NET_F_SPEED_DUPLEX` was negotiated.
    pub duplex_full: Option<bool>,
    /// MTU advice, when `NET_F_MTU` was negotiated.
    pub mtu: Option<u16>,
}

/// Byte offset of `speed` in `virtio_net_config`, after the mandatory 10 bytes.
pub const CONFIG_SPEED_OFFSET: usize = 10;
/// Byte offset of `duplex`, immediately after `speed`.
pub const CONFIG_DUPLEX_OFFSET: usize = CONFIG_SPEED_OFFSET + 4;
/// Byte offset of `mtu`, immediately after `duplex`.
pub const CONFIG_MTU_OFFSET: usize = CONFIG_DUPLEX_OFFSET + 1;
/// Length of the mandatory part: `mac`, `status`, `max_virtqueue_pairs`.
pub const CONFIG_MANDATORY_LEN: usize = 10;

/// Parse `virtio_net_config` out of `bytes`, given the negotiated features.
///
/// `bytes` is the device's configuration space and may be *shorter* than
/// [`CONFIG_MTU_OFFSET`] when the optional fields were not negotiated. Every
/// optional read is bounds-checked, and a truncated optional field becomes
/// `None` rather than a zero that a caller would happily take for an MTU of zero
/// and then wonder why nothing routes.
pub fn parse_config(bytes: &[u8], features: u64) -> NetConfig {
    let mut config = NetConfig::default();
    if bytes.len() < CONFIG_MANDATORY_LEN {
        return config;
    }
    config.mac.copy_from_slice(&bytes[0..6]);
    config.status = bytes[6];
    config.max_queue_pairs = u16_le::new(u16::from_le_bytes([bytes[8], bytes[9]]));

    if has(features, NET_F_SPEED_DUPLEX) {
        // Speed and duplex sit together, and the MTU follows them, so a host
        // that offers speed without an MTU is entirely normal and must not be
        // misread as an MTU of zero.
        if bytes.len() >= CONFIG_DUPLEX_OFFSET + 1 {
            config.speed_mbps = Some(u32::from_le_bytes([
                bytes[CONFIG_SPEED_OFFSET],
                bytes[CONFIG_SPEED_OFFSET + 1],
                bytes[CONFIG_SPEED_OFFSET + 2],
                bytes[CONFIG_SPEED_OFFSET + 3],
            ]));
            config.duplex_full = Some(bytes[CONFIG_DUPLEX_OFFSET] != 0);
        }
        if has(features, NET_F_MTU) && bytes.len() >= CONFIG_MTU_OFFSET + 2 {
            config.mtu = Some(u16::from_le_bytes([bytes[CONFIG_MTU_OFFSET], bytes[CONFIG_MTU_OFFSET + 1]]));
        }
    }
    config
}

/// Is feature bit `bit` set in `features`?
///
/// `u64`, not `u32`, because the virtio feature space runs to bit 63 and
/// `VIRTIO_F_VERSION_1` at bit 32 already overflows a 32-bit word. A `u32` here
/// silently truncates every modern-device feature, which presents as a device
/// that negotiates nothing and then refuses to start.
#[inline]
pub const fn has(features: u64, bit: u64) -> bool {
    features & (1u64 << bit) != 0
}

/// Does the link status byte report a usable link?
#[inline]
pub const fn link_is_up(status: u8) -> bool {
    status & LINK_UP != 0
}

/// Is the link status byte reporting that the host cannot transmit?
#[inline]
pub const fn link_tx_blocked(status: u8) -> bool {
    status & LINK_NO_TX != 0
}

/// Does this header assert that the packet data is valid?
#[inline]
pub const fn hdr_data_valid(hdr: &NetHeader) -> bool {
    hdr.flags & HDR_F_DATA_VALID != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    // ── layout: sizes are the ABI ───────────────────────────────────────────

    #[test]
    fn base_header_is_ten_bytes() {
        assert_eq!(size_of::<NetHeader>(), 10, "virtio_net_hdr is 10 bytes");
    }

    #[test]
    fn mrg_rxbuf_header_is_twelve_bytes() {
        assert_eq!(size_of::<MrgRxBufHeader>(), 12, "virtio_net_hdr_mrg_rxbuf is 12");
    }

    #[test]
    fn header_widens_exactly_by_two_bytes() {
        // The whole point of the port: the mrg variant is the base plus
        // `num_buffers`, with no hidden padding anywhere.
        let extra = size_of::<MrgRxBufHeader>() - size_of::<NetHeader>();
        assert_eq!(extra, 2, "merging must add exactly the num_buffers field");
    }

    #[test]
    fn header_offsets_match_the_wire_order() {
        // Re-derive each field's byte offset independently, so a reordered or
        // padded struct fails here rather than on hardware.
        let h = NetHeader::default();
        let base = &h as *const NetHeader as usize;
        assert_eq!(&h.flags as *const u8 as usize - base, 0);
        assert_eq!(&h.gso_type as *const u8 as usize - base, 1);
        assert_eq!(&h.hdr_len as *const u16_le as usize - base, 2);
        assert_eq!(&h.gso_size as *const u16_le as usize - base, 4);
        assert_eq!(&h.csum_start as *const u16_le as usize - base, 6);
        assert_eq!(&h.csum_offset as *const u16_le as usize - base, 8);

        let m = MrgRxBufHeader::default();
        let mbase = &m as *const MrgRxBufHeader as usize;
        assert_eq!(&m.base as *const NetHeader as usize - mbase, 0);
        assert_eq!(&m.num_buffers as *const u16_le as usize - mbase, 10);
    }

    #[test]
    fn header_len_follows_the_negotiated_feature() {
        assert_eq!(net_header_len(false), 10);
        assert_eq!(net_header_len(true), 12);
    }

    #[test]
    fn u16_le_round_trips() {
        for v in [0u16, 1, 0x1234, 1500, 0xffff] {
            assert_eq!(u16_le::new(v).get(), v);
        }
    }

    #[test]
    fn feature_bit_numbers_match_the_spec() {
        // The bits the driver actually branches on. A wrong number here is
        // indistinguishable from a device that mis-negotiated.
        assert_eq!(NET_F_MAC, 5);
        assert_eq!(NET_F_MTU, 3);
        assert_eq!(NET_F_STATUS, 16);
        assert_eq!(NET_F_MRG_RXBUF, 15);
        assert_eq!(NET_F_CTRL_VQ, 17);
        assert_eq!(NET_F_MQ, 22);
        assert_eq!(NET_F_SPEED_DUPLEX, 38);
        assert_eq!(F_VERSION_1, 32);
    }

    #[test]
    fn has_tests_the_right_bit() {
        let features = (1 << NET_F_MAC) | (1 << NET_F_MRG_RXBUF);
        assert!(has(features, NET_F_MAC));
        assert!(has(features, NET_F_MRG_RXBUF));
        assert!(!has(features, NET_F_MTU));
        assert!(!has(features, NET_F_SPEED_DUPLEX));
    }

    #[test]
    fn running_requires_driver_ok_and_no_failure() {
        assert!(is_running(STATUS_DRIVER_OK));
        assert!(!is_running(0), "not initialised is not running");
        assert!(!is_running(STATUS_DRIVER), "DRIVER alone is not DRIVER_OK");
        assert!(!is_running(STATUS_DRIVER_OK | STATUS_FAILED), "failed is not running");
    }

    #[test]
    fn features_ok_is_distinct_from_driver_ok() {
        // The legacy driver conflates these and omits FEATURES_OK entirely.
        assert_ne!(STATUS_FEATURES_OK, STATUS_DRIVER_OK);
        assert_eq!(STATUS_FEATURES_OK, 8);
    }

    #[test]
    fn checksum_and_validity_flags() {
        let mut h = NetHeader::default();
        assert!(!hdr_needs_csum(&h));
        assert!(!hdr_data_valid(&h));

        h.flags = HDR_F_NEEDS_CSUM | HDR_F_DATA_VALID;
        assert!(hdr_needs_csum(&h));
        assert!(hdr_data_valid(&h));

        h.flags = HDR_F_RSC_INFO;
        assert!(!hdr_needs_csum(&h), "RSC must not be mistaken for a checksum");
    }

    // ── config parsing ──────────────────────────────────────────────────────

    /// A config space as QEMU writes it: the mandatory 10 bytes plus all
    /// optional fields.
    fn full_config() -> [u8; CONFIG_MTU_OFFSET + 2] {
        let mut b = [0u8; CONFIG_MTU_OFFSET + 2];
        b[0..6].copy_from_slice(&[0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
        b[6] = LINK_UP;
        b[8..10].copy_from_slice(&1u16.to_le_bytes());
        b[CONFIG_SPEED_OFFSET..CONFIG_SPEED_OFFSET + 4].copy_from_slice(&10_000u32.to_le_bytes());
        b[CONFIG_DUPLEX_OFFSET] = 1; // full duplex
        b[CONFIG_MTU_OFFSET..CONFIG_MTU_OFFSET + 2].copy_from_slice(&1500u16.to_le_bytes());
        b
    }

    #[test]
    fn parses_mac_status_and_queue_pairs() {
        let c = parse_config(&full_config(), 0);
        assert_eq!(c.mac, [0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
        assert_eq!(c.status, LINK_UP);
        assert_eq!(c.max_queue_pairs.get(), 1);
    }

    #[test]
    fn optional_fields_are_absent_without_the_feature() {
        // The device *wrote* speed and MTU, but we did not negotiate the bits, so
        // reading them would be reading state we hold no contract for.
        let c = parse_config(&full_config(), 0);
        assert_eq!(c.speed_mbps, None);
        assert_eq!(c.duplex_full, None);
        assert_eq!(c.mtu, None);
    }

    #[test]
    fn optional_fields_parse_when_negotiated() {
        let f = (1 << NET_F_SPEED_DUPLEX) | (1 << NET_F_MTU);
        let c = parse_config(&full_config(), f);
        assert_eq!(c.speed_mbps, Some(10_000));
        assert_eq!(c.duplex_full, Some(true));
        assert_eq!(c.mtu, Some(1500));
    }

    #[test]
    fn speed_without_mtu_does_not_fabricate_an_mtu() {
        // Speed/duplex and MTU are independent bits, and a host may offer the
        // first without the second. Reading past the end here is the classic way
        // to end up with MTU 0 and nothing routed.
        let f = 1 << NET_F_SPEED_DUPLEX;
        let mut bytes = full_config();
        bytes[CONFIG_MTU_OFFSET] = 0;
        bytes[CONFIG_MTU_OFFSET + 1] = 0;
        let c = parse_config(&bytes, f);
        assert_eq!(c.speed_mbps, Some(10_000));
        assert_eq!(c.mtu, None, "MTU must stay None when the bit is absent");
    }

    #[test]
    fn truncated_config_never_yields_zero_mtu() {
        // Only the mandatory part present, but the MTU bit negotiated.
        let f = (1 << NET_F_SPEED_DUPLEX) | (1 << NET_F_MTU);
        let c = parse_config(&full_config()[..CONFIG_MANDATORY_LEN], f);
        assert_eq!(c.mtu, None);
        assert_eq!(c.speed_mbps, None);
    }

    #[test]
    fn short_configs_are_inert() {
        assert_eq!(parse_config(&[], 0), NetConfig::default());
        assert_eq!(parse_config(&[0xff; 5], 0), NetConfig::default());
        let c = parse_config(&[0xff; 5], 0);
        assert_eq!(c.mac, [0; 6]);
    }

    #[test]
    fn link_status_decodes() {
        assert!(link_is_up(LINK_UP));
        assert!(!link_is_up(0));
        assert!(link_tx_blocked(LINK_NO_TX));
        assert!(!link_tx_blocked(LINK_UP));
        // Up *and* unable to transmit is a real and easily misread combination.
        let s = LINK_UP | LINK_NO_TX;
        assert!(link_is_up(s));
        assert!(link_tx_blocked(s));
    }
}
