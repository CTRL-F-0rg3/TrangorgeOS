use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicIsize, Ordering};
use core::ops::{Deref, DerefMut};

pub struct RwLock<T> {
    // -1 = write locked, >0 = number of readers, 0 = unlocked
    state: AtomicIsize,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send + Sync> Sync for RwLock<T> {}
unsafe impl<T: Send> Send for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self { state: AtomicIsize::new(0), data: UnsafeCell::new(data) }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            let val = self.state.load(Ordering::Relaxed);
            if val >= 0 {
                if self.state.compare_exchange_weak(
                    val, val + 1, Ordering::Acquire, Ordering::Relaxed
                ).is_ok() {
                    return RwLockReadGuard { lock: self };
                }
            }
            crate::task::scheduler::yield_now();
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        loop {
            if self.state.compare_exchange_weak(
                0, -1, Ordering::Acquire, Ordering::Relaxed
            ).is_ok() {
                return RwLockWriteGuard { lock: self };
            }
            crate::task::scheduler::yield_now();
        }
    }
}

pub struct RwLockReadGuard<'a, T> { lock: &'a RwLock<T> }
impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
}
impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) { self.lock.state.fetch_sub(1, Ordering::Release); }
}

pub struct RwLockWriteGuard<'a, T> { lock: &'a RwLock<T> }
impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
}
impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T { unsafe { &mut *self.lock.data.get() } }
}
impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) { self.lock.state.store(0, Ordering::Release); }
}