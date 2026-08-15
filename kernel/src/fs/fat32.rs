use crate::fs::disc::BlockDevice;
use crate::fs::share::{DirEntry, FileMetadata, FsError, FsResult};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const FAT32_EOC_MIN: u32 = 0x0FFF_FFF8;
pub const FAT32_ENTRY_MASK: u32 = 0x0FFF_FFFF;
pub const ATTR_DIRECTORY: u8 = 0x10;
pub const ATTR_LONG_NAME: u8 = 0x0F;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Fat32BootSector {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media: u8,
    pub fat_size_16: u16,
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],
}

impl Fat32BootSector {
    pub fn first_fat_sector(&self) -> u32 {
        self.reserved_sector_count as u32
    }

    pub fn fat_region_sectors(&self) -> u32 {
        self.fat_size_32 * self.num_fats as u32
    }

    pub fn first_data_sector(&self) -> u32 {
        self.first_fat_sector() + self.fat_region_sectors()
    }

    pub fn cluster_to_sector(&self, cluster: u32) -> u32 {
        self.first_data_sector() + (cluster - 2) * self.sectors_per_cluster as u32
    }

    pub fn bytes_per_cluster(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct DirEntryRaw {
    name: [u8; 11],
    attr: u8,
    nt_reserved: u8,
    create_time_tenth: u8,
    create_time: u16,
    create_date: u16,
    last_access_date: u16,
    cluster_high: u16,
    write_time: u16,
    write_date: u16,
    cluster_low: u16,
    file_size: u32,
}

impl DirEntryRaw {
    fn is_free(&self) -> bool {
        self.name[0] == 0x00 || self.name[0] == 0xE5
    }

    fn is_long_name(&self) -> bool {
        self.attr == ATTR_LONG_NAME
    }

    fn is_directory(&self) -> bool {
        self.attr & ATTR_DIRECTORY != 0
    }

    fn first_cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) | self.cluster_low as u32
    }
}

pub fn parse_boot_sector(sector: &[u8; 512]) -> FsResult<Fat32BootSector> {
    let bpb = unsafe { core::ptr::read_unaligned(sector.as_ptr() as *const Fat32BootSector) };
    if bpb.bytes_per_sector != 512 {
        return Err(FsError::Unsupported);
    }
    if bpb.fat_size_32 == 0 {
        return Err(FsError::Unsupported);
    }
    if sector[510] != 0x55 || sector[511] != 0xAA {
        return Err(FsError::Corrupt);
    }
    Ok(bpb)
}

pub fn read_fat_entry<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    cluster: u32,
) -> FsResult<u32> {
    let fat_offset = cluster * 4;
    let sector = bpb.first_fat_sector() + fat_offset / bpb.bytes_per_sector as u32;
    let offset_in_sector = (fat_offset % bpb.bytes_per_sector as u32) as usize;

    let mut buf = [0u8; 512];
    device
        .read_block(sector as u64, &mut buf)
        .map_err(|_| FsError::Io)?;

    let raw = u32::from_le_bytes([
        buf[offset_in_sector],
        buf[offset_in_sector + 1],
        buf[offset_in_sector + 2],
        buf[offset_in_sector + 3],
    ]);
    Ok(raw & FAT32_ENTRY_MASK)
}

pub fn list_directory<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    start_cluster: u32,
) -> FsResult<Vec<DirEntry>> {
    let mut entries = Vec::new();
    let mut cluster = start_cluster;
    let entries_per_sector = bpb.bytes_per_sector as usize / core::mem::size_of::<DirEntryRaw>();

    loop {
        for s in 0..bpb.sectors_per_cluster as u32 {
            let sector = bpb.cluster_to_sector(cluster) + s;
            let mut buf = [0u8; 512];
            device
                .read_block(sector as u64, &mut buf)
                .map_err(|_| FsError::Io)?;

            for i in 0..entries_per_sector {
                let offset = i * core::mem::size_of::<DirEntryRaw>();
                let raw = unsafe {
                    core::ptr::read_unaligned(buf[offset..].as_ptr() as *const DirEntryRaw)
                };
                if raw.name[0] == 0x00 {
                    return Ok(entries);
                }
                if raw.is_free() || raw.is_long_name() {
                    continue;
                }
                entries.push(DirEntry {
                    name: format_short_name(&raw.name),
                    metadata: FileMetadata {
                        size_bytes: raw.file_size as u64,
                        is_directory: raw.is_directory(),
                    },
                });
                let _ = raw.first_cluster();
            }
        }

        let next = read_fat_entry(device, bpb, cluster)?;
        if next >= FAT32_EOC_MIN {
            break;
        }
        cluster = next;
    }

    Ok(entries)
}

