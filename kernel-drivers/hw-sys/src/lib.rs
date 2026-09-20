#![no_std]

//! `hw-sys` — the FFI boundary between the exported hardware drivers and the
//! OS runtime (kernel). The drivers never link against the kernel crate; they
//! call these thin `extern "C"` symbols, which the kernel (or any future
//! runtime) is expected to provide.
//!
//! This keeps every `pcie`/`usb`/`audio`/… library a standalone, buildable
//! `no_std` crate that can be reused by the rest of the system.

extern "C" {
    /// printf-style logging (mirrors the kernel `kprintf`).
    pub fn kprintf(fmt: *const u8, ...);

    /// Allocate a DMA-coherent buffer; returns phys + virt.
    pub fn dma_alloc_coherent(
        bytes: usize,
        zone_max: u64,
        out_phys: *mut u64,
        out_virt: *mut *mut u8,
    ) -> bool;

    pub fn dma_free_coherent(phys: u64, virt: *mut u8, bytes: usize);

    /// Map device MMIO into the higher half; returns a virtual address.
    pub fn mmio_map(phys: u64, len: usize, out_virt: *mut u64) -> bool;

    pub fn mmio_unmap(virt: u64, len: usize);

    /// Is `virt` actually present in the current page table?
    pub fn paging_is_mapped(virt: u64) -> bool;

    /// Query the primary framebuffer (used by `hdmi`/`gfx`).
    pub fn fb_info(out_w: *mut u32, out_h: *mut u32, out_stride: *mut u32, out_phys: *mut u64) -> bool;
}

pub mod log {
    /// Print a plain (already formatted) message, `%s`-style.
    pub fn info(msg: &str) {
        let mut buf = [0u8; 256];
        let len = msg.len().min(255);
        buf[..len].copy_from_slice(&msg.as_bytes()[..len]);
        buf[len] = 0;
        unsafe { super::kprintf(b"%s\n\0".as_ptr(), buf.as_ptr()); }
    }
}
