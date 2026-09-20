use core::sync::atomic::{AtomicI32, Ordering};

pub struct Semaphore {
    count: AtomicI32,
}

impl Semaphore {
    pub const fn new(initial: i32) -> Self {
        Self { count: AtomicI32::new(initial) }
    }

    pub fn acquire(&self) {
        loop {
            let val = self.count.load(Ordering::Relaxed);
            if val > 0 {
                if self.count.compare_exchange_weak(
                    val, val - 1, Ordering::Acquire, Ordering::Relaxed
                ).is_ok() {
                    return;
                }
            }
            crate::task::scheduler::yield_now();
        }
    }

    pub fn try_acquire(&self) -> bool {
        let val = self.count.load(Ordering::Relaxed);
        if val > 0 {
            self.count.compare_exchange(
                val, val - 1, Ordering::Acquire, Ordering::Relaxed
            ).is_ok()
        } else {
            false
        }
    }

    pub fn release(&self) {
        self.count.fetch_add(1, Ordering::Release);
        // TODO: Wake up one thread from the waitqueue
    }
}