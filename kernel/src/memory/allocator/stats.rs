
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryStats {
    pub frames_allocated: AtomicUsize,
    pub frames_freed: AtomicUsize,
    pub heap_bytes_allocated: AtomicUsize,
    pub heap_bytes_freed: AtomicUsize,
    pub heap_allocation_failures: AtomicUsize,
}

impl MemoryStats {
    pub const fn new() -> Self {
        Self {
            frames_allocated: AtomicUsize::new(0),
            frames_freed: AtomicUsize::new(0),
            heap_bytes_allocated: AtomicUsize::new(0),
            heap_bytes_freed: AtomicUsize::new(0),
            heap_allocation_failures: AtomicUsize::new(0),
        }
    }

    pub fn record_frame_alloc(&self) { self.frames_allocated.fetch_add(1, Ordering::Relaxed); }
    pub fn record_frame_free(&self) { self.frames_freed.fetch_add(1, Ordering::Relaxed); }
    pub fn record_heap_alloc(&self, bytes: usize) { self.heap_bytes_allocated.fetch_add(bytes, Ordering::Relaxed); }
    pub fn record_heap_free(&self, bytes: usize) { self.heap_bytes_freed.fetch_add(bytes, Ordering::Relaxed); }
    pub fn record_heap_failure(&self) { self.heap_allocation_failures.fetch_add(1, Ordering::Relaxed); }

    pub fn snapshot(&self) -> MemoryStatsSnapshot {
        MemoryStatsSnapshot {
            frames_allocated: self.frames_allocated.load(Ordering::Relaxed),
            frames_freed: self.frames_freed.load(Ordering::Relaxed),
            heap_bytes_allocated: self.heap_bytes_allocated.load(Ordering::Relaxed),
            heap_bytes_freed: self.heap_bytes_freed.load(Ordering::Relaxed),
            heap_allocation_failures: self.heap_allocation_failures.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStatsSnapshot {
    pub frames_allocated: usize,
    pub frames_freed: usize,
    pub heap_bytes_allocated: usize,
    pub heap_bytes_freed: usize,
    pub heap_allocation_failures: usize,
}

pub static STATS: MemoryStats = MemoryStats::new();