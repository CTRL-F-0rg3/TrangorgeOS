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
        let mut erst = DmaBuf::new(16)?;

        // `DmaBuf::new` hands back unallocated, non-zeroed memory, and the
        // Event Ring Segment Table is walked as a *32-byte* structure whose
        // reserved bits are read as data. Anything left as garbage there is a
        // segment size or a ring count, so the table is zeroed before it is
        // filled in.
        erst.zero();

        // One ERST entry, per the xHCI Event Ring Segment Table layout (32 bytes):
        //
        //   bits 63:4  ring segment base address
        //   bits 31:24 XHCI extended ERST size   — only meaningful on entry 0
        //   bits 23:16 ring segment size          — TRBs *in this segment*
        //   bit  0     cycle
        //
        // The ring segment size and the cycle bit are what make the entry
        // usable at all, and both live in the first 64 bits. `count` therefore
        // has to be shifted into bits 23:16 of the *first* word; the previous
        // code wrote it to the second word, which is entirely reserved, and
        // left the controller with a segment size of zero and no cycle — so it
        // never linked the event ring and no command ever completed.
        unsafe {
            let e = erst.virt as *mut u64;
            let entry = (buf.phys & 0xFFFF_FFFF_FFFF_FFF0) | ((count as u64) << 16) | 1;
            e.add(0).write_volatile(entry);
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

    pub fn erdp(&self) -> u64 {
        (self.buf.phys + (self.dequeue as u64) * 16) & 0xFFFF_FFFF_FFFF_FF00
    }
}
