//! Block device abstraction traits.

use kapi_abi::errors::DsError;

/// Represents a single I/O request to a block device.
pub struct BlockRequest {
    pub lba: u64,
    pub block_count: u32,
    pub buffer_ptr: *mut u8,
    pub buffer_len: usize,
    pub is_write: bool,
}

/// Core trait for all block devices.
/// Drivers must implement this to integrate with the block subsystem.
pub trait BlockDevice {
    /// Read blocks from the device starting at LBA.
    fn read_blocks(&mut self, lba: u64, block_count: u32, buffer: &mut [u8]) -> Result<(), DsError>;
    
    /// Write blocks to the device starting at LBA.
    fn write_blocks(&mut self, lba: u64, block_count: u32, buffer: &[u8]) -> Result<(), DsError>;
    
    /// Flush internal caches to physical media.
    fn flush(&mut self) -> Result<(), DsError>;
    
    /// Get device geometry (block size, total blocks).
    fn geometry(&self) -> BlockGeometry;
    
    /// Check if device supports TRIM/discard.
    fn supports_trim(&self) -> bool { false }
    
    /// Optional: Submit async request (for advanced schedulers).
    fn submit_request(&mut self, _req: BlockRequest) -> Result<(), DsError> {
        Err(DsError::Unknown)
    }
}

/// Physical geometry of a block device.
#[derive(Debug, Clone, Copy)]
pub struct BlockGeometry {
    pub block_size: u32,
    pub total_blocks: u64,
    pub max_transfer_blocks: u32,
}

impl BlockGeometry {
    pub fn total_bytes(&self) -> u64 {
        self.total_blocks * self.block_size as u64
    }
}