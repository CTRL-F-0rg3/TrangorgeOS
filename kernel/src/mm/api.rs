use super::ffi;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;

pub fn kmalloc(size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::kmalloc(size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn kzalloc(size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::kzalloc(size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::krealloc(ptr as *mut c_void, size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn kfree(ptr: *mut u8) {
    unsafe { ffi::kfree(ptr as *mut c_void) }
}

/// Page-aligned allocation (stosy jądra, DMA buffers). Uzupełnia surface
/// `mm::api` tak, by była zgodna z backendem RISC-V (`riscv::api`),
/// który już wystawia te same funkcje — bez duplikowania logiki allokatora.
pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::kalloc_pages(pages) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
    unsafe { ffi::kfree_pages(ptr as *mut c_void, _pages) }
}

pub fn virt_to_phys(ptr: *mut u8) -> u64 {
    unsafe { ffi::kvirt_to_phys(ptr as *mut c_void) }
}

pub struct KernelAlloc;

#[global_allocator]
static ALLOCATOR: KernelAlloc = KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = if layout.align() <= 16 {
            ffi::kmalloc(layout.size())
        } else {
            ffi::kmalloc_aligned(layout.size(), layout.align())
        };

        p as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ffi::kfree(ptr as *mut c_void)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = if layout.align() <= 16 {
            ffi::kzalloc(layout.size())
        } else {
            ffi::kmalloc_aligned(layout.size(), layout.align())
        };

        p as *mut u8
    }
}

pub fn self_test() -> Result<&'static str, &'static str> {
    // kmalloc of various sizes + write/read
    let sizes: [usize; 6] = [8, 64, 256, 1024, 4096, 16384];
    let mut keep: [*mut u8; 6] = [core::ptr::null_mut(); 6];

    for (i, &sz) in sizes.iter().enumerate() {
        let p = kmalloc(sz).ok_or("api: kmalloc failed")?;

        unsafe { core::ptr::write_bytes(p, 0x5A, sz); }

        for k in 0..sz {
            if unsafe { *p.add(k) } != 0x5A {
                kfree(p);
                return Err("api: kmalloc write/read mismatch");
            }
        }

        keep[i] = p;
    }

    for p in keep.iter() {
        kfree(*p);
    }

    // kzalloc musi zerować
    let z = kzalloc(512).ok_or("api: kzalloc failed")?;

    for k in 0..512 {
        if unsafe { *z.add(k) } != 0 {
            kfree(z);
            return Err("api: kzalloc not zeroed");
        }
    }

    kfree(z);

    // kcalloc (FFI) — mnożenie + zerowanie
    let c = unsafe { ffi::kcalloc(4, 256) } as *mut u8;

    if c.is_null() {
        return Err("api: kcalloc failed");
    }

    for k in 0..(4 * 256) {
        if unsafe { *c.add(k) } != 0 {
            unsafe { ffi::kfree(c as *mut c_void) };
            return Err("api: kcalloc not zeroed");
        }
    }

    unsafe { ffi::kfree(c as *mut c_void) };

    // kmalloc_aligned (FFI) — wyrównanie
    let a = unsafe { ffi::kmalloc_aligned(1024, 256) } as *mut u8;

    if a.is_null() {
        return Err("api: kmalloc_aligned failed");
    }

    if (a as usize) % 256 != 0 {
        unsafe { ffi::kfree(a as *mut c_void) };
        return Err("api: kmalloc_aligned not aligned");
    }

    unsafe { ffi::kfree(a as *mut c_void) };

    // krealloc: rośnięcie + zachowanie danych, potem zmniejszenie
    let p = kmalloc(64).ok_or("api: kmalloc(p) failed")?;
    unsafe { core::ptr::write_bytes(p, 0x33, 64); }

    let q = krealloc(p, 4096).ok_or("api: krealloc grow failed")?;

    for k in 0..64 {
        if unsafe { *q.add(k) } != 0x33 {
            kfree(q);
            return Err("api: krealloc lost data on grow");
        }
    }

    let r = krealloc(q, 32).ok_or("api: krealloc shrink failed")?;

    for k in 0..32 {
        if unsafe { *r.add(k) } != 0x33 {
            kfree(r);
            return Err("api: krealloc lost data on shrink");
        }
    }

    kfree(r);

    // virt_to_phys dla wskaźnika z kmalloc
    let v = kmalloc(128).ok_or("api: kmalloc(v) failed")?;
    let phys = virt_to_phys(v);

    if phys == 0 || phys == u64::MAX {
        kfree(v);
        return Err("api: virt_to_phys failed");
    }

    kfree(v);

    Ok("heap api roundtrip")
}