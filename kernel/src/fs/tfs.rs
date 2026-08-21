use crate::fs::driver::block::BlockDevice;
use core::fmt::Write;

pub const SUPER_MAGIC: [u8; 4] = *b"TFS1";

const DIR_BLOCKS: u32 = 16;
const DATA_START: u32 = 1 + 1 + DIR_BLOCKS;
const ENTRY_SIZE: usize = 64;
const ENTRIES_PER_BLOCK: usize = 512 / ENTRY_SIZE;
const MAX_NAME: usize = 48;

pub const ROOT_DIR: u32 = 2;

const KIND_EMPTY: u8 = 0;
const KIND_FILE: u8 = 1;
const KIND_DIR: u8 = 2;

#[derive(Debug)]
pub enum FsError {
    NoDevice,
    Io,
    Corrupt,
    NotFound,
    NameTooLong,
    DiskFull,
    NotDir,
    NotEmpty,
}

pub type Result<T> = core::result::Result<T, FsError>;

#[derive(Clone, Copy)]
pub struct Superblock {
    pub total_blocks: u32,
    pub free_start: u32,
    pub file_count: u32,
}

fn rd32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn wr32(b: &mut [u8], off: usize, v: u32) {
    let bytes = v.to_le_bytes();
    b[off..off + 4].copy_from_slice(&bytes);
}

pub fn format(dev: &dyn BlockDevice) -> Result<()> {
    let mut sb = [0u8; 512];
    sb[0..4].copy_from_slice(&SUPER_MAGIC);
    wr32(&mut sb, 4, 1);
    wr32(&mut sb, 8, 512);
    wr32(&mut sb, 12, dev.block_count() as u32);
    wr32(&mut sb, 16, DIR_BLOCKS);
    wr32(&mut sb, 20, DATA_START);
    wr32(&mut sb, 24, DATA_START);
    wr32(&mut sb, 28, 0);

    dev.write_block(1, &sb).map_err(|_| FsError::Io)?;

    let zero = [0u8; 512];
    for i in 0..DIR_BLOCKS {
        dev.write_block(ROOT_DIR as u64 + i as u64, &zero).map_err(|_| FsError::Io)?;
    }

    Ok(())
}

fn read_superblock(dev: &dyn BlockDevice) -> Result<Superblock> {
    let mut sb = [0u8; 512];
    dev.read_block(1, &mut sb).map_err(|_| FsError::Io)?;

    if &sb[0..4] != &SUPER_MAGIC {
        return Err(FsError::Corrupt);
    }

    Ok(Superblock {
        total_blocks: rd32(&sb, 12),
        free_start: rd32(&sb, 24),
        file_count: rd32(&sb, 28),
    })
}

fn write_superblock(dev: &dyn BlockDevice, sb: &Superblock) -> Result<()> {
    let mut buf = [0u8; 512];
    buf[0..4].copy_from_slice(&SUPER_MAGIC);
    wr32(&mut buf, 4, 1);
    wr32(&mut buf, 8, 512);
    wr32(&mut buf, 12, sb.total_blocks);
    wr32(&mut buf, 16, DIR_BLOCKS);
    wr32(&mut buf, 20, DATA_START);
    wr32(&mut buf, 24, sb.free_start);
    wr32(&mut buf, 28, sb.file_count);
    dev.write_block(1, &buf).map_err(|_| FsError::Io)
}

struct DirEntry {
    name: [u8; MAX_NAME],
    kind: u8,
    size: u32,
    first_block: u32,
}

const EMPTY_ENTRY: DirEntry = DirEntry {
    name: [0; MAX_NAME],
    kind: KIND_EMPTY,
    size: 0,
    first_block: 0,
};

fn read_entry(block: &[u8], idx: usize) -> DirEntry {
    let off = idx * ENTRY_SIZE;
    let mut name = [0u8; MAX_NAME];
    name.copy_from_slice(&block[off..off + MAX_NAME]);
    DirEntry {
        name,
        kind: block[off + MAX_NAME],
        size: rd32(block, off + 52),
        first_block: rd32(block, off + 56),
    }
}

