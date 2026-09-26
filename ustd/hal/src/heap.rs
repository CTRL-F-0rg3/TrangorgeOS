//! The userspace heap: `mmap` regions, size-class free lists, and the
//! bookkeeping `malloc` needs.
//!
//! `std` will not link without a `#[global_allocator]`, and the only sensible
//! one here is the system allocator, which is what `tgs-libc` builds on top of
//! this module. So this is not an optimisation — it is load-bearing.
//!
//! # Design
//!
//! A **region** is one `mmap` of [`ARENA_BYTES`]. It is carved by a bump
//! pointer; when the bump pointer runs out, a new region is mapped. Freed
//! blocks go onto one of nine free lists, one per power-of-two size class from
//! [`MIN_CLASS`] (16 bytes) to [`MAX_CLASS`] (4096 bytes). A request larger
//! than [`MAX_CLASS`] gets a dedicated mapping of its own, which is `munmap`ped
//! on free: that keeps the common case cheap and the rare case trivially
//! correct.
//!
//! Two consequences worth stating, because both are deliberate:
//!
//! * **A freed block remembers only its size class**, taken from the header
//!   immediately below the payload. `realloc` that grows into a larger class
//!   therefore frees the old block without the caller having to say how big it
//!   was, and `dealloc` never has to trust a `Layout` the caller may have
//!   changed.
//! * **Nothing is coalesced.** A program that churns through many differently
//!   sized allocations eventually fragments and an allocation fails. That is a
//!   real limitation, called out in `README.md`, not an oversight: the right
//!   place to solve it is the kernel's page allocator, once `mmap` can be asked
//!   to hand physical pages back in bulk.
//!
//! # Thread safety
//!
//! None, deliberately. The kernel does not program `fs_base`, so there is no
//! TLS and no way to notice a second thread. [`Heap::alloc`] therefore
//! documents that it is single-threaded, and `tgs-rt` refuses to spawn threads
//! rather than silently corrupting the free lists.

use core::alloc::Layout;
use core::ptr::{self, NonNull};

use crate::errno::{self, Result};
use crate::gate::Syscall;
use crate::sysnum;

/// The page size the kernel and the rest of this crate assume.
pub const PAGE: usize = 4096;

/// The smallest block this allocator manages. A smaller request is rounded up,
/// which is what every real allocator does.
pub const MIN_CLASS: usize = 16;

/// The largest block served from a free list. Anything bigger gets its own
/// mapping.
pub const MAX_CLASS: usize = 4096;

/// How much one region reserves from the kernel.
pub const ARENA_BYTES: usize = 64 * 1024;

/// The number of size classes: 16, 32, ..., 4096.
pub const CLASSES: usize = 9;

/// How many regions a process may hold at once.
///
/// A fixed array, in `.bss`, and deliberately so: the allocator is reached from
/// `malloc`, and `malloc` is the global allocator every `Vec` in the program
/// also goes through. A region list that grew by allocating would recurse into
/// itself the first time it needed to grow. The same rule is why this module has
/// no `alloc` import at all.
pub const MAX_REGIONS: usize = 64;

/// One region handed out by `mmap`.
#[derive(Debug, Clone, Copy)]
struct Region {
    /// The address `mmap` returned.
    base: usize,
    /// How many bytes it covers.
    len: usize,
    /// The offset of the next never-used byte.
    bump: usize,
    /// Whether this region backs a single oversized allocation, and so has to
    /// be `munmap`ped on free rather than reused.
    dedicated: bool,
}

/// A free block, threaded through the block itself so the free lists need no
/// storage of their own.
#[derive(Debug, Clone, Copy)]
struct Free {
    /// The next free block in this class, or 0.
    next: usize,
}

/// The marker value in a [`Header::class`] for a block that has a mapping of its
/// own. It has to be a value no class can take.
const DEDICATED: usize = usize::MAX;

