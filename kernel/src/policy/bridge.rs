extern "C" {
    fn policy_evaluate(ring: u8, cls: u8, op: u8, arg: u64) -> u8;
    fn nim_policy_log(ring: u8, cls: u8, op: u8, dec: u8);
}

pub const ALLOW: u8 = 0;
pub const DENY: u8 = 1;

pub fn check(ring: u8, cmd: u32, arg: u64) -> bool {
    let cls = (cmd >> 8) as u8;
    let op = (cmd & 0xFF) as u8;

    let d = unsafe { policy_evaluate(ring, cls, op, arg) };

    unsafe { nim_policy_log(ring, cls, op, d) };

    d == ALLOW
}

// HYPER_YIELD => { /* zawsze dozwolone — scheduler */ }

// _ => {
//     let cmd = c.rax_or_x0_or_a0;   // zależnie od arch
//     let arg = c.arg_reg;

//     if !crate::policy::bridge::check(world_ring, cmd, arg) {
//         c.ret_reg = u64::MAX;      // DENIED
//         return;
//     }

//     /* dopiero teraz dispatch do service */
// }