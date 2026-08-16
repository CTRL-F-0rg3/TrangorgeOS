#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PciDev {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
}

unsafe fn outl(port: u16, v: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") v);
}

unsafe fn inl(port: u16) -> u32 {
    let v: u32;
    core::arch::asm!("in eax, dx", out("eax") v, in("dx") port);
    v
}

fn cfg_addr(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
    0x8000_0000 | (bus << 16) | (dev << 11) | (func << 8) | (off & 0xFC)
}

pub fn read32(d: PciDev, off: u32) -> u32 {
    unsafe {
        outl(0xCF8, cfg_addr(d.bus as u32, d.dev as u32, d.func as u32, off));
        inl(0xCFC)
    }
}

pub fn write32(d: PciDev, off: u32, v: u32) {
    unsafe {
        outl(0xCF8, cfg_addr(d.bus as u32, d.dev as u32, d.func as u32, off));
        outl(0xCFC, v);
    }
}

pub fn read16(d: PciDev, off: u32) -> u16 {
    (read32(d, off) >> ((off & 2) * 8)) as u16
}

pub fn read8(d: PciDev, off: u32) -> u8 {
    (read32(d, off) >> ((off & 3) * 8)) as u8
}

impl PciDev {
    pub fn vendor(self) -> u16 {
        read16(self, 0x00)
    }

    pub fn device_id(self) -> u16 {
        read16(self, 0x02)
    }

    pub fn class(self) -> u8 {
        (read32(self, 0x08) >> 24) as u8
    }

    pub fn subclass(self) -> u8 {
        (read32(self, 0x08) >> 16) as u8
    }

    pub fn prog_if(self) -> u8 {
        (read32(self, 0x08) >> 8) as u8
    }

    pub fn bar(self, idx: u32) -> u64 {
        let lo = read32(self, 0x10 + idx * 4) as u64;

        if lo & 0x04 != 0 {
            let hi = read32(self, 0x10 + (idx + 1) * 4) as u64;
            (hi << 32) | (lo & 0xFFFF_FFF0)
        } else {
            lo & 0xFFFF_FFF0
        }
    }

    pub fn enable_mmio(self) {
        let cmd = read16(self, 0x04) as u32;
        write32(self, 0x04, cmd | 0x0006);
    }
}

pub fn find_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciDev> {
    for bus in 0u32..256 {
        for dev in 0u32..32 {
            for func in 0u32..8 {
                let d = PciDev {
                    bus: bus as u8,
                    dev: dev as u8,
                    func: func as u8,
                };

                if d.vendor() == 0xFFFF {
                    continue;
                }

                if d.class() == class && d.subclass() == subclass && d.prog_if() == prog_if {
                    return Some(d);
                }

                if func == 0 {
                    break;
                }
            }
        }
    }

    None
}