/// Bytes of bookkeeping immediately below every payload: the block's size class
/// and the block's own address.
///
/// Placing the header *before* the payload rather than inside it is what makes
/// the rest of this file correct. Two things depend on it:
///
/// * `dealloc` learns the block's address from the header alone, so it never has
///   to be told the alignment — which matters because a caller may well hand
///   over a stale `Layout`.
/// * Writing to a payload cannot destroy the metadata. With the header inside
///   the payload, a single `memcpy` into the first eight bytes of a `malloc`ed
///   block silently rewrote its own size class, and the block became
///   unreleasable on the next `free`.
pub const HEADER: usize = core::mem::size_of::<Header>();

/// The bookkeeping immediately below a payload.
#[derive(Debug, Clone, Copy)]
struct Header {
    /// The size class, or [`DEDICATED`] for an oversized block.
    class: usize,
    /// The first byte of the block. This is what the free lists are threaded
    /// through, because it is the only address a split or a free can recover
    /// without knowing the alignment.
    block: usize,
}

/// A snapshot of the heap's state, for the runtime's start-up banner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeapStats {
    /// How many regions are mapped.
    pub regions: usize,
    /// Total bytes mapped.
    pub mapped: usize,
    /// Bytes currently handed out.
    pub allocated: usize,
    /// Cumulative `mmap` calls.
    pub maps: usize,
    /// Cumulative `munmap` calls.
    pub unmaps: usize,
}

/// The system heap.
pub struct Heap {
    gate: &'static dyn Syscall,
    /// The regions in use, `regions[..used]`. A `Dedicated` region can be
    /// [`Heap::swap_remove`]d from the middle; an arena cannot, because bumping
    /// it depends on everything after it being intact.
    regions: [Region; MAX_REGIONS],
    used: usize,
    /// Head of each free list, as an address, or 0 when empty.
    free: [usize; CLASSES],
    /// Bytes currently handed out, for [`Heap::stats`].
    allocated: usize,
    /// Cumulative `mmap` calls, for [`Heap::stats`].
    maps: usize,
    /// Cumulative `munmap` calls, for [`Heap::stats`].
    unmaps: usize,
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("regions", &self.used)
            .field("allocated", &self.allocated)
            .field("maps", &self.maps)
            .field("unmaps", &self.unmaps)
            .finish()
    }
}

impl Heap {
    /// A heap that reaches the kernel through `gate`.
    pub const fn new(gate: &'static dyn Syscall) -> Self {
        Self {
            gate,
            regions: [Region {
                base: 0,
                len: 0,
                bump: 0,
                dedicated: false,
            }; MAX_REGIONS],
            used: 0,
            free: [0; CLASSES],
            allocated: 0,
            maps: 0,
            unmaps: 0,
        }
    }

    /// The system heap, reaching the real kernel through [`crate::gate::Gate`].
    ///
    /// `const` so a `tgs-libc` can put one in a `static` and be the
    /// `#[global_allocator]` without any lazy initialisation — which matters,
    /// because the first `malloc` happens long before any code of ours runs.
    pub const fn system() -> Self {
        Self::new(&crate::gate::GATE)
    }

    /// What the heap knows about itself, for the runtime's start-up banner.
    pub fn stats(&self) -> HeapStats {
        HeapStats {
            regions: self.used,
            mapped: self.regions[..self.used].iter().map(|r| r.len).sum(),
            allocated: self.allocated,
            maps: self.maps,
            unmaps: self.unmaps,
        }
    }

    /// Take the region at `i` out of the list by moving the last one into its
    /// place.
    fn swap_remove(&mut self, i: usize) -> Region {
        self.used -= 1;
        self.regions.swap(i, self.used);
        self.regions[self.used]
    }

    /// Add a region, or report [`errno::ENOMEM`] if the table is full.
    fn push(&mut self, region: Region) -> Result<()> {
        let at = self.used;
        if at == MAX_REGIONS {
            return Err(errno::ENOMEM);
        }
        self.regions[at] = region;
        self.used += 1;
        Ok(())
    }

