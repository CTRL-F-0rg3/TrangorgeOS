//! Threads, and the `pthread` surface that `std` links whether or not it uses it.
//!
//! # Single-threaded, on purpose
//!
//! The kernel does not program `fs_base`, so there is no thread-local storage, and
//! `tgs_hal::heap`'s free lists have no lock. A second thread would therefore
//! corrupt the heap, not merely fail. So:
//!
//! * `pthread_create` reports `EAGAIN` rather than pretending. A program that
//!   checks the return value degrades to running on one thread, which is a
//!   recoverable outcome; a program that assumed success would corrupt its heap.
//! * `std::thread::spawn` goes through the same call, so it panics with a message
//!   that says what is missing rather than deadlocking.
//! * The keys still work, one value each, because `thread_local!` with a
//!   destructor is compiled by `std` into a `pthread_key_create` call regardless
//!   of how many threads exist.
//!
//! When the kernel grows `fs_base` and a `clone` that maps a stack, this module
//! becomes the place that has to grow with it, and `tgs_hal::heap` needs a lock.
//! Both are called out in `../README.md`.

#![allow(non_camel_case_types)]

use core::cell::UnsafeCell;

use tgs_hal::errno;

use crate::shim;
use crate::types::{c_int, c_uint, c_void};

/// `unsigned int`
pub type pthread_t = c_uint;

/// `pthread_key_t`
pub type pthread_key_t = c_uint;

/// `PTHREAD_KEYS_MAX` — a generous cap; the real cost is the slot array.
const MAX_KEYS: usize = 128;

struct Keys(UnsafeCell<[*mut c_void; MAX_KEYS]>);

// SAFETY: one task, so a key table needs no lock.
unsafe impl Sync for Keys {}

static KEYS: Keys = Keys(UnsafeCell::new([core::ptr::null_mut(); MAX_KEYS]));

/// A `pthread_key_t`, or 0 for "invalid".
#[no_mangle]
pub extern "C" fn pthread_key_create(key: *mut pthread_key_t, _destructor: *const c_void) -> c_int {
    if key.is_null() {
        return errno::EFAULT.0;
    }
    // SAFETY: the caller passed a non-null `pthread_key_t *`.
    unsafe {
        for (i, slot) in (*KEYS.0.get()).iter_mut().enumerate() {
            if slot.is_null() {
                *slot = core::ptr::null_mut();
                let n = (i + 1) as pthread_key_t;
                *key = n;
                return 0;
            }
        }
    }
    errno::EAGAIN.0
}

/// `int pthread_key_delete(pthread_key_t key)`
#[no_mangle]
pub extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    set_specific(key, core::ptr::null_mut());
    0
}

/// `int pthread_getspecific(pthread_key_t key)`
#[no_mangle]
pub extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    // Key 0 is `PTHREAD_KEYS_MAX`-style "invalid" in glibc and is never a real key
    // here either, since `pthread_key_create` hands out 1-based indices.
    if key == 0 || key as usize > MAX_KEYS {
        return core::ptr::null_mut();
    }
    // SAFETY: `key` is in range, checked above.
    unsafe { (*KEYS.0.get())[key as usize - 1] }
}

/// `int pthread_setspecific(pthread_key_t key, const void *value)`
#[no_mangle]
pub extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    set_specific(key, value.cast_mut());
    0
}

/// The body both setters share.
fn set_specific(key: pthread_key_t, value: *mut c_void) {
    if key == 0 || key as usize > MAX_KEYS {
        return;
    }
    // SAFETY: `key` is in range, checked above.
    unsafe { (*KEYS.0.get())[key as usize - 1] = value };
}

/// `int __pthread_key_create(pthread_key_t *key, void (*destructor)(void *))`
///
/// The glibc-internal spelling, which `std`'s `thread_local!` slow path uses.
#[no_mangle]
pub extern "C" fn __pthread_key_create(key: *mut pthread_key_t, destructor: *const c_void) -> c_int {
    pthread_key_create(key, destructor)
}

/// `int pthread_create(...)`
///
/// `EAGAIN`, not a stub that succeeds. See the module docs.
#[no_mangle]
pub extern "C" fn pthread_create(
    thread: *mut pthread_t,
    _attr: *const c_void,
    _start: Option<extern "C" fn(*mut c_void) -> *mut c_void>,
    _arg: *mut c_void,
) -> c_int {
    if thread.is_null() {
        return errno::EFAULT.0;
    }
    errno::EAGAIN.0
}

/// `int pthread_join(pthread_t thread, void **retval)`
#[no_mangle]
pub extern "C" fn pthread_join(_thread: pthread_t, _retval: *mut *mut c_void) -> c_int {
    errno::ESRCH.0
}

/// `pthread_t pthread_self(void)` — always 1, because there is one thread.
#[no_mangle]
pub extern "C" fn pthread_self() -> pthread_t {
    1
}

/// `int pthread_equal(pthread_t a, pthread_t b)`
#[no_mangle]
pub extern "C" fn pthread_equal(a: pthread_t, b: pthread_t) -> c_int {
    (a == b) as c_int
}

/// `int pthread_kill(pthread_t thread, int signum)`
#[no_mangle]
pub extern "C" fn pthread_kill(_thread: pthread_t, _signum: c_int) -> c_int {
    errno::ESRCH.0
}

