use super::ffi;
use core::ffi::c_void;
use core::ops::BitOr;

/*
 * P1 (sekcja 4.5 planu ulepszeń: "bezpieczne flagi ochrony"). Wcześniej
 * `PROT_*`/`MAP_*` były zwykłymi stałymi `u32`, dokładnie tego samego
 * typu co niepowiązane `virt::*` (kernel/src/mm/virt.rs, flagi VMM_FLAG_*
 * z INNĄ numeracją bitów — np. `virt::WRITE == 1` to co innego niż
 * `space::PROT_WRITE == 2`). Ponieważ oba zestawy to gołe `u32`, nic nie
 * chroniło przed pomyleniem ich przy wywołaniu (np. przez pomyłkę
 * `virt::alloc(len, space::PROT_WRITE)` skompilowałoby się i po cichu
 * ustawiłoby zupełnie inne uprawnienia niż zamierzone). Dodatkowo
 * `AddressSpace::mmap(addr, len, prot: u32, flags: u32)` miało dwa
 * sąsiadujące parametry TEGO SAMEGO typu — łatwo je pomylić przy
 * wywołaniu, a kompilator by tego nie wyłapał.
 *
 * `ProtFlags`/`MapFlags` to `#[repr(transparent)]` opakowania nad `u32`
 * (więc wciąż w pełni zgodne z ABI C na granicy FFI — `.bits()` daje
 * dokładnie tę samą wartość liczbową co wcześniej), ale jako ODRĘBNE
 * typy Rust: przekazanie `VmmFlags` tam, gdzie oczekiwane jest
 * `ProtFlags` (albo odwrotnie), jest teraz błędem kompilacji, a nie
 * cichym niedopasowaniem numeracji bitów w czasie wykonania. Istniejące
 * wywołania w stylu `space::PROT_READ | space::PROT_WRITE` działają bez
 * zmian, bo `ProtFlags` implementuje `BitOr` z tym samym zachowaniem.
 */
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProtFlags(u32);

impl ProtFlags {
    pub const NONE: ProtFlags = ProtFlags(0);
    pub const READ: ProtFlags = ProtFlags(1 << 0);
    pub const WRITE: ProtFlags = ProtFlags(1 << 1);
    pub const EXEC: ProtFlags = ProtFlags(1 << 2);
    pub const USER: ProtFlags = ProtFlags(1 << 3);
    /*
     * Wcześniej BRAKUJĄCE w Rust, mimo że C (paging.h: `PROT_DEVICE (1u
     * << 4)`) je definiuje i `driverspaceinit/init/service.rs` już się
     * do niego odwoływał jako `space::PROT_DEVICE` — co bez tej stałej
     * jest błędem kompilacji "cannot find value `PROT_DEVICE`"
     * (E0425), a nie problemem uruchomieniowym. Naprawione przy okazji
     * wprowadzania tego typu, z tą samą wartością bitową co po stronie C.
     */
    pub const DEVICE: ProtFlags = ProtFlags(1 << 4);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for ProtFlags {
    type Output = ProtFlags;

    fn bitor(self, rhs: ProtFlags) -> ProtFlags {
        ProtFlags(self.0 | rhs.0)
    }
}

pub const PROT_READ: ProtFlags = ProtFlags::READ;
pub const PROT_WRITE: ProtFlags = ProtFlags::WRITE;
pub const PROT_EXEC: ProtFlags = ProtFlags::EXEC;
pub const PROT_USER: ProtFlags = ProtFlags::USER;
pub const PROT_DEVICE: ProtFlags = ProtFlags::DEVICE;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MapFlags(u32);

impl MapFlags {
    pub const NONE: MapFlags = MapFlags(0);
    pub const ANONYMOUS: MapFlags = MapFlags(1 << 0);
    pub const PRIVATE: MapFlags = MapFlags(1 << 1);
    pub const FIXED: MapFlags = MapFlags(1 << 3);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for MapFlags {
    type Output = MapFlags;

    fn bitor(self, rhs: MapFlags) -> MapFlags {
        MapFlags(self.0 | rhs.0)
    }
}

pub const MAP_ANONYMOUS: MapFlags = MapFlags::ANONYMOUS;
pub const MAP_PRIVATE: MapFlags = MapFlags::PRIVATE;
pub const MAP_FIXED: MapFlags = MapFlags::FIXED;

pub struct AddressSpace {
    ptr: *mut c_void,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::aspace_create() };

        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn handle(&self) -> *mut c_void {
    unsafe { ffi::aspace_paging_handle(self.ptr) }
    }

    pub fn cr3(&self) -> u64 {
        unsafe { ffi::paging_aspace_cr3(self.ptr) }
    }
    pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
        unsafe { ffi::paging_aspace_map(self.handle(), virt, phys, len, prot.bits()) }
    }

    pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
        let at = unsafe { ffi::aspace_map_anon(self.ptr, hint, len, prot.bits()) };

        if at == 0 { None } else { Some(at) }
    }

    pub fn mmap(&self, addr: u64, len: usize, prot: ProtFlags, flags: MapFlags) -> Option<u64> {
        let at = unsafe { ffi::mmap(self.ptr, addr, len, prot.bits(), flags.bits()) };

        if at == 0 { None } else { Some(at) }
    }

    pub fn munmap(&self, addr: u64, len: usize) -> bool {
        unsafe { ffi::munmap(self.ptr, addr, len) }
    }

    pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
        unsafe { ffi::aspace_protect(self.ptr, addr, len, prot.bits()) }
    }

    pub fn brk(&self, new_brk: u64) -> u64 {
        unsafe { ffi::aspace_brk(self.ptr, new_brk) }
    }

    pub fn switch(&self) {
        let handle = unsafe { ffi::aspace_paging_handle(self.ptr) };
        unsafe { ffi::paging_aspace_switch(handle) }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        unsafe { ffi::aspace_destroy(self.ptr) }
    }
}

pub fn self_test() -> Result<&'static str, &'static str> {
    let aspace = AddressSpace::new().ok_or("aspace: create failed")?;

    let a1 = aspace
        .map_anon(0, 4096, PROT_READ | PROT_WRITE)
        .ok_or("aspace: map_anon failed")?;

    if a1 == 0 {
        return Err("aspace: map_anon returned 0");
    }

    let a2 = aspace
        .mmap(0, 8192, PROT_READ | PROT_WRITE, MAP_ANONYMOUS | MAP_PRIVATE)
        .ok_or("aspace: mmap failed")?;

    if a2 == 0 {
        return Err("aspace: mmap returned 0");
    }

    if !aspace.protect(a1, 4096, PROT_READ) {
        return Err("aspace: protect failed");
    }

    if !aspace.munmap(a1, 4096) {
        return Err("aspace: munmap(a1) failed");
    }

    if !aspace.munmap(a2, 8192) {
        return Err("aspace: munmap(a2) failed");
    }

    Ok("address space create/map/protect/unmap roundtrip")
}