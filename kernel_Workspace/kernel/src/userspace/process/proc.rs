pub const MAX_PROCS: usize = 8;
pub const MAILBOX_LEN: usize = 8;

#[derive(Clone, Copy, Default)]
pub struct IpcMsg {
    pub from: u32,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
}

pub struct Process {
    pub pid: u32,
    pub world: usize,
    pub parent: u32,
    pub alive: bool,
    pub exit_code: i32,
    pub mbox: [IpcMsg; MAILBOX_LEN],
    pub mbox_head: u32,
    pub mbox_tail: u32,
}

static mut PROCS: [Option<Process>; MAX_PROCS] =
    [None, None, None, None, None, None, None, None];

static mut NEXT_PID: u32 = 1;

pub fn register(world: usize, parent: u32) -> Option<u32> {
    unsafe {
        for slot in PROCS.iter_mut() {
            if slot.is_none() {
                let pid = NEXT_PID;
                NEXT_PID += 1;

                *slot = Some(Process {
                    pid,
                    world,
                    parent,
                    alive: true,
                    exit_code: 0,
                    mbox: [IpcMsg::default(); MAILBOX_LEN],
                    mbox_head: 0,
                    mbox_tail: 0,
                });

                return Some(pid);
            }
        }
    }

    None
}

pub fn by_pid(pid: u32) -> Option<&'static mut Process> {
    unsafe {
        for slot in PROCS.iter_mut() {
            if let Some(p) = slot {
                if p.pid == pid {
                    return Some(p);
                }
            }
        }
    }

    None
}

pub fn by_world(world: usize) -> Option<&'static mut Process> {
    unsafe {
        for slot in PROCS.iter_mut() {
            if let Some(p) = slot {
                if p.world == world {
                    return Some(p);
                }
            }
        }
    }

    None
}

pub fn send(dst: u32, msg: IpcMsg) -> bool {
    let p = match by_pid(dst) {
        Some(p) => p,
        None => return false,
    };

    let next = (p.mbox_head + 1) % MAILBOX_LEN as u32;

    if next == p.mbox_tail {
        return false;
    }

    p.mbox[p.mbox_head as usize] = msg;
    p.mbox_head = next;

    true
}

pub fn recv(pid: u32) -> Option<IpcMsg> {
    let p = by_pid(pid)?;

    if p.mbox_tail == p.mbox_head {
        return None;
    }

    let m = p.mbox[p.mbox_tail as usize];
    p.mbox_tail = (p.mbox_tail + 1) % MAILBOX_LEN as u32;

    Some(m)
}