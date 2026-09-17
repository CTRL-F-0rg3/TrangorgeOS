use crate::mm::ffi;
use core::ptr;

pub const TRAMPOLINE_BASE: u64 = 0x8000;

const PTE_PRESENT: u64 = 1;
const PTE_WRITABLE: u64 = 2;

extern "C" {
    static mut trampoline_start: u8;
    static mut trampoline_end: u8;
    static mut trampoline_cr3: u64;
    static mut trampoline_entry: u64;
    static mut trampoline_stack: u64;
    static mut trampoline_arg: u64;
}

unsafe fn field_phys(field: usize) -> u64 {
    let start = &trampoline_start as *const u8 as usize;
    TRAMPOLINE_BASE + (field - start) as u64
}

pub fn install(cr3: u64, entry: u64) {
    let start = unsafe { &trampoline_start as *const u8 };
    let end = unsafe { &trampoline_end as *const u8 };
    let len = end as usize - start as usize;

    let pages = (len + 4095) / 4096;
    for i in 0..pages {
        let phys = TRAMPOLINE_BASE + (i as u64) * 4096;
        unsafe {
            ffi::paging_map_page(phys, phys, PTE_PRESENT | PTE_WRITABLE);
        }
    }

    unsafe {
        ptr::copy_nonoverlapping(start, TRAMPOLINE_BASE as *mut u8, len);
    }

    unsafe {
        let cr3_addr = field_phys(&trampoline_cr3 as *const u64 as usize);
        let entry_addr = field_phys(&trampoline_entry as *const u64 as usize);
        ptr::write_volatile(cr3_addr as *mut u64, cr3);
        ptr::write_volatile(entry_addr as *mut u64, entry);
    }
}

pub fn set_stack_and_arg(stack_top: u64, arg: u64) {
    unsafe {
        let s = field_phys(&trampoline_stack as *const u64 as usize);
        let a = field_phys(&trampoline_arg as *const u64 as usize);
        ptr::write_volatile(s as *mut u64, stack_top);
        ptr::write_volatile(a as *mut u64, arg);
    }
}
