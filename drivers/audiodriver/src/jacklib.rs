use driverspacelib as ds;
use ds::jack::JackInfo;

pub struct JackMgr {
    query_id: Option<u64>,
    last: Option<JackInfo>,
    amp_wanted: bool,
}

impl JackMgr {
    pub const fn new() -> Self {
        Self { query_id: None, last: None, amp_wanted: true }
    }

    pub fn tick(&mut self) {
        if self.query_id.is_none() {
            self.query_id = Some(ds::jack::query_req());
            return;
        }

        let id = self.query_id.unwrap();

        if let Some(info) = ds::jack::query_take(id) {
            self.query_id = None;

            let changed = match self.last {
                Some(l) => l.present != info.present,
                None => true,
            };

            if changed {
                if info.present {
                    ds::log::ds_log("jack: plugged");
                } else {
                    ds::log::ds_log("jack: unplugged");
                }

                if info.present && self.amp_wanted {
                    ds::jack::set_amp(true);
                } else {
                    ds::jack::set_amp(false);
                }
            }

            self.last = Some(info);
        }
    }

    pub fn present(&self) -> bool {
        self.last.map(|l| l.present).unwrap_or(false)
    }
}