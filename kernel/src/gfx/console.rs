pub fn test_fill(r: u32, g: u32, b: u32) {
    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for y in 0..h {
        for x in 0..w {
            fb().set(x, y, rgb(r, g, b));
        }
    }
}

pub fn init(fb_addr: u64, width: u32, height: u32, stride: u32) -> bool {
    if width == 0 || height == 0 || stride == 0 {
        return false;
    }

    if !unsafe { ffi::mm_ready() } {
        return false;
    }

    let width = width as usize;
    let height = height as usize;

    let stride = if stride as usize >= width * 4 {
        stride as usize / 4
    } else {
        stride as usize
    };

    let size = stride * height * 4;

    let ptr = if fb_addr >= 0xFFFF800000000000 {
        fb_addr
    } else {
        let mut virt = 0u64;

        if !unsafe { ffi::vmm_map_device(fb_addr, size, &mut virt) } {
            return false;
        }

        virt
    };

    unsafe {
        FB = Some(Framebuffer {
            ptr: ptr as *mut u32,
            width,
            height,
            stride,
        });
    }

    for t in (0..=256).step_by(16) {
        galaxy::render(fb(), t);
        delay();
    }

    let mut buf = 0u64;

    if !unsafe { ffi::vmm_alloc(size, 1, &mut buf) } {
        return false;
    }

    unsafe {
        CLEAN = buf as *mut u32;

        core::ptr::copy_nonoverlapping(
            fb().ptr as *const u32,
            CLEAN,
            fb().stride * fb().height,
        );
    }

    refresh();

    true
}