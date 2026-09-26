//! The bridge to the GPU driver: find the output, map it, and present into it.
//!
//! # What this talks to
//!
//! The driver, over `kapi-syscall`. Three pieces of information matter, and
//! they are asked for separately because they have different lifetimes: the
//! framebuffer's *geometry* (a mode, which can change), its *physical* address
//! (a BAR, which does not), and a *mapping* of that address (which must be
//! undone).
//!
//! # Why `stride` is asked for rather than derived
//!
//! A scanline is `width * bytes_per_pixel` only when the driver packs rows
//! back to back. Real drivers pad rows to a larger boundary, and a compositor
//! that assumes `width * 4` writes every frame to the wrong offsets once it
//! does. So the size is computed from the stride the driver reports.

use ds_log::{ds_error, ds_info};
use kapi_abi::DsCmd;
use kapi_syscall::sys_ipc_call;

/// The geometry and mapping of the output the driver is scanning out.
pub struct GpuBridge {
    /// Physical base of the scanout framebuffer, as the driver reports it.
    fb_phys: u64,
    /// Virtual address of the mapping the driver granted.
    fb_virt: u64,
    /// Bytes one scanline occupies, padding included.
    stride: u32,
    /// Whole-frame size in bytes: `stride * height`.
    fb_size: usize,
    width: u32,
    height: u32,
    ready: bool,
}

impl GpuBridge {
    pub fn new() -> Self {
        Self {
            fb_phys: 0,
            fb_virt: 0,
            stride: 0,
            fb_size: 0,
            width: 0,
            height: 0,
            ready: false,
        }
    }

    /// Ask the driver about the output and map it, or report why not.
    pub fn init(&mut self) -> bool {
        ds_info!("GPU-Bridge: requesting framebuffer info from the GPU driver");

        let reply = sys_ipc_call(DsCmd::VideoFbInfo, 0, 0, 0);
        if !reply.is_ok() {
            ds_error!("GPU-Bridge: framebuffer info request failed");
            return false;
        }

        // The reply packs geometry as `(width << 16) | height`, with the stride
        // in `arg1` and the physical base in `arg2`. Packing rather than a
        // struct is what `tgcomm::dispatch` puts on the wire, so it is decoded
        // in exactly one place here.
        let packed = reply.arg0 as u32;
        self.width = packed >> 16;
        self.height = packed & 0xFFFF;
        self.stride = reply.arg1 as u32;
        self.fb_phys = reply.arg2;

        if self.width == 0 || self.height == 0 {
            ds_error!("GPU-Bridge: driver reported a zero-sized output");
            return false;
        }
        if self.fb_phys == 0 {
            ds_error!("GPU-Bridge: driver reported no framebuffer");
            return false;
        }
        if self.stride == 0 {
            // A driver that omits the stride is claiming rows are packed, which
            // is the one assumption that is safe to make on its behalf.
            self.stride = self.width * 4;
        }

        self.fb_size = (self.stride as usize) * (self.height as usize);

        ds_info!(
            "GPU-Bridge: output {}x{}, stride {}, {} KiB at phys 0x{:x}",
            self.width,
            self.height,
            self.stride,
            self.fb_size / 1024,
            self.fb_phys
        );

        let map_reply = sys_ipc_call(DsCmd::ReqMmio, self.fb_phys, self.fb_size as u64, 0);
        if !map_reply.is_ok() {
            ds_error!("GPU-Bridge: mapping the framebuffer failed");
            return false;
        }

        self.fb_virt = map_reply.arg0;
        if self.fb_virt == 0 {
            ds_error!("GPU-Bridge: mapping returned a null address");
            return false;
        }

        self.ready = true;
        ds_info!("GPU-Bridge: framebuffer mapped at 0x{:x}", self.fb_virt);
        true
    }

    /// The output's geometry, or `(0, 0)` before [`Self::init`] succeeds.
    pub fn mode(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Whether the output is mapped and can be presented into.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Copy a composited frame into the scanout framebuffer.
    ///
    /// This is a CPU copy, and it is a placeholder for the flip the driver
    /// should be doing: `GpuDevice::present` already exists in `ds-fw-gpu` and
    /// is the right home for this. Until that is wired up, presenting means
    /// memcpy into the mapped framebuffer.
    ///
    /// `size` is honoured rather than assumed, and the copy is bounded by the
    /// mapped size: a frame larger than the output would otherwise write past
    /// the mapping, and a mapping's end is not a guard page.
    pub fn present(&self, src_buffer: *const u32, size: usize) {
        if !self.ready || self.fb_virt == 0 || src_buffer.is_null() || size == 0 {
            return;
        }

        let copy_bytes = size.min(self.fb_size);
        let words = copy_bytes / core::mem::size_of::<u32>();

        // SAFETY: `src_buffer` is the compositor's staging frame, which
        // `gfx_server_main` allocated and which is at least `size` bytes long;
        // the destination is the driver's mapping, which the driver sized at
        // `fb_size`; and `words` is bounded by both. The two are different
        // allocations, so they cannot overlap.
        unsafe {
            core::ptr::copy_nonoverlapping(src_buffer, self.fb_virt as *mut u32, words);
        }
    }
}