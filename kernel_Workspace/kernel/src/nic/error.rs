#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    DeviceNotReady,
    DeviceNeedsReset,
    UnsupportedFeatures { offered: u64, requested: u64 },
    InvalidQueueSize,
    QueueFull,
    NoFreeDescriptor,
    BadDescriptor,
    DmaAddressUnavailable,
    TransmissionFailed,
    ReceiveFailed,
    BufferTooSmall,
    Timeout,
    /// A device BAR was not mapped into the address space, so its registers
    /// cannot be read. Distinct from `DeviceNotReady`: the device is there, the
    /// kernel just did not map it.
    MmioUnmapped,
    /// A frame larger than the driver's buffers, i.e. larger than an MTU.
    /// `BufferTooSmall` is the receive-side name for the same condition.
    FrameTooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketError {
    Truncated,
    InvalidEtherType,
    InvalidArp,
    InvalidIpv4Version,
    InvalidIpv4HeaderLength,
    InvalidIpv4Length,
    InvalidIpv4Checksum,
    FragmentedIpv4,
    InvalidIcmp,
    InvalidIcmpChecksum,
    UnsupportedProtocol,
}