pub fn read_file<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    start_cluster: u32,
    size_bytes: u64,
) -> FsResult<Vec<u8>> {
    let mut data = Vec::with_capacity(size_bytes as usize);
    let mut cluster = start_cluster;
    let cluster_bytes = bpb.bytes_per_cluster() as usize;
    let mut buf = vec![0u8; cluster_bytes];

    while (data.len() as u64) < size_bytes {
        for s in 0..bpb.sectors_per_cluster as u32 {
            let sector = bpb.cluster_to_sector(cluster) + s;
            let start = (s as usize) * bpb.bytes_per_sector as usize;
            let end = start + bpb.bytes_per_sector as usize;
            device
                .read_block(sector as u64, &mut buf[start..end])
                .map_err(|_| FsError::Io)?;
        }

        let remaining = (size_bytes - data.len() as u64) as usize;
        let take = remaining.min(cluster_bytes);
        data.extend_from_slice(&buf[..take]);

        if (data.len() as u64) >= size_bytes {
            break;
        }

        let next = read_fat_entry(device, bpb, cluster)?;
        if next >= FAT32_EOC_MIN {
            break;
        }
        cluster = next;
    }

    Ok(data)
}

pub fn write_fat_entry<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    cluster: u32,
    value: u32,
) -> FsResult<()> {
    let fat_offset = cluster * 4;
    let sector = bpb.first_fat_sector() + fat_offset / bpb.bytes_per_sector as u32;
    let offset_in_sector = (fat_offset % bpb.bytes_per_sector as u32) as usize;

    let mut buf = [0u8; 512];
    device
        .read_block(sector as u64, &mut buf)
        .map_err(|_| FsError::Io)?;

    let masked = value & FAT32_ENTRY_MASK;
    buf[offset_in_sector..offset_in_sector + 4].copy_from_slice(&masked.to_le_bytes());

    device
        .write_block(sector as u64, &buf)
        .map_err(|_| FsError::Io)?;

    for fat_index in 1..bpb.num_fats as u32 {
        let alt_sector = sector + fat_index * bpb.fat_size_32;
        device
            .write_block(alt_sector as u64, &buf)
            .map_err(|_| FsError::Io)?;
    }

    Ok(())
}

pub fn allocate_cluster<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
) -> FsResult<u32> {
    let total_clusters =
        (bpb.total_sectors_32 - bpb.first_data_sector()) / bpb.sectors_per_cluster as u32 + 2;
    for cluster in 2..total_clusters {
        let entry = read_fat_entry(device, bpb, cluster)?;
        if entry == 0 {
            write_fat_entry(device, bpb, cluster, FAT32_EOC_MIN)?;
            return Ok(cluster);
        }
    }
    Err(FsError::OutOfSpace)
}

pub fn write_file_data<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    start_cluster: u32,
    data: &[u8],
) -> FsResult<()> {
    let cluster_bytes = bpb.bytes_per_cluster() as usize;
    let mut cluster = start_cluster;
    let mut offset = 0usize;

    loop {
        let mut buf = vec![0u8; cluster_bytes];
        let take = (data.len() - offset).min(cluster_bytes);
        buf[..take].copy_from_slice(&data[offset..offset + take]);
        offset += take;

        for s in 0..bpb.sectors_per_cluster as u32 {
            let sector = bpb.cluster_to_sector(cluster) + s;
            let start = (s as usize) * bpb.bytes_per_sector as usize;
            let end = start + bpb.bytes_per_sector as usize;
            device
                .write_block(sector as u64, &buf[start..end])
                .map_err(|_| FsError::Io)?;
        }

        if offset >= data.len() {
            write_fat_entry(device, bpb, cluster, FAT32_EOC_MIN)?;
            break;
        }

        let next_cluster = allocate_cluster(device, bpb)?;
        write_fat_entry(device, bpb, cluster, next_cluster)?;
        cluster = next_cluster;
    }

    Ok(())
}

