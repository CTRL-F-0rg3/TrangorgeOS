//! The `malloc` family, and the `#[global_allocator]` `std` will be built with.
//!
//! `std`'s default allocator is `std::alloc::System`, which on a unix target is
//! a thin wrapper over exactly these functions. So this module is the only place
//! a `Vec` in a TrangorgeOS program can get its memory from — if it is wrong,
//! nothing else in the sysroot matters.
//!
//! # Error reporting
//!
//! `malloc` returns null and sets `errno`; it does not panic, and it must not.
//! A panic inside the allocator, from a program whose heap is already broken,
//! is a guaranteed double fault. Every failure path here sets `ENOMEM` and
//! returns null.
//!
//! # `free(NULL)` and `realloc(NULL, n)`
//!
//! Both are defined by C to be well defined, and both are what `std` emits in
//! ordinary code — `Vec` growth, `Vec::truncate` on an empty vector. Handling
//! them here rather than trusting the caller checks is the difference between
//! "works" and "works until a `Vec` happens to grow".

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};

use tgs_hal::errno;
use tgs_hal::heap::{Heap, HEADER, MIN_CLASS};


use crate::errno::set_errno;
use crate::types::{c_int, size_t};

/// The system allocator, so `malloc` and `#[global_allocator]` are one code path.
///
/// An `UnsafeCell<Heap>` rather than a bare `Heap`, because `GlobalAlloc`'s
/// methods take `&self` — there is nowhere to put a `&mut` — and the heap is
/// reached through a `static`.
pub struct System(UnsafeCell<Heap>);

// SAFETY: `System` is only ever reachable from one task. The kernel does not
// program `fs_base` yet, so there is no second task to race with — and `tgs-rt`
// refuses to spawn one rather than letting the assumption break silently. The
// moment the kernel grows real threads this needs a real lock, and the
// `tgs_hal::heap` module docs already say so.
unsafe impl Sync for System {}

impl System {
    /// The process-wide allocator. `const`, so it needs no lazy initialisation:
    /// the first `malloc` can happen before any of our code runs.
    const fn new() -> Self {
        Self(UnsafeCell::new(Heap::system()))
    }

    /// Run `f` against the heap.
    fn with<R>(&self, f: impl FnOnce(&mut Heap) -> R) -> R {
        // SAFETY: one task, and `f` cannot re-enter the allocator without
        // recursing into `with` — which nothing in this crate does.
        f(unsafe { &mut *self.0.get() })
    }
}

// SAFETY: every method forwards to `tgs_hal::heap::Heap`, whose contract is
// identical: a block comes from `alloc`/`realloc`/`alloc_zeroed` and goes back to
// `dealloc` exactly once, with the same alignment it left with. `realloc`
// preserves the surviving bytes, and a failed allocation returns null rather
// than unwinding.
unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.with(|h| h.alloc(layout))
            .map_or(ptr::null_mut(), |p| p.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(p) = NonNull::new(ptr) {
            // SAFETY: the caller promises `ptr` came from `System::alloc` and has
            // not been freed since. `Heap::dealloc` re-derives everything it
            // needs from the header below the payload, so a stale `layout` cannot
            // mislead it.
            self.with(|h| h.dealloc(p, layout));
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // Zeroing in the caller rather than in the heap: `calloc` has the same
        // requirement, and doing it here means `calloc` can be a thin wrapper
        // instead of a second implementation that could disagree about sizes.
        let p = unsafe { self.alloc(layout) };
        if !p.is_null() {
            // SAFETY: `p` is a fresh block of `layout.size()` bytes from `alloc`.
            unsafe { ptr::write_bytes(p, 0, layout.size()) };
        }
        p
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Some(old) = NonNull::new(ptr) else {
            // `realloc(NULL, n)` is `malloc(n)`.
            let l = Layout::from_size_align_unchecked(new_size.max(1), layout.align());
            return unsafe { self.alloc(l) };
        };
        // SAFETY: the caller promises `old` is a live block from this allocator.
        self.with(|h| h.realloc(old, layout, new_size))
            .map_or(ptr::null_mut(), |p| p.as_ptr())
    }
}

