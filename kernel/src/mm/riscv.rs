//! Native RISC-V (riscv64gc) memory-management backend.
//!
//! The x86_64 build backs `mm::` with the C bridge (`libmm.a` — `mm/ffi.rs`).
//! RISC-V has no such bridge yet, so this module provides the same public
//! API surface (`mm::phys`, `mm::api`, `mm::virt`, `mm::space`) natively:
//!
//! * `phys`  — bitmap physical-frame allocator over a static 4 MiB pool,
//! * `api`   — kmalloc/kfree/krealloc… over a size-headered free-list heap,
//! * `virt`  — virtual-range allocator (VA window; no MMU backing yet),
//! * `space` — Sv39 page tables maintained in software (3 levels, 4 KiB
//!             pages). The MMU stays in bare mode under OpenSBI for now —
//!             tables are built and walked, but `satp` is not switched.
//!
//! Identity mapping: in bare mode VA == PA in the low half, which makes
//! direct frame access (`phys_to_virt`) trivial here.

pub mod phys {
    use spin::Mutex;

    const FRAME_SIZE: usize = 4096;
    const FRAME_POOL_BYTES: usize = 4 * 1024 * 1024;
    const FRAME_COUNT: usize = FRAME_POOL_BYTES / FRAME_SIZE; // 1024 frames
    const BITMAP_WORDS: usize = FRAME_COUNT / 64; // 16 x u64

    #[repr(C, align(4096))]
    struct FramePool([u8; FRAME_POOL_BYTES]);

    static POOL: FramePool = FramePool([0; FRAME_POOL_BYTES]);

    struct State {
        bitmap: [u64; BITMAP_WORDS],
        used: usize,
    }

    static STATE: Mutex<State> = Mutex::new(State {
        bitmap: [0; BITMAP_WORDS],
        used: 0,
    });

    fn pool_base() -> u64 {
        &POOL as *const FramePool as u64
    }

    /// Zero the bitmap explicitly (QEMU's ELF loader already clears BSS;
    /// this keeps the invariant obvious and re-init safe).
    pub fn init() {
        let mut s = STATE.lock();
        s.bitmap = [0; BITMAP_WORDS];
        s.used = 0;
    }

    fn bit_set(b: &mut [u64; BITMAP_WORDS], idx: usize) {
        b[idx / 64] |= 1u64 << (idx % 64);
    }

    fn bit_clear(b: &mut [u64; BITMAP_WORDS], idx: usize) {
        b[idx / 64] &= !(1u64 << (idx % 64));
    }

    fn bit_get(b: &[u64; BITMAP_WORDS], idx: usize) -> bool {
        b[idx / 64] & (1u64 << (idx % 64)) != 0
    }

    /// Reserve [base, base+len) — marks frames overlapping the pool as used.
    pub fn reserve(base: u64, len: u64) {
        let mut s = STATE.lock();
        let start = base.max(pool_base());
        let end = base
            .saturating_add(len)
            .min(pool_base() + FRAME_POOL_BYTES as u64);
        if end <= start {
            return;
        }
        let first = ((start - pool_base()) / FRAME_SIZE as u64) as usize;
        let last = ((end - 1 - pool_base()) / FRAME_SIZE as u64) as usize;
        for idx in first..=last {
            if !bit_get(&s.bitmap, idx) {
                bit_set(&mut s.bitmap, idx);
                s.used += 1;
            }
        }
    }

    /// Find `count` consecutive free frames; the first must satisfy the
    /// frame-index alignment (`align_frames = 512` → 2 MiB alignment).
    fn scan_run(s: &mut State, count: usize, align_frames: usize) -> Option<usize> {
        let mut run = 0usize;
        for idx in 0..FRAME_COUNT {
            if bit_get(&s.bitmap, idx) {
                run = 0;
                continue;
            }
            if run == 0 {
                if idx % align_frames.max(1) != 0 {
                    continue;
                }
                run = 1;
                if run == count {
                    return Some(idx);
                }
                continue;
            }
            run += 1;
            if run == count {
                return Some(idx + 1 - count);
            }
        }
        None
    }

