use crate::fs::driver::BlockDevice;
use crate::fs::vfs::Result;

#[repr(C, packed)]
pub struct Superblock {
    pub magic: [u8; 8],
    pub uuid: [u8; 16],
    pub label: [u8; 64],
    pub version: u32,
    pub block_size: u32,
    pub block_size_log2: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub root_inode: u64,
    pub journal_inode: u64,
    pub journal_size: u32,
    pub last_mount: u64,
    pub mount_count: u64,
    pub features_compat: u32,
    pub features_incompat: u32,
    pub features_ro_compat: u32,
    pub checksum: u32,
}

impl Superblock {
    pub fn read(device: &dyn BlockDevice) -> Result<Self> {
        let mut buf = [0u8; 4096];
        device.read_blocks(0, 1, &mut buf)?;
        
        let sb: Self = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const Self) };
        
        // Verify checksum
        let expected_checksum = sb.checksum;
        let mut sb_copy = sb.clone();
        sb_copy.checksum = 0;
        
        let actual_checksum = Self::calculate_checksum(&sb_copy);
        
        if actual_checksum != expected_checksum {
            return Err("Superblock checksum mismatch");
        }
        
        Ok(sb)
    }
    
    pub fn write(&self, device: &dyn BlockDevice) -> Result<()> {
        let mut buf = [0u8; 4096];
        unsafe {
            core::ptr::write_unaligned(buf.as_mut_ptr() as *mut Self, *self);
        }
        
        device.write_blocks(0, 1, &buf)?;
        Ok(())
    }
    
    fn calculate_checksum(sb: &Superblock) -> u32 {
        // Simple CRC32 implementation
        let bytes = unsafe {
            core::slice::from_raw_parts(
                sb as *const Superblock as *const u8,
                core::mem::size_of::<Superblock>() - 4,
            )
        };
        
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in bytes {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }
    
    pub fn generate_uuid() -> [u8; 16] {
        // Simple UUID v4 generation
        let mut uuid = [0u8; 16];
        for i in 0..16 {
            uuid[i] = (i as u8 * 17 + 42) ^ 0xA5;
        }
        uuid[6] = (uuid[6] & 0x0F) | 0x40; // Version 4
        uuid[8] = (uuid[8] & 0x3F) | 0x80; // Variant 1
        uuid
    }
}

impl Clone for Superblock {
    fn clone(&self) -> Self {
        unsafe { core::ptr::read_unaligned(self as *const Self) }
    }
}