fn write_entry(block: &mut [u8], idx: usize, e: &DirEntry) {
    let off = idx * ENTRY_SIZE;
    block[off..off + MAX_NAME].copy_from_slice(&e.name);
    block[off + MAX_NAME] = e.kind;
    wr32(block, off + 52, e.size);
    wr32(block, off + 56, e.first_block);
}

fn entry_name(e: &DirEntry) -> &str {
    let slen = e.name.iter().position(|&c| c == 0).unwrap_or(MAX_NAME);
    core::str::from_utf8(&e.name[..slen]).unwrap_or("?")
}

fn find_entry(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<(usize, DirEntry)> {
    for blk in 0..DIR_BLOCKS as u64 {
        let mut buf = [0u8; 512];
        dev.read_block(dir as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;

        for i in 0..ENTRIES_PER_BLOCK {
            let e = read_entry(&buf, i);
            if e.kind == KIND_EMPTY {
                continue;
            }
            if entry_name(&e) == name {
                return Ok((blk as usize * ENTRIES_PER_BLOCK + i, e));
            }
        }
    }

    Err(FsError::NotFound)
}

pub fn list_dir(dev: &dyn BlockDevice, dir: u32, out: &mut impl Write) -> Result<()> {
    let sb = read_superblock(dev)?;

    writeln!(out, "TFS: {} block(s), {} file(s), next free {}",
             sb.total_blocks, sb.file_count, sb.free_start).ok();

    let mut found = 0usize;

    for blk in 0..DIR_BLOCKS as u64 {
        let mut buf = [0u8; 512];
        dev.read_block(dir as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;

        for i in 0..ENTRIES_PER_BLOCK {
            let e = read_entry(&buf, i);
            if e.kind == KIND_EMPTY {
                continue;
            }
            let tag = if e.kind == KIND_DIR { "dir " } else { "    " };
            writeln!(out, "  {}{}  ({} bytes @ block {})", tag, entry_name(&e), e.size, e.first_block).ok();
            found += 1;
        }
    }

    if found == 0 {
        writeln!(out, "  (empty)").ok();
    }

    Ok(())
}

pub fn write_file(dev: &dyn BlockDevice, dir: u32, name: &str, data: &[u8]) -> Result<()> {
    if name.is_empty() || name.len() >= MAX_NAME {
        return Err(FsError::NameTooLong);
    }

    let mut sb = read_superblock(dev)?;

    if find_entry(dev, dir, name).is_ok() {
        remove(dev, dir, name)?;
        sb = read_superblock(dev)?;
    }

    let blocks_needed = ((data.len() + 511) / 512).max(1) as u32;

    if sb.free_start + blocks_needed > sb.total_blocks {
        return Err(FsError::DiskFull);
    }

    for i in 0..blocks_needed {
        let mut block = [0u8; 512];
        let start = i as usize * 512;
        let end = core::cmp::min(start + 512, data.len());
        if start < data.len() {
            block[..end - start].copy_from_slice(&data[start..end]);
        }
        dev.write_block(sb.free_start as u64 + i as u64, &block).map_err(|_| FsError::Io)?;
    }

    let mut slot = None;
    'outer: for blk in 0..DIR_BLOCKS as u64 {
        let mut buf = [0u8; 512];
        dev.read_block(dir as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;

        for i in 0..ENTRIES_PER_BLOCK {
            if read_entry(&buf, i).kind == KIND_EMPTY {
                let mut name_buf = [0u8; MAX_NAME];
                name_buf[..name.len()].copy_from_slice(name.as_bytes());
                let e = DirEntry {
                    name: name_buf,
                    kind: KIND_FILE,
                    size: data.len() as u32,
                    first_block: sb.free_start,
                };
                write_entry(&mut buf, i, &e);
                dev.write_block(dir as u64 + blk, &buf).map_err(|_| FsError::Io)?;
                slot = Some(());
                break 'outer;
            }
        }
    }

    if slot.is_none() {
        return Err(FsError::DiskFull);
    }

    sb.free_start += blocks_needed;
    sb.file_count += 1;
    write_superblock(dev, &sb)
}

pub fn remove(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
    let (idx, e) = find_entry(dev, dir, name)?;

    if e.kind == KIND_DIR {
        for blk in 0..DIR_BLOCKS as u64 {
            let mut buf = [0u8; 512];
            dev.read_block(e.first_block as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;
            for i in 0..ENTRIES_PER_BLOCK {
                if read_entry(&buf, i).kind != KIND_EMPTY {
                    return Err(FsError::NotEmpty);
                }
            }
        }
    }

    let blk = idx / ENTRIES_PER_BLOCK;
    let slot = idx % ENTRIES_PER_BLOCK;

    let mut buf = [0u8; 512];
    dev.read_block(dir as u64 + blk as u64, &mut buf).map_err(|_| FsError::Io)?;
    write_entry(&mut buf, slot, &EMPTY_ENTRY);
    dev.write_block(dir as u64 + blk as u64, &buf).map_err(|_| FsError::Io)?;

    let mut sb = read_superblock(dev)?;
    sb.file_count = sb.file_count.saturating_sub(1);
    write_superblock(dev, &sb)
}

pub fn mkdir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
    if name.is_empty() || name.len() >= MAX_NAME {
        return Err(FsError::NameTooLong);
    }
    if find_entry(dev, dir, name).is_ok() {
        return Err(FsError::NotDir);
    }

    let mut sb = read_superblock(dev)?;

    if sb.free_start + DIR_BLOCKS > sb.total_blocks {
        return Err(FsError::DiskFull);
    }

    let first = sb.free_start;
    let zero = [0u8; 512];
    for i in 0..DIR_BLOCKS {
        dev.write_block(first as u64 + i as u64, &zero).map_err(|_| FsError::Io)?;
    }

    let mut slot = None;
    'outer: for blk in 0..DIR_BLOCKS as u64 {
        let mut buf = [0u8; 512];
        dev.read_block(dir as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;
        for i in 0..ENTRIES_PER_BLOCK {
            if read_entry(&buf, i).kind == KIND_EMPTY {
                let mut name_buf = [0u8; MAX_NAME];
                name_buf[..name.len()].copy_from_slice(name.as_bytes());
                let e = DirEntry {
                    name: name_buf,
                    kind: KIND_DIR,
                    size: 0,
                    first_block: first,
                };
                write_entry(&mut buf, i, &e);
                dev.write_block(dir as u64 + blk, &buf).map_err(|_| FsError::Io)?;
                slot = Some(());
                break 'outer;
            }
        }
    }

    if slot.is_none() {
        return Err(FsError::DiskFull);
    }

    sb.free_start += DIR_BLOCKS;
    sb.file_count += 1;
    write_superblock(dev, &sb)
}

pub fn find_dir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<u32> {
    let (_, e) = find_entry(dev, dir, name)?;
    if e.kind != KIND_DIR {
        return Err(FsError::NotDir);
    }
    Ok(e.first_block)
}

pub fn entries(dev: &dyn BlockDevice, dir: u32) -> Result<alloc::vec::Vec<(alloc::string::String, u32, u8)>> {
    let mut out = alloc::vec::Vec::new();

    for blk in 0..DIR_BLOCKS as u64 {
        let mut buf = [0u8; 512];
        dev.read_block(dir as u64 + blk, &mut buf).map_err(|_| FsError::Io)?;

        for i in 0..ENTRIES_PER_BLOCK {
            let e = read_entry(&buf, i);
            if e.kind == KIND_EMPTY {
                continue;
            }
            out.push((alloc::string::String::from(entry_name(&e)), e.size, e.kind));
        }
    }

    Ok(out)
}

pub fn read_file(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<alloc::vec::Vec<u8>> {
    let (_, e) = find_entry(dev, dir, name)?;

    let mut data = alloc::vec![0u8; e.size as usize];
    let blocks = (e.size as usize + 511) / 512;

    for i in 0..blocks {
        let mut block = [0u8; 512];
        dev.read_block(e.first_block as u64 + i as u64, &mut block).map_err(|_| FsError::Io)?;
        let start = i * 512;
        let end = core::cmp::min(start + 512, data.len());
        data[start..end].copy_from_slice(&block[..end - start]);
    }

    Ok(data)
}
