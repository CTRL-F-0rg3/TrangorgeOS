use super::aut;
use crate::driverspaceinit::abi::abi::DsMsg;
use crate::driverspaceinit::init::init;
use crate::driverspaceinit::init::initabi::DS_SCRATCH_SIZE;

extern "C" {
    fn bt_init() -> bool;
    fn bt_ready() -> bool;
    fn bt_info(ver: *mut u8, bdaddr: *mut u8);
    fn bt_hci_cmd(opcode: u16, params: *const u8, len: u8) -> bool;
    fn bt_event_poll(buf: *mut u8, cap: u8, len: *mut u8) -> bool;
    fn bt_acl_send(data: *const u8, len: u16) -> bool;
    fn bt_acl_recv(data: *mut u8, cap: u16, len: *mut u16) -> bool;
}

const HCI_EVENT_MAX: usize = 64;
const ACL_MAX: usize = 256;
const HCI_PARAM_MAX: usize = 64;

pub fn bt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
    if !aut::authorize(ring, op) {
        return -1;
    }

    if op == aut::BT_INIT {
        return if unsafe { bt_init() } { 0 } else { -2 };
    }

    if !unsafe { bt_ready() } {
        return -2;
    }

    let scratch: *mut u8 = match init::scratch_view() {
        Some(ptr) => ptr,
        None => return -2,
    };

    match op {
        aut::BT_INFO => {
            let mut ver = 0u8;
            let mut addr = [0u8; 6];
            unsafe { bt_info(&mut ver, addr.as_mut_ptr()) };
            r.arg0 = ver as u64;
            r.arg1 = u64::from_le_bytes([addr[0], addr[1], addr[2], addr[3], addr[4], addr[5], 0, 0]);
            0
        }
        aut::BT_CMD => {
            if DS_SCRATCH_SIZE < HCI_PARAM_MAX {
                return -3;
            }
            let len = (m.arg1 as usize).min(HCI_PARAM_MAX) as u8;
            if unsafe { bt_hci_cmd(m.arg0 as u16, scratch as *const u8, len) } { 0 } else { -1 }
        }
        aut::BT_EVT => {
            if DS_SCRATCH_SIZE < HCI_EVENT_MAX {
                return -3;
            }
            let mut len = 0u8;
            if !unsafe { bt_event_poll(scratch, HCI_EVENT_MAX as u8, &mut len) } {
                return 1;
            }
            r.arg0 = len as u64;
            0
        }
        aut::BT_ACL_OUT => {
            if DS_SCRATCH_SIZE < ACL_MAX {
                return -3;
            }
            let len = (m.arg0 as usize).min(ACL_MAX) as u16;
            if unsafe { bt_acl_send(scratch as *const u8, len) } { 0 } else { -1 }
        }
        aut::BT_ACL_IN => {
            if DS_SCRATCH_SIZE < ACL_MAX {
                return -3;
            }
            let mut len = 0u16;
            if !unsafe { bt_acl_recv(scratch, ACL_MAX as u16, &mut len) } {
                return 1;
            }
            r.arg0 = len as u64;
            0
        }
        _ => -1,
    }
}
