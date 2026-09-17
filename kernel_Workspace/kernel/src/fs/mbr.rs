use crate::fs::driver::block::{BlockDevice, DriverError};
use crate::fs::driver::registry;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

#[derive(Clone, Copy, Default)]
pub struct MbrEntry {
    pub ty: u8,
    pub start: u32,
    pub size: u32,
    pub bootable: bool,
}

pub struct Partition {
    pub base: &'static dyn BlockDevice,
    pub start: u64,
    pub count: u64,
    pub ty: u8,
    pub index: u8,
}

static mut PARTS: [Option<Partition>; 8] =
    [None, None, None, None, None, None, None, None];

impl BlockDevice for Partition {
    fn name(&self) -> &'static str {
        "part"
    }

    fn block_size(&self) -> usize {
        self.base.block_size()
    }

    fn block_count(&self) -> u64 {
        self.count
    }

    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
        if block >= self.count {
            return Err(DriverError::InvalidBlock);
        }

        self.base.read_block(self.start + block, buf)
    }

    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
        if block >= self.count {
            return Err(DriverError::InvalidBlock);
        }

        self.base.write_block(self.start + block, buf)
    }
}

pub fn parse_mbr(buf: &[u8]) -> Option<[MbrEntry; 4]> {
    if buf.len() < 512 || buf[510] != 0x55 || buf[511] != 0xAA {
        return None;
    }

    let mut out = [MbrEntry::default(); 4];

    for i in 0..4 {
        let off = 446 + i * 16;

        out[i] = MbrEntry {
            bootable: buf[off] == 0x80,
            ty: buf[off + 4],
            start: u32::from_le_bytes([buf[off + 8], buf[off + 9],
                                       buf[off + 10], buf[off + 11]]),
            size: u32::from_le_bytes([buf[off + 12], buf[off + 13],
                                      buf[off + 14], buf[off + 15]]),
        };
    }

    Some(out)
}

pub fn probe_disk(d: &'static dyn BlockDevice) -> usize {
    let mut buf = [0u8; 512];

    if d.read_block(0, &mut buf).is_err() {
        return 0;
    }

    let entries = match parse_mbr(&buf) {
        Some(e) => e,
        None => return 0,
    };

    let mut added = 0;

    for (i, e) in entries.iter().enumerate() {
        if e.ty == 0 || e.size == 0 {
            continue;
        }

        let part = Partition {
            base: d,
            start: e.start as u64,
            count: e.size as u64,
            ty: e.ty,
            index: i as u8,
        };

        unsafe {
            for slot in PARTS.iter_mut() {
                if slot.is_none() {
                    *slot = Some(part);
                    break;
                }
            }

            for slot in PARTS.iter() {
                if let Some(p) = slot {
                    if p.base as *const _ == d as *const _ && p.index == i as u8 {
                        registry::register(p);
                    }
                }
            }
        }

        unsafe {
            kprintf(b"fs: partition %d type=%x start=%d size=%d\n\0".as_ptr(),
                    i as u32,
                    e.ty as u32,
                    e.start,
                    e.size);
        }

        added += 1;
    }

    added
}

pub fn init() {
    let n = registry::count();

    for i in 0..n {
        if let Some(d) = registry::get(i) {
            probe_disk(d);
        }
    }
}
