use super::super::abi::abi::*;
use super::super::abi::src::RingView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsError {
    Timeout,
    QueueFull,
    BadStatus(i32),
    NoAspace,
    NoMemory,
    NotPrepared,
}

pub struct InitHandshake {
    pub k2d: RingView,
    pub d2k: RingView,
    pub next_id: u64,
}

impl InitHandshake {
    pub fn send(&mut self, cmd: DsCmd,
                a0: u64, a1: u64, a2: u64) -> Result<DsMsg, DsError> {
        let id = self.next_id;
        self.next_id += 1;

        let msg = DsMsg {
            id,
            cmd: cmd as u32,
            flags: DS_FLAG_RESPONSE,
            arg0: a0,
            arg1: a1,
            arg2: a2,
            status: 0,
            pad: 0,
        };

        if !self.k2d.push(&msg) {
            return Err(DsError::QueueFull);
        }

        for _ in 0..2_000_000 {
            if let Some(r) = self.d2k.pop() {
                if r.id == id {
                    if r.status == 0 {
                        return Ok(r);
                    }

                    return Err(DsError::BadStatus(r.status));
                }

                continue;
            }

            core::hint::spin_loop();
        }

        Err(DsError::Timeout)
    }

    pub fn run(&mut self, params_va: u64) -> Result<(), DsError> {
        self.send(DsCmd::Init, params_va, DS_MAGIC, DS_VERSION as u64)?;
        self.send(DsCmd::Caps, 0, 0, 0)?;
        self.send(DsCmd::Ready, 0, 0, 0)?;

        Ok(())
    }
}