    pub fn alloc_frame() -> Option<u64> {
        alloc_frames(1)
    }

    pub fn alloc_zero_frame() -> Option<u64> {
        let pa = alloc_frame()?;
        // Identity map (bare mode): writing through the physical address
        // writes the backing pool memory directly.
        unsafe {
            core::ptr::write_bytes(pa as *mut u8, 0, FRAME_SIZE);
        }
        Some(pa)
    }

    pub fn alloc_frames(count: usize) -> Option<u64> {
        alloc_frames_aligned(count, 1)
    }

    /// Allocate `count` contiguous frames, first frame aligned to
    /// `align_frames * 4096` bytes.
    pub fn alloc_frames_aligned(count: usize, align_frames: usize) -> Option<u64> {
        if count == 0 || count > FRAME_COUNT {
            return None;
        }
        let mut s = STATE.lock();
        let start = scan_run(&mut s, count, align_frames)?;
        for idx in start..start + count {
            bit_set(&mut s.bitmap, idx);
        }
        s.used += count;
        Some(pool_base() + (start * FRAME_SIZE) as u64)
    }

    pub fn free_frame(pa: u64) -> bool {
        free_frames(pa, 1)
    }

    pub fn free_frames(pa: u64, count: usize) -> bool {
        if count == 0 {
            return false;
        }
        let base = pool_base();
        if pa < base || pa % FRAME_SIZE as u64 != 0 {
            return false;
        }
        let span_end = pa + (count * FRAME_SIZE) as u64;
        if span_end > base + FRAME_POOL_BYTES as u64 {
            return false;
        }
        let first = ((pa - base) / FRAME_SIZE as u64) as usize;
        let mut s = STATE.lock();
        for idx in first..first + count {
            if !bit_get(&s.bitmap, idx) {
                return false; // double free / foreign frame
            }
        }
        for idx in first..first + count {
            bit_clear(&mut s.bitmap, idx);
        }
        s.used -= count;
        true
    }

    /// Pool + heap bytes (loose equivalent of `mm_total_ram`).
    pub fn total_bytes() -> u64 {
        FRAME_POOL_BYTES as u64 + super::api::pool_bytes()
    }

    pub fn free_bytes() -> u64 {
        let s = STATE.lock();
        let frames_free = (FRAME_COUNT - s.used) as u64 * FRAME_SIZE as u64;
        frames_free + super::api::heap_free_bytes()
    }

    /// Low-half identity map in bare mode: phys == virt.
    pub const DIRECT_MAP_BASE: u64 = 0;

    pub fn phys_to_virt(phys: u64) -> *mut u8 {
        phys as *mut u8
    }
}

// ==== PART3: api — size-headered free-list heap =============================

pub mod api {
    use core::ffi::c_void;
    use spin::Mutex;

    const HEAP_BYTES: usize = 2 * 1024 * 1024;
    /// Block header: [data_size: u64][block_base: u64][block_size: u64][pad].
    const HDR: usize = 32;
    const MIN_ALIGN: usize = 16;

    #[repr(C, align(16))]
    struct HeapPool([u8; HEAP_BYTES]);

    static POOL: HeapPool = HeapPool([0; HEAP_BYTES]);

    struct HeapState {
        /// Address of the first free block (0 = none).
        head: usize,
    }

    static STATE: Mutex<HeapState> = Mutex::new(HeapState { head: 0 });

    fn pool_base() -> usize {
        &POOL as *const HeapPool as usize
    }

    /// Called once from `init()` — the whole pool becomes one free block.
    pub fn init() {
        let base = pool_base();
        let mut s = STATE.lock();
        unsafe {
            // [size = whole pool][base = self][block_size][pad]
            *(base as *mut u64) = HEAP_BYTES as u64;
            *((base + 8) as *mut u64) = base as u64;
            *((base + 16) as *mut u64) = HEAP_BYTES as u64;
        }
        s.head = base;
    }

