//! Kernel connection for executive actions.
//!
//! Userspace never touches hardware or privileged state directly; it asks the
//! kernel through the shared-memory protocol. [`KernelClient::acquire_framebuffer`]
//! is the executive action the graphical environment uses to obtain the
//! framebuffer (a kernel-provided buffer mapped into the user address space).

use tg_comm::{Client, Layer};

/// A kernel-provided framebuffer (mapped into the user address space).
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    pub base: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

/// Video-class opcode: query framebuffer info (see lib/src/opcodes.rs).
const OP_VIDEO_FB_INFO: u32 = (5 << 8) | 1; // class Video(5), op 1

/// Connection to the kernel for executive actions.
pub struct KernelClient {
    client: Client,
}

impl KernelClient {
    /// # Safety
    /// `tx_base`/`rx_base` must be the kernel-mapped shared rings.
    pub unsafe fn connect(tx_base: *mut u8, rx_base: *mut u8) -> Self {
        Self {
            client: Client::attach(Layer::Userspace, tx_base, rx_base, 1),
        }
    }

    /// Ask the kernel for the framebuffer. On a real system the kernel replies
    /// with `(width<<16 | height, stride, physical base)` and maps the buffer
    /// into the user address space; here the reply is parsed from the shared
    /// ring.
    pub fn acquire_framebuffer(&mut self) -> Option<Framebuffer> {
        let id = self.client.request(OP_VIDEO_FB_INFO, 0, 0, 0);
        if id == 0 {
            return None;
        }

        // Drain replies until we find ours.
        for _ in 0..64 {
            self.yield_now();
            if let Some(reply) = self.client.recv() {
                if reply.seq == id && reply.status == 0 {
                    let packed = reply.a0;
                    return Some(Framebuffer {
                        base: reply.a2,
                        width: (packed >> 16) as u32,
                        height: (packed & 0xFFFF) as u32,
                        stride: reply.a1 as u32,
                    });
                }
            }
        }

        None
    }

    /// Yield to the kernel so it can poll the shared ring.
    pub fn yield_now(&self) {
        uspace_comm::yield_to_kernel();
    }
}
