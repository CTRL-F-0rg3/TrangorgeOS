use super::proc;

pub const FD_MAX: usize = 16;

const POOL_SLOTS: usize = 4;
const POOL_SIZE: usize = 16 * 1024;

#[derive(Clone, Copy, PartialEq)]
pub enum FdKind {
    None = 0,
    File,
    Stdio,
}

#[derive(Clone, Copy)]
pub struct Fd {
    pub kind: FdKind,
    pub pool: i16,
    pub offset: u32,
    pub len: u32,
}

impl Fd {
    pub const NONE: Fd = Fd { kind: FdKind::None, pool: -1, offset: 0, len: 0 };
    pub const STDIO: Fd = Fd { kind: FdKind::Stdio, pool: -1, offset: 0, len: 0 };
}

static mut POOL: [[u8; POOL_SIZE]; POOL_SLOTS] = [[0; POOL_SIZE]; POOL_SLOTS];
static mut POOL_USED: [bool; POOL_SLOTS] = [false; POOL_SLOTS];

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

fn pool_take() -> Option<i16> {
    unsafe {
        for i in 0..POOL_SLOTS {
            if !POOL_USED[i] {
                POOL_USED[i] = true;
                return Some(i as i16);
            }
        }
    }

    None
}

fn pool_drop(slot: i16) {
    if slot >= 0 && (slot as usize) < POOL_SLOTS {
        unsafe { POOL_USED[slot as usize] = false; }
    }
}

pub fn open_for(pid: u32, path: &str) -> i32 {
    let p = match proc::by_pid(pid) {
        Some(p) => p,
        None => return -1,
    };

    let mut idx = -1i32;

    for i in 0..FD_MAX {
        if p.fds[i].kind == FdKind::None {
            idx = i as i32;
            break;
        }
    }

    if idx < 0 {
        return -1;
    }

    let slot = match pool_take() {
        Some(s) => s,
        None => return -1,
    };

    let n = match crate::fs::vfs::root() {
        Some(fs) => {
            let buf = unsafe { &mut POOL[slot as usize] };
            match fs.read_path(path, buf) {
                Some(n) => n,
                None => {
                    pool_drop(slot);
                    return -1;
                }
            }
        }
        None => {
            pool_drop(slot);
            return -1;
        }
    };

    p.fds[idx as usize] = Fd {
        kind: FdKind::File,
        pool: slot,
        offset: 0,
        len: n as u32,
    };

    idx
}

pub fn read_for(pid: u32, fd: i32, out: &mut [u8]) -> i32 {
    let p = match proc::by_pid(pid) {
        Some(p) => p,
        None => return -1,
    };

    if fd < 0 || fd as usize >= FD_MAX {
        return -1;
    }

    let f = p.fds[fd as usize];

    match f.kind {
        FdKind::Stdio => {
            /* fd 0 = klawiatura (nieblokująco, 1 znak) */
            if fd == 0 {
                match crate::drivers::usb::class::hid::keyboard::take_char() {
                    Some(c) => {
                        if !out.is_empty() {
                            out[0] = c;
                        }
                        1
                    }
                    None => 0,
                }
            } else {
                -1
            }
        }

        FdKind::File => {
            let off = f.offset as usize;

            if off >= f.len as usize {
                return 0;
            }

            let n = out.len().min(f.len as usize - off);
            let slot = f.pool as usize;

            unsafe {
                out[..n].copy_from_slice(&POOL[slot][off..off + n]);
            }

            p.fds[fd as usize].offset += n as u32;

            n as i32
        }

        FdKind::None => -1,
    }
}

pub fn write_for(pid: u32, fd: i32, data: &[u8]) -> i32 {
    let p = match proc::by_pid(pid) {
        Some(p) => p,
        None => return -1,
    };

    if fd < 0 || fd as usize >= FD_MAX {
        return -1;
    }

    let f = p.fds[fd as usize];

    match f.kind {
        FdKind::Stdio => {
            if fd == 1 || fd == 2 {
                for &b in data {
                    unsafe { kprintf(b"%c\0".as_ptr(), b as u32); }
                }

                data.len() as i32
            } else {
                -1
            }
        }

        /* VFS jest read-only w v1 */
        FdKind::File => -1,
        FdKind::None => -1,
    }
}

pub fn close_for(pid: u32, fd: i32) -> i32 {
    let p = match proc::by_pid(pid) {
        Some(p) => p,
        None => return -1,
    };

    if fd < 3 || fd as usize >= FD_MAX {
        return -1;   /* stdio niezamykalne */
    }

    let f = p.fds[fd as usize];

    if f.kind == FdKind::None {
        return -1;
    }

    pool_drop(f.pool);

    p.fds[fd as usize] = Fd::NONE;

    0
}