    /// The number of bytes a request of `size` bytes at alignment `align` has to
    /// fit in, header included.
    ///
    /// The alignment term is the subtle one. A payload sits at
    /// `align_up(block + HEADER, align)`, so the most it can drift from
    /// `block + HEADER` is `align - 1`. Reserving a whole `align` on top of the
    /// header therefore always leaves at least one spare byte, and
    /// `payload + size` provably stays inside the block. Alignments up to
    /// [`HEADER`] need no reservation at all, because the block itself is
    /// [`HEADER`]-aligned, which is why an ordinary 8- or 16-byte `malloc` pays
    /// nothing for it.
    const fn needed(size: usize, align: usize) -> usize {
        let slack = if align > HEADER { align } else { 0 };
        size.saturating_add(HEADER).saturating_add(slack)
    }

    /// The payload address of a block that starts at `block`.
    const fn payload_addr(block: usize, align: usize) -> usize {
        Self::align_up(block + HEADER, align)
    }

    /// The class `size` bytes at alignment `align` land in, or `None` when the
    /// request is oversized.
    const fn class_of(size: usize, align: usize) -> Option<usize> {
        let want = Self::needed(size, align);
        let mut i = 0;
        let mut c = MIN_CLASS;
        while i < CLASSES {
            if want <= c {
                return Some(i);
            }
            i += 1;
            c *= 2;
        }
        None
    }

    /// The total size of a block in class `i`, header included.
    const fn class_bytes(i: usize) -> usize {
        MIN_CLASS << i
    }

    /// Round `addr` up to a multiple of `align`, which is a power of two.
    const fn align_up(addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }

    /// Allocate a block for `layout`, or return [`errno::ENOMEM`].
    ///
    /// The block is `layout.align()`-aligned and at least `layout.size()`
    /// bytes long.
    ///
    /// # Safety
    ///
    /// This crate is single-threaded until the kernel programs `fs_base`; see
    /// the module docs. Calling this from two tasks at once corrupts the free
    /// lists.
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>> {
        let size = layout.size().max(1);
        let align = layout.align().max(MIN_CLASS);

        match Self::class_of(size, align) {
            Some(class) => self.alloc_classed(class, align),
            None => self.alloc_dedicated(size, align),
        }
    }

    /// Serve `class` from a free list, a wider free list, or a fresh region.
    fn alloc_classed(&mut self, class: usize, align: usize) -> Result<NonNull<u8>> {
        // First-fit from the matching free list, then widen to the next class up
        // and split. Widening is what stops a 4096-byte block from being useless
        // to a 48-byte request.
        for c in class..CLASSES {
            if self.free[c] == 0 {
                continue;
            }
            let block = self.free[c];
            // SAFETY: a free list only ever holds addresses this heap produced
            // and the caller later gave back, so `block` is a live block of at
            // least `class_bytes(c)` bytes with a `Free` header in it.
            let next = unsafe { *(block as *const Free) }.next;
            self.free[c] = next;
            let total = Self::class_bytes(c);
            self.carve(block, total, class, align);
            return Ok(self.payload(block, total, class, align));
        }

        let total = Self::class_bytes(class);
        let block = self.bump_alloc(total, align)?;
        self.carve(block, total, class, align);
        Ok(self.payload(block, total, class, align))
    }

    /// Shrink a block of `total` bytes down to `class`, returning the tail to a
    /// free list when it is big enough to be useful.
    fn carve(&mut self, block: usize, total: usize, class: usize, align: usize) {
        let want = Self::class_bytes(class);
        if total <= want {
            return;
        }
        let Some(tail_class) = Self::class_of(total - want, align) else {
            return;
        };
        // A tail has to be able to hold a header plus a minimal payload, or it
        // is not worth a free list.
        if Self::class_bytes(tail_class) < HEADER + MIN_CLASS {
            return;
        }
        // `want` is a power of two of at least `MIN_CLASS`, and every block
        // starts `MIN_CLASS`-aligned, so the tail is too and can carry a `Free`
        // header at its own base.
        let tail = block + want;
        // SAFETY: `tail` is inside the block the caller is about to take for
        // itself, `class_bytes(tail_class) <= total - want` by construction, and
        // `tail` is `MIN_CLASS`-aligned.
        unsafe {
            ptr::write(
                tail as *mut Free,
                Free {
                    next: self.free[tail_class],
                },
            );
        }
        self.free[tail_class] = tail;
    }

