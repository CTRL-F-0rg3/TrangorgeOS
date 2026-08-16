//! Minimalny parser ACPI: wyszukanie RSDP i wyciągnięcie informacji z MADT
//! (lista CPU, adres Local APIC, I/O APIC). Odczyt pamięci fizycznej odbywa
//! się przez `physical_memory_offset` (map_physical_memory bootloadera).

use alloc::vec::Vec;
use core::ptr;

pub struct CpuEntry {
    pub apic_id: u32,
    pub enabled: bool,
}

pub struct IoApic {
    pub id: u8,
    pub address: u32,
    pub gsi_base: u32,
}

pub struct MadtInfo {
    pub lapic_base: u64,
    pub cpus: Vec<CpuEntry>,
    pub io_apics: Vec<IoApic>,
}

pub struct Rsdp {
    pub revision: u8,
    pub rsdt_addr: u32,
    pub xsdt_addr: u64,
}

const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
const MADT_SIGNATURE: &[u8; 4] = b"APIC";

unsafe fn phys_read<T: Copy>(phys_offset: u64, phys: u64) -> T {
    ptr::read_unaligned((phys_offset + phys) as *const T)
}

unsafe fn phys_slice<'a>(phys_offset: u64, phys: u64, len: usize) -> &'a [u8] {
    core::slice::from_raw_parts((phys_offset + phys) as *const u8, len)
}

fn checksum(bytes: &[u8]) -> bool {
    bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
}

unsafe fn check_rsdp(phys_offset: u64, addr: u64) -> Option<u64> {
    if phys_slice(phys_offset, addr, 8) != RSDP_SIGNATURE {
        return None;
    }
    if !checksum(phys_slice(phys_offset, addr, 20)) {
        return None;
    }
    Some(addr)
}

/// Wyszukuje RSDP w EBDA i obszarze 0xE0000..0xFFFFF.
pub fn find_rsdp(phys_offset: u64) -> Option<u64> {
    unsafe {
        let ebda_seg = u16::from_le_bytes([
            phys_read::<u8>(phys_offset, 0x40E),
            phys_read::<u8>(phys_offset, 0x40F),
        ]);
        if ebda_seg != 0 {
            let ebda = (ebda_seg as u64) << 4;
            let mut addr = ebda;
            while addr < ebda + 0x400 {
                if let Some(rsdp) = check_rsdp(phys_offset, addr) {
                    return Some(rsdp);
                }
                addr += 16;
            }
        }

        let mut addr = 0xE0000u64;
        while addr < 0x100000 {
            if let Some(rsdp) = check_rsdp(phys_offset, addr) {
                return Some(rsdp);
            }
            addr += 16;
        }
    }
    None
}

pub unsafe fn parse_rsdp(phys_offset: u64, addr: u64) -> Rsdp {
    let revision = phys_read::<u8>(phys_offset, addr + 15);
    let rsdt_addr = phys_read::<u32>(phys_offset, addr + 16);
    let xsdt_addr = if revision >= 2 {
        phys_read::<u64>(phys_offset, addr + 24)
    } else {
        0
    };
    Rsdp {
        revision,
        rsdt_addr,
        xsdt_addr,
    }
}

unsafe fn is_table(phys_offset: u64, addr: u64, sig: &[u8; 4]) -> bool {
    addr != 0 && phys_slice(phys_offset, addr, 4) == sig
}

pub unsafe fn find_madt(phys_offset: u64, rsdp: &Rsdp) -> Option<u64> {
    if rsdp.xsdt_addr != 0 {
        let len = phys_read::<u32>(phys_offset, rsdp.xsdt_addr + 4) as usize;
        let count = len.saturating_sub(36) / 8;
        for i in 0..count {
            let entry = phys_read::<u64>(phys_offset, rsdp.xsdt_addr + 36 + (i as u64) * 8);
            if is_table(phys_offset, entry, MADT_SIGNATURE) {
                return Some(entry);
            }
        }
        None
    } else if rsdp.rsdt_addr != 0 {
        let len = phys_read::<u32>(phys_offset, rsdp.rsdt_addr as u64 + 4) as usize;
        let count = len.saturating_sub(36) / 4;
        for i in 0..count {
            let entry =
                phys_read::<u32>(phys_offset, rsdp.rsdt_addr as u64 + 36 + (i as u64) * 4) as u64;
            if is_table(phys_offset, entry, MADT_SIGNATURE) {
                return Some(entry);
            }
        }
        None
    } else {
        None
    }
}

pub unsafe fn parse_madt(phys_offset: u64, madt: u64) -> MadtInfo {
    let lapic_base = phys_read::<u32>(phys_offset, madt + 36) as u64;
    let len = phys_read::<u32>(phys_offset, madt + 4) as usize;

    let mut cpus = Vec::new();
    let mut io_apics = Vec::new();
    let mut lapic_base_override: Option<u64> = None;

    // nagłówek (36) + adres LAPIC (4) + flagi (4) = 44
    let mut off = 44usize;
    while off + 2 <= len {
        let etype = phys_read::<u8>(phys_offset, madt + off as u64);
        let elen = phys_read::<u8>(phys_offset, madt + off as u64 + 1) as usize;
        if elen < 2 {
            break;
        }

        match etype {
            0 => {
                // Processor Local APIC
                if off + 8 <= len {
                    let apic_id = phys_read::<u8>(phys_offset, madt + off as u64 + 3);
                    let flags = phys_read::<u32>(phys_offset, madt + off as u64 + 4);
                    cpus.push(CpuEntry {
                        apic_id: apic_id as u32,
                        enabled: flags & 1 != 0,
                    });
                }
            }
            1 => {
                // I/O APIC
                if off + 12 <= len {
                    let id = phys_read::<u8>(phys_offset, madt + off as u64 + 2);
                    let address = phys_read::<u32>(phys_offset, madt + off as u64 + 4);
                    let gsi_base = phys_read::<u32>(phys_offset, madt + off as u64 + 8);
                    io_apics.push(IoApic {
                        id,
                        address,
                        gsi_base,
                    });
                }
            }
            9 => {
                // Processor Local x2APIC
                if off + 16 <= len {
                    let x2apic_id = phys_read::<u32>(phys_offset, madt + off as u64 + 4);
                    let flags = phys_read::<u32>(phys_offset, madt + off as u64 + 8);
                    cpus.push(CpuEntry {
                        apic_id: x2apic_id,
                        enabled: flags & 1 != 0,
                    });
                }
            }
            5 => {
                // Local APIC Address Override (64-bit)
                if off + 12 <= len {
                    lapic_base_override =
                        Some(phys_read::<u64>(phys_offset, madt + off as u64 + 4));
                }
            }
            _ => {}
        }

        off += elen;
    }

    MadtInfo {
        lapic_base: lapic_base_override.unwrap_or(lapic_base),
        cpus,
        io_apics,
    }
}
