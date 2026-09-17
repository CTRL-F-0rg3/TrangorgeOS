use crate::fs::driver::block::BlockDevice;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatError {
    Io,
    BadSignature,
    NotFound,
    NotDir,
    NotFile,
}

pub struct Fat32 {
    disk: &'static dyn BlockDevice,
    bps: u64,
    spc: u64,
    fat_start: u64,
    data_start: u64,
    root_cluster: u32,
}

pub struct FatEntry {
    pub name: Vec<u8>,
    pub is_dir: bool,
    pub size: u64,
    pub cluster: u32,
}

fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

impl Fat32 {
    pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, FatError> {
        let mut b = vec![0u8; 512];

        disk.read_block(0, &mut b).map_err(|_| FatError::Io)?;

        if b[510] != 0x55 || b[511] != 0xAA {
            return Err(FatError::BadSignature);
        }

        let bps = u16le(&b, 0x0B) as u64;
        let spc = b[0x0D] as u64;
        let reserved = u16le(&b, 0x0E) as u64;
        let nfats = b[0x10] as u64;
        let fat_size = u32le(&b, 0x24) as u64;
        let root_cluster = u32le(&b, 0x2C);

        if bps == 0 || spc == 0 || fat_size == 0 {
            return Err(FatError::BadSignature);
        }

        let fat_start = reserved;
        let data_start = reserved + nfats * fat_size;

        Ok(Self { disk, bps, spc, fat_start, data_start, root_cluster })
    }

    fn read_at(&self, off: u64, buf: &mut [u8]) -> Result<(), FatError> {
        let sec = self.disk.block_size() as u64;
        let mut pos = 0usize;

        while pos < buf.len() {
            let cur = off + pos as u64;
            let s = cur / sec;
            let within = (cur % sec) as usize;

            let mut tmp = vec![0u8; sec as usize];
            self.disk.read_block(s, &mut tmp).map_err(|_| FatError::Io)?;

            let n = (sec as usize - within).min(buf.len() - pos);
            buf[pos..pos + n].copy_from_slice(&tmp[within..within + n]);
            pos += n;
        }

        Ok(())
    }

    fn cluster_off(&self, clus: u32) -> u64 {
        (self.data_start + (clus as u64 - 2) * self.spc) * self.bps
    }

    fn read_cluster(&self, clus: u32) -> Result<Vec<u8>, FatError> {
        let mut buf = vec![0u8; (self.spc * self.bps) as usize];
        self.read_at(self.cluster_off(clus), &mut buf)?;
        Ok(buf)
    }

    fn next_cluster(&self, clus: u32) -> Option<u32> {
        let off = self.fat_start * self.bps + clus as u64 * 4;

        let mut b = [0u8; 4];

        if self.read_at(off, &mut b).is_err() {
            return None;
        }

        let v = u32le(&b, 0) & 0x0FFF_FFFF;

        if v >= 0x0FFF_FFF8 {
            return None;
        }

        Some(v)
    }

    fn short_name(raw: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();

        for i in 0..8 {
            if raw[i] != b' ' {
                out.push(raw[i]);
            }
        }

        let mut ext = Vec::new();

        for i in 8..11 {
            if raw[i] != b' ' {
                ext.push(raw[i]);
            }
        }

        if !ext.is_empty() {
            out.push(b'.');
            out.extend(ext);
        }

        out
    }

    fn lfn_char(c: u16) -> u8 {
        if c < 128 && c != 0 { c as u8 } else { b'_' }
    }

    pub fn read_dir_cluster(&self, start: u32) -> Result<Vec<FatEntry>, FatError> {
        let mut out = Vec::new();

        let mut pending: Vec<(u8, [u16; 13])> = Vec::new();

        let mut clus = Some(start);

        while let Some(c) = clus {
            let data = self.read_cluster(c)?;

            let mut off = 0usize;

            while off + 32 <= data.len() {
                let first = data[off];

                if first == 0x00 {
                    return Ok(out);
                }

                let attr = data[off + 11];

                if first != 0xE5 && attr == 0x0F {
                    let ord = data[off] & 0x3F;

                    let mut part = [0u16; 13];

                    for i in 0..5 {
                        part[i] = u16le(&data, off + 1 + i * 2);
                    }
                    for i in 0..6 {
                        part[5 + i] = u16le(&data, off + 14 + i * 2);
                    }
                    for i in 0..2 {
                        part[11 + i] = u16le(&data, off + 28 + i * 2);
                    }

                    pending.push((ord, part));
                } else if first != 0xE5 && attr & 0x08 == 0 {
                    let is_dir = attr & 0x10 != 0;

                    let name = if !pending.is_empty() {
                        pending.sort_by_key(|p| p.0);

                        let mut n = Vec::new();

                        for (_, part) in pending.iter() {
                            for ch in part.iter() {
                                if *ch == 0 || *ch == 0xFFFF {
                                    break;
                                }
                                n.push(Self::lfn_char(*ch));
                            }
                        }

                        pending.clear();
                        n
                    } else {
                        Self::short_name(&data[off..off + 11])
                    };

                    if name != b"." && name != b".." {
                        let hi = u16le(&data, off + 20) as u32;
                        let lo = u16le(&data, off + 26) as u32;

                        out.push(FatEntry {
                            name,
                            is_dir,
                            size: u32le(&data, off + 28) as u64,
                            cluster: (hi << 16) | lo,
                        });
                    }
                } else {
                    pending.clear();
                }

                off += 32;
            }

            clus = self.next_cluster(c);
        }

        Ok(out)
    }

    pub fn resolve(&self, path: &str) -> Result<FatEntry, FatError> {
        let mut cur = FatEntry {
            name: Vec::new(),
            is_dir: true,
            size: 0,
            cluster: self.root_cluster,
        };

        for comp in path.split('/') {
            if comp.is_empty() {
                continue;
            }

            if !cur.is_dir {
                return Err(FatError::NotDir);
            }

            let entries = self.read_dir_cluster(cur.cluster)?;

            let mut found = None;

            for e in entries {
                if e.name.len() == comp.len()
                    && e.name.iter().zip(comp.bytes()).all(|(a, b)| {
                        a.to_ascii_lowercase() == b.to_ascii_lowercase()
                    })
                {
                    found = Some(e);
                    break;
                }
            }

            cur = found.ok_or(FatError::NotFound)?;
        }

        Ok(cur)
    }

    pub fn read_file(&self, entry: &FatEntry,
                     buf: &mut [u8]) -> Result<usize, FatError> {
        if entry.is_dir {
            return Err(FatError::NotFile);
        }

        let mut done = 0usize;
        let mut clus = Some(entry.cluster);

        while done < buf.len() && done < entry.size as usize {
            let c = clus.ok_or(FatError::Io)?;

            let data = self.read_cluster(c)?;

            let n = data.len().min(buf.len() - done)
                .min(entry.size as usize - done);

            buf[done..done + n].copy_from_slice(&data[..n]);
            done += n;

            clus = self.next_cluster(c);
        }

        Ok(done)
    }

    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, FatError> {
        let e = self.resolve(path)?;
        self.read_file(&e, buf)
    }

    pub fn list_path(&self, path: &str) -> Result<Vec<FatEntry>, FatError> {
        let e = self.resolve(path)?;

        if !e.is_dir {
            return Err(FatError::NotDir);
        }

        self.read_dir_cluster(e.cluster)
    }
}
