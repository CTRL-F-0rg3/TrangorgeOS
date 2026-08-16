#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkError {
    DeviceNotReady,
    InitializationFailed,
    TransmissionFailed,
    ReceiveFailed,
    BufferUnavailable,
    Unsupported,
    InvalidFrameLength,
}
