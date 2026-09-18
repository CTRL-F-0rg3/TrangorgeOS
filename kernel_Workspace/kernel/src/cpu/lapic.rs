use core::ptr; 
use x86_64::registers::model_specific::Msr;

const APIC_BASE_MSR: u32 = 0x1B;

const REG_ID: u32 = 0x020;
const REG_VERSION: u32 = 0x030;
const REG_SVR: u32 = 0x0F0;
const REG_EOI: u32 = 0x0B0;
const REG_ICR0: u32 = 0x300;
const REG_ICR1: u32 = 0x310;
const REG_LVT_LINT0: u32 = 0x350;
const REG_LVT_LINT1: u32 = 0x360;

const SPURIOUS_ENABLE: u32 = 1 << 8;

static mut LAPIC_BASE: usize = 0;
static mut X2APIC: bool = false;

fn x2apic_msr(reg: u32) -> u32 {
    0x800 + (reg >> 4)
}

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

pub fn id() -> u32 {
    if unsafe { X2APIC } {
        read(REG_ID)
    } else {
        read(REG_ID) >> 24
    }
}

pub fn version() -> u32 {
    read(REG_VERSION)
}

pub fn enable_bsp() {
    write(REG_SVR, SPURIOUS_ENABLE | 0xFF);
    write(REG_LVT_LINT0, 0x700);
}

pub fn enable_ap() {
    write(REG_SVR, SPURIOUS_ENABLE | 0xFF);
    write(REG_LVT_LINT0, 1 << 16);
    write(REG_LVT_LINT1, 1 << 16);
}

pub fn eoi() {
    write(REG_EOI, 0);
}

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

pub fn send_init_ipi() {

    send_ipi((5 << 8) | (1 << 14) | (1 << 15) | (3 << 18), 0);

    send_ipi((5 << 8) | (3 << 18), 0);
}

pub fn send_startup_ipi(apic_id: u32, vector: u8) {
    send_ipi((6 << 8) | (vector as u32), apic_id);
}
