#![allow(dead_code)]

pub const ARCH_RAW_MEM_USABLE: u32 = 1;
pub const ARCH_RAW_MEM_RESERVED: u32 = 2;
pub const ARCH_RAW_MEM_ACPI_RECLAIM: u32 = 3;
pub const ARCH_RAW_MEM_ACPI_NVS: u32 = 4;
pub const ARCH_RAW_MEM_BAD: u32 = 5;
pub const ARCH_RAW_MEM_BOOTLOADER: u32 = 0x100;

const MAX_RAW_ENTRIES: usize = 256;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawMemEntry {
    pub base: u64,
    pub len: u64,
    pub typ: u32,
    pub reserved: u32,
}

impl RawMemEntry {
    const fn zero() -> Self {
        Self {
            base: 0,
            len: 0,
            typ: 0,
            reserved: 0,
        }
    }
}

extern "C" {
    fn arch_memory_init(
        entries: *const RawMemEntry,
        count: usize,
        kernel_phys_start: u64,
        kernel_phys_end: u64,
        initrd_phys_start: u64,
        initrd_phys_end: u64,
    );

    fn arch_memory_dump();
}

static mut RAW_ENTRIES: [RawMemEntry; MAX_RAW_ENTRIES] =
    [RawMemEntry::zero(); MAX_RAW_ENTRIES];

static mut RAW_COUNT: usize = 0;

pub unsafe fn mm_add_region(base: u64, len: u64, typ: u32) {
    if len == 0 {
        return;
    }

    let i = RAW_COUNT;

    if i >= MAX_RAW_ENTRIES {
        // Increase ARCH_MAX_MEM_REGIONS in memory.h and MAX_RAW_ENTRIES here.
        panic!("mm_bridge: too many memory map entries");
    }

    RAW_ENTRIES[i] = RawMemEntry {
        base,
        len,
        typ,
        reserved: 0,
    };

    RAW_COUNT += 1;
}

pub unsafe fn mm_finalize(
    kernel_phys_start: u64,
    kernel_phys_end: u64,
    initrd_phys_start: u64,
    initrd_phys_end: u64,
) {
    arch_memory_init(
        RAW_ENTRIES.as_ptr(),
        RAW_COUNT,
        kernel_phys_start,
        kernel_phys_end,
        initrd_phys_start,
        initrd_phys_end,
    );

    
}