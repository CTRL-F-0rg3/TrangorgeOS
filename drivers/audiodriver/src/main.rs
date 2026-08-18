use crate as ad;
use driverspacelib as ds;

pub struct JackMgr {
    present: bool,
    amp_on: bool,
}

impl JackMgr {
    pub const fn new() -> Self {
        Self { present: false, amp_on: false }
    }

    pub fn tick(&mut self) {
        let now = ad::jack_present();

        if now != self.present {
            self.present = now;

            if now {
                ds::log::ds_log("jack: present");
                self.set_amp(true);
            } else {
                ds::log::ds_log("jack: gone");
                self.set_amp(false);
            }
        }
    }

    pub fn set_amp(&mut self, on: bool) {
        self.amp_on = on;
        ad::set_amp(on);
    }

    pub fn present(&self) -> bool {
        self.present
    }
}