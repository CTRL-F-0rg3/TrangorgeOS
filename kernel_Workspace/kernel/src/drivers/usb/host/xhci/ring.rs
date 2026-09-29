use super::trb::Trb;
use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::UsbError;

pub struct CmdRing {
    buf: DmaBuf,
    len: usize,
    enqueue: usize,
    cycle: bool,
}

impl CmdRing {
    pub fn new(count: usize) -> Result<Self, UsbError> {
        let len = count + 1;
        let buf = DmaBuf::new(len * 16)?;

        unsafe {
            (buf.virt as *mut Trb).add(count).write_volatile(Trb::link(buf.phys));
        }

        Ok(Self { buf, len, enqueue: 0, cycle: true })
    }

    pub fn phys(&self) -> u64 {
        self.buf.phys
    }

    /// Where the next TRB will land, for diagnostics.
    pub fn enqueue_pos(&self) -> usize {
        self.enqueue
    }

    pub fn enqueue(&mut self, mut trb: Trb) {
        if self.enqueue == self.len - 1 {
            self.enqueue = 0;
            self.cycle = !self.cycle;
        }

        if self.cycle {
            trb.control |= 1;
        } else {
            trb.control &= !1;
        }

        unsafe {
            (self.buf.virt as *mut Trb)
                .add(self.enqueue)
                .write_volatile(trb);
        }

        self.enqueue += 1;
    }
}

pub struct EventRing {
    buf: DmaBuf,
    erst: DmaBuf,
    len: usize,
    dequeue: usize,
    cycle: bool,
}

pub struct TransferRing {
    buf: DmaBuf,
    len: usize,
    enqueue: usize,
    cycle: bool,
}

impl TransferRing {
    pub fn new(count: usize) -> Result<Self, UsbError> {
        let len = count + 1;
        let buf = DmaBuf::new(len * 16)?;

        unsafe {
            (buf.virt as *mut Trb).add(count).write_volatile(Trb::link(buf.phys));
        }

        Ok(Self { buf, len, enqueue: 0, cycle: true })
    }

    pub fn phys(&self) -> u64 {
        self.buf.phys
    }

    pub fn enqueue(&mut self, mut trb: Trb) {
        if self.enqueue == self.len - 1 {
            self.enqueue = 0;
            self.cycle = !self.cycle;
        }

        if self.cycle {
            trb.control |= 1;
        } else {
            trb.control &= !1;
        }

        unsafe {
            (self.buf.virt as *mut Trb).add(self.enqueue).write_volatile(trb);
        }

        self.enqueue += 1;
    }
}

impl EventRing {
    pub fn new(count: usize) -> Result<Self, UsbError> {
        let buf = DmaBuf::new(count * 16)?;
        // An Event Ring Segment Table entry is 32 bytes, not 16. The controller
        // walks the table in 32-byte strides, so a 16-byte allocation leaves it
        // reading the second half of the *next* thing in memory and interpreting
        // it as segment size and cycle bit. The 16 bytes that were written were
        // enough for the fields this driver sets, and not enough for the ones
        // the controller reads.
        let mut erst = DmaBuf::new(32)?;

        // `DmaBuf::new` hands back unallocated, non-zeroed memory, and the
        // Event Ring Segment Table is walked as a *32-byte* structure whose
        // reserved bits are read as data. Anything left as garbage there is a
        // segment size or a ring count, so the table is zeroed before it is
        // filled in.
        erst.zero();

        // One ERST entry. The 32-byte layout (see `struct xhci_erst_entry` and
        // `xhci_alloc_erst()` in Linux's drivers/usb/host/xhci-mem.c) is:
        //
        //   dword 0..1  64-bit event ring segment base address
        //   dword 2     segment size, in TRBs
        //   dword 3     reserved, zero
        //
        // The size is a *separate dword*, not a field sharing the address word.
        // Writing it at `count << 16` put the value in the middle of the address
        // instead, so the controller read a segment size of zero and never
        // linked the event ring.
        //
        // There is deliberately no cycle bit here. Linux leaves `seg_addr` as the
        // bare segment DMA address and sets `rsvd` to zero; OR-ing a cycle bit
        // into the low dword perturbs `er_start` by one, which pushes `erdp` to
        // the far end of the segment and makes the controller reject it.
        unsafe {
            let e = erst.virt as *mut u32;
            let addr = buf.phys & 0xFFFF_FFFF_FFFF_FFF0;

            e.add(0).write_volatile(addr as u32);
            e.add(1).write_volatile((addr >> 32) as u32);
            e.add(2).write_volatile(count as u32);
            e.add(3).write_volatile(0);
        }

        Ok(Self { buf, erst, len: count, dequeue: 0, cycle: true })
    }

    pub fn erst_phys(&self) -> u64 {
        self.erst.phys
    }

    pub fn pending(&self) -> Option<Trb> {
        let t = unsafe {
            (self.buf.virt as *const Trb).add(self.dequeue).read_volatile()
        };

        if t.cycle() == self.cycle {
            Some(t)
        } else {
            None
        }
    }

    pub fn pop(&mut self) {
        self.dequeue += 1;

        if self.dequeue == self.len {
            self.dequeue = 0;
            self.cycle = !self.cycle;
        }
    }

    /// Event Ring Dequeue Pointer.
    ///
    /// TRB-aligned, i.e. 16 bytes - not 256. Masking to `0xFF00` rounds the
    /// pointer down to a 256-byte boundary, which for a 4 KiB ring folds the
    /// first sixteen positions back onto position zero. ERDP then points at an
    /// event the driver has not consumed yet, the cycle comparison matches a TRB
    /// it already popped, and command completion looks like a stall.
    pub fn erdp(&self) -> u64 {
        (self.buf.phys + (self.dequeue as u64) * 16) & 0xFFFF_FFFF_FFFF_FFF0
    }
}