#[global_allocator]
static ALLOCATOR: System = System::new();

/// The live allocator.
///
/// A `static mut` would trip the `static_mut_refs` lint and, more to the point,
/// would let a future caller take two `&mut` at once. This is the only way
/// anything in the crate reaches the heap.
pub(crate) fn allocator() -> &'static System {
    &ALLOCATOR
}

fn oom() -> *mut u8 {
    set_errno(errno::ENOMEM);
    ptr::null_mut()
}

/// `void *malloc(size_t size)`
#[no_mangle]
pub extern "C" fn malloc(size: size_t) -> *mut u8 {
    // A zero-sized request must still get a distinct, freeable pointer: the C
    // standard allows null here, but `std` asserts the pointer is non-null in
    // several places, and a null that some code reads as success and other code
    // reads as failure is a trap.
    let size = size.max(1);
    let Ok(layout) = Layout::from_size_align(size, 16) else {
        return oom();
    };
    allocator().with(|h| h.alloc(layout)).map_or(oom(), |p| p.as_ptr())
}

/// `void free(void *ptr)`
#[no_mangle]
pub extern "C" fn free(ptr: *mut u8) {
    // `free(NULL)` is defined to do nothing.
    let Some(p) = NonNull::new(ptr) else {
        return;
    };
    // Recover the size the block was allocated with. `Heap::dealloc` ignores
    // this, but it is the honest thing to hand it, and a future allocator that
    // *does* need the size will not silently start trusting a stale one.
    let size = usable_size(ptr);
    let Ok(layout) = Layout::from_size_align(size.max(16), 16) else {
        return;
    };
    // SAFETY: `p` came from `malloc` on this heap and is freed once, which C
    // requires of every pointer passed to `free`.
    // SAFETY: the C contract of `free` is exactly this one.
    unsafe { allocator().with(|h| h.dealloc(p, layout)) };
}

/// `void *calloc(size_t n, size_t size)`
#[no_mangle]
pub extern "C" fn calloc(n: size_t, size: size_t) -> *mut u8 {
    // C requires the product to be treated as overflow rather than wrapping: a
    // `calloc` that quietly returns a small buffer for a huge request is a heap
    // overflow waiting to be found.
    let Some(total) = n.checked_mul(size) else {
        return oom();
    };
    let total = total.max(1);
    let Ok(layout) = Layout::from_size_align(total, 16) else {
        return oom();
    };
    // SAFETY: `allocator()` is the only heap, and the block is fresh.
    // SAFETY: `allocator()` is the only heap, and the block is fresh.
    let p = unsafe { allocator().alloc_zeroed(layout) };
    if p.is_null() {
        oom()
    } else {
        p
    }
}

/// `void *realloc(void *ptr, size_t size)`
#[no_mangle]
pub extern "C" fn realloc(ptr: *mut u8, size: size_t) -> *mut u8 {
    let Some(old) = NonNull::new(ptr) else {
        return malloc(size);
    };
    let old_size = usable_size(old.as_ptr());
    let size = size.max(1);
    // The new size is handed to the heap as a length, not as a `Layout`: the
    // heap re-derives the layout from its own class table, so validating it here
    // would only duplicate the check in a place that could disagree with it.
    if Layout::from_size_align(size, 16).is_err() {
        return oom();
    }
    let Ok(old_layout) = Layout::from_size_align(old_size.max(16), 16) else {
        return oom();
    };
    // SAFETY: `old` came from `malloc` and is still live, which is exactly
    // `Heap::realloc`'s contract; it copies `min(old_size, size)` bytes out.
    // SAFETY: the C contract of `realloc` is exactly this one.
    unsafe { allocator().with(|h| h.realloc(old, old_layout, size)) }
        .map_or(oom(), |p| p.as_ptr())
}

/// `void *aligned_alloc(size_t alignment, size_t size)`
#[no_mangle]
pub extern "C" fn aligned_alloc(alignment: size_t, size: size_t) -> *mut u8 {
    let Some(layout) = aligned_layout(alignment, size) else {
        return oom();
    };
    // SAFETY: `allocator()` is the only heap, and the block is fresh.
    allocator()
        .with(|h| h.alloc(layout))
        .map_or(oom(), |p| p.as_ptr())
}

