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

/// 尝试从服务下行环上取一条请求；环空时立即返回 `None`，不阻塞。
///
/// 这是“服务方”侧的原语：`sys_ipc_call` 面向客户端（发请求、等回复），
/// 而驱动进程反过来要被动接收 `ds-manager` 下发的 IPC 消息。
pub fn sys_try_recv() -> Option<DsMsg> {
    let ring = k2d()?;
    // SAFETY: `k2d` 只在 `init` 写入过指针，之后不再改变。
    unsafe { ring.pop() }
}

/// 回复队列里还有多少条未读消息。
pub fn sys_recv_pending() -> usize {
    match k2d() {
        // SAFETY: 见 `sys_try_recv`。
        Some(ring) => unsafe { ring.len() },
        None => 0,
    }
}

/// 下行环；未初始化时返回 `None`。
fn k2d() -> Option<&'static RingBuffer> {
    let ptr = unsafe { &raw const K2D_RING };
    // SAFETY: 指针由 `init` 写入一次。
    if unsafe { (*ptr).is_null() } {
        None
    } else {
        // SAFETY: `init` 指向一个 `'static` 的 `RingBuffer`。
        Some(unsafe { &*(*ptr) })
    }
}

/// 上行环；未初始化时返回 `None`。
fn d2k() -> Option<&'static RingBuffer> {
    let ptr = unsafe { &raw const D2K_RING };
    // SAFETY: 指针由 `init` 写入一次。
    if unsafe { (*ptr).is_null() } {
        None
    } else {
        // SAFETY: `init` 指向一个 `'static` 的 `RingBuffer`。
        Some(unsafe { &*(*ptr) })
    }
}

/// 把一条回复推回上行环，并在必要时附带一段载荷。
///
/// `payload` 会被拷贝到 `kapi-syscall` 自己的载荷缓冲里，因此调用方可以在
/// 返回后立刻回收自己的栈缓冲。载荷超过 [`MAX_REPLY_PAYLOAD`] 时只复制能容纳
/// 的部分——调用方应通过 `msg.arg2` 里的长度判断是否被截断。
pub const MAX_REPLY_PAYLOAD: usize = 64;

static mut REPLY_PAYLOAD: [u8; MAX_REPLY_PAYLOAD] = [0u8; MAX_REPLY_PAYLOAD];

pub fn sys_ipc_reply(mut msg: DsMsg, payload: &[u8], len: usize) {
    let Some(ring) = d2k() else { return };

    let len = len.min(MAX_REPLY_PAYLOAD);
    if len > 0 {
        let copy = len.min(payload.len());
        // SAFETY: 驱动进程是唯一写者；`copy` 同时受 `len` 与 `payload.len()`
        // 约束，不会越界。
        unsafe {
            let dst = &raw mut REPLY_PAYLOAD;
            let base = (*dst).as_mut_ptr();
            core::ptr::copy_nonoverlapping(payload.as_ptr(), base, copy);
        }
        // SAFETY: 只读取缓冲的地址。
        let addr = unsafe { (*(&raw const REPLY_PAYLOAD)).as_ptr() as u64 };
        msg.arg0 = addr;
        msg.arg1 = len as u64;
    }

    // SAFETY: `d2k` 指向一个存活的环。
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