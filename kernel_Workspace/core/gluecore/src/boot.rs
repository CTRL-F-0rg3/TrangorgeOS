use kstd_base::{PhysAddr, Size};

extern "C" {
    fn boot_get_memory_map(out_ptr: *mut u64, out_size: *mut usize) -> bool;
    fn boot_get_cmdline(out_ptr: *mut u64, out_len: *mut usize) -> bool;
    fn boot_get_framebuffer(out_phys: *mut u64, out_w: *mut u32, out_h: *mut u32, out_pitch: *mut u32) -> bool;
}

pub struct BootMemoryMap {
    pub ptr: u64,
    pub size: usize,
}

pub fn get_memory_map() -> Option<BootMemoryMap> {
    let mut ptr: u64 = 0;
    let mut size: usize = 0;
    if unsafe { boot_get_memory_map(&mut ptr, &mut size) } {
        Some(BootMemoryMap { ptr, size })
    } else {
        None
    }
}

pub struct BootFramebuffer {
    pub phys: PhysAddr,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
}

pub fn get_framebuffer() -> Option<BootFramebuffer> {
    let mut phys: u64 = 0;
    let mut w: u32 = 0;
    let mut h: u32 = 0;
    let mut pitch: u32 = 0;
    if unsafe { boot_get_framebuffer(&mut phys, &mut w, &mut h, &mut pitch) } {
        Some(BootFramebuffer { phys: PhysAddr(phys), width: w, height: h, pitch })
    } else {
        None
    }
}