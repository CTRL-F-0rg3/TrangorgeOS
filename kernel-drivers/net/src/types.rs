//! Addresses, ethertypes and frame sizes.
//!
//! The types and the vocabulary are unchanged from
//! `kernel_Workspace/kernel/src/nic/types.rs` and `ethernet.rs`, so the protocol
//! stack above compiles against either. What is added is validation that belongs
//! at the boundary: a driver should refuse a frame the wire cannot carry *before*
//! the hardware sees it.

use core::fmt;

use crate::error::PacketError;

/// A 48-bit Ethernet address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// The null address, which a card may report before it is configured.
    pub const ZERO: Self = Self([0; 6]);
    /// The all-ones address: every frame is delivered to it.
    pub const BROADCAST: Self = Self([0xff; 6]);

    /// Is this the broadcast address?
    #[inline]
    pub fn is_broadcast(self) -> bool {
        self.0 == Self::BROADCAST.0
    }

    /// Is this the null address?
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Is this a multicast address?
    ///
    /// The group bit is the low bit of the first octet. Locally administered
    /// addresses use the next bit, and the two are independent - a card's own
    /// address is usually `02:...`, which is neither multicast nor broadcast.
    #[inline]
    pub fn is_multicast(self) -> bool {
        self.0[0] & 1 == 1
    }

    /// Is this locally administered rather than vendor-assigned?
    #[inline]
    pub fn is_locally_administered(self) -> bool {
        self.0[0] & 2 == 2
    }

    /// Is this a valid unicast address for a card to own?
    ///
    /// False for null, broadcast and multicast. A card reporting any of those as
    /// its own address has not finished initialising, and handing it to the
    /// stack produces frames nobody can receive.
    #[inline]
    pub fn is_usable_unicast(self) -> bool {
        !self.is_zero() && !self.is_broadcast() && !self.is_multicast()
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// A 32-bit IPv4 address, in network byte order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    /// `0.0.0.0`.
    pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
    /// `255.255.255.255`.
    pub const LIMITED_BROADCAST: Self = Self([255, 255, 255, 255]);

    /// Build from four octets.
    #[inline]
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    /// The address as a big-endian `u32`, for masking.
    #[inline]
    pub fn as_u32_be(self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    /// Is this `0.0.0.0`?
    #[inline]
    pub fn is_unspecified(self) -> bool {
        self == Self::UNSPECIFIED
    }

    /// Is this `255.255.255.255`?
    #[inline]
    pub fn is_limited_broadcast(self) -> bool {
        self == Self::LIMITED_BROADCAST
    }

    /// Is this a loopback address?
    ///
    /// The whole `127.0.0.0/8`, not just `127.0.0.1`: a machine sending to
    /// `127.0.0.2` expects it to work.
    #[inline]
    pub fn is_loopback(self) -> bool {
        self.0[0] == 127
    }

    /// Is this private, per RFC 1918?
    #[inline]
    pub fn is_private(self) -> bool {
        match self.0[0] {
            10 => true,
            172 => (16..=31).contains(&self.0[1]),
            192 => self.0[1] == 168,
            _ => false,
        }
    }

    /// Is this inside `netmask`'s network with `other`?
    #[inline]
    pub fn is_in_subnet(self, other: Self, netmask: Self) -> bool {
        (self.as_u32_be() & netmask.as_u32_be()) == (other.as_u32_be() & netmask.as_u32_be())
    }
}

impl fmt::Debug for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// An EtherType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Ethertype(pub u16);

impl Ethertype {
    /// IPv4.
    pub const IPV4: Self = Self(0x0800);
    /// ARP.
    pub const ARP: Self = Self(0x0806);
    /// IPv6.
    pub const IPV6: Self = Self(0x86dd);
    /// VLAN-tagged: the real payload follows a four-byte tag.
    pub const VLAN: Self = Self(0x8100);
    /// IEEE 802.1ad, the double-tagged form.
    pub const QINQ: Self = Self(0x88a8);
    /// A frame length rather than a type, a form nothing uses any more.
    pub const LENGTH: Self = Self(0x0000);

    /// Does this ethertype carry a VLAN tag?
    #[inline]
    pub fn is_vlan(self) -> bool {
        self == Self::VLAN || self == Self::QINQ
    }
}

/// Bytes in an Ethernet header, excluding the FCS.
pub const HEADER_LEN: usize = 14;

/// The smallest frame the wire carries, excluding the FCS.
///
/// Sixty bytes. A shorter one is illegal, and some cards drop it in hardware
/// without telling anyone - which presents as a driver bug.
pub const MIN_FRAME: usize = 60;

/// The default MTU: 1500 bytes of payload.
pub const MTU_DEFAULT: usize = 1500;

/// The largest frame a card may hold, with room for a jumbo MTU.
pub const MAX_FRAME: usize = 16 * 1024;

/// Is `len` a frame length the wire allows?
#[inline]
pub fn is_valid_frame_len(len: usize) -> bool {
    (MIN_FRAME..=MAX_FRAME).contains(&len)
}

/// Parse a 14-byte Ethernet header, returning `(dst, src, ethertype, payload)`.
///
/// The length is not checked beyond the header: whether a *frame* is long enough
/// is the ring's business, and a caller that only wants the ethertype should not
/// have to care.
pub fn parse_header(
    bytes: &[u8],
) -> Result<(MacAddress, MacAddress, Ethertype, &[u8]), PacketError> {
    if bytes.len() < HEADER_LEN {
        return Err(PacketError::Truncated);
    }
    let destination = MacAddress([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]);
    let source = MacAddress([bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]]);
    let ether_type = Ethertype(u16::from_be_bytes([bytes[12], bytes[13]]));
    Ok((destination, source, ether_type, &bytes[HEADER_LEN..]))
}

