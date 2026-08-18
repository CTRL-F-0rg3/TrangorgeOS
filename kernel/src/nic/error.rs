/// Błędy urządzenia, kolejek oraz operacji wymagających czasu.
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
}

/// Przyczyna odrzucenia niezaufanego pakietu.
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
