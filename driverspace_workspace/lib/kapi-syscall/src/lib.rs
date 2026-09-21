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

pub fn sys_yield() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}