/// Write a 14-byte Ethernet header and return the payload slice.
pub fn build_header(
    out: &mut [u8],
    destination: MacAddress,
    source: MacAddress,
    ether_type: Ethertype,
) -> Result<&mut [u8], PacketError> {
    if out.len() < HEADER_LEN {
        return Err(PacketError::Truncated);
    }
    out[0..6].copy_from_slice(&destination.0);
    out[6..12].copy_from_slice(&source.0);
    out[12..14].copy_from_slice(&ether_type.0.to_be_bytes());
    Ok(&mut out[HEADER_LEN..])
}

/// Pad a frame up to [`MIN_FRAME`] and return its wire length.
///
/// Padding must be zeroes, and it must happen in the buffer the card reads: a
/// card that sees a short frame on the wire drops it, so an unpadded ARP request
/// is an ARP request that silently never arrives.
pub fn pad_to_minimum(frame: &mut [u8], logical_len: usize) -> Result<usize, PacketError> {
    if logical_len < HEADER_LEN || logical_len > frame.len() {
        return Err(PacketError::Truncated);
    }
    let wire_len = if logical_len < MIN_FRAME { MIN_FRAME } else { logical_len };
    if wire_len > frame.len() {
        return Err(PacketError::Truncated);
    }
    frame[logical_len..wire_len].fill(0);
    Ok(wire_len)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_predicates_disagree_where_they_should() {
        // These must not overlap. A card satisfying two of them has a bug, and a
        // stack trusting the wrong one sends frames nowhere.
        assert!(MacAddress::BROADCAST.is_broadcast());
        assert!(!MacAddress::BROADCAST.is_usable_unicast());

        assert!(MacAddress::ZERO.is_zero());
        assert!(!MacAddress::ZERO.is_usable_unicast());

        let multicast = MacAddress([0x01, 0x00, 0x5e, 0, 0, 1]);
        assert!(multicast.is_multicast());
        assert!(!multicast.is_usable_unicast());

        let unicast = MacAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);
        assert!(unicast.is_usable_unicast());
        assert!(unicast.is_locally_administered());
        assert!(!unicast.is_multicast(), "locally administered is not multicast");
    }

    #[test]
    fn ipv4_classification() {
        assert!(Ipv4Address::new(10, 0, 0, 1).is_private());
        assert!(Ipv4Address::new(172, 16, 0, 1).is_private());
        assert!(Ipv4Address::new(172, 31, 255, 1).is_private());
        assert!(!Ipv4Address::new(172, 32, 0, 1).is_private(), "172.32/16 is not private");
        assert!(Ipv4Address::new(192, 168, 1, 1).is_private());
        assert!(!Ipv4Address::new(8, 8, 8, 8).is_private());

        assert!(Ipv4Address::new(127, 0, 0, 2).is_loopback(), "the whole /8");
        assert!(!Ipv4Address::new(128, 0, 0, 1).is_loopback());
    }

    #[test]
    fn subnet_membership_uses_the_mask() {
        let local = Ipv4Address::new(192, 168, 10, 3);
        let mask = Ipv4Address::new(255, 255, 255, 0);
        assert!(local.is_in_subnet(Ipv4Address::new(192, 168, 10, 99), mask));
        assert!(!local.is_in_subnet(Ipv4Address::new(192, 168, 11, 1), mask));
    }

    #[test]
    fn a_header_round_trips() {
        let src = MacAddress([2, 0, 0, 0, 0, 1]);
        let mut bytes = [0u8; MIN_FRAME];
        let payload = build_header(&mut bytes, MacAddress::BROADCAST, src, Ethertype::ARP).unwrap();
        payload[..3].copy_from_slice(&[1, 2, 3]);

        let (dst, s, et, body) = parse_header(&bytes).unwrap();
        assert_eq!(dst, MacAddress::BROADCAST);
        assert_eq!(s, src);
        assert_eq!(et, Ethertype::ARP);
        assert_eq!(&body[..3], &[1, 2, 3]);
    }

    #[test]
    fn a_short_buffer_is_truncated_not_a_panic() {
        let mut small = [0u8; 8];
        assert_eq!(
            build_header(&mut small, MacAddress::ZERO, MacAddress::ZERO, Ethertype::IPV4)
                .unwrap_err(),
            PacketError::Truncated
        );
        assert_eq!(parse_header(&[0u8; 8]).unwrap_err(), PacketError::Truncated);
    }

    #[test]
    fn a_short_frame_is_padded_to_the_wire_minimum() {
        // A card drops an unpadded frame in hardware, so this is not cosmetic.
        let src = MacAddress([2, 0, 0, 0, 0, 1]);
        let mut bytes = [0xffu8; MIN_FRAME];
        let payload = build_header(&mut bytes, MacAddress::BROADCAST, src, Ethertype::ARP).unwrap();
        payload[..1].copy_from_slice(&[7]);
        let len = pad_to_minimum(&mut bytes, HEADER_LEN + 1).unwrap();
        assert_eq!(len, MIN_FRAME);
        assert!(bytes[HEADER_LEN + 1..].iter().all(|b| *b == 0), "padding is zeroes");
    }

    #[test]
    fn a_long_frame_is_left_alone() {
        let mut bytes = [0u8; 100];
        assert_eq!(pad_to_minimum(&mut bytes, 90).unwrap(), 90);
    }

    #[test]
    fn frame_lengths_are_bounded() {
        assert!(!is_valid_frame_len(0));
        assert!(!is_valid_frame_len(MIN_FRAME - 1), "a 59-byte frame is not on the wire");
        assert!(is_valid_frame_len(MIN_FRAME));
        assert!(is_valid_frame_len(MAX_FRAME));
        assert!(!is_valid_frame_len(MAX_FRAME + 1));
    }

    #[test]
    fn vlan_ethertypes_are_recognised() {
        assert!(Ethertype::VLAN.is_vlan());
        assert!(Ethertype::QINQ.is_vlan());
        assert!(!Ethertype::IPV4.is_vlan());
    }
}

