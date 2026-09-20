#![no_std]
#![allow(dead_code)]

//! `pcie` — PCI/PCIe bus enumeration and configuration-space access,
//! extracted from the legacy kernel (`kernel/src/pci.rs` and
//! `kernel/src/drivers/pci`). This is the foundation the `usb` (xHCI) host
//! driver builds on.
//!
//! "Concrete actions" exposed here: read/write config space, read BARs,
//! enable MMIO, find a device by class/subclass/prog-if, and enumerate the
//! whole bus.

extern crate alloc;

use alloc::vec::Vec;
use core::arch::asm;

/// A device location on the PCI bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciDev {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
}

/// A fully-probed device (as returned by [`enumerate`]).
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub address: PciDev,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
}

// ---------------------------------------------------------------------------
// Port I/O (config mechanism #1)
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn outl(port: u16, v: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") v);
}

#[inline(always)]
unsafe fn inl(port: u16) -> u32 {
    let v: u32;
    asm!("in eax, dx", out("eax") v, in("dx") port);
    v
}

fn cfg_addr(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
    0x8000_0000 | (bus << 16) | (dev << 11) | (func << 8) | (off & 0xFC)
}

// ---------------------------------------------------------------------------
// Config-space access
// ---------------------------------------------------------------------------

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

    /// Read BAR `idx`, handling both 32-bit and 64-bit BARs.
    pub fn bar(self, idx: u32) -> u64 {
        let lo = read32(self, 0x10 + idx * 4) as u64;
        if lo & 0x04 != 0 {
            let hi = read32(self, 0x10 + (idx + 1) * 4) as u64;
            (hi << 32) | (lo & 0xFFFF_FFF0)
        } else {
            lo & 0xFFFF_FFF0
        }
    }

    /// Enable memory-space access and bus mastering.
    pub fn enable_mmio(self) {
        let cmd = read16(self, 0x04) as u32;
        write32(self, 0x04, cmd | 0x0006);
    }
}

/// Find the first device matching class/subclass/prog-if.
pub fn find_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciDev> {
    for bus in 0u32..256 {
        for dev in 0u32..32 {
            let mut func = 0u32;
            let d = PciDev { bus: bus as u8, dev: dev as u8, func: 0 };
            if d.vendor() == 0xFFFF {
                continue;
            }
            let multifunction = (read8(d, 0x0E) & 0x80) != 0;
            while func < 8 {
                let f = PciDev { bus: bus as u8, dev: dev as u8, func: func as u8 };
                if f.vendor() != 0xFFFF
                    && f.class() == class
                    && f.subclass() == subclass
                    && f.prog_if() == prog_if
                {
                    return Some(f);
                }
                if !multifunction {
                    break;
                }
                func += 1;
            }
        }
    }
    None
}

fn probe(bus: u8, dev: u8, func: u8) -> Option<PciDevice> {
    let address = PciDev { bus, dev, func };
    let vendor_id = read16(address, 0x00);
    if vendor_id == 0xFFFF {
        return None;
    }
    Some(PciDevice {
        address,
        vendor_id,
        device_id: read16(address, 0x02),
        class_code: read8(address, 0x0B),
        subclass: read8(address, 0x0A),
        prog_if: read8(address, 0x09),
        header_type: read8(address, 0x0E),
    })
}

/// Enumerate the whole bus into a `Vec` of probed devices.
pub fn enumerate() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    for bus in 0u8..=255 {
        for dev in 0u8..32 {
            if let Some(d0) = probe(bus, dev, 0) {
                let multifunction = d0.header_type & 0x80 != 0;
                devices.push(d0);
                if multifunction {
                    for func in 1u8..8 {
                        if let Some(dn) = probe(bus, dev, func) {
                            devices.push(dn);
                        }
                    }
                }
            }
        }
        if bus == 255 {
            break;
        }
    }
    devices
}
