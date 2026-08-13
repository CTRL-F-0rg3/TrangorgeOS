use crate::fs::disc::BlockDevice;
use crate::fs::share::{FsError, FsResult};
use alloc::vec::Vec;

pub struct Fat32FormatOptions {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub total_sectors: u32,
}

pub fn format_fat32<D: BlockDevice + ?Sized>(
    device: &mut D,
    options: &Fat32FormatOptions,
) -> FsResult<()> {
    if options.bytes_per_sector as u64 != device.block_size() {
        return Err(FsError::Unsupported);
    }

    let data_sectors = options
        .total_sectors
        .saturating_sub(options.reserved_sectors as u32);
    let cluster_size_sectors = (options.sectors_per_cluster as u32).max(1);
    let approx_clusters = data_sectors / cluster_size_sectors;
    let fat_size_32 =
        (approx_clusters * 4).div_ceil(options.bytes_per_sector as u32) + 1;

    let mut boot_sector = [0u8; 512];
    boot_sector[0] = 0xEB;
    boot_sector[1] = 0x58;
    boot_sector[2] = 0x90;
    boot_sector[11..13].copy_from_slice(&options.bytes_per_sector.to_le_bytes());
    boot_sector[13] = options.sectors_per_cluster;
    boot_sector[14..16].copy_from_slice(&options.reserved_sectors.to_le_bytes());
    boot_sector[16] = options.num_fats;
    boot_sector[32..36].copy_from_slice(&options.total_sectors.to_le_bytes());
    boot_sector[36..40].copy_from_slice(&fat_size_32.to_le_bytes());
    boot_sector[44..48].copy_from_slice(&2u32.to_le_bytes());
    boot_sector[510] = 0x55;
    boot_sector[511] = 0xAA;

    device
        .write_block(0, &boot_sector)
        .map_err(|_| FsError::Io)?;

    let zero = [0u8; 512];
    let mut first_fat_sector = zero;
    first_fat_sector[8..12].copy_from_slice(&0x0FFF_FFF8u32.to_le_bytes());
    first_fat_sector[12..16].copy_from_slice(&0x0FFF_FFFFu32.to_le_bytes());

    for fat_index in 0..options.num_fats as u32 {
        let fat_start = options.reserved_sectors as u32 + fat_index * fat_size_32;
        for i in 0..fat_size_32 {
            let sector_data = if i == 0 { &first_fat_sector } else { &zero };
            device
                .write_block((fat_start + i) as u64, sector_data)
                .map_err(|_| FsError::Io)?;
        }
    }

    let root_sector = options.reserved_sectors as u32 + options.num_fats as u32 * fat_size_32;
    for s in 0..options.sectors_per_cluster as u32 {
        device
            .write_block((root_sector + s) as u64, &zero)
            .map_err(|_| FsError::Io)?;
    }

    Ok(())
}

struct TestDisc {
    data: Vec<u8>,
}

impl BlockDevice for TestDisc {
    fn block_size(&self) -> u64 {
        512
    }

    fn block_count(&self) -> u64 {
        (self.data.len() / 512) as u64
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> crate::fs::disc::Result<()> {
        let offset = (lba * 512) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(crate::fs::disc::DiscError::OutOfRange);
        }
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> crate::fs::disc::Result<()> {
        let offset = (lba * 512) as usize;
        if offset + buf.len() > self.data.len() {
            return Err(crate::fs::disc::DiscError::OutOfRange);
        }
        self.data[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

crate::test_module!({
    use alloc::vec;

    let mut disc = TestDisc {
        data: vec![0u8; 512 * 16],
    };

    let options = Fat32FormatOptions {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 1,
        num_fats: 1,
        total_sectors: 16,
    };

    if format_fat32(&mut disc, &options).is_err() {
        return Err("format_fat32 failed on a fresh synthetic disc");
    }

    let mut boot_sector_buf = [0u8; 512];
    if disc.read_block(0, &mut boot_sector_buf).is_err() {
        return Err("failed to read back the formatted boot sector");
    }
    let bpb = match crate::fs::fat32::parse_boot_sector(&boot_sector_buf) {
        Ok(bpb) => bpb,
        Err(_) => return Err("formatted boot sector did not parse as valid FAT32"),
    };

    let entries = match crate::fs::fat32::list_directory(&mut disc, &bpb, bpb.root_cluster) {
        Ok(entries) => entries,
        Err(_) => return Err("failed to list the freshly formatted root directory"),
    };
    if !entries.is_empty() {
        return Err("freshly formatted root directory should be empty");
    }

    Ok("format_fat32 write + fat32 read-back round trip verified")
});