    pub fn pool_bytes() -> u64 {
        HEAP_BYTES as u64
    }

    pub fn heap_free_bytes() -> u64 {
        let s = STATE.lock();
        let mut cur = s.head;
        let mut total = 0u64;
        while cur != 0 {
            unsafe {
                let bsize = *(cur as *const u64) as usize;
                total += bsize as u64;
                cur = *((cur + HDR) as *const usize); // next pointer in data area
            }
        }
        total
    }

    fn push_free(s: &mut HeapState, block: usize) {
        unsafe {
            *((block + HDR) as *mut usize) = s.head; // next
        }
        s.head = block;
    }

    /// Allocate `size` bytes, 16-byte aligned (first fit). Blocks are carved
    /// from the front of free blocks; remainders stay on the freelist. No
    /// coalescing yet — bounded by pool size, fine for the scaffold heap.
    fn raw_alloc(size: usize, align: usize) -> *mut u8 {
        let mut s = STATE.lock();
        if s.head == 0 {
            return core::ptr::null_mut();
        }

        let mut prev: usize = 0;
        let mut cur = s.head;
        while cur != 0 {
            let bsize = unsafe { *(cur as *const u64) } as usize;
            let want = if align <= MIN_ALIGN {
                size + HDR
            } else {
                size + align + HDR // slack for over-aligned placement
            };
            if bsize >= want {
                let (hdr, take, next) = if align <= MIN_ALIGN {
                    let take = if bsize - (size + HDR) < HDR {
                        bsize
                    } else {
                        size + HDR
                    };
                    let next = if take < bsize { cur + take } else { 0 };
                    (cur, take, next)
                } else {
                    let data = ((cur + HDR + align - 1) & !(align - 1))
                        .max(cur + HDR);
                    (data - HDR, bsize, 0) // hdr placed at aligned position
                };

                // Unlink cur; possibly re-link front and back remainders.
                if prev == 0 {
                    s.head = unsafe { *((cur + HDR) as *const usize) };
                } else {
                    unsafe {
                        *((prev + HDR) as *mut usize) =
                            *((cur + HDR) as *const usize);
                    }
                }

                if align <= MIN_ALIGN {
                    if next != 0 {
                        unsafe {
                            *(next as *mut u64) = (bsize - take) as u64;
                            *((next + 8) as *mut u64) = next as u64;
                            *((next + 16) as *mut u64) = (bsize - take) as u64;
                            *((next + HDR) as *mut usize) =
                                *((cur + HDR) as *const usize);
                        }
                        if prev == 0 {
                            s.head = next;
                        } else {
                            unsafe { *((prev + HDR) as *mut usize) = next };
                        }
                    }
                } else {
                    let front = hdr - cur;
                    if front >= HDR {
                        unsafe {
                            *(cur as *mut u64) = front as u64;
                            *((cur + 8) as *mut u64) = cur as u64;
                            *((cur + 16) as *mut u64) = front as u64;
                        }
                        push_free(&mut s, cur);
                    }
                    let back_start = hdr + HDR + size;
                    let back_end = cur + bsize;
                    if back_end >= back_start + HDR {
                        unsafe {
                            *(back_start as *mut u64) =
                                (back_end - back_start) as u64;
                            *((back_start + 8) as *mut u64) = back_start as u64;
                            *((back_start + 16) as *mut u64) =
                                (back_end - back_start) as u64;
                        }
                        push_free(&mut s, back_start);
                    }
                }

                unsafe {
                    *(hdr as *mut u64) = size as u64; // data size
                    *((hdr + 8) as *mut u64) = cur as u64; // block base
                    *((hdr + 16) as *mut u64) = take as u64; // block size
                }
                return (hdr + HDR) as *mut u8;
            }
            prev = cur;
            cur = unsafe { *((cur + HDR) as *const usize) };
        }
        core::ptr::null_mut()
    }

