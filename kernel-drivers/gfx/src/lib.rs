#![no_std]

//! `gfx` — primary framebuffer info, extracted from the kernel graphics
//! console (`kernel/src/gfx`). This exposes the framebuffer the rest of the
//! system (hdmi, terminal, …) can build on; the actual MMIO mapping happens in
//! the runtime via `hw_sys`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub phys: u64,
}

pub fn fb_info() -> Option<FramebufferInfo> {
    let (mut w, mut h, mut s, mut p) = (0u32, 0u32, 0u32, 0u64);

    unsafe {
        if hw_sys::fb_info(&mut w, &mut h, &mut s, &mut p) {
            Some(FramebufferInfo { width: w, height: h, stride: s, phys: p })
        } else {
            None
        }
    }
}
