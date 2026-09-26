//! Errors from the device layer and the packet layer.
//!
//! Kept byte-for-byte compatible with `kernel_Workspace/kernel/src/nic/error.rs`
//! so the protocol stack above this crate keeps compiling against the same
//! variants. New failure modes get new variants; nothing is renamed.

/// A failure from the card, the ring, or the transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    /// The card is not initialised, or has been down.
    DeviceNotReady,
    /// The card reported an error that only a reset clears.
    DeviceNeedsReset,
    /// The card did not offer a feature the driver insists on.
    ///
    /// Carries both sets, because the useful diagnostic is *which* feature, and
    /// printing "unsupported" alone sends somebody to read the driver source.
    UnsupportedFeatures { offered: u64, requested: u64 },
    /// A queue size the card will not accept.
    InvalidQueueSize,
    /// No room in the ring. Backpressure, not a fault.
    QueueFull,
    /// No free descriptor.
    NoFreeDescriptor,
    /// A descriptor that does not describe anything valid.
    BadDescriptor,
    /// A DMA address could not be obtained.
    DmaAddressUnavailable,
    /// The card refused to send.
    TransmissionFailed,
    /// The card could not receive.
    ReceiveFailed,
    /// A buffer was too small for what it was asked to hold.
    BufferTooSmall,
    /// The card did not respond in time.
    Timeout,
}

/// A failure decoding a frame.
///
/// Separate from [`NetworkError`] on purpose: a malformed packet is the
/// network's fault and must not reset the card, while a device error is the
/// card's and might. A driver that returns both through one type ends up
/// resetting a NIC because somebody sent it a bad ARP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketError {
    /// Fewer bytes than the header needs.
    Truncated,
    /// The ethertype is not one this stack handles.
    InvalidEtherType,
    /// An ARP packet that does not parse.
    InvalidArp,
    /// An IPv4 version field that is not 4.
    InvalidIpv4Version,
    /// An IPv4 header length that is not 20 or more.
    InvalidIpv4HeaderLength,
    /// A total length that does not match the buffer.
    InvalidIpv4Length,
    /// An IPv4 header checksum that does not verify.
    InvalidIpv4Checksum,
    /// A fragment, which this stack does not reassemble.
    FragmentedIpv4,
    /// An ICMP packet that does not parse.
    InvalidIcmp,
    /// An ICMP checksum that does not verify.
    InvalidIcmpChecksum,
    /// A protocol with no handler.
    UnsupportedProtocol,
}
