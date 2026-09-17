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
        let erst = DmaBuf::new(16)?;

        unsafe {
            let e = erst.virt as *mut u64;
            e.add(0).write_volatile(buf.phys);
            e.add(1).write_volatile(count as u64);
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
