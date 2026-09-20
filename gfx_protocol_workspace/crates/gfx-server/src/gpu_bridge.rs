use kapi_abi::{DsCmd, DsMsg};
use kapi_syscall;
use ds_log::{ds_info, ds_error};

pub struct GpuBridge {
    fb_phys: u64,
    fb_virt: u64,
    fb_size: usize,
    width: u32,
    height: u32,
}

impl GpuBridge {
    pub fn new() -> Self {
        Self { fb_phys: 0, fb_virt: 0, fb_size: 0, width: 0, height: 0 }
    }

    pub fn init(&mut self) -> bool {
        ds_info!("GPU-Bridge", "Requesting framebuffer info from vgpu via DS IPC");

        let reply = kapi_syscall::sys_ipc_call(DsCmd::VideoFbInfo, 0, 0, 0);
        if !reply.is_ok() {
            ds_error!("GPU-Bridge", "Failed to get FB info from vgpu");
            return false;
        }

        self.fb_phys = reply.arg0;
        self.width = reply.arg1 as u32;
        self.height = reply.arg2 as u32;
        self.fb_size = (self.width * self.height * 4) as usize;

        ds_info!("GPU-Bridge", "FB: phys=0x{:x} {}x{}", self.fb_phys, self.width, self.height);

        let map_reply = kapi_syscall::sys_ipc_call(
            DsCmd::ReqMmio, self.fb_phys, self.fb_size as u64, 0
        );
        if !map_reply.is_ok() {
            ds_error!("GPU-Bridge", "Failed to map framebuffer MMIO");
            return false;
        }

        self.fb_virt = map_reply.arg0;
        ds_info!("GPU-Bridge", "Framebuffer mapped at virt=0x{:x}", self.fb_virt);
        true
    }

    pub fn present(&self, src_buffer: *const u32, size: usize) {
        if self.fb_virt == 0 { return; }
        unsafe {
            core::ptr::copy_nonoverlapping(
                src_buffer,
                self.fb_virt as *mut u32,
                size / 4,
            );
        }
    }
}