/// `int pthread_attr_init(pthread_attr_t *attr)`
#[no_mangle]
pub extern "C" fn pthread_attr_init(attr: *mut c_void) -> c_int {
    if attr.is_null() {
        return errno::EFAULT.0;
    }
    // A zeroed attribute block reads as "defaults", which is what the struct is
    // documented to mean.
    // SAFETY: the caller passed a non-null `pthread_attr_t *`.
    unsafe { core::ptr::write_bytes(attr, 0, 1) };
    0
}

/// `int pthread_attr_destroy(pthread_attr_t *attr)`
#[no_mangle]
pub extern "C" fn pthread_attr_destroy(_attr: *mut c_void) -> c_int {
    0
}

/// `int pthread_attr_setstacksize(pthread_attr_t *attr, size_t stacksize)`
#[no_mangle]
pub extern "C" fn pthread_attr_setstacksize(_attr: *mut c_void, _stacksize: usize) -> c_int {
    0
}

/// `int pthread_attr_setguardsize(pthread_attr_t *attr, size_t guardsize)`
#[no_mangle]
pub extern "C" fn pthread_attr_setguardsize(_attr: *mut c_void, _guardsize: usize) -> c_int {
    0
}

/// `unsigned long *__pthread_get_minstack(pthread_t *thread)`
///
/// glibc internal. Returning null is the documented "no minimum stack" answer;
/// `std` only uses it to check that a thread's stack is not absurdly small.
#[no_mangle]
pub extern "C" fn __pthread_get_minstack(_thread: *const pthread_t) -> *const c_ulong_t {
    core::ptr::null()
}

/// The type `__pthread_get_minstack` returns.
type c_ulong_t = u64;

/// `int pthread_once(pthread_once_t *once_control, void (*init_routine)(void))`
///
/// `std` uses this for its own one-time initialisation. Running the routine
/// unconditionally and reporting success gives the same observable behaviour for
/// a single-threaded program.
#[no_mangle]
pub extern "C" fn pthread_once(_once: *mut c_int, init_routine: Option<extern "C" fn()>) -> c_int {
    if let Some(f) = init_routine {
        f();
    }
    0
}

/// `int pthread_mutex_init(pthread_mutex_t *mutex, const pthread_mutexattr_t *attr)`
#[no_mangle]
pub extern "C" fn pthread_mutex_init(mutex: *mut c_void, _attr: *const c_void) -> c_int {
    if mutex.is_null() {
        return errno::EFAULT.0;
    }
    // SAFETY: the caller passed a non-null `pthread_mutex_t *`. A zeroed mutex is
    // the unlocked state in the x86_64 layout.
    unsafe { core::ptr::write_bytes(mutex, 0, 1) };
    0
}

/// `int pthread_mutex_destroy(pthread_mutex_t *mutex)`
#[no_mangle]
pub extern "C" fn pthread_mutex_destroy(_mutex: *mut c_void) -> c_int {
    0
}

/// `int pthread_mutex_lock(pthread_mutex_t *mutex)` — a no-op, with no contention
/// to lose because there is one thread.
#[no_mangle]
pub extern "C" fn pthread_mutex_lock(_mutex: *mut c_void) -> c_int {
    0
}

/// `int pthread_mutex_trylock(pthread_mutex_t *mutex)`
#[no_mangle]
pub extern "C" fn pthread_mutex_trylock(_mutex: *mut c_void) -> c_int {
    0
}

/// `int pthread_mutex_unlock(pthread_mutex_t *mutex)`
#[no_mangle]
pub extern "C" fn pthread_mutex_unlock(_mutex: *mut c_void) -> c_int {
    0
}

/// `int pthread_cond_init(pthread_cond_t *cond, const pthread_condattr_t *attr)`
#[no_mangle]
pub extern "C" fn pthread_cond_init(cond: *mut c_void, _attr: *const c_void) -> c_int {
    if cond.is_null() {
        return errno::EFAULT.0;
    }
    // SAFETY: the caller passed a non-null `pthread_cond_t *`.
    unsafe { core::ptr::write_bytes(cond, 0, 1) };
    0
}

/// `int pthread_cond_destroy(pthread_cond_t *cond)`
#[no_mangle]
pub extern "C" fn pthread_cond_destroy(_cond: *mut c_void) -> c_int {
    0
}

/// `int pthread_cond_wait(pthread_cond_t *cond, pthread_mutex_t *mutex)`
///
/// There is nothing to wait for with one thread, so a wait would deadlock. The
/// honest answer is `EINVAL` rather than blocking forever.
#[no_mangle]
pub extern "C" fn pthread_cond_wait(_cond: *mut c_void, _mutex: *mut c_void) -> c_int {
    errno::EINVAL.0
}

/// `int pthread_cond_signal(pthread_cond_t *cond)`
#[no_mangle]
pub extern "C" fn pthread_cond_signal(_cond: *mut c_void) -> c_int {
    0
}

/// `int pthread_cond_broadcast(pthread_cond_t *cond)`
#[no_mangle]
pub extern "C" fn pthread_cond_broadcast(_cond: *mut c_void) -> c_int {
    0
}

/// `void *pthread_getspecific`-adjacent: glibc's `__libc_single_threaded`.
///
/// Read by `std` to decide whether it needs to lock around `RandomState`. `1` is
/// the correct answer and lets a lot of code take its fast path.
#[no_mangle]
pub static __libc_single_threaded: c_int = 1;

/// `int shim_unused(void)` — keeps the `shim` import used in this module.
#[doc(hidden)]
pub fn __keep_shim_linked() -> c_int {
    shim::fail(errno::ENOSYS) as c_int
}