/// `void *memalign(size_t alignment, size_t size)`
///
/// Not in C11, but in POSIX and in glibc, and `std` reaches for it on some
/// paths. It differs from `aligned_alloc` in that `size` need not be a multiple
/// of `alignment`, which [`aligned_layout`] deliberately does not require.
#[no_mangle]
pub extern "C" fn memalign(alignment: size_t, size: size_t) -> *mut u8 {
    aligned_alloc(alignment, size)
}

/// `void *valloc(size_t size)` — page-aligned.
///
/// Legacy and deprecated in glibc, but cheap to provide and still called by some
/// C code.
#[no_mangle]
pub extern "C" fn valloc(size: size_t) -> *mut u8 {
    let Some(layout) = aligned_layout(tgs_hal::heap::PAGE, size) else {
        return oom();
    };
    // SAFETY: `allocator()` is the only heap, and the block is fresh.
    allocator()
        .with(|h| h.alloc(layout))
        .map_or(oom(), |p| p.as_ptr())
}

/// `int posix_memalign(void **memptr, size_t alignment, size_t size)`
///
/// Note the inverted convention: it returns the *error number* (0 on success)
/// and does not touch `errno`. Getting that backwards is the classic way to
/// break every `posix_memalign` caller.
#[no_mangle]
pub extern "C" fn posix_memalign(
    memptr: *mut *mut u8,
    alignment: size_t,
    size: size_t,
) -> c_int {
    if memptr.is_null() {
        return errno::EINVAL.0;
    }
    // POSIX requires the alignment to be a power of two *and* a multiple of
    // `sizeof(void *)`.
    if !alignment.is_power_of_two() || alignment < core::mem::size_of::<*mut u8>() {
        return errno::EINVAL.0;
    }
    let Some(layout) = aligned_layout(alignment, size) else {
        return errno::ENOMEM.0;
    };
    // SAFETY: `allocator()` is the only heap, and the block is fresh.
    match allocator().with(|h| h.alloc(layout)) {
        Ok(p) => {
            // SAFETY: POSIX guarantees `memptr` points at writable storage for
            // one pointer when it is non-null.
            unsafe { *memptr = p.as_ptr() };
            0
        }
        Err(e) => e.0,
    }
}

/// `size_t malloc_usable_size(void *ptr)`
///
/// A glibc extension. `std` does not call it, but C code layered on top of this
/// one might, and a wrong answer here is a silent heap overflow.
#[no_mangle]
pub extern "C" fn malloc_usable_size(ptr: *mut u8) -> size_t {
    if ptr.is_null() {
        return 0;
    }
    usable_size(ptr)
}

/// The number of bytes the caller may actually write to `ptr`.
///
/// The class comes out of the header below the payload, so it is exact for
/// classed blocks. A dedicated block reports up to the next power of two above
/// the pointer, which overstates by the alignment slack — an overstatement is the
/// safe direction for a function whose only job is to stop a caller writing past
/// the end.
fn usable_size(ptr: *mut u8) -> size_t {
    const DEDICATED: size_t = usize::MAX;
    // SAFETY: `ptr` is non-null and came from this heap, so the word `HEADER`
    // bytes below it is the class this heap wrote. A pointer that did not come
    // from this heap is a caller bug, and C already makes that undefined.
    let class = unsafe { *(ptr.sub(HEADER) as *const size_t) };

    if class == DEDICATED {
        let granule = MIN_CLASS.next_power_of_two();
        let off = ptr as usize & (granule - 1);
        return (granule - off).max(MIN_CLASS);
    }
    let class = class.min(tgs_hal::heap::CLASSES - 1);
    (MIN_CLASS << class).saturating_sub(HEADER)
}

/// A layout for `size` bytes at `alignment`, or `None` if the request is
/// impossible.
fn aligned_layout(alignment: size_t, size: size_t) -> Option<Layout> {
    let alignment = alignment.max(16);
    if !alignment.is_power_of_two() {
        return None;
    }
    Layout::from_size_align(size.max(1), alignment).ok()
}
