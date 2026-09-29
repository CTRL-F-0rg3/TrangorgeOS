use super::regs::*;
use super::ring::{CmdRing, EventRing};
use super::trb::*;
use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::UsbError;

// `kprintf` is a C variadic; a `u8` cannot be passed through `...` because the
// callee reads a full word. See the same cast in `hid::attach`.
use core::ffi::c_uint;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

/// Yield budget for every wait on the controller. See the loops.
pub const WAIT_YIELDS: u32 = 4_000;

/// Offsets of the interrupter 0 register set, relative to the *runtime* block
/// base (`RTSOFF`).
///
/// Interrupter 0 does not start at the block base: MFINDEX occupies `0x00`, and
/// the register sets begin at `0x20` (xHCI 1.0 §4.2.2, "Interrupter Register
/// Set", stride 0x20). QEMU resolves the set number as `(reg - 0x20) / 0x20` and
/// silently drops any access outside a set it knows, so writing ERSTSZ at
/// `RTSOFF + 0x08` lands on MFINDEX: the controller is never told where its
/// event ring segment table is, posts no command completion, and the first
/// command - Enable Slot - expires as a timeout.
pub const RT_IMAN: usize = 0x20;
pub const RT_IMOD: usize = 0x24;
pub const RT_ERSTSZ: usize = 0x28;
pub const RT_ERSTBA: usize = 0x30;
pub const RT_ERDP: usize = 0x38;

pub struct Xhci {
    pub regs: XhciRegs,
    pub cmd: CmdRing,
    pub ev: EventRing,
    pub dcbaa: DmaBuf,
    pub ctx_size: usize,
    pub slots: u32,
    pub ports: u32,

    /// Ports already attached, one bit per port.
    ///
    /// The input loop rescans the ports so a keyboard plugged in after boot is
    /// picked up. Without this, every rescan re-enumerates devices that are
    /// already attached, which fills the two HID slots with duplicates and
    /// leaves the real keyboard unable to report anything.
    pub attached: u32,

    /// How many command-ring TRBs this driver has told the controller to read.
    ///
    /// # Why this has to be a counter
    ///
    /// The Host Controller Doorbell (DBOFF + 0) is not a flag. Per xHCI 1.0
    /// §4.7.16.1 its value is the *doorbell target*: the number of TRBs the
    /// controller should have processed. The controller compares it against its
    /// own position and only looks at the ring when the target is ahead.
    ///
    /// Writing a constant zero therefore means "you have nothing to do", and the
    /// controller never reads the TRB that was just enqueued. Every command then
    /// times out, no device is ever enabled, and the consequence reaches all the
    /// way up to a keyboard that never enumerates - which looks like a broken
    /// input path and is not one.
    ///
    /// It starts at 0, matching a controller that has been reset and believes it
    /// has processed nothing.
    pub cmd_doorbell: u32,
}

/// One iteration of a wait on the controller.
///
/// # Why this is a plain spin, and why that is a bug we have not fixed yet
///
/// A controller modelled by QEMU is stepped by the host from its own event
/// loop, and a guest in a tight `pause` loop never yields to it - so in
/// principle the wait has to hand the CPU over with `hlt`.
///
/// That was tried and it is worse. Each iteration became one timer interrupt,
/// which turned a wait bounded by an iteration count into one bounded by
/// millions of wake-ups, and the machine stopped booting. The iteration budget
/// would also have to be restated in wall-clock terms, which this code has no
/// way to do without a timer read.
///
/// So the spin is deliberately left as it is: it keeps the wait bounded and the
/// system bootable, and it leaves the timeout in place to be reported rather
/// than a hang. Whether a real controller *needs* the yield is unresolved -
/// with `hlt` removed, a command that never completes is a `Timeout` again,
/// which at least is a diagnosable state.
#[inline]
pub fn wait_step() {
    core::hint::spin_loop();
}