pub fn find_entry<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    start_cluster: u32,
    name: &str,
) -> FsResult<Option<(u32, FileMetadata)>> {
    let target = to_short_name(name);
    let mut cluster = start_cluster;
    let entries_per_sector = bpb.bytes_per_sector as usize / core::mem::size_of::<DirEntryRaw>();

    loop {
        for s in 0..bpb.sectors_per_cluster as u32 {
            let sector = bpb.cluster_to_sector(cluster) + s;
            let mut buf = [0u8; 512];
            device
                .read_block(sector as u64, &mut buf)
                .map_err(|_| FsError::Io)?;

            for i in 0..entries_per_sector {
                let offset = i * core::mem::size_of::<DirEntryRaw>();
                let raw = unsafe {
                    core::ptr::read_unaligned(buf[offset..].as_ptr() as *const DirEntryRaw)
                };
                if raw.name[0] == 0x00 {
                    return Ok(None);
                }
                if raw.is_free() || raw.is_long_name() {
                    continue;
                }
                if raw.name == target {
                    return Ok(Some((
                        raw.first_cluster(),
                        FileMetadata {
                            size_bytes: raw.file_size as u64,
                            is_directory: raw.is_directory(),
                        },
                    )));
                }
            }
        }

        let next = read_fat_entry(device, bpb, cluster)?;
        if next >= FAT32_EOC_MIN {
            return Ok(None);
        }
        cluster = next;
    }
}

pub fn write_file<D: BlockDevice + ?Sized>(
    device: &mut D,
    bpb: &Fat32BootSector,
    root_cluster: u32,
    name: &str,
    data: &[u8],
) -> FsResult<()> {
    let short_name = to_short_name(name);
    let entries_per_sector = bpb.bytes_per_sector as usize / core::mem::size_of::<DirEntryRaw>();

    let mut cluster = root_cluster;
    loop {
        for s in 0..bpb.sectors_per_cluster as u32 {
            let sector = bpb.cluster_to_sector(cluster) + s;
            let mut buf = [0u8; 512];
            device
                .read_block(sector as u64, &mut buf)
                .map_err(|_| FsError::Io)?;

            for i in 0..entries_per_sector {
                let offset = i * core::mem::size_of::<DirEntryRaw>();
                let raw = unsafe {
                    core::ptr::read_unaligned(buf[offset..].as_ptr() as *const DirEntryRaw)
                };

                let matches_existing =
                    !raw.is_free() && !raw.is_long_name() && raw.name == short_name;
                let is_free_slot = raw.is_free();

                if matches_existing || is_free_slot {
                    let start_cluster = if matches_existing {
                        raw.first_cluster()
                    } else {
                        allocate_cluster(device, bpb)?
                    };

                    write_file_data(device, bpb, start_cluster, data)?;

                    let new_entry = DirEntryRaw {
                        name: short_name,
                        attr: 0x20,
                        nt_reserved: 0,
                        create_time_tenth: 0,
                        create_time: 0,
                        create_date: 0,
                        last_access_date: 0,
                        cluster_high: (start_cluster >> 16) as u16,
                        write_time: 0,
                        write_date: 0,
                        cluster_low: (start_cluster & 0xFFFF) as u16,
                        file_size: data.len() as u32,
                    };

                    let entry_bytes = unsafe {
                        core::slice::from_raw_parts(
                            &new_entry as *const _ as *const u8,
                            core::mem::size_of::<DirEntryRaw>(),
                        )
                    };
                    buf[offset..offset + entry_bytes.len()].copy_from_slice(entry_bytes);
                    device
                        .write_block(sector as u64, &buf)
                        .map_err(|_| FsError::Io)?;

                    return Ok(());
                }
            }
        }

        let next = read_fat_entry(device, bpb, cluster)?;
        if next >= FAT32_EOC_MIN {
            return Err(FsError::OutOfSpace);
        }
        cluster = next;
    }
}

fn to_short_name(name: &str) -> [u8; 11] {
    let mut result = [b' '; 11];
    let upper = name.to_ascii_uppercase();
    let (base, ext) = match upper.split_once('.') {
        Some((b, e)) => (b, e),
        None => (upper.as_str(), ""),
    };
    for (i, b) in base.bytes().take(8).enumerate() {
        result[i] = b;
    }
    for (i, b) in ext.bytes().take(3).enumerate() {
        result[8 + i] = b;
    }
    result
}

