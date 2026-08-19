// bridge.rs
use super::aut;
use crate::driverspaceinit::abi::{DsMsg, DS_SCRATCH_VA, DS_SCRATCH_SIZE};

extern "C" {
    fn bt_ready() -> bool;
    fn bt_info(ver: *mut u8, bdaddr: *mut u8);
    fn bt_hci_cmd(opcode: u16, params: *const u8, len: u8) -> bool;
    fn bt_event_poll(buf: *mut u8, len: *mut u8) -> bool;
    fn bt_acl_send(data: *const u8, len: u16) -> bool;
    fn bt_acl_recv(data: *mut u8, len: *mut u16) -> bool;
}

pub fn bt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op as u8) {
        return -1;
    }

    let scratch = DS_SCRATCH_VA as *mut u8;

    match op {
        x if x == aut::BT_INFO => {
            let mut ver = 0u8;
            let mut addr = [0u8; 6];

            unsafe { bt_info(&mut ver, addr.as_mut_ptr()) };

            r.arg0 = ver as u64;

            let mut packed = 0u64;

            for i in 0..6 {
                packed |= (addr[i] as u64) << (i * 8);
            }

            r.arg1 = packed;
            0
        }

        x if x == aut::BT_CMD => {
            let opcode = m.arg0 as u16;
            let len = (m.arg1 as u8).min(64);

            let ok = unsafe { bt_hci_cmd(opcode, scratch, len) };

            if ok { 0 } else { -1 }
        }

        x if x == aut::BT_EVT => {
            let mut len = 0u8;

            let ok = unsafe { bt_event_poll(scratch, &mut len) };

            if !ok {
                return 1;
            }

            r.arg0 = len as u64;
            0
        }

        x if x == aut::BT_ACL_OUT => {
            let len = (m.arg0 as u16).min(256);

            let ok = unsafe { bt_acl_send(scratch, len) };

            if ok { 0 } else { -1 }
        }

        x if x == aut::BT_ACL_IN => {
            let mut len = 0u16;

            let ok = unsafe { bt_acl_recv(scratch, &mut len) };

            if !ok {
                return 1;
            }

            r.arg0 = len as u64;
            0
        }

        _ => -1,
    }
}