    /// Data size of a live allocation (0 for null/foreign pointers).
    unsafe fn data_size(ptr: *mut u8) -> usize {
        if ptr.is_null() {
            return 0;
        }
        let hdr = ptr as usize - HDR;
        if hdr < pool_base() || hdr >= pool_base() + HEAP_BYTES {
            return 0;
        }
        *(hdr as *const u64) as usize
    }

    /// Allocate `size` bytes; returns `None` on failure (matches x86 `mm::api`).
    pub fn kmalloc(size: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }
        let p = raw_alloc(size, MIN_ALIGN);
        if p.is_null() { None } else { Some(p) }
    }

    pub fn kmalloc_aligned(size: usize, align: usize) -> Option<*mut u8> {
        if size == 0 || align == 0 || !align.is_power_of_two() {
            return None;
        }
        let p = raw_alloc(size, align.max(MIN_ALIGN));
        if p.is_null() { None } else { Some(p) }
    }

    pub fn kzalloc(size: usize) -> Option<*mut u8> {
        let p = kmalloc(size)?;
        unsafe { core::ptr::write_bytes(p, 0, size) };
        Some(p)
    }

    pub fn kcalloc(count: usize, size: usize) -> Option<*mut u8> {
        count.checked_mul(size).and_then(kzalloc)
    }

    pub fn kfree(ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            let hdr = ptr as usize - HDR;
            if hdr < pool_base() || hdr >= pool_base() + HEAP_BYTES {
                return; // foreign pointer — ignore
            }
            let base = *((hdr + 8) as *const usize) as usize;
            if base < pool_base() || base >= pool_base() + HEAP_BYTES {
                return;
            }
            let mut s = STATE.lock();
            push_free(&mut s, base);
        }
    }

    pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
        if ptr.is_null() {
            return kmalloc(size);
        }
        if size == 0 {
            kfree(ptr);
            return None;
        }
        unsafe {
            let old = data_size(ptr);
            let p = kmalloc(size)?;
            core::ptr::copy_nonoverlapping(ptr, p, old.min(size));
            kfree(ptr);
            Some(p)
        }
    }

    pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
        pages.checked_mul(4096).and_then(|b| kmalloc_aligned(b, 4096))
    }

    pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
        kfree(ptr)
    }

    /// Identity map in bare mode: VA == PA for the heap pool.
    pub fn kvirt_to_phys(ptr: *mut u8) -> u64 {
        let p = ptr as usize;
        if p >= pool_base() && p < pool_base() + HEAP_BYTES {
            p as u64
        } else {
            0
        }
    }
}

// ==== PART4: virt — VA range allocator ======================================

pub mod virt {
    use alloc::vec::Vec;
    use core::ops::BitOr;
    use spin::Mutex;

    /*
     * Same flag numeration as the x86_64 `virt.rs` (VMM_FLAG_* from
     * mm/alloc/virtual/vmm.h). Ranges are only RESERVED here — the MMU runs
     * in bare mode, so a reserved VA window is not backed by page tables yet.
     * The window sits below QEMU-virt RAM (0x8000_0000) and above the
     * peripheral block, in otherwise unused address space.
     */
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct VmmFlags(u32);

    impl VmmFlags {
        pub const NONE: VmmFlags = VmmFlags(0);
        pub const WRITE: VmmFlags = VmmFlags(1 << 0);
        pub const USER: VmmFlags = VmmFlags(1 << 1);
        pub const NX: VmmFlags = VmmFlags(1 << 2);
        pub const DEVICE: VmmFlags = VmmFlags(1 << 3);
        pub const ZERO: VmmFlags = VmmFlags(1 << 4);

        pub const fn bits(self) -> u32 {
            self.0
        }
    }

    impl BitOr for VmmFlags {
        type Output = VmmFlags;