fn spin_wait<F: Fn() -> bool>(f: F) -> Result<(), UsbError> {
            // The budget counts *yields*, not nanoseconds: with `hlt` in the wait,
        // one iteration is one timer interrupt, so the old 2,000,000 meant
        // millions of wake-ups - a hang dressed as a slow timeout. A successful
        // command completes within a couple of yields, because the controller
        // posts its event the moment the CPU hands the slot over, so a few
        // thousand is generous for success and still bounded for failure.
        for _ in 0..WAIT_YIELDS {
        if f() {
            return Ok(());
        }

        wait_step();
    }

    Err(UsbError::Timeout)
}

fn op_write64(regs: &XhciRegs, off: usize, v: u64) {
    regs.op_write(off, (v & 0xFFFF_FFFF) as u32);
    regs.op_write(off + 4, (v >> 32) as u32);
}

/// Write a 64-bit runtime register.
///
/// Public to the crate rather than to the parent module because the HID class
/// driver also has to advance ERDP, and it is a 64-bit register: writing it
/// through the 32-bit `rt_write` silently truncates the physical address
/// whenever the allocation lands above 4 GiB, and the controller then keeps
/// posting events into a segment it cannot name.
pub(crate) fn rt_write64(regs: &XhciRegs, off: usize, v: u64) {
    regs.rt_write(off, (v & 0xFFFF_FFFF) as u32);
    regs.rt_write(off + 4, (v >> 32) as u32);
}

pub fn init(regs: XhciRegs) -> Result<Xhci, UsbError> {
    let cmd0 = regs.op_read(OP_USBCMD);

    if cmd0 & CMD_RS != 0 {
        regs.op_write(OP_USBCMD, cmd0 & !CMD_RS);
        spin_wait(|| regs.op_read(OP_USBSTS) & STS_HCH != 0)?;
    }

    regs.op_write(OP_USBCMD, CMD_HCRST);

    spin_wait(|| regs.op_read(OP_USBCMD) & CMD_HCRST == 0)?;
    spin_wait(|| regs.op_read(OP_USBSTS) & STS_CNR == 0)?;

    // PAGESIZE is at operational offset 0x08. Bit 0 is reserved and reads 0;
    // bits 2:1 are the maximum burst size and must be 0 for 4 KiB pages. QEMU
    // returns the literal value 1 there, which is that same encoding of "4 KiB,
    // no burst", so the check has to tolerate it rather than reject it.
    if regs.op_read(OP_PAGESIZE) & 0x7 > 0x1 {
        return Err(UsbError::Invalid);
    }

    let slots = regs.max_slots.min(64);
    let ports = regs.max_ports;

    let mut dcbaa = DmaBuf::new(64 * 8)?;
    dcbaa.zero();

    let cmd = CmdRing::new(64)?;
    let ev = EventRing::new(256)?;

    // CONFIG carries three fields, not one: MaxSlotsEn (bits 7:0),
    // MaxPortsEn (bits 15:8) and MaxIntrsEn (bits 23:16). Leaving the port field
    // at zero tells the controller it has no ports to use, which it is entitled
    // to believe — it is a "how many of the device's ports I enable" field, not a
    // readback of the hardware.
    let max_intrs = regs.max_intrs.min(1);
    regs.op_write(OP_CONFIG, (slots & 0xFF) | ((ports & 0xFF) << 8) | ((max_intrs & 0xFF) << 16));
    op_write64(&regs, OP_DCBAAP, dcbaa.phys);
    op_write64(&regs, OP_CRCR, cmd.phys() | 1);

    // Runtime register setup order, as it appears on the wire in a working
    // enumeration (verified against QEMU's `usb_xhci_*` trace):
    //
    //   ERSTSZ -> ERDP -> ERSTBA -> USBCMD.RS
    //
    // ERDP is written *before* ERSTBA. QEMU only latches the event ring from
    // the `case 0x14` arm of `xhci_runtime_write` (the high half of ERSTBA), so
    // at that point it validates ERSTSZ and resolves the segment table. Writing
    // IMOD/IMAN between ERSTBA and ERDP is harmless, but reordering ERSTSZ past
    // ERSTBA is not: `xhci_er_reset` then sees `erstsz == 0`, treats the
    // interrupter as disabled, leaves `er_size` at 0, and every later
    // `xhci_event` fails its own bounds check and raises HCE. From the guest side
    // that looks like a dead controller - no completions, every command timing
    // out - even though the command ring itself was programmed correctly.
    regs.rt_write(RT_ERSTSZ, 1);
    rt_write64(&regs, RT_ERDP, ev.erdp());
    regs.rt_write(RT_IMOD, 0);
    regs.rt_write(RT_IMAN, 0);
    rt_write64(&regs, RT_ERSTBA, ev.erst_phys());

    regs.op_write(OP_USBCMD, CMD_RS);

    spin_wait(|| regs.op_read(OP_USBSTS) & STS_HCH == 0)?;

    // Read the ring pointers back before trusting any of them. Every failure
    // that follows - a command that times out, a device that never enables -
    // looks identical, and the first question is always whether the controller
    // kept what we told it. A register that reads back as zero is the
    // difference between "the controller has the wrong pointer" and "the
    // controller is ignoring the ring", and those need opposite fixes.
    let crcr_lo = regs.op_read(OP_CRCR);
    let crcr_hi = regs.op_read(OP_CRCR + 4);
    let dcbaa_lo = regs.op_read(OP_DCBAAP);
    let erstba_lo = regs.rt_read(RT_ERSTBA);
    let erstsz = regs.rt_read(RT_ERSTSZ);
    let db0 = regs.db_read(0);
    crate::serial::write_str(&alloc::format!(
        "[xhci] ring: rt=0x{:x} db=0x{:x} CRCR=0x{:x}{:08x} (pisano 0x{:x}) \
         DCBAAP.lo=0x{:08x} ERSTBA.lo=0x{:08x} ERSTSZ={} DB0={} cfg=0x{:x}\n",
        regs.rt_offset(),
        regs.db_offset(),
        crcr_hi,
        crcr_lo,
        cmd.phys(),
        dcbaa_lo,
        erstba_lo,
        erstsz,
        db0,
        regs.op_read(OP_CONFIG),
    ));

    let ctx_size = if regs.csz64 { 64 } else { 32 };

    Ok(Xhci {
        regs,
        cmd,
        ev,
        dcbaa,
        ctx_size,
        slots,
        ports,
        attached: 0,
        // A controller that has just come out of HCRST believes it has
        // processed no command-ring TRBs, so the first doorbell target is 1.
        cmd_doorbell: 0,
    })
}

