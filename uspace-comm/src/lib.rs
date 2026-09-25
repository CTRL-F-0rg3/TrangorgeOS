//! Userspace (ring 3) client for the TrangorgeOS `tg_comm` shared-memory
//! protocol.
//!
//! This is the counterpart of `kernel_Workspace/kernel/src/tgcomm/userspace.rs`.
//! The kernel maps two shared rings into the user address space and hands the
//! process a capability table; this client builds `CommMsg` requests, posts
//! them to the user->kernel ring, yields with `int 0x80` so the kernel polls,
//! and reads the replies from the kernel->user ring.

#![no_std]

use tg_comm::{Client, CommMsg, Layer, MsgKind};

/// User-space virtual addresses of the shared rings (see the kernel's
/// `tgcomm::userspace` for the mapping side).
pub const US_K2U_VA: u64 = 0x7000_0000; // kernel -> user (replies)
pub const US_U2K_VA: u64 = 0x7000_1000; // user -> kernel (requests)
pub const US_RING_SLOTS: u32 = 256;

/// Capability handles handed to the userspace init process.
pub const US_CAP_DEVICE: u32 = 1;
pub const US_CAP_MEMORY: u32 = 2;

/// Attach to the kernel-mapped shared rings.
pub fn attach() -> Client {
    // SAFETY: the kernel maps these rings at fixed VAs before entering ring 3.
    unsafe {
        Client::attach(
            Layer::Userspace,
            US_U2K_VA as *mut u8,
            US_K2U_VA as *mut u8,
            US_CAP_DEVICE,
        )
    }
}

/// Build a well-formed request message for this layer.
pub fn request(opcode: u32, cap: u32, a0: u64, a1: u64, a2: u64) -> CommMsg {
    let mut m = CommMsg::new(MsgKind::Request, Layer::Userspace, Layer::Kernel, opcode);
    m.cap = cap;
    m.a0 = a0;
    m.a1 = a1;
    m.a2 = a2;
    m
}

/// Yield to the kernel so it polls the shared ring (HYPER_YIELD via int 0x80).
#[inline]
pub fn yield_to_kernel() {
    unsafe {
        core::arch::asm!("int 0x80", in("rax") 1u64);
    }
}