        fn bitor(self, rhs: VmmFlags) -> VmmFlags {
            VmmFlags(self.0 | rhs.0)
        }
    }

    pub const WRITE: VmmFlags = VmmFlags::WRITE;
    pub const USER: VmmFlags = VmmFlags::USER;
    pub const NX: VmmFlags = VmmFlags::NX;
    pub const DEVICE: VmmFlags = VmmFlags::DEVICE;
    pub const ZERO: VmmFlags = VmmFlags::ZERO;

    const WINDOW_BASE: u64 = 0x4000_0000;
    const WINDOW_LEN: u64 = 0x4000_0000; // 1 GiB

    struct DeviceMap {
        va: u64,
        phys: u64,
        len: u64,
    }

    struct State {
        cursor: u64,
        taken: Vec<(u64, u64)>, // (va, len) reserved ranges
        devices: Vec<DeviceMap>,
    }

    static STATE: Mutex<State> = Mutex::new(State {
        cursor: WINDOW_BASE,
        taken: Vec::new(),
        devices: Vec::new(),
    });

    pub fn init() {
        let mut s = STATE.lock();
        s.cursor = WINDOW_BASE;
        s.taken.clear();
        s.devices.clear();
    }

    fn overlaps(s: &State, va: u64, len: u64) -> bool {
        s.taken
            .iter()
            .any(|(t, l)| va < t + *l && *t < va + len)
    }

    fn take_range(s: &mut State, len: u64) -> Option<u64> {
        // scan from cursor to window end
        let mut va = s.cursor;
        while va + len <= WINDOW_BASE + WINDOW_LEN {
            if !overlaps(s, va, len) {
                s.taken.push((va, len));
                s.cursor = va + len;
                return Some(va);
            }
            va += 0x1000;
        }
        // wrap-around scan
        va = WINDOW_BASE;
        while va < s.cursor {
            if !overlaps(s, va, len) {
                s.taken.push((va, len));
                return Some(va);
            }
            va += 0x1000;
        }
        None
    }

    /// Reserve a `bytes`-sized (page-rounded) VA range.
    pub fn alloc(bytes: usize, _flags: VmmFlags) -> Option<u64> {
        if bytes == 0 {
            return None;
        }
        let len = ((bytes + 0xFFF) & !0xFFF) as u64;
        let mut s = STATE.lock();
        take_range(&mut s, len)
    }

    /// Release a previously reserved range.
    pub fn free(va: u64, bytes: usize) -> bool {
        let len = ((bytes + 0xFFF) & !0xFFF) as u64;
        let mut s = STATE.lock();
        match s.taken.iter().position(|&(t, l)| t == va && l == len) {
            Some(i) => {
                s.taken.remove(i);
                true
            }
            None => false,
        }
    }

    /// Reserve a window for a device at `phys` (MMIO). The VA→PA pairing is
    /// recorded so `unmap_device` can validate the call; actual mapping
    /// happens when Sv39 paging is enabled (identity access works today).
    pub fn map_device(phys: u64, len: usize) -> Option<u64> {
        let plen = ((len + 0xFFF) & !0xFFF) as u64;
        let va = {
            let mut s = STATE.lock();
            take_range(&mut s, plen)?
        };
        let mut s = STATE.lock();
        s.devices.push(DeviceMap { va, phys, len: plen });
        Some(va)
    }

    pub fn unmap_device(va: u64, len: usize) -> bool {
        let plen = ((len + 0xFFF) & !0xFFF) as u64;
        let mut s = STATE.lock();
        match s.devices.iter().position(|d| d.va == va && d.len == plen) {
            Some(i) => {
                s.devices.remove(i);
                s.taken
                    .retain(|&(t, l)| !(t == va && l == plen));
                true
            }
            None => false,
        }
    }
}

// ==== PART5: space — Sv39 page tables (software-managed) ====================