impl Xhci {
    /// Submit one command-ring TRB and wait for its completion event.
    pub fn command(&mut self, trb: Trb) -> Result<Trb, UsbError> {
        // Which command is this? The completion event carries no opcode, so
        // without this the timeout log says only "a command timed out" and
        // there is nothing to attach a trace to.
        let wanted_type = trb.typ();

        self.cmd.enqueue(trb);

        // The doorbell target, not a flag. See `cmd_doorbell` for why a constant
        // here meant the controller was never told to read the ring. Written
        // *after* the TRB and *before* the wait, and never on a failure path - a
        // doorbell rung for a command that was never enqueued would make the
        // controller read the wrong TRB and desynchronise the ring.
        self.cmd_doorbell = self.cmd_doorbell.wrapping_add(1);
        self.regs.doorbell(0, self.cmd_doorbell, 0);

                // The budget counts *yields*, not nanoseconds: with `hlt` in the wait,
        // one iteration is one timer interrupt, so the old 2,000,000 meant
        // millions of wake-ups - a hang dressed as a slow timeout. A successful
        // command completes within a couple of yields, because the controller
        // posts its event the moment the CPU hands the slot over, so a few
        // thousand is generous for success and still bounded for failure.
        for _ in 0..WAIT_YIELDS {
            if let Some(t) = self.ev.pending() {
                let t = t;
                self.ev.pop();
                rt_write64(&self.regs, RT_ERDP, self.ev.erdp());

                if t.typ() == TRB_CMD_COMPLETION {
                    if t.completion_code() == CC_SUCCESS {
                        return Ok(t);
                    }

                    return Err(UsbError::Transfer(t.completion_code()));
                }

                continue;
            }

            wait_step();
        }

        // A timeout here used to be indistinguishable from a broken controller.
        // What is reported is everything needed to tell the three candidates
        // apart: the doorbell we rang and what the register reads back (a
        // controller that ignores it reads 0), whether the event ring moved at
        // all, and the position the command ring had reached.
        let db_back = self.regs.db_read(0);
        crate::serial::write_str(&alloc::format!(
            "[xhci] TIMEOUT typ=0x{:02x} dzwonek={} wrocil={} cmd_enq={} erdp=0x{:x} \
             USBSTS=0x{:x} (CRST={} CNR={} CFGCHG={})\n",
            wanted_type,
            self.cmd_doorbell,
            db_back,
            self.cmd.enqueue_pos(),
            self.ev.erdp(),
            self.regs.op_read(OP_USBSTS),
            u8::from(self.regs.op_read(OP_USBSTS) & (1 << 0) != 0),
            u8::from(self.regs.op_read(OP_USBSTS) & (1 << 11) != 0),
            u8::from(self.regs.op_read(OP_USBSTS) & (1 << 2) != 0),
        ));

        Err(UsbError::Timeout)
    }

