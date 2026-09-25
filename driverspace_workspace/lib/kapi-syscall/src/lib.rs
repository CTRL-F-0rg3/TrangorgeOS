#![no_std]

pub mod channel;

use kapi_abi::{DsCmd, DsMsg};
use channel::RingBuffer;
use core::sync::atomic::{AtomicBool, Ordering};

static mut K2D_RING: *mut RingBuffer = core::ptr::null_mut();
static mut D2K_RING: *mut RingBuffer = core::ptr::null_mut();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub unsafe fn init(k2d_va: *mut u8, d2k_va: *mut u8) {
    unsafe {
        K2D_RING = RingBuffer::from_ptr(k2d_va);
        D2K_RING = RingBuffer::from_ptr(d2k_va);
    }
    INITIALIZED.store(true, Ordering::Release);
}

pub fn sys_ipc_call(cmd: DsCmd, arg0: u64, arg1: u64, arg2: u64) -> DsMsg {
    let mut msg = DsMsg::new(cmd as u32);
    msg.arg0 = arg0;
    msg.arg1 = arg1;
    msg.arg2 = arg2;

    unsafe {
        while !(*K2D_RING).push(&msg) {
            sys_yield();
        }
    }

    sys_poll();
    
    unsafe {
        (*D2K_RING).pop().unwrap_or_default()
    }
}

pub fn sys_poll() {
    unsafe {
        while (*D2K_RING).is_empty() {
            sys_yield();
        }
    }
}

/// Try to take one request from the service downlink ring; returns `None`
/// immediately when the ring is empty, without blocking.
///
/// This is the *server-side* primitive: `sys_ipc_call` faces the client (send a
/// request, wait for a reply), while a driver process passively receives IPC
/// messages pushed down by `ds-manager`.
pub fn sys_try_recv() -> Option<DsMsg> {
    let ring = k2d()?;
    // SAFETY: `k2d` only ever reads a pointer written once by `init`.
    unsafe { ring.pop() }
}

/// How many messages are still unread in the inbound queue.
pub fn sys_recv_pending() -> usize {
    match k2d() {
        // SAFETY: see `sys_try_recv`.
        Some(ring) => unsafe { ring.len() },
        None => 0,
    }
}

/// The downlink ring, or `None` when not initialised yet.
fn k2d() -> Option<&'static RingBuffer> {
    let ptr = unsafe { &raw const K2D_RING };
    // SAFETY: the pointer is written exactly once, by `init`.
    if unsafe { (*ptr).is_null() } {
        None
    } else {
        // SAFETY: `init` points at a `'static` `RingBuffer`.
        Some(unsafe { &*(*ptr) })
    }
}

/// The uplink ring, or `None` when not initialised yet.
fn d2k() -> Option<&'static RingBuffer> {
    let ptr = unsafe { &raw const D2K_RING };
    // SAFETY: the pointer is written exactly once, by `init`.
    if unsafe { (*ptr).is_null() } {
        None
    } else {
        // SAFETY: `init` points at a `'static` `RingBuffer`.
        Some(unsafe { &*(*ptr) })
    }
}

/// Push one reply onto the uplink ring, attaching a payload if there is one.
///
/// `payload` is copied into a buffer owned by `kapi-syscall`, so the caller may
/// reclaim its own stack buffer as soon as this returns. A payload longer than
/// [`MAX_REPLY_PAYLOAD`] is truncated; callers should check the length in
/// `msg.arg2` to notice that.
pub const MAX_REPLY_PAYLOAD: usize = 64;

static mut REPLY_PAYLOAD: [u8; MAX_REPLY_PAYLOAD] = [0u8; MAX_REPLY_PAYLOAD];

pub fn sys_ipc_reply(mut msg: DsMsg, payload: &[u8], len: usize) {
    let Some(ring) = d2k() else { return };

    let len = len.min(MAX_REPLY_PAYLOAD);
    if len > 0 {
        let copy = len.min(payload.len());
        // SAFETY: the driver process is the only writer, and `copy` is bounded
        // by both `len` and `payload.len()`, so it cannot overrun.
        unsafe {
            let dst = &raw mut REPLY_PAYLOAD;
            let base = (*dst).as_mut_ptr();
            core::ptr::copy_nonoverlapping(payload.as_ptr(), base, copy);
        }
        // SAFETY: only reads the buffer's address.
        let addr = unsafe { (*(&raw const REPLY_PAYLOAD)).as_ptr() as u64 };
        msg.arg0 = addr;
        msg.arg1 = len as u64;
    }

    // SAFETY: `d2k` points at a live ring.
    unsafe {
        while !ring.push(&msg) {
            sys_yield();
        }
    }
}

pub fn sys_yield() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}