use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU8, Ordering};
use core::ops::{Deref, DerefMut};

const UNLOCKED: u8 = 0;
const LOCKED: u8 = 1;

// A sleeping mutex. Uses atomics for the fast path. 
// In a full implementation, the slow path would enqueue the thread 
// into a WaitQueue and call the scheduler. Here we use yield as a placeholder.
pub struct Mutex<T> {
    state: AtomicU8,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU8::new(UNLOCKED),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            if self.state.compare_exchange_weak(
                UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed
            ).is_ok() {
                return MutexGuard { lock: self };
            }
            // Slow path: yield CPU instead of busy-spinning
            crate::task::scheduler::yield_now();
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.state.compare_exchange(
            UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed
        ).is_ok() {
            Some(MutexGuard { lock: self })
        } else {
            None
        }
    }

    fn unlock(&self) {
        self.state.store(UNLOCKED, Ordering::Release);
        // TODO: Wake up one thread from the waitqueue if any
    }
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T { unsafe { &mut *self.lock.data.get() } }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) { self.lock.unlock(); }
}