#![no_std]

//! `gfx` — primary framebuffer info, extracted from the kernel graphics
//! console (`kernel/src/gfx`). This exposes the framebuffer the rest of the
//! system (hdmi, terminal, …) can build on; the actual MMIO mapping happens in
//! the runtime via `hw_sys`.

/// The framebuffer the system is drawing on.
///
/// # Units
///
/// `stride` is in **bytes** and `phys` is a **physical** address, because that
/// is what `hw_sys::fb_info` and the kernel's implementation agree on, and what
/// a device register has to be programmed with. A caller that wants pixels per
/// scanline divides by the format's pixel size itself, rather than being handed
/// a number whose unit depends on which function produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub width: u32,
    pub height: u32,
    /// Bytes one scanline occupies, padding included.
    pub stride: u32,
    /// Physical base address. `0` means there is no framebuffer.
    pub phys: u64,
}

impl FramebufferInfo {
    /// The frame's size in bytes: `stride * height`.
    ///
    /// Derived from the stride rather than `width * height * 4`, so a driver
    /// that pads its rows reports the size it actually occupies.
    pub const fn size_bytes(&self) -> u64 {
        (self.stride as u64) * (self.height as u64)
    }

    /// Whether this describes a framebuffer that can be mapped.
    ///
    /// A zero base is the configuration space saying "this BAR is
    /// unimplemented", and mapping it maps whatever physical page is there.
    pub const fn is_usable(&self) -> bool {
        self.phys != 0 && self.width != 0 && self.height != 0 && self.stride != 0
    }
}

/// The current framebuffer, or `None` when there is none.
///
/// Returns `None` rather than a descriptor of zeros: a zero base is the one
/// value a caller must not map, and reporting it as a success is how a driver
/// ends up writing over the configuration space.
pub fn fb_info() -> Option<FramebufferInfo> {
    let (mut w, mut h, mut s, mut p) = (0u32, 0u32, 0u32, 0u64);

    unsafe {
        if hw_sys::fb_info(&mut w, &mut h, &mut s, &mut p) {
            let info = FramebufferInfo { width: w, height: h, stride: s, phys: p };
            if info.is_usable() { Some(info) } else { None }
        } else {
            None
        }
    }
}
