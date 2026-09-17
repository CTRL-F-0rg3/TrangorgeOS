//! Runtime for code running in driver space.
//!
//! Mirrors the C client in `libs/dsclient.c`:
//! - [`init_once`] points the rings at the kernel-provided buffers,
//! - [`request`] / [`request_raw`] push a request into the d2k ring
//!   and return an id,
//! - [`tick`] drains kernel responses from the k2d ring into a cache,
//! - [`take_resp`] pulls the matching response by id,
//! - [`yield_to_kernel`] hands control back to the kernel.

use crate::abi::*;
use crate::driver::{DeviceInfo, Driver};
use core::ptr;

const RESP_CACHE: usize = 16;

const EMPTY_MSG: DsMsg = DsMsg {
    id: 0,
    cmd: 0,
    flags: 0,
    arg0: 0,
    arg1: 0,
    arg2: 0,
    status: 0,
    pad: 0,
};

static mut K2D: *mut DsRing = ptr::null_mut();
static mut D2K: *mut DsRing = ptr::null_mut();
static mut K2D_MSGS: *mut DsMsg = ptr::null_mut();
static mut D2K_MSGS: *mut DsMsg = ptr::null_mut();
static mut NEXT_ID: u64 = 5000;
static mut CACHE_MSG: [DsMsg; RESP_CACHE] = [EMPTY_MSG; RESP_CACHE];
static mut CACHE_USED: [bool; RESP_CACHE] = [false; RESP_CACHE];

/// Reads the boot parameters block and points our ring views at the
/// kernel-provided buffers.
pub fn init_once(params_va: u64) {
    unsafe {
        let p = &*(params_va as *const DsInitParams);

        K2D = p.k2d_va as *mut DsRing;
        D2K = p.d2k_va as *mut DsRing;
        K2D_MSGS = (p.k2d_va + DS_RING_HDR_SIZE as u64) as *mut DsMsg;
        D2K_MSGS = (p.d2k_va + DS_RING_HDR_SIZE as u64) as *mut DsMsg;
    }
}

/// Registers a driver (by calling its `Driver::init` hook) and notifies
/// the kernel that this driver space is now managed.
pub fn register<D: Driver>(drv: &mut D) {
    let _ = drv.init(&DeviceInfo::default());
    let _ = request(DsCmd::RegisterDriver, 0, 0, 0);
}

unsafe fn push_raw(cmd: u32, a0: u64, a1: u64, a2: u64) -> u64 {
    let ring = D2K;
    if ring.is_null() {
        return 0;
    }

    let id = NEXT_ID;
    NEXT_ID = NEXT_ID.wrapping_add(1);

    let head = (*ring).head;
    let cap = (*ring).cap;
    if cap == 0 {
        return 0;
    }

    let next = (head + 1) % cap;
    if next == (*ring).tail {
        return 0; // ring full
    }

    let slot = D2K_MSGS.add(head as usize);
    *slot = DsMsg {
        id,
        cmd,
        flags: DS_FLAG_RESPONSE,
        arg0: a0,
        arg1: a1,
        arg2: a2,
        status: 0,
        pad: 0,
    };
    (*ring).head = next;

    id
}

/// Sends a driver-space command to the kernel. Returns the id used later
/// with [`take_resp`].
pub fn request(cmd: DsCmd, a0: u64, a1: u64, a2: u64) -> u64 {
    unsafe { push_raw(cmd as u32, a0, a1, a2) }
}

/// Sends a raw (service-class packed) command to the kernel.
pub fn request_raw(cmd: u32, a0: u64, a1: u64, a2: u64) -> u64 {
    unsafe { push_raw(cmd, a0, a1, a2) }
}

/// Drains pending kernel->driver responses into the local cache.
pub fn tick() {
    unsafe {
        let ring = K2D;
        if ring.is_null() {
            return;
        }

        while (*ring).tail != (*ring).head {
            let slot = K2D_MSGS.add((*ring).tail as usize);
            let copy = *slot;
            (*ring).tail = ((*ring).tail + 1) % (*ring).cap;

            for i in 0..RESP_CACHE {
                if !CACHE_USED[i] {
                    CACHE_MSG[i] = copy;
                    CACHE_USED[i] = true;
                    break;
                }
            }
        }
    }
}

/// Returns (and consumes) the response with the given request id.
pub fn take_resp(id: u64) -> Option<DsMsg> {
    unsafe {
        for i in 0..RESP_CACHE {
            if CACHE_USED[i] && CACHE_MSG[i].id == id {
                let m = CACHE_MSG[i];
                CACHE_USED[i] = false;
                return Some(m);
            }
        }
    }
    None
}

/// Switches execution back to the kernel (never returns).
#[unsafe(naked)]
pub extern "C" fn yield_to_kernel() {
    core::arch::naked_asm!(
        "mov rax, {sw}",
        "mov rbx, [rax + 0]",
        "mov cr3, rbx",
        "mov rsp, [rax + 24]",
        "jmp [rax + 16]",
        sw = const DS_SWITCH_VA,
    );
}