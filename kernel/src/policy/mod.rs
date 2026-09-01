use crate::caps::{audit, check, store, Capability};
use spin::Mutex;

pub const ALLOW: u8 = 0;
pub const DENY: u8 = 1;

pub const RING_KERNEL: u8 = 0;
pub const RING_DRIVER: u8 = 1;
pub const RING_USER: u8 = 2;

pub const CLS_SYS: u8 = 0;
pub const CLS_VIDEO: u8 = 1;
pub const CLS_AUDIO: u8 = 2;
pub const CLS_INPUT: u8 = 3;
pub const CLS_BLOCK: u8 = 4;
pub const CLS_NET: u8 = 5;

pub const CLS_BT: u8 = 6;

pub const BLK_WRITE: u8 = 3;

pub const fn cmd(class: u8, op: u8) -> u32 {
    ((class as u32) << 8) | (op as u32)
}

pub const fn class_of(cmd: u32) -> u8 {
    (cmd >> 8) as u8
}

pub const fn op_of(cmd: u32) -> u8 {
    (cmd & 0xFF) as u8
}

pub fn evaluate(ring: u8, class: u8, op: u8, _arg: u64) -> u8 {
    if ring == RING_KERNEL {
        return ALLOW;
    }

    if ring == RING_USER && class == CLS_BLOCK && op == BLK_WRITE {
        return DENY;
    }

    if ring == RING_USER && class == CLS_NET {
        return DENY;
    }

    if ring == RING_DRIVER {
        return ALLOW;
    }

    match class {
        CLS_SYS | CLS_VIDEO | CLS_AUDIO | CLS_INPUT | CLS_BLOCK => ALLOW,
        CLS_NET => DENY,
        _ => DENY,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolicyEntry {
    pub ring: u8,
    pub cls: u8,
    pub op: u8,
    pub dec: u8,
}

const LOG_CAP: usize = 256;

struct LogInner {
    buf: [Option<PolicyEntry>; LOG_CAP],
    head: usize,
    total: u64,
    denies: u64,
}

static LOG: Mutex<LogInner> = Mutex::new(LogInner {
    buf: [None; LOG_CAP],
    head: 0,
    total: 0,
    denies: 0,
});

pub fn policy_log(ring: u8, cls: u8, op: u8, dec: u8) {
    let mut l = LOG.lock();
    let entry = PolicyEntry { ring, cls, op, dec };
    let idx = l.head;
    l.buf[idx] = Some(entry);
    l.head = (idx + 1) % LOG_CAP;
    l.total += 1;
    if dec == DENY {
        l.denies += 1;
    }
}

pub fn denies() -> u64 {
    LOG.lock().denies
}

pub fn total() -> u64 {
    LOG.lock().total
}

#[allow(dead_code)]
pub fn entry(idx: usize) -> Option<PolicyEntry> {
    let l = LOG.lock();
    if idx >= LOG_CAP {
        None
    } else {
        l.buf[idx]
    }
}

pub fn ring_of(world: u32) -> u8 {
    if world == check::kernel_world_id() {
        return RING_KERNEL;
    }
    if store::world_has_cap(world, Capability::Ring0)
        || store::world_has_cap(world, Capability::Driver)
    {
        RING_DRIVER
    } else {
        RING_USER
    }
}

pub fn required_caps(class: u8) -> &'static [Capability] {
    match class {
        CLS_SYS => &[Capability::SyscallRestricted, Capability::SyscallAll],
        CLS_VIDEO | CLS_AUDIO => &[Capability::DevMmio],
        CLS_INPUT => &[Capability::DevIrq, Capability::DevMmio],
        CLS_BLOCK => &[Capability::FsRead, Capability::FsWrite],
        CLS_NET | CLS_BT => &[Capability::IpcPrivileged, Capability::SyscallAll],
        _ => &[],
    }
}

pub fn hook(world: u32, cap: Capability) -> bool {
    let class = match cap {
        Capability::SyscallAll | Capability::SyscallRestricted => CLS_SYS,
        Capability::DevMmio => CLS_VIDEO,
        Capability::DevPci | Capability::DevPort | Capability::DevIrq => CLS_INPUT,
        Capability::FsRead | Capability::FsWrite | Capability::FsCreate
        | Capability::FsMount => CLS_BLOCK,
        Capability::IpcPrivileged => CLS_NET,
        _ => CLS_SYS,
    };
    evaluate(ring_of(world), class, cap.id(), 0) == ALLOW
}

pub fn install() {
    crate::caps::policy::set_hook(hook);
}

pub fn decide(world: u32, cmd: u32, arg: u64) -> Result<(), &'static str> {
    let cls = class_of(cmd);
    let op = op_of(cmd);
    let ring = ring_of(world);

    let decision = evaluate(ring, cls, op, arg);
    policy_log(ring, cls, op, decision);

    if decision == DENY {
        return Err("policy: reguła odmawia (ring/class/op)");
    }

    if ring == RING_KERNEL {
        return Ok(());
    }

    let required = required_caps(cls);
    if required.is_empty() {
        return Err("policy: nieznana klasa usługi");
    }

    for &cap in required {
        if store::world_has_cap(world, cap) {
            audit::log_check(world, cap, true);
            return Ok(());
        }
    }

    audit::log_check(world, required[0], false);
    Err("policy: world nie ma wymaganej capability")
}

pub fn decide_current(cmd: u32, arg: u64) -> Result<(), &'static str> {
    decide(check::current_world_id_pub(), cmd, arg)
}

#[cfg(all(target_arch = "x86_64", feature = "policy-foreign"))]
pub mod bridge;