fn format_short_name(raw: &[u8; 11]) -> String {
    let mut name = String::new();
    for &b in &raw[0..8] {
        if b == b' ' {
            break;
        }
        name.push(b as char);
    }
    if raw[8] != b' ' {
        name.push('.');
        for &b in &raw[8..11] {
            if b == b' ' {
                break;
            }
            name.push(b as char);
        }
    }
    name
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
    let mut image = vec![0u8; 512 * 6];

    image[11..13].copy_from_slice(&512u16.to_le_bytes());
    image[13] = 1;
    image[14..16].copy_from_slice(&1u16.to_le_bytes());
    image[16] = 1;
    image[32..36].copy_from_slice(&6u32.to_le_bytes());
    image[36..40].copy_from_slice(&1u32.to_le_bytes());
    image[44..48].copy_from_slice(&2u32.to_le_bytes());
    image[510] = 0x55;
    image[511] = 0xAA;

    let fat_sector_offset = 512;
    image[fat_sector_offset + 8..fat_sector_offset + 12]
        .copy_from_slice(&0x0FFF_FFFFu32.to_le_bytes());
    image[fat_sector_offset + 12..fat_sector_offset + 16]
        .copy_from_slice(&0x0FFF_FFFFu32.to_le_bytes());

    let root_offset = 512 * 2;
    image[root_offset..root_offset + 11].copy_from_slice(b"HELLO   TXT");
    image[root_offset + 11] = 0x20;
    let first_cluster: u16 = 3;
    image[root_offset + 26..root_offset + 28].copy_from_slice(&first_cluster.to_le_bytes());
    let file_size: u32 = 5;
    image[root_offset + 28..root_offset + 32].copy_from_slice(&file_size.to_le_bytes());

    let data_offset = 512 * 3;
    image[data_offset..data_offset + 5].copy_from_slice(b"hello");

    let mut disc = TestDisc { data: image };

    let mut boot_sector_buf = [0u8; 512];
    if disc.read_block(0, &mut boot_sector_buf).is_err() {
        return Err("failed to read synthetic boot sector");
    }
    let bpb = match parse_boot_sector(&boot_sector_buf) {
        Ok(bpb) => bpb,
        Err(_) => return Err("failed to parse synthetic FAT32 boot sector"),
    };

    let entries = match list_directory(&mut disc, &bpb, bpb.root_cluster) {
        Ok(entries) => entries,
        Err(_) => return Err("failed to list synthetic root directory"),
    };
    if entries.len() != 1 {
        return Err("expected exactly one directory entry");
    }
    if entries[0].name != "HELLO.TXT" {
        return Err("parsed short name did not match the synthetic entry");
    }
    if entries[0].metadata.size_bytes != 5 {
        return Err("parsed file size did not match the synthetic entry");
    }

    let content = match read_file(&mut disc, &bpb, 3, 5) {
        Ok(content) => content,
        Err(_) => return Err("failed to read synthetic file content"),
    };
    if content.as_slice() != b"hello" {
        return Err("read_file did not return the expected synthetic content");
    }

    if write_file(&mut disc, &bpb, bpb.root_cluster, "NEW.TXT", b"world").is_err() {
        return Err("write_file failed on synthetic FAT32 image");
    }

    let (new_cluster, new_meta) = match find_entry(&mut disc, &bpb, bpb.root_cluster, "NEW.TXT") {
        Ok(Some(pair)) => pair,
        Ok(None) => return Err("find_entry did not locate the freshly written file"),
        Err(_) => return Err("find_entry returned an error"),
    };
    if new_meta.size_bytes != 5 {
        return Err("freshly written file has the wrong size");
    }

    let new_content = match read_file(&mut disc, &bpb, new_cluster, new_meta.size_bytes) {
        Ok(c) => c,
        Err(_) => return Err("failed to read back the freshly written file"),
    };
    if new_content.as_slice() != b"world" {
        return Err("freshly written file content did not match what was written");
    }

    Ok("FAT32 read + write_file + find_entry round trip verified")
});
