use crate::drivers::usb::UsbError;
use crate::mm::ffi;

pub const OP_USBCMD: usize = 0x00;
pub const OP_USBSTS: usize = 0x04;
pub const OP_PAGESIZE: usize = 0x08;
pub const OP_CRCR: usize = 0x10;
pub const OP_DCBAAP: usize = 0x30;
pub const OP_CONFIG: usize = 0x38;
pub const OP_PORTSC: usize = 0x400;

pub const CMD_RS: u32 = 1 << 0;
pub const CMD_HCRST: u32 = 1 << 1;
pub const CMD_INTE: u32 = 1 << 2;

pub const STS_HCH: u32 = 1 << 0;
pub const STS_CNR: u32 = 1 << 11;

pub const PORTSC_CCS: u32 = 1 << 0;
pub const PORTSC_PED: u32 = 1 << 1;
pub const PORTSC_PR: u32 = 1 << 4;
pub const PORTSC_PRC: u32 = 1 << 21;
pub const PORTSC_CSC: u32 = 1 << 17;
pub const PORTSC_SPEED: u32 = 0xF << 10;

const MAP_SIZE: usize = 0x10000;

unsafe fn r32(p: *const u8) -> u32 {
    (p as *const u32).read_volatile()
}

unsafe fn w32(p: *mut u8, v: u32) {
    (p as *mut u32).write_volatile(v)
}

pub struct XhciRegs {
    base: *mut u8,
    pub cap_len: usize,
    pub db_off: usize,
    pub rt_off: usize,
    pub max_slots: u32,
    pub max_intrs: u32,
    pub max_ports: u32,
    pub addr64: bool,
    pub csz64: bool,
}

impl XhciRegs {
    pub fn new(phys: u64) -> Result<Self, UsbError> {
        let ptr = if phys >= 0xFFFF800000000000 {
            phys
        } else {
            let mut virt = 0u64;

            if !unsafe { ffi::vmm_map_device(phys, MAP_SIZE, &mut virt) } {
                return Err(UsbError::MapFailed);
            }

            virt
        };

        unsafe {
            let base = ptr as *mut u8;

            let cap_len = base.read_volatile() as usize;
            let hcs1 = r32(base.add(0x04));
            let hcc1 = r32(base.add(0x10));
            let db = (r32(base.add(0x14)) & !0x3) as usize;
            let rt = (r32(base.add(0x18)) & !0xF) as usize;

            Ok(Self {
                base,
                cap_len,
                db_off: db,
                rt_off: rt,
                max_slots: hcs1 & 0xFF,
                max_intrs: (hcs1 >> 8) & 0x7FF,
                max_ports: (hcs1 >> 24) & 0xFF,
                addr64: hcc1 & 1 != 0,
                csz64: hcc1 & (1 << 2) != 0,
            })
        }
    }

    fn op(&self) -> *mut u8 {
        unsafe { self.base.add(self.cap_len) }
    }

    fn rt(&self) -> *mut u8 {
        unsafe { self.base.add(self.rt_off) }
    }

    fn db(&self) -> *mut u8 {
        unsafe { self.base.add(self.db_off) }
    }

    pub fn op_read(&self, off: usize) -> u32 {
        unsafe { r32(self.op().add(off)) }
    }

    pub fn op_write(&self, off: usize, v: u32) {
        unsafe { w32(self.op().add(off), v) }
    }

    pub fn rt_read(&self, off: usize) -> u32 {
        unsafe { r32(self.rt().add(off)) }
    }

    pub fn rt_write(&self, off: usize, v: u32) {
        unsafe { w32(self.rt().add(off), v) }
    }

    pub fn doorbell(&self, slot: u32, target: u32, task: u32) {
        unsafe {
            w32(self.db().add(slot as usize * 4),
                (target & 0xFF) | ((task & 0xFFFF) << 16));
        }
    }

    pub fn port_sc(&self, port: u32) -> u32 {
        self.op_read(OP_PORTSC + (port as usize - 1) * 0x10)
    }

    pub fn port_sc_write(&self, port: u32, v: u32) {
        self.op_write(OP_PORTSC + (port as usize - 1) * 0x10, v);
    }

    pub fn port_speed(&self, port: u32) -> u32 {
        (self.port_sc(port) & PORTSC_SPEED) >> 10
    }
}