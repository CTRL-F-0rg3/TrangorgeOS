use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::UsbError;

pub struct Contexts {
    pub input: DmaBuf,
    pub output: DmaBuf,
    pub ctx_size: usize,
}

impl Contexts {
    pub fn setup_configure_ep(&mut self, speed: u32, port: u32,
                              epidx: u32, ep_type: u32, mps: u16,
                              interval: u32, ring_phys: u64) {
    self.input.zero();
    self.in_add_drop(1 | (1 << epidx), 0);

    let s = self.slot_ptr();

    unsafe {
        s.add(0).write_volatile((speed << 20) | (2 << 27));
        s.add(1).write_volatile(port << 16);
    }

    let e = unsafe {
        self.input.virt.add(self.ctx_size * (1 + epidx as usize)) as *mut u32
    };

    unsafe {
        e.add(0).write_volatile((3 << 1) | (ep_type << 3) | ((interval & 0xFF) << 16));
        e.add(1).write_volatile((mps as u32) << 16);
        e.add(2).write_volatile((ring_phys & 0xFFFF_FFFF_FFFF_FF00) as u32 | 1);
        e.add(3).write_volatile((ring_phys >> 32) as u32);
        e.add(4).write_volatile(mps as u32);
    }
}

pub fn setup_configure_bulk_pair(&mut self, speed: u32, port: u32,
                                 out_idx: u32, in_idx: u32,
                                 mps_out: u16, mps_in: u16,
                                 ring_out: u64, ring_in: u64) {
    self.input.zero();
    self.in_add_drop(1 | (1 << out_idx) | (1 << in_idx), 0);

    let s = self.slot_ptr();

    unsafe {
        s.add(0).write_volatile((speed << 20) | (3 << 27));
        s.add(1).write_volatile(port << 16);
    }

    for (idx, typ, mps, ring) in [
        (out_idx, 2u32, mps_out, ring_out),
        (in_idx, 6u32, mps_in, ring_in),
    ] {
        let e = unsafe {
            self.input.virt.add(self.ctx_size * (1 + idx as usize)) as *mut u32
        };

        unsafe {
            e.add(0).write_volatile((3 << 1) | (typ << 3));
            e.add(1).write_volatile((mps as u32) << 16);
            e.add(2).write_volatile((ring & 0xFFFF_FFFF_FFFF_FF00) as u32 | 1);
            e.add(3).write_volatile((ring >> 32) as u32);
            e.add(4).write_volatile(512);
        }
    }
}
}

impl Contexts {
    pub fn new(ctx_size: usize) -> Result<Self, UsbError> {
        let mut input = DmaBuf::new(ctx_size * 3)?;
        let output = DmaBuf::new(ctx_size * 2)?;

        input.zero();

        Ok(Self { input, output, ctx_size })
    }

    fn in_add_drop(&mut self, add: u32, drop: u32) {
        let p = self.input.virt as *mut u32;

        unsafe {
            p.add(0).write_volatile(drop);
            p.add(1).write_volatile(add);
        }
    }

    fn slot_ptr(&mut self) -> *mut u32 {
        unsafe { self.input.virt.add(self.ctx_size) as *mut u32 }
    }

    fn ep0_ptr(&mut self) -> *mut u32 {
        unsafe { self.input.virt.add(self.ctx_size * 2) as *mut u32 }
    }

    pub fn setup_address_device(&mut self, speed: u32, port: u32,
                                 mps: u16, ring_phys: u64) {
        self.input.zero();
        self.in_add_drop(3, 0);

        let s = self.slot_ptr();

        unsafe {
            s.add(0).write_volatile((speed << 20) | (1 << 27));
            s.add(1).write_volatile(port << 16);
        }

        let e = self.ep0_ptr();

        unsafe {
            e.add(0).write_volatile((3 << 1) | (4 << 3));
            e.add(1).write_volatile((mps as u32) << 16);
            e.add(2).write_volatile((ring_phys & 0xFFFF_FFFF_FFFF_FF00) as u32 | 1);
            e.add(3).write_volatile((ring_phys >> 32) as u32);
            e.add(4).write_volatile(8);
        }
    }

    pub fn setup_evaluate_mps(&mut self, mps: u16) {
        self.input.zero();
        self.in_add_drop(2, 0);

        let e = self.ep0_ptr();

        unsafe {
            e.add(1).write_volatile((mps as u32) << 16);
        }
    }
}