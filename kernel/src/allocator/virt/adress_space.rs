use crate::allocator::config::{HEAP_END, HEAP_START};
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::VirtAddr;

pub struct AddressSpaceAllocator {
    next: AtomicU64,
    end: u64,
}

crate::test_module!({
    let allocator = AddressSpaceAllocator::new(0x1000, 0x1000 + 4096 * 3);

    let a = match allocator.reserve(4096, 4096) {
        Some(addr) => addr,
        None => return Err("failed to reserve a range in a fresh allocator"),
    };
    let b = match allocator.reserve(4096, 4096) {
        Some(addr) => addr,
        None => return Err("failed to reserve a second range"),
    };
    if a == b {
        return Err("reserve returned overlapping addresses");
    }
    if b.as_u64() < a.as_u64() + 4096 {
        return Err("reserved ranges overlap");
    }

    if allocator.reserve(4096, 4096).is_none() {
        return Err("failed to reserve the third and final available range");
    }
    if allocator.reserve(4096, 4096).is_some() {
        return Err("reserve succeeded past the configured end of the range");
    }

    Ok("virtual address space bump allocator verified")
});

impl AddressSpaceAllocator {
    pub const fn new(start: u64, end: u64) -> Self {
        AddressSpaceAllocator {
            next: AtomicU64::new(start),
            end,
        }
    }

    pub fn reserve(&self, size: usize, align: usize) -> Option<VirtAddr> {
        let align = align.max(1) as u64;
        loop {
            let current = self.next.load(Ordering::Relaxed);
            let aligned = (current + align - 1) & !(align - 1);
            let next = aligned.checked_add(size as u64)?;
            if next > self.end {
                return None;
            }
            if self
                .next
                .compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return Some(VirtAddr::new(aligned));
            }
        }
    }
}

pub static HEAP_REGION: AddressSpaceAllocator = AddressSpaceAllocator::new(HEAP_START, HEAP_END);
