use super::init::Xhci;
use super::trb::*;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

impl Xhci {
    pub fn drain_events(&mut self) {
        while let Some(t) = self.ev.pending() {
            let t = t;
            self.ev.pop();
            super::init::rt_write64(&self.regs, super::init::RT_ERDP, self.ev.erdp());

            match t.typ() {
                TRB_PORT_STATUS => unsafe {
                    kprintf(b"usb: port status change, port=%d\n\0".as_ptr(),
                            (t.param >> 24) as u32);
                },
                TRB_HC_EVENT => unsafe {
                    kprintf(b"usb: hc event, cc=%d\n\0".as_ptr(),
                            t.completion_code() as u32);
                },
                _ => {}
            }
        }
    }
}