pub mod space {
    use alloc::vec::Vec;
    use core::ffi::c_void;
    use core::ops::BitOr;
    use spin::Mutex;

    // Sv39 PTE bits
    const PTE_V: u64 = 1 << 0;
    const PTE_R: u64 = 1 << 1;
    const PTE_W: u64 = 1 << 2;
    const PTE_X: u64 = 1 << 3;
    const PTE_U: u64 = 1 << 4;
    const PTE_A: u64 = 1 << 6;
    const PTE_D: u64 = 1 << 7;

    const PAGE: usize = 4096;

    /// Same bit numeration as the x86_64 `space.rs` (PROT_* / MAP_* from
    /// paging.h) so call sites stay source-compatible across backends.
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct ProtFlags(u32);

    impl ProtFlags {
        pub const NONE: ProtFlags = ProtFlags(0);
        pub const READ: ProtFlags = ProtFlags(1 << 0);
        pub const WRITE: ProtFlags = ProtFlags(1 << 1);
        pub const EXEC: ProtFlags = ProtFlags(1 << 2);
        pub const USER: ProtFlags = ProtFlags(1 << 3);
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

    fn sfence_all() {
        unsafe {
            core::arch::asm!("sfence.vma zero, zero", options(nostack));
        }
    }

    fn pte_for_prot(prot: ProtFlags) -> u64 {
        let b = prot.bits();
        let mut f = PTE_A | PTE_D;
        if b == 0 || b & (1 << 0) != 0 {
            f |= PTE_R;
        }
        if b & (1 << 1) != 0 {
            f |= PTE_R | PTE_W;
        }
        if b & (1 << 2) != 0 {
            f |= PTE_X;
        }
        if b & (1 << 3) != 0 {
            f |= PTE_U;
        }
        f
    }

    struct Range {
        va: u64,
        len: usize,
        frames: Vec<u64>,
    }

    struct Inner {
        next_va: u64,
        brk: u64,
        ranges: Vec<Range>,
        /// PA of every page table owned by this space (root first) — freed on Drop.
        tables: Vec<u64>,
    }

    pub struct AddressSpace {
        /// Root table address (VA == PA in bare mode).
        root: usize,
        inner: Mutex<Inner>,
    }

    fn alloc_table() -> Option<usize> {
        let t = super::api::kzalloc(PAGE)? as usize;
        Some(t)
    }

    impl AddressSpace {
        pub fn new() -> Option<Self> {
            let root = alloc_table()?;
            Some(Self {
                root,
                inner: Mutex::new(Inner {
                    next_va: 0x0040_0000, // per-space VA cursor (above low hole)
                    brk: 0,
                    ranges: Vec::new(),
                    tables: alloc::vec![root as u64],
                }),
            })
        }

        /// Paging handle = root table pointer (FFI-shape compatibility).
        pub fn handle(&self) -> *mut c_void {
            self.root as *mut c_void
        }

        /// Sv39 `satp` value this space would install (MODE=8 | PPN of root).
        /// NOT loaded yet — enabling the MMU is the next milestone.
        pub fn cr3(&self) -> u64 {
            (8u64 << 60) | ((self.root as u64) >> 12)
        }

        fn ensure_table(entry: u64, tables: &mut Vec<u64>) -> Option<usize> {
            if entry & PTE_V != 0 {
                return Some(((entry >> 10) << 12) as usize);
            }
            let t = alloc_table()?;
            tables.push(t as u64);
            Some(t)
        }

