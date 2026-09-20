#![no_std]
#![no_main]

extern crate ds_ipc;
extern crate ds_log;
extern crate ds_mem;
extern crate kapi_abi;
extern crate kapi_syscall;

use ds_log::{ds_info, ds_error, ds_warn};
use ds_mem::dma::mapping::{MmioRegion, Volatile};
use kapi_abi::{DsCmd, Handle, Status};
use core::sync::atomic::{AtomicBool, Ordering};

const VENDOR_ID: u16 = 0x1234;
const DEVICE_ID: u16 = 0x1111;
const PCI_CLASS_VGA: u32 = 0x0300;

const VBE_INDEX_PORT: u16 = 0x01CE;
const VBE_DATA_PORT: u16 = 0x01CF;

const VBE_ID: u16 = 0;
const VBE_WIDTH: u16 = 1;
const VBE_HEIGHT: u16 = 2;
const VBE_BPP: u16 = 3;
const VBE_ENABLE: u16 = 4;

const VBE_ENABLE_ON: u16 = 0x01;
const VBE_ENABLE_LINEAR: u16 = 0x40;
const VBE_ENABLE_INIT: u16 = 0x80;

static mut G_INFO: VgpuInfo = VgpuInfo {
    fb: core::ptr::null_mut(),
    width: 0,
    height: 0,
    bpp: 0,
    stride: 0,
};

static mut G_READY: bool = false;

struct VgpuInfo {
    fb: *mut u32,
    width: u32,
    height: u32,
    bpp: u32,
    stride: u32,
}

#[no_mangle]
pub extern "C" fn driver_main() {
    ds_info!("VGPU", "Virtual GPU driver starting...");

    if !vgpu_init(1024, 768) {
        ds_error!("VGPU", "Failed to initialize VGPU");
        return;
    }

    ds_info!("VGPU", "VGPU initialized successfully");
    vgpu_clear(0xFF101020);

    loop {
        kapi_syscall::sys_yield();
    }
}

fn vgpu_init(w: u32, h: u32) -> bool {
    unsafe {
        if G_READY {
            return true;
        }
    }

    ds_info!("VGPU", "Searching for VGA device...");
    
    let bdf = match find_vga_device() {
        Some(bdf) => bdf,
        None => {
            ds_error!("VGPU", "VGA device not found");
            return false;
        }
    };

    ds_info!("VGPU", "Found VGA device at BDF: {}", bdf);

    let vendor_device = read_pci_config(bdf, 0);
    let expected = (VENDOR_ID as u32) | ((DEVICE_ID as u32) << 16);
    
    if vendor_device != expected {
        ds_error!("VGPU", "Vendor/Device ID mismatch: expected 0x{:x}, got 0x{:x}", expected, vendor_device);
        return false;
    }

    let fb_phys = read_pci_bar(bdf, 0);
    if fb_phys == 0 {
        ds_error!("VGPU", "Failed to read BAR0");
        return false;
    }

    ds_info!("VGPU", "Framebuffer physical address: 0x{:x}", fb_phys);

    enable_pci_device(bdf);

    vbe_write(VBE_ENABLE, 0);
    vbe_write(VBE_ID, 0xB0C4);
    vbe_write(VBE_WIDTH, w as u16);
    vbe_write(VBE_HEIGHT, h as u16);
    vbe_write(VBE_BPP, 32);
    vbe_write(VBE_ENABLE, VBE_ENABLE_ON | VBE_ENABLE_LINEAR | VBE_ENABLE_INIT);

    let fb_size = (w * h * 4) as u64;
    let va_hint = 0x45000000u64;

    ds_info!("VGPU", "Requesting MMIO mapping for framebuffer...");
    
    let region = match MmioRegion::map(Handle(0), fb_phys, fb_size) {
        Ok(r) => r,
        Err(e) => {
            ds_error!("VGPU", "Failed to map MMIO: {:?}", e);
            return false;
        }
    };

    ds_info!("VGPU", "Framebuffer mapped at virtual address: 0x{:x}", region.virt_addr());

    unsafe {
        G_INFO.fb = region.virt_addr() as *mut u32;
        G_INFO.width = w;
        G_INFO.height = h;
        G_INFO.bpp = 32;
        G_INFO.stride = w;
        G_READY = true;
    }

    true
}

fn find_vga_device() -> Option<u64> {
    let mut msg = kapi_abi::DsMsg::new(DsCmd::PciFind as u32);
    msg.arg0 = PCI_CLASS_VGA as u64;

    let reply = kapi_syscall::sys_ipc_call(DsCmd::PciFind, PCI_CLASS_VGA as u64, 0, 0);
    
    if reply.is_ok() && reply.arg0 != 0 {
        Some(reply.arg0)
    } else {
        None
    }
}

fn read_pci_config(bdf: u64, offset: u32) -> u32 {
    let reply = kapi_syscall::sys_ipc_call(DsCmd::PciRead, bdf, offset as u64, 0);
    
    if reply.is_ok() {
        reply.arg0 as u32
    } else {
        0xFFFFFFFF
    }
}

fn read_pci_bar(bdf: u64, bar: u32) -> u64 {
    let offset = 0x10 + (bar * 4);
    let val = read_pci_config(bdf, offset);
    
    if val == 0 || val == 0xFFFFFFFF {
        return 0;
    }
    
    (val & 0xFFFFFFF0) as u64
}

fn enable_pci_device(bdf: u64) {
    let cmd_offset = 0x04;
    let mut cmd = read_pci_config(bdf, cmd_offset);
    cmd |= 0x07;
    
    let mut msg = kapi_abi::DsMsg::new(DsCmd::PciWrite as u32);
    msg.arg0 = bdf;
    msg.arg1 = cmd_offset as u64;
    msg.arg2 = cmd as u64;
    
    kapi_syscall::sys_ipc_call(DsCmd::PciWrite, bdf, cmd_offset as u64, cmd as u64);
}

fn vbe_write(idx: u16, val: u16) {
    unsafe {
        core::arch::asm!("out dx, ax", in("dx") VBE_INDEX_PORT, in("ax") idx, options(nomem, nostack, preserves_flags));
        core::arch::asm!("out dx, ax", in("dx") VBE_DATA_PORT, in("ax") val, options(nomem, nostack, preserves_flags));
    }
}

fn vgpu_clear(color: u32) {
    unsafe {
        if !G_READY {
            return;
        }
        
        let fb = G_INFO.fb;
        let n = (G_INFO.width * G_INFO.height) as usize;
        
        for i in 0..n {
            core::ptr::write_volatile(fb.add(i), color);
        }
    }
}

fn vgpu_pixel(x: u32, y: u32, color: u32) {
    unsafe {
        if !G_READY || x >= G_INFO.width || y >= G_INFO.height {
            return;
        }
        
        let offset = (y * G_INFO.stride + x) as usize;
        core::ptr::write_volatile(G_INFO.fb.add(offset), color);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ds_error!("VGPU", "PANIC!");
    loop {
        kapi_syscall::sys_yield();
    }
}