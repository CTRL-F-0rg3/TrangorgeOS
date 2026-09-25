//! Runtime for code running in driver space, over `tg_comm` shared memory.
//!
//! The wire transport is the 64-byte `tg_comm::CommMsg` over a shared-memory
//! `tg_comm::SpscRing`, gated by the kernel's capability authorization.
//! - [`init_once`] attaches the two rings at the kernel-mapped fixed VAs,
//! - [`request`] / [`request_raw`] push a request into the d2k ring,
//! - [`tick`] drains kernel replies from the k2d ring into a cache,
//! - [`take_resp`] pulls the matching reply by id,
//! - [`yield_to_kernel`] triggers the kernel to poll the ring.

use crate::abi::*;
use tg_comm::{CommMsg, Layer, MsgKind, SpscRing};
use crate::driver::{DeviceInfo, Driver};

const RESP_CACHE: usize = 16;

static mut K2D: Option<SpscRing> = None; // kernel -> driver (replies)
static mut D2K: Option<SpscRing> = None; // driver -> kernel (requests)
static mut NEXT_ID: u32 = 5000;
static mut CACHE_MSG: [DsMsg; RESP_CACHE] = [DsMsg::EMPTY; RESP_CACHE];
static mut CACHE_USED: [bool; RESP_CACHE] = [false; RESP_CACHE];

/// Attach the shared rings at the kernel-mapped fixed virtual addresses.
pub fn init_once(_params_va: u64) {
    unsafe {
        K2D = Some(SpscRing::attach(DS_K2D_VA as *mut u8));
        D2K = Some(SpscRing::attach(DS_D2K_VA as *mut u8));
    }
}

/// Registers a driver and notifies the kernel that this driver space is now
/// managed.
pub fn register<D: Driver>(drv: &mut D) {
    let _ = drv.init(&DeviceInfo::default());
    let _ = request(DsCmd::RegisterDriver, 0, 0, 0);
}

/// Sends a driver-space command to the kernel. Returns the id used later with
/// [`take_resp`].
pub fn request(cmd: DsCmd, a0: u64, a1: u64, a2: u64) -> u64 {
    request_raw(cmd as u32, a0, a1, a2)
}

/// Sends a raw (opcode) request to the kernel.
pub fn request_raw(opcode: u32, a0: u64, a1: u64, a2: u64) -> u64 {
    unsafe {
        let d2k = match D2K.as_ref() {
            Some(r) => r,
            None => return 0,
        };

        let id = NEXT_ID;
        NEXT_ID = NEXT_ID.wrapping_add(1);

        let mut m = CommMsg::new(MsgKind::Request, Layer::Driverspace, Layer::Kernel, opcode);
        m.seq = id;
        m.cap = DS_CAP_DEVICE;
        m.a0 = a0;
        m.a1 = a1;
        m.a2 = a2;

        if d2k.push(&m) {
            id as u64
        } else {
            0
        }
    }
}

/// Drains pending kernel->driver replies into the local cache.
pub fn tick() {
    unsafe {
        let k2d = match K2D.as_ref() {
            Some(r) => r,
            None => return,
        };

        while let Some(m) = k2d.pop() {
            let resp = DsMsg {
                id: m.seq as u64,
                cmd: m.opcode,
                flags: 0,
                arg0: m.a0,
                arg1: m.a1,
                arg2: m.a2,
                status: m.status,
                pad: 0,
            };

            for i in 0..RESP_CACHE {
                if !CACHE_USED[i] {
                    CACHE_MSG[i] = resp;
                    CACHE_USED[i] = true;
                    break;
                }
            }
        }
    }
}

/// Returns (and consumes) the reply with the given request id.
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

/// Hands control back to the kernel (triggers a poll of the shared ring).
#[inline]
pub fn yield_to_kernel() {
    unsafe {
        core::arch::asm!("int 0x80", in("rax") 1u64);
    }
}