        /// Map one 4 KiB page through the 3-level Sv39 walk, allocating
        /// intermediate tables as needed.
        fn map_page(
            root: usize,
            tables: &mut Vec<u64>,
            va: u64,
            pa: u64,
            flags: u64,
        ) -> Option<()> {
            unsafe {
                let i2 = ((va >> 30) & 0x1FF) as usize;
                let e2 = *((root + i2 * 8) as *const u64);
                let mid = Self::ensure_table(e2, tables)?;
                if e2 & PTE_V == 0 {
                    *((root + i2 * 8) as *mut u64) =
                        (((mid as u64) >> 12) << 10) | PTE_V;
                }
                let i1 = ((va >> 21) & 0x1FF) as usize;
                let e1 = *((mid + i1 * 8) as *const u64);
                let leaf = Self::ensure_table(e1, tables)?;
                if e1 & PTE_V == 0 {
                    *((mid + i1 * 8) as *mut u64) =
                        (((leaf as u64) >> 12) << 10) | PTE_V;
                }
                let i0 = ((va >> 12) & 0x1FF) as usize;
                *((leaf + i0 * 8) as *mut u64) = ((pa >> 12) << 10) | flags | PTE_V;
            }
            Some(())
        }

        /// Software walk: PA behind `va`, if mapped.
        pub fn translate(&self, va: u64) -> Option<u64> {
            unsafe {
                let root = self.root;
                let i2 = ((va >> 30) & 0x1FF) as usize;
                let e2 = *((root + i2 * 8) as *const u64);
                if e2 & PTE_V == 0 {
                    return None;
                }
                let mid = ((e2 >> 10) << 12) as usize;
                let i1 = ((va >> 21) & 0x1FF) as usize;
                let e1 = *((mid + i1 * 8) as *const u64);
                if e1 & PTE_V == 0 {
                    return None;
                }
                let leaf = ((e1 >> 10) << 12) as usize;
                let i0 = ((va >> 12) & 0x1FF) as usize;
                let e0 = *((leaf + i0 * 8) as *const u64);
                if e0 & PTE_V == 0 {
                    return None;
                }
                Some(((e0 >> 10) << 12) | (va & 0xFFF))
            }
        }
    }

    impl AddressSpace {
        /// Map physical memory into the space (per 4 KiB page).
        pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
            let pages = (len + PAGE - 1) / PAGE;
            let flags = pte_for_prot(prot);
            let mut inner = self.inner.lock();
            for i in 0..pages {
                let va = virt + (i * PAGE) as u64;
                let pa = phys + (i * PAGE) as u64;
                if Self::map_page(self.root, &mut inner.tables, va, pa, flags).is_none() {
                    return false;
                }
            }
            sfence_all();
            true
        }

        /// Allocate anonymous zeroed pages; `hint` (page-aligned) is honored,
        /// otherwise the per-space cursor hands out fresh VAs.
        pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
            if len == 0 {
                return None;
            }
            let pages = (len + PAGE - 1) / PAGE;
            let flags = pte_for_prot(prot);
            let mut inner = self.inner.lock();
            let va = if hint != 0 && hint % PAGE as u64 == 0 {
                hint
            } else {
                inner.next_va
            };
            let mut frames = Vec::new();
            for i in 0..pages {
                let page_va = va + (i * PAGE) as u64;
                let pa = super::phys::alloc_zero_frame()?;
                if Self::map_page(self.root, &mut inner.tables, page_va, pa, flags)
                    .is_none()
                {
                    let _ = super::phys::free_frame(pa);
                    for f in frames.drain(..) {
                        let _ = super::phys::free_frame(f);
                    }
                    return None;
                }
                frames.push(pa);
            }
            let end = va + (pages * PAGE) as u64;
            inner.next_va = inner.next_va.max(end);
            inner.ranges.push(Range { va, len, frames });
            sfence_all();
            Some(va)
        }

        /// POSIX-flavoured wrapper: `MAP_FIXED` with a non-zero `addr` maps at
        /// that address, otherwise the cursor picks one.
        pub fn mmap(
            &self,
            addr: u64,
            len: usize,
            prot: ProtFlags,
            flags: MapFlags,
        ) -> Option<u64> {
            if addr != 0 && flags.bits() & (1 << 3) != 0 {
                self.map_anon(addr, len, prot)
            } else {
                self.map_anon(0, len, prot)
            }
        }

