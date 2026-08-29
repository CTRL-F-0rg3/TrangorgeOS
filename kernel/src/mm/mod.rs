//! Memory-management subsystem.
//!
//! Two backends, one API surface (`mm::phys`, `mm::api`, `mm::virt`,
//! `mm::space`) and one shared `self_test`:
//! * x86_64 — thin FFI wrappers over the C bridge (`libmm.a`, see `ffi.rs`);
//! * riscv64 — the native Rust backend in `riscv.rs` (bitmap PMM, free-list
//!   heap, VA range allocator, software-managed Sv39 tables).

#[cfg(target_arch = "x86_64")]
pub mod api;
#[cfg(target_arch = "x86_64")]
pub mod ffi;
#[cfg(target_arch = "riscv64")]
pub mod riscv;
#[cfg(target_arch = "x86_64")]
pub mod phys;
#[cfg(target_arch = "x86_64")]
pub mod space;
#[cfg(target_arch = "x86_64")]
pub mod virt;

// One API surface: on RISC-V the backend modules ARE `mm::phys` etc.
#[cfg(target_arch = "riscv64")]
pub use riscv::{api, phys, space, virt};

use core::cell::UnsafeCell;

#[cfg(target_arch = "x86_64")]
pub fn init(params: &ffi::MmBootParams) -> bool {
    unsafe { ffi::mm_init(params as *const ffi::MmBootParams) }
}

#[cfg(target_arch = "x86_64")]
pub fn ready() -> bool {
    unsafe { ffi::mm_ready() }
}

#[cfg(target_arch = "x86_64")]
pub fn dump() {
    unsafe { ffi::mm_dump() }
}

/// RISC-V: bring up the native backend (frames, heap, vmm, Sv39 tables).
#[cfg(target_arch = "riscv64")]
pub fn init_riscv() -> bool {
    riscv::init()
}

#[cfg(target_arch = "x86_64")]
const MAX_RAW_ENTRIES: usize = 256;

#[cfg(target_arch = "x86_64")]
const ZERO_ENTRY: ffi::RawMemEntry = ffi::RawMemEntry {
    base: 0,
    len: 0,
    typ: 0,
    reserved: 0,
};

#[cfg(target_arch = "x86_64")]
struct EntryStorage(UnsafeCell<[ffi::RawMemEntry; MAX_RAW_ENTRIES]>);

// mm initialization happens exactly once, at startup, before any concurrency
// exists — we safely force Sync.
#[cfg(target_arch = "x86_64")]
unsafe impl Sync for EntryStorage {}

/// Builds the parameters for `mm_init` from the bootloader-provided
/// information and initializes the whole memory subsystem (arch memory ->
/// paging -> pmm -> vmm -> heap -> cache -> address spaces).
#[cfg(target_arch = "x86_64")]
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

/// Dokładny test alokatora: ramki fizyczne, heap, vmm i przestrzenie adresowe.
pub fn self_test() -> Result<&'static str, &'static str> {
    // --- 1. Ramki fizyczne (PMM) ---
    let mut frames = [0u64; 8];

    for f in frames.iter_mut() {
        *f = phys::alloc_frame().ok_or("phys: frame alloc failed")?;
        if *f % 4096 != 0 {
            return Err("phys: frame not page-aligned");
        }
    }

    for i in 0..frames.len() {
        for j in (i + 1)..frames.len() {
            if frames[i] == frames[j] {
                return Err("phys: frames not distinct");
            }
        }
    }

    for &f in frames.iter() {
        if !phys::free_frame(f) {
            return Err("phys: frame free failed");
        }
    }

    // --- 2. Heap: kmalloc różnych rozmiarów + write/read ---
    let sizes: [usize; 6] = [8, 64, 256, 1024, 4096, 16384];
    let mut keep: [*mut u8; 6] = [core::ptr::null_mut(); 6];

    for (i, &sz) in sizes.iter().enumerate() {
        let p = api::kmalloc(sz).ok_or("heap: kmalloc failed")?;

        unsafe { core::ptr::write_bytes(p, 0x5A, sz); }

        for k in 0..sz {
            if unsafe { *p.add(k) } != 0x5A {
                api::kfree(p);
                return Err("heap: kmalloc write/read mismatch");
            }
        }

        keep[i] = p;
    }

    for p in keep.iter() {
        api::kfree(*p);
    }

    // --- 3. kzalloc musi zerować ---
    let z = api::kzalloc(512).ok_or("heap: kzalloc failed")?;

    for k in 0..512 {
        if unsafe { *z.add(k) } != 0 {
            api::kfree(z);
            return Err("heap: kzalloc not zeroed");
        }
    }

    api::kfree(z);

    // --- 4. krealloc: rośnięcie + zachowanie danych ---
    let a = api::kmalloc(64).ok_or("heap: kmalloc(a) failed")?;

    unsafe { core::ptr::write_bytes(a, 0x33, 64); }

    let b = api::krealloc(a, 4096).ok_or("heap: krealloc grow failed")?;

    for k in 0..64 {
        if unsafe { *b.add(k) } != 0x33 {
            api::kfree(b);
            return Err("heap: krealloc lost data");
        }
    }

    api::kfree(b);

    // --- 5. vmm: alokacja/zwolnienie regionu ---
    let v = virt::alloc(8192, virt::WRITE).ok_or("vmm: alloc failed")?;

    if v == 0 {
        return Err("vmm: alloc returned 0");
    }

    if !virt::free(v, 8192) {
        return Err("vmm: free failed");
    }

    // --- 6. AddressSpace: create / map_anon / munmap ---
    let aspace = space::AddressSpace::new().ok_or("aspace: create failed")?;
    let addr = aspace
        .map_anon(0, 4096, space::PROT_READ | space::PROT_WRITE)
        .ok_or("aspace: map_anon failed")?;

    if addr == 0 {
        return Err("aspace: map_anon returned 0");
    }

    if !aspace.munmap(addr, 4096) {
        return Err("aspace: munmap failed");
    }

    Ok("phys + heap + vmm + aspace roundtrip")
}