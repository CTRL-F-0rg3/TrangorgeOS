pub mod api;
pub mod ffi;
pub mod phys;
pub mod space;
pub mod virt;

use core::cell::UnsafeCell;

pub fn init(params: &ffi::MmBootParams) -> bool {
    unsafe { ffi::mm_init(params as *const ffi::MmBootParams) }
}

pub fn ready() -> bool {
    unsafe { ffi::mm_ready() }
}

pub fn dump() {
    unsafe { ffi::mm_dump() }
}

const MAX_RAW_ENTRIES: usize = 256;
const ZERO_ENTRY: ffi::RawMemEntry = ffi::RawMemEntry {
    base: 0,
    len: 0,
    typ: 0,
    reserved: 0,
};

struct EntryStorage(UnsafeCell<[ffi::RawMemEntry; MAX_RAW_ENTRIES]>);

// Inicjalizacja mm dzieje się jednokrotnie, na starcie, zanim pojawi się
// jakakolwiek współbieżność — bezpiecznie wymuszamy Sync.
unsafe impl Sync for EntryStorage {}

/// Buduje parametry dla `mm_init` z informacji przekazanych przez bootloader
/// i inicjalizuje cały podsystem pamięci (arch memory -> paging -> pmm ->
/// vmm -> heap -> cache -> address spaces).
pub fn init_from_boot_info(boot_info: &'static bootloader::BootInfo) -> bool {
    use bootloader::bootinfo::MemoryRegionType;

    static ENTRIES: EntryStorage =
        EntryStorage(UnsafeCell::new([ZERO_ENTRY; MAX_RAW_ENTRIES]));

    let entries = unsafe { &mut *ENTRIES.0.get() };
    let mut count = 0usize;

    for region in boot_info.memory_map.iter() {
        if count >= MAX_RAW_ENTRIES {
            break;
        }

        let start = region.range.start_addr();
        let end = region.range.end_addr();

        if end <= start {
            continue;
        }

        let typ = match region.region_type {
            MemoryRegionType::Usable => 1u32,
            MemoryRegionType::Reserved => 2u32,
            MemoryRegionType::AcpiReclaimable => 3u32,
            MemoryRegionType::AcpiNvs => 4u32,
            MemoryRegionType::BadMemory => 5u32,
            MemoryRegionType::Bootloader => 0x100,
            _ => 2u32,
        };

        entries[count] = ffi::RawMemEntry {
            base: start,
            len: end - start,
            typ,
            reserved: 0,
        };
        count += 1;
    }

    let params = ffi::MmBootParams {
        memmap: entries.as_ptr(),
        memmap_count: count,
        kernel_phys_start: 0,
        kernel_phys_end: 0,
        initrd_phys_start: 0,
        initrd_phys_end: 0,
        boot_phys_offset: boot_info.physical_memory_offset,
    };

    unsafe { ffi::mm_init(&params) }
}