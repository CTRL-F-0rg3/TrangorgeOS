use core::sync::atomic::{AtomicUsize, Ordering};

pub static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
pub static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
pub static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
pub static BYTES_FREED: AtomicUsize = AtomicUsize::new(0);
pub static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);

crate::test_module!({
    let start_allocs = ALLOCATIONS.load(Ordering::Relaxed);
    let start_in_use = bytes_in_use();

    record_alloc(128);
    record_alloc(256);
    record_dealloc(128);

    if ALLOCATIONS.load(Ordering::Relaxed) != start_allocs + 2 {
        return Err("allocation counter did not increment correctly");
    }
    if bytes_in_use() != start_in_use + 256 {
        return Err("bytes_in_use accounting is inconsistent after alloc/dealloc");
    }

    record_dealloc(256);
    if bytes_in_use() != start_in_use {
        return Err("bytes_in_use did not return to baseline after freeing everything");
    }

    Ok("allocation stats accounting verified")
});

pub fn record_alloc(size: usize) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    let total_allocated = BYTES_ALLOCATED.fetch_add(size, Ordering::Relaxed) + size;
    let freed = BYTES_FREED.load(Ordering::Relaxed);
    let in_use = total_allocated.saturating_sub(freed);

    let mut peak = PEAK_BYTES.load(Ordering::Relaxed);
    while in_use > peak {
        match PEAK_BYTES.compare_exchange_weak(peak, in_use, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => break,
            Err(actual) => peak = actual,
        }
    }
}

pub fn record_dealloc(size: usize) {
    DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    BYTES_FREED.fetch_add(size, Ordering::Relaxed);
}

pub fn bytes_in_use() -> usize {
    BYTES_ALLOCATED
        .load(Ordering::Relaxed)
        .saturating_sub(BYTES_FREED.load(Ordering::Relaxed))
}
