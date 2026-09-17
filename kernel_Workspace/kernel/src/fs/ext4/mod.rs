use crate::fs::driver::block::BlockDevice;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtError {
    Io,
    BadMagic,
    Unsupported,
    NotFound,
    NotDir,
    NotFile,
}

pub struct Superblock {
    pub block_size: u64,
    pub blocks_count: u64,
    pub inodes_count: u32,
    pub blocks_per_group: u32,
    pub inodes_per_group: u32,
    pub inode_size: u32,
    pub desc_size: u32,
    pub first_data_block: u32,
    pub incompat: u32,
}

pub struct RawInode {
    pub mode: u16,
    pub size: u64,
    pub extent_area: [u8; 60],
}

pub struct DirEntry {
    pub inode: u32,
    pub name: Vec<u8>,
    pub ftype: u8,
}

pub struct Ext4 {
    disk: &'static dyn BlockDevice,
    sb: Superblock,
    bgdt_block: u64,
}

const EXT4_MAGIC: u16 = 0xEF53;
const INCOMPAT_EXTENTS: u32 = 0x40;
const INCOMPAT_64BIT: u32 = 0x80;

fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn u64le(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes([
        b[o], b[o + 1], b[o + 2], b[o + 3],
        b[o + 4], b[o + 5], b[o + 6], b[o + 7],
    ])
}

impl Ext4 {
    pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, ExtError> {
        let mut raw = vec![0u8; 1024];

        Self::read_at(disk, 1024, &mut raw).map_err(|_| ExtError::Io)?;

        if u16le(&raw, 0x38) != EXT4_MAGIC {
            return Err(ExtError::BadMagic);
        }

        let log_bs = u32le(&raw, 0x18);

        if log_bs > 3 {
            return Err(ExtError::Unsupported);
        }

        let incompat = u32le(&raw, 0x60);

        if incompat & INCOMPAT_EXTENTS == 0 {
            return Err(ExtError::Unsupported);
        }

        let sb = Superblock {
            block_size: 1024u64 << log_bs,
            blocks_count: u32le(&raw, 0x04) as u64,
            inodes_count: u32le(&raw, 0x10),
            blocks_per_group: u32le(&raw, 0x20),
            inodes_per_group: u32le(&raw, 0x28),
            inode_size: u16le(&raw, 0x58) as u32,
            desc_size: if incompat & INCOMPAT_64BIT != 0 {
                u16le(&raw, 0xFE) as u32
            } else {
                32
            },
            first_data_block: u32le(&raw, 0x14),
            incompat,
        };

        let bgdt_block = if sb.block_size == 1024 { 2 } else { 1 };

        Ok(Self { disk, sb, bgdt_block })
    }

    fn read_at(disk: &'static dyn BlockDevice,
               off: u64,
               buf: &mut [u8]) -> Result<(), crate::fs::driver::block::DriverError> {
        let sec = disk.block_size() as u64;
        let mut pos = 0usize;

        while pos < buf.len() {
            let cur = off + pos as u64;
            let s = cur / sec;
            let within = (cur % sec) as usize;

            let mut tmp = vec![0u8; sec as usize];
            disk.read_block(s, &mut tmp)?;

            let n = (sec as usize - within).min(buf.len() - pos);
            buf[pos..pos + n].copy_from_slice(&tmp[within..within + n]);
            pos += n;
        }

        Ok(())
    }

    pub fn read_blk(&self, blk: u64) -> Result<Vec<u8>, ExtError> {
        let mut buf = vec![0u8; self.sb.block_size as usize];

        Self::read_at(self.disk, blk * self.sb.block_size, &mut buf)
            .map_err(|_| ExtError::Io)?;

        Ok(buf)
    }

    fn inode_table(&self, ino: u32) -> Result<u64, ExtError> {
        let group = (ino - 1) / self.sb.inodes_per_group;
        let index = (ino - 1) % self.sb.inodes_per_group;

        let bg = self.read_blk(self.bgdt_block
            + (group as u64 * self.sb.desc_size as u64) / self.sb.block_size)?;

        let off = ((group as u64 * self.sb.desc_size as u64) % self.sb.block_size) as usize;

        let mut table = u32le(&bg, off + 0x08) as u64;

        if self.sb.desc_size >= 64 {
            table |= (u32le(&bg, off + 0x24) as u64) << 32;
        }

        Ok(table * self.sb.block_size + index as u64 * self.sb.inode_size as u64)
    }

