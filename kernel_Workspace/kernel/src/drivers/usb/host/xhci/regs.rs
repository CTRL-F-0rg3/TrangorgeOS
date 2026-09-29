use crate::drivers::usb::UsbError;
use crate::mm::ffi;

// Operational registers, xHCI 1.0 §5.4.2, relative to `CAPOFF` (the operational
// block starts at CAPLENGTH, but the offsets below are all measured from there):
//
//   0x00 USBCMD   0x04 USBSTS   0x08 PAGESIZE  0x14 DNCTRL
//   0x18 CRCR     0x30 DCBAAP   0x38 CONFIG    0x400 PORTSC
//
// `DCBAAP` and `CRCR` were previously at 0x40 and 0x48. Those offsets are not
// registers: QEMU resolves an unknown operational offset through
// `trace_unimplemented` and drops the write, so the controller was never given
// a device-context array or a command ring at all. Every command then expired
// with no completion posted, which reads exactly like a dead USB controller.
// `CONFIG` at 0x38 happened to be correct, which is the only reason the port
// enumeration that follows still worked.
pub const OP_USBCMD: usize = 0x00;
pub const OP_USBSTS: usize = 0x04;
pub const OP_PAGESIZE: usize = 0x08;
pub const OP_DNCTRL: usize = 0x14;
pub const OP_CRCR: usize = 0x18;
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

            // The legacy MM can leave the device MMIO unmapped even when
            // vmm_map_device reports success; reading it would page-fault.
            if !unsafe { ffi::paging_is_mapped(virt) } {
                return Err(UsbError::MapFailed);
            }

            virt
        };

        unsafe {
            let base = ptr as *mut u8;

            let cap_len = base.read_volatile() as usize;
            let hcs1 = r32(base.add(0x04));
            let hcc1 = r32(base.add(0x10));

            // The capability register block layout (xHCI 1.0 §5.2) is:
            //
            //   0x00 CAPLENGTH | 0x04 HCSPARAMS1 | 0x08 HCSPARAMS2
            //   0x0C HCSPARAMS3 | 0x10 HCCPARAMS1 | 0x14 DBOFF
            //   0x18 RTSOFF     | 0x1C HCCPARAMS2
            //
            // DBOFF and RTSOFF are byte offsets measured from the *capability*
            // base, i.e. from `base`. The operational block is the only one
            // placed at `base + CAPLENGTH`. These are 16-bit fields.
            let db_off = r32(base.add(0x14)) as usize & 0xFFFF;
            let rt_off = r32(base.add(0x18)) as usize & 0xFFFF;

            // Both blocks must fall inside the BAR we mapped, otherwise the
            // runtime/doorbell accesses below would land outside the device
            // window instead of failing loudly here.
            if db_off >= MAP_SIZE || rt_off >= MAP_SIZE {
                return Err(UsbError::MapFailed);
            }

            Ok(Self {
                base,
                cap_len,
                db_off,
                rt_off,
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

    /// The doorbell array offset, for diagnostics.
    ///
    /// Reported because "the doorbell register was never where we thought" is
    /// indistinguishable from "the controller ignored the doorbell" once both
    /// end as a command timeout.
    pub fn db_offset(&self) -> usize {
        self.db_off
    }

    /// Offset of the runtime register block, read from `RTSOFF` (cap+0x18).
    ///
    /// For QEMU this is `0x1000`. Deriving it any other way lands the
    /// interrupter/ERST registers in the middle of the doorbell array, which
    /// is silent corruption rather than an obvious failure.
    pub fn rt_offset(&self) -> usize {
        self.rt_off
    }

    /// Read a doorbell back without ringing it.
    ///
    /// Only meaningful for a device slot. QEMU's `xhci_doorbell_read` returns an
    /// unconditional 0 for every doorbell offset ("doorbells always read as 0"),
    /// so a zero readback here says nothing about whether the write landed and
    /// must not be used as evidence that the controller ignored it.
    pub fn db_read(&self, slot: u32) -> u32 {
        unsafe { r32(self.db().add(slot as usize * 4)) }
    }

    /// Ring the doorbell.
    ///
    /// `slot == 0` is the *Host Controller* doorbell and `target` is then the
    /// doorbell target — the number of command-ring TRBs the controller should
    /// have processed, which is a counter and not a flag. For a device slot the
    /// value is `(target | task << 16)`, and the field widths are 8 and 11 bits
    /// (xHCI 1.0 §4.7.16.3); the task mask was `0xFFFF` and the top five bits
    /// were silently dropped by the controller.
    pub fn doorbell(&self, slot: u32, target: u32, task: u32) {
        let v = if slot == 0 {
            // Host Controller Doorbell. QEMU treats this offset as a *poll*
            // rather than a counter: `xhci_doorbell_write` runs
            // `xhci_process_commands()` only when the written value is exactly
            // 0, and logs "bad doorbell 0 write" for anything else. Writing the
            // spec-legal "number of TRBs the controller should have processed"
            // therefore stops the controller from ever looking at the command
            // ring, and every command expires as a timeout. Zero is the correct
            // value for this controller; the counter is kept in software.
            0
        } else {
            (target & 0xFF) | ((task & 0x7FF) << 16)
        };
        unsafe { w32(self.db().add(slot as usize * 4), v) }
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
