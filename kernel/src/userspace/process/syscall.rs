use super::proc as P;
use crate::trampoline_rings as tr;

pub const SYS_EXIT: u64    = 0x1000;
pub const SYS_SPAWN: u64   = 0x1001;
pub const SYS_GETPID: u64  = 0x1002;
pub const SYS_IPC_SEND: u64 = 0x1010;
pub const SYS_IPC_RECV: u64 = 0x1011;
pub const SYS_KEY: u64     = 0x1040;

extern "C" {
    fn paging_translate_in(pml4: u64, virt: u64) -> u64;
    fn kprintf(fmt: *const u8, ...);
}

const DIRECT_BASE: u64 = 0xFFFF888000000000;

fn user_copy_in(cr3: u64, src: u64, dst: &mut [u8]) -> bool {
    let mut pos = 0usize;

    while pos < dst.len() {
        let va = src + pos as u64;
        let page = va & !0xFFFu64;
        let off = (va & 0xFFF) as usize;

        let phys = unsafe { paging_translate_in(cr3, page) };

        if phys == u64::MAX {
            return false;
        }

        let n = (4096 - off).min(dst.len() - pos);

        unsafe {
            let p = (DIRECT_BASE + phys + off as u64) as *const u8;
            core::ptr::copy_nonoverlapping(p, dst.as_mut_ptr().add(pos), n);
        }

        pos += n;
    }

    true
}

fn user_cstr(cr3: u64, ptr: u64, buf: &mut [u8]) -> Option<&str> {
    for i in 0..buf.len() {
        let mut b = [0u8; 1];

        if !user_copy_in(cr3, ptr + i as u64, &mut b) {
            return None;
        }

        if b[0] == 0 {
            return core::str::from_utf8(&buf[..i]).ok();
        }

        buf[i] = b[0];
    }

    None
}

static mut ELF_BUF: [u8; 64 * 1024] = [0; 64 * 1024];

fn do_spawn(cr3: u64, path_ptr: u64, parent_pid: u32) -> i64 {
    let mut tmp = [0u8; 128];

    let path = match user_cstr(cr3, path_ptr, &mut tmp) {
        Some(s) => s,
        None => return -1,
    };

    let buf = unsafe { &mut ELF_BUF };

    let n = match crate::fs::vfs::root() {
        Some(fs) => match fs.read_path(path, buf) {
            Some(n) => n,
            None => return -1,
        },
        None => return -1,
    };

    let loaded = match super::elf::load(&buf[..n]) {
        Ok(l) => l,
        Err(_) => return -1,
    };

    let stack_phys = match crate::mm::phys::alloc_frames(4) {
        Some(p) => p,
        None => return -1,
    };

    let prot = crate::mm::space::PROT_READ
             | crate::mm::space::PROT_WRITE
             | crate::mm::space::PROT_USER;

    if !loaded.aspace.map_phys(tr::USER_STACK_TOP - 0x4000,
                               stack_phys, 0x4000, prot) {
        return -1;
    }

    let world = match tr::add_world(tr::RING_USER,
                                    loaded.aspace.cr3(),
                                    loaded.entry,
                                    tr::USER_STACK_TOP, 0) {
        Some(w) => w,
        None => return -1,
    };

    match P::register(world, parent_pid) {
        Some(pid) => pid as i64,
        None => -1,
    }
}

pub fn handle(world: usize, c: &mut tr::CpuCtx) {
    let cr3 = tr::world_cr3(world);

    let me = P::by_world(world)
        .map(|p| p.pid)
        .unwrap_or(0);

    let num = c.rax;
    let a0 = c.rdi;
    let a1 = c.rsi;
    let a2 = c.rdx;

    c.rax = match num {
        SYS_EXIT => {
            tr::exit_from(world, c, a0 as i32);
            return;
        }

        SYS_SPAWN => do_spawn(cr3, a0, me) as u64,

        SYS_GETPID => me as u64,

        SYS_IPC_SEND => {
            let msg = P::IpcMsg {
                from: me,
                a0: a1,
                a1: a2,
                a2: 0,
                a3: 0,
            };

            if P::send(a0 as u32, msg) { 0 } else { u64::MAX }
        }

        SYS_IPC_RECV => {
            match P::recv(me) {
                Some(m) => {
                    c.rdi = m.a0;
                    c.rsi = m.a1;
                    m.from as u64
                }
                None => u64::MAX,
            }
        }

        SYS_KEY => {
            match crate::drivers::usb::class::hid::keyboard::take_char() {
                Some(ch) => ch as u64,
                None => 0,
            }
        }

        _ => u64::MAX,
    };
}


// add to trampoline'
// pub const USER_STACK_TOP: u64 = 0x7FFF_0000_0000;

// pub fn current() -> usize {
//     unsafe { CURRENT.unwrap() }
// }

// pub fn world_cr3(world: usize) -> u64 {
//     unsafe { WORLDS[world].as_ref().unwrap().cr3 }
// }

// pub fn kill_world(world: usize) {
//     unsafe {
//         if let Some(w) = WORLDS[world].as_mut() {
//             w.alive = false;
//         }
//     }
// }

// pub fn exit_from(world: usize, c: &mut CpuCtx, code: i32) {
//     unsafe {
//         WORLDS[world].as_mut().unwrap().ctx = *c;
//         kill_world(world);

//         if let Some(p) = crate::process::proc::by_world(world) {
//             p.alive = false;
//             p.exit_code = code;
//         }

//         let next = pick_next(Some(world));
//         CURRENT = Some(next);

//         let w = WORLDS[next].as_mut().unwrap();

//         write_cr3(w.cr3);
//         tr_restore_ctx(&mut w.ctx);
//     }
// }