pub mod host;
pub mod pci_glue;
pub mod dma;
pub mod core;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    NoController,
    MapFailed,
    Timeout,
    NotReady,
    BadDescriptor,
    Invalid,
    Transfer(u8),
}