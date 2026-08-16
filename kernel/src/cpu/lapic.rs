//! Sterownik Local APIC (xAPIC przez MMIO, z fallbackiem x2APIC przez MSR).

use core::ptr;
use x86_64::registers::model_specific::Msr;

/// MSR IA32_APIC_BASE — status i położenie Local APIC.
const APIC_BASE_MSR: u32 = 0x1B;

/* Offsety rejestrów xAPIC (MMIO). */
const REG_ID: u32 = 0x020;
const REG_VERSION: u32 = 0x030;
const REG_SVR: u32 = 0x0F0;
const REG_EOI: u32 = 0x0B0;
const REG_ICR0: u32 = 0x300;
const REG_ICR1: u32 = 0x310;
const REG_LVT_LINT0: u32 = 0x350;

const SPURIOUS_ENABLE: u32 = 1 << 8; // APIC Software Enable

static mut LAPIC_BASE: usize = 0;
static mut X2APIC: bool = false;

/// Mapowanie xAPIC -> x2APIC MSR: 0x800 + (offset >> 4).
fn x2apic_msr(reg: u32) -> u32 {
    0x800 + (reg >> 4)
}

/// Inicjalizacja: wybiera tryb (xAPIC MMIO lub x2APIC MSR) na podstawie
/// stanu MSR IA32_APIC_BASE.
pub fn init(base_phys: u64) -> bool {
    unsafe {
        let msr = Msr::new(APIC_BASE_MSR).read();
        if msr & (1 << 10) != 0 {
            X2APIC = true;
            return true;
        }

        let base = if base_phys >= 0xFFFF800000000000 {
            base_phys
        } else {
            match crate::mm::virt::map_device(base_phys, 0x1000) {
                Some(v) => v,
                None => return false,
            }
        };

        LAPIC_BASE = base as usize;
        true
    }
}

pub fn is_x2apic() -> bool {
    unsafe { X2APIC }
}

pub fn read(reg: u32) -> u32 {
    unsafe {
        if X2APIC {
            Msr::new(x2apic_msr(reg)).read() as u32
        } else {
            ptr::read_volatile((LAPIC_BASE as *const u8).add(reg as usize) as *const u32)
        }
    }
}

pub fn write(reg: u32, val: u32) {
    unsafe {
        if X2APIC {
            Msr::new(x2apic_msr(reg)).write(val as u64);
        } else {
            ptr::write_volatile((LAPIC_BASE as *mut u8).add(reg as usize) as *mut u32, val);
        }
    }
}

/// Identyfikator Local APIC bieżącego rdzenia.
pub fn id() -> u32 {
    if unsafe { X2APIC } {
        read(REG_ID)
    } else {
        read(REG_ID) >> 24
    }
}

/// Wersja i maksymalny LVT entry (starsze 8 bitów / młodsze 8 bitów).
pub fn version() -> u32 {
    read(REG_VERSION)
}

/// Włącza Local APIC (SVR) i konfiguruje LINT0 jako ExtINT (virtual wire),
/// żeby 8259 PIC dalej dostarczał IRQ na BSP.
pub fn enable() {
    write(REG_SVR, SPURIOUS_ENABLE | 0xFF);
    write(REG_LVT_LINT0, 0x700); // ExtINT, unmasked
}

/// Sygnalizacja końca przerwania (EOI).
pub fn eoi() {
    write(REG_EOI, 0);
}

/// Wysłanie IPI: `icr` = wartość ICR low, `dest_apic_id` = adresat.
pub fn send_ipi(icr: u32, dest_apic_id: u32) {
    unsafe {
        if X2APIC {
            let val = ((dest_apic_id as u64) << 32) | icr as u64;
            Msr::new(x2apic_msr(REG_ICR0)).write(val);
        } else {
            write(REG_ICR1, dest_apic_id << 24);
            write(REG_ICR0, icr);
        }
    }
}

/// INIT IPI do wszystkich AP (assert + deassert).
pub fn send_init_ipi() {
    // delivery mode = 101 (INIT), level = 1 (assert), shorthand = all-excl-self
    send_ipi((5 << 8) | (1 << 14) | (1 << 15) | (3 << 18), 0);
    // deassert (level = 0)
    send_ipi((5 << 8) | (3 << 18), 0);
}

/// Startup IPI (SIPI): delivery mode = 110, vector = numer strony startowej.
pub fn send_startup_ipi(apic_id: u32, vector: u8) {
    send_ipi((6 << 8) | (vector as u32), apic_id);
}