    pub fn read_inode(&self, ino: u32) -> Result<RawInode, ExtError> {
        let byte = self.inode_table(ino)?;

        let mut raw = vec![0u8; self.sb.inode_size as usize];

        Self::read_at(self.disk, byte, &mut raw).map_err(|_| ExtError::Io)?;

        let mut area = [0u8; 60];
        area.copy_from_slice(&raw[0x28..0x28 + 60]);

        Ok(RawInode {
            mode: u16le(&raw, 0x00),
            size: u32le(&raw, 0x04) as u64 | ((u32le(&raw, 0x80) as u64) << 32),
            extent_area: area,
        })
    }

    fn block_map(&self, inode: &RawInode, file_blk: u32) -> Option<u64> {
        let mut data = inode.extent_area.to_vec();

        loop {
            if u16le(&data, 0) != 0xF30A {
                return None;
            }

            let entries = u16le(&data, 2) as usize;
            let depth = u16le(&data, 6);

            for i in 0..entries {
                let o = 12 + i * 12;

                if depth == 0 {
                    let ee_block = u32le(&data, o);
                    let ee_len = u16le(&data, o + 4) as u32;

                    if ee_len == 0 || file_blk < ee_block
                        || file_blk >= ee_block + ee_len {
                        continue;
                    }

                    let lo = u32le(&data, o + 8) as u64;
                    let hi = u16le(&data, o + 6) as u64;

                    return Some((hi << 32 | lo) + (file_blk - ee_block) as u64);
                } else {
                    let ei_block = u32le(&data, o);

                    if file_blk < ei_block {
                        continue;
                    }

                    let lo = u32le(&data, o + 4) as u64;
                    let hi = u16le(&data, o + 8) as u64;

                    if let Ok(b) = self.read_blk(hi << 32 | lo) {
                        data = b;
                        break;
                    }

                    return None;
                }
            }

            if depth != 0 {
                continue;
            }

            return None;
        }
    }

    pub fn read_file(&self, inode: &RawInode,
                     off: u64,
                     buf: &mut [u8]) -> Result<usize, ExtError> {
        if inode.mode & 0xF000 != 0x8000 {
            return Err(ExtError::NotFile);
        }

        let bs = self.sb.block_size;
        let mut done = 0usize;

        while done < buf.len() {
            let pos = off + done as u64;

            if pos >= inode.size {
                break;
            }

            let fblk = (pos / bs) as u32;

            let dblk = self.block_map(inode, fblk).ok_or(ExtError::Io)?;

            let blk = self.read_blk(dblk)?;

            let within = (pos % bs) as usize;
            let n = (bs as usize - within)
                .min(buf.len() - done)
                .min((inode.size - pos) as usize);

            buf[done..done + n].copy_from_slice(&blk[within..within + n]);
            done += n;
        }

        Ok(done)
    }

    pub fn read_dir(&self, inode: &RawInode) -> Result<Vec<DirEntry>, ExtError> {
        if inode.mode & 0xF000 != 0x4000 {
            return Err(ExtError::NotDir);
        }

        let mut raw = vec![0u8; inode.size as usize];
        self.read_file(inode, 0, &mut raw)?;

        let mut out = Vec::new();
        let mut off = 0usize;

        while off + 8 <= raw.len() {
            let ino = u32le(&raw, off);
            let rec = u16le(&raw, off + 4) as usize;
            let nlen = raw[off + 6] as usize;
            let ftype = raw[off + 7];

            if rec < 8 {
                break;
            }

            if ino != 0 {
                let name = raw[off + 8..off + 8 + nlen].to_vec();

                if name != b"." && name != b".." {
                    out.push(DirEntry { inode: ino, name, ftype });
                }
            }

            off += rec;
        }

        Ok(out)
    }

    pub fn resolve(&self, path: &str) -> Result<RawInode, ExtError> {
        let mut ino = 2u32;

        for comp in path.split('/') {
            if comp.is_empty() {
                continue;
            }

            let dir = self.read_inode(ino)?;
            let entries = self.read_dir(&dir)?;

            let mut found = None;

            for e in entries {
                if e.name.as_slice() == comp.as_bytes() {
                    found = Some(e.inode);
                    break;
                }
            }

            ino = found.ok_or(ExtError::NotFound)?;
        }

        let inode = self.read_inode(ino)?;

        if inode.mode & 0xF000 == 0xA000 && inode.size <= 60 {
            let target = core::str::from_utf8(&inode.extent_area[..inode.size as usize])
                .map_err(|_| ExtError::NotFound)?;

            return self.resolve(target);
        }

        Ok(inode)
    }

    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, ExtError> {
        let inode = self.resolve(path)?;
        self.read_file(&inode, 0, buf)
    }

    pub fn list_path(&self, path: &str) -> Result<Vec<DirEntry>, ExtError> {
        let inode = self.resolve(path)?;
        self.read_dir(&inode)
    }
}