    pub fn scan_ports(&mut self) {
        for p in 1..=self.ports {
            // Skip ports that have already been enumerated. The rescan exists to
            // catch devices plugged in after boot; without this guard it would
            // also re-attach the ones already there.
            if self.attached & (1 << (p - 1)) != 0 {
                continue;
            }

            let sc = self.regs.port_sc(p);

            if sc & PORTSC_CCS == 0 {
                // Silent. An empty port is the normal case on almost every
                // machine, and this scan is repeated for as long as the menu
                // waits: reporting every idle port turns the boot log into a
                // wall of lines that says nothing eight times over. A port with
                // something on it is reported below, which is the case worth
                // a line.
                continue;
            }

            let speed = (sc & PORTSC_SPEED) >> 10;

            unsafe {
                kprintf(b"usb: port %d connected, speed=%d\n\0".as_ptr(), p, speed);
            }

            if sc & PORTSC_PED == 0 && speed <= 3 {
                self.regs.port_sc_write(p, sc | PORTSC_PR);

                let ok = spin_wait(|| {
                    self.regs.port_sc(p) & PORTSC_PRC != 0
                }).is_ok();

                if ok {
                    self.regs.port_sc_write(p, PORTSC_PRC | PORTSC_CSC);
                }
            }

            let sc2 = self.regs.port_sc(p);
            let enabled = sc2 & PORTSC_PED != 0;

            unsafe {
                kprintf(b"usb: port %d enabled=%d\n\0".as_ptr(),
                        p,
                        enabled as u32);
            }

            if !enabled {
                continue;
            }

            // Marked before attaching, not after: if attach fails the port still
            // holds a device that is not working, and retrying it on every
            // rescan would restart enumeration for the rest of the boot.
            self.attached |= 1 << (p - 1);

            self.attach_port(p);
        }
    }

    fn attach_port(&mut self, p: u32) {
        let mut dev = match crate::drivers::usb::core::enumerate::enumerate(self, p) {
            Ok(d) => d,
            Err(_) => {
                unsafe {
                    kprintf(b"usb: port %d enumeration failed\n\0".as_ptr(), p);
                }
                return;
            }
        };

        use crate::drivers::usb::class::{hid, mass};

        match mass::attach(self, &mut dev) {
            Ok(true) => return,
            Ok(false) => {}
            Err(_) => unsafe {
                kprintf(b"usb: port %d mass-storage attach failed\n\0".as_ptr(), p);
            },
        }

        match hid::attach(self, &mut dev) {
            Ok(true) => {}
            Ok(false) => unsafe {
                kprintf(b"usb: port %d: no matching class driver\n\0".as_ptr(), p);
            },
            Err(_) => unsafe {
                kprintf(b"usb: port %d HID attach failed\n\0".as_ptr(), p);
            },
        }
    }
}