    /// The payload address of a block, with its header written.
    fn payload(
        &mut self,
        block: usize,
        total: usize,
        class: usize,
        align: usize,
    ) -> NonNull<u8> {
        let at = Self::payload_addr(block, align);
        self.allocated += total;
        // SAFETY: `align_up(block + HEADER, align)` is at least `block + HEADER`,
        // so the header is inside the block, and `needed` reserved enough room
        // that `at + size` stays inside it too. `at` is never 0.
        unsafe {
            ptr::write(
                (at - HEADER) as *mut Header,
                Header {
                    class,
                    block,
                },
            );
            NonNull::new_unchecked(at as *mut u8)
        }
    }

    /// Find room in an existing region, or map a new one.
    fn bump_alloc(&mut self, total: usize, align: usize) -> Result<usize> {
        for r in self.regions[..self.used].iter_mut() {
            if r.dedicated {
                continue;
            }
            let start = Self::align_up(r.base + r.bump, align);
            let end = start
                .checked_sub(r.base)
                .and_then(|o| o.checked_add(total));
            if end.is_some_and(|e| e <= r.len) {
                r.bump = start - r.base + total;
                return Ok(start);
            }
        }

        let len = ARENA_BYTES.max(total).next_multiple_of(PAGE);
        let addr = self.map(len)?;
        self.push(Region {
            base: addr,
            len,
            bump: total,
            dedicated: false,
        })?;
        Ok(addr)
    }

    /// Serve an oversized request with a mapping of its own.
    fn alloc_dedicated(&mut self, want: usize, align: usize) -> Result<NonNull<u8>> {
        let len = Self::needed(want, align).next_multiple_of(PAGE);
        let base = self.map(len)?;
        debug_assert_eq!(
            base % align,
            0,
            "the kernel must honour the alignment mmap was asked for",
        );
        self.push(Region {
            base,
            len,
            bump: len,
            dedicated: true,
        })?;
        // `payload` is what accounts the bytes; adding them here as well would
        // double-count the mapping and make `dealloc` leave a permanent residue.
        Ok(self.payload(base, len, DEDICATED, align))
    }

    /// Hand a block back.
    ///
    /// `layout` is accepted for parity with the `GlobalAlloc` trait and
    /// deliberately ignored: the header immediately below the payload records the
    /// class and the block address, so this stays correct even when a caller
    /// hands over a stale layout.
    ///
    /// # Safety
    ///
    /// `ptr` must have come from [`Heap::alloc`] on this heap, must not have
    /// been freed since, and must not overlap any other live block.
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        let at = ptr.as_ptr() as usize;
        // SAFETY: the caller promises the pointer came from `alloc`, so the
        // header this heap wrote immediately below it is present and readable.
        let h = unsafe { *((at - HEADER) as *const Header) };

        if h.class != DEDICATED {
            let class = h.class.min(CLASSES - 1);
            self.allocated = self.allocated.saturating_sub(Self::class_bytes(class));
            // SAFETY: the block starts at `h.block`, is at least
            // `class_bytes(class)` long and is ours again. The `Free` header
            // overwrites the `Header`, which is fine: a free block's metadata is
            // the free list link, and `payload` rewrites both when it is reused.
            unsafe {
                ptr::write(
                    h.block as *mut Free,
                    Free {
                        next: self.free[class],
                    },
                );
            }
            self.free[class] = h.block;
            return;
        }

        // A dedicated mapping: hand the pages back to the kernel. A block whose
        // address matches no region is left alone rather than unmapped blindly.
        if let Some(i) = self.regions[..self.used]
            .iter()
            .position(|r| r.dedicated && r.base == h.block)
        {
            let r = self.swap_remove(i);
            self.allocated = self.allocated.saturating_sub(r.len);
            self.unmap(r.base, r.len);
        }
    }

    /// Resize a block, in place when the physical block is already big enough.
    ///
    /// # Safety
    ///
    /// `ptr` must have come from this heap with `layout`, and must not have been
    /// freed since.
    pub unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>> {
        let at = ptr.as_ptr() as usize;
        // SAFETY: the caller promises `ptr` is a live block from this heap, so
        // the header immediately below the payload is ours to read.
        let h = unsafe { *((at - HEADER) as *const Header) };

        if h.class != DEDICATED {
            let old_total = Self::class_bytes(h.class.min(CLASSES - 1));
            let align = layout.align().max(MIN_CLASS);
            // Too big for any free list? Then only a dedicated block could help,
            // and a free-list block never is.
            let fits = match Self::class_of(new_size, align) {
                Some(i) => Self::class_bytes(i) <= old_total,
                None => false,
            };
            if fits {
                // The header is deliberately left alone: it describes the
                // physical block, which is what `dealloc` needs.
                return Ok(ptr);
            }
        }

        // SAFETY: the caller promises `ptr` is a live block from this heap, and
        // `alloc_copy`'s contract is exactly that.
        unsafe { self.alloc_copy(ptr, layout, new_size) }
    }

    /// Allocate, copy the surviving bytes, free the old block.
    unsafe fn alloc_copy(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<NonNull<u8>> {
        // A `realloc` can legitimately be asked for a size the old alignment
        // cannot express (`Layout` requires size + align - 1 to fit in a `usize`).
        // Halving the alignment until it fits beats panicking inside an
        // allocator, and beats silently keeping an alignment the caller did not
        // ask for.
        let new_layout = {
            let mut align = layout.align();
            loop {
                if let Ok(l) = Layout::from_size_align(new_size, align) {
                    break l;
                }
                match align.checked_next_power_of_two() {
                    Some(0) | Some(1) | None => {
                        // Nothing left to give up. A 1-byte alignment always
                        // works, because `new_size` is itself a `usize`.
                        // SAFETY: `new_size >= 1` and an alignment of 1 is the
                        // weakest possible, so the pair is always valid.
                        break unsafe { Layout::from_size_align_unchecked(new_size, 1) };
                    }
                    Some(next) => align = next / 2,
                }
            }
        };

        let fresh = self.alloc(new_layout)?;
        // SAFETY: the caller promises `ptr` is a live block of at least
        // `layout.size()` bytes and still owned; `fresh` is at least
        // `new_size` bytes and cannot overlap `ptr`, because it came either from
        // a free list or from a mapping that did not exist when `ptr` was handed
        // out.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), fresh.as_ptr(), layout.size().min(new_size));
            self.dealloc(ptr, layout);
        }
        Ok(fresh)
    }

    /// Ask the kernel for `len` bytes of anonymous, private, read-write memory.
    fn map(&mut self, len: usize) -> Result<usize> {
        // SAFETY: the mapping is anonymous, so the kernel copies nothing out of
        // the arguments and the result carries no obligation that outlives the
        // call.
        let raw = unsafe {
            self.gate.call(
                sysnum::MMAP,
                [
                    0,
                    len as u64,
                    (sysnum::PROT_READ | sysnum::PROT_WRITE) as u64,
                    (sysnum::MAP_PRIVATE | sysnum::MAP_ANONYMOUS) as u64,
                    -1i64 as u64, // no file
                    0,
                ],
            )
        };
        self.maps += 1;
        let addr = errno::Errno::from_raw(raw)?;
        if addr == 0 {
            // A kernel that reports success but hands back address 0 has failed.
            // Every later write would hit the unmapped page at zero, so turn it
            // into the error it actually is.
            return Err(errno::ENOMEM);
        }
        Ok(addr)
    }

    /// Give `len` bytes at `addr` back to the kernel.
    fn unmap(&mut self, addr: usize, len: usize) {
        // SAFETY: `addr` and `len` are exactly what [`Heap::map`] was given for
        // this region, and no live block points into it because it was
        // dedicated.
        unsafe {
            self.gate
                .call(sysnum::MUNMAP, [addr as u64, len as u64, 0, 0, 0, 0]);
        }
        self.unmaps += 1;
    }
}