        /// Unmap a previously created range (frees the anonymous frames).
        pub fn munmap(&self, addr: u64, len: usize) -> bool {
            let mut inner = self.inner.lock();
            let idx = match inner
                .ranges
                .iter()
                .position(|r| r.va == addr && r.len == len)
            {
                Some(i) => i,
                None => return false,
            };
            let range = inner.ranges.remove(idx);
            for (i, pa) in range.frames.iter().enumerate() {
                let _ = unmap_page_root(self.root, addr + (i * PAGE) as u64);
                let _ = super::phys::free_frame(*pa);
            }
            sfence_all();
            true
        }

        /// Change protection of every page in [addr, addr+len).
        pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
            let pages = (len + PAGE - 1) / PAGE;
            let flags = pte_for_prot(prot);
            let mut ok = true;
            for i in 0..pages {
                let va = addr + (i * PAGE) as u64;
                ok &= self.set_pte_flags(va, flags);
            }
            sfence_all();
            ok
        }

        fn set_pte_flags(&self, va: u64, flags: u64) -> bool {
            unsafe {
                let root = self.root;
                let i2 = ((va >> 30) & 0x1FF) as usize;
                let e2 = *((root + i2 * 8) as *const u64);
                if e2 & PTE_V == 0 {
                    return false;
                }
                let mid = ((e2 >> 10) << 12) as usize;
                let i1 = ((va >> 21) & 0x1FF) as usize;
                let e1 = *((mid + i1 * 8) as *const u64);
                if e1 & PTE_V == 0 {
                    return false;
                }
                let leaf = ((e1 >> 10) << 12) as usize;
                let i0 = ((va >> 12) & 0x1FF) as usize;
                let pte = (leaf + i0 * 8) as *mut u64;
                if *pte & PTE_V == 0 {
                    return false;
                }
                let frame = *pte & !0x3FF;
                *pte = frame | flags | PTE_V;
            }
            true
        }

        /// Program break: grows monotonically (mapping is up to the caller).
        pub fn brk(&self, new_brk: u64) -> u64 {
            let mut inner = self.inner.lock();
            if new_brk > inner.brk {
                inner.brk = new_brk;
            }
            inner.brk
        }

        /// Address-space switch. The MMU runs in bare mode under OpenSBI for
        /// now, so this only invalidates TLB state; installing `satp` comes
        /// with the paging-enable milestone.
        pub fn switch(&self) {
            sfence_all();
        }
    }

    fn unmap_page_root(root: usize, va: u64) -> bool {
        unsafe {
            let i2 = ((va >> 30) & 0x1FF) as usize;
            let e2 = *((root + i2 * 8) as *const u64);
            if e2 & PTE_V == 0 {
                return false;
            }
            let mid = ((e2 >> 10) << 12) as usize;
            let i1 = ((va >> 21) & 0x1FF) as usize;
            let e1 = *((mid + i1 * 8) as *const u64);
            if e1 & PTE_V == 0 {
                return false;
            }
            let leaf = ((e1 >> 10) << 12) as usize;
            let i0 = ((va >> 12) & 0x1FF) as usize;
            let pte = (leaf + i0 * 8) as *mut u64;
            if *pte & PTE_V == 0 {
                return false;
            }
            *pte = 0;
        }
        true
    }

    impl Drop for AddressSpace {
        fn drop(&mut self) {
            let mut inner = self.inner.lock();
            for range in inner.ranges.drain(..) {
                for (i, pa) in range.frames.iter().enumerate() {
                    let _ = unmap_page_root(self.root, range.va + (i * PAGE) as u64);
                    let _ = super::phys::free_frame(*pa);
                }
            }
            for t in inner.tables.drain(..) {
                super::api::kfree(t as *mut u8);
            }
        }
    }
}

/// Initialize the whole RISC-V MM backend (call once, after the arch heap).
pub fn init() -> bool {
    phys::init();
    api::init();
    virt::init();
    true
}

