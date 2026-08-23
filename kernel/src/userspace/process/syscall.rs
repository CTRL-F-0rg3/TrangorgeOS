use super::proc as P;
use crate::trampoline_rings as tr;

pub const SYS_EXIT: u64    = 0x1000;
pub const SYS_SPAWN: u64   = 0x1001;
pub const SYS_GETPID: u64  = 0x1002;
pub const SYS_IPC_SEND: u64 = 0x1010;
pub const SYS_IPC_RECV: u64 = 0x1011;
pub const SYS_KEY: u64     = 0x1040;
pub const SYS_READDIR: u64 = 0x1025;
pub const SYS_RUNCL: u64   = 0x1050;
pub const SYS_UI_OPEN: u64 = 0x1060;

pub const UI_FB_VA: u64   = 0x5000_0000;
pub const UI_FONT_VA: u64 = 0x6000_0000;
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
        SYS_READDIR => {
            let idx = a0 as usize;
            let cap = (a2 as usize).min(127);

            match crate::fs::vfs::root().and_then(|fs| fs.list_path("/")) {
                Some(v) if idx < v.len() => {
                    let e = &v[idx];
                    let bytes = e.name.as_bytes();
                    let n = bytes.len().min(cap);

                    if !user_copy_out(cr3, a1, &bytes[..n]) {
                        return c.rax = u64::MAX;
                    }

                    let _ = user_copy_out(cr3, a1 + n as u64, &[0u8]);

                    r.arg1 = e.size;

                    if e.is_dir { 2 } else { 1 }
                }
                _ => 0,
            }
        }

        SYS_RUNCL => {
            let mut tmp = [0u8; 128];

            match user_cstr(cr3, a0, &mut tmp) {
                Some(path) => super::runcl::run(path) as u64,
                None => u64::MAX,
            }
        }

        SYS_UI_OPEN => {
            extern "C" {
                fn hdmi_caps_raw(w: *mut u32, h: *mut u32,
                                 s: *mut u32, phys: *mut u64);
                fn kvirt_to_phys(p: *const u8) -> u64;
                fn paging_map_page_in(pml4: u64, virt: u64,
                                      phys: u64, flags: u64) -> bool;
            }

            extern "C" {
                static font8x8: u8;
            }

            unsafe {
                let (mut w, mut h, mut s, mut fp) = (0u32, 0u32, 0u32, 0u64);

                hdmi_caps_raw(&mut w, &mut h, &mut s, &mut fp);

                if fp == 0 || w == 0 {
                    return c.rax = u64::MAX;
                }

                /* fb: RW + user + NX (W^X) */
                let fb_bytes = (s * h * 4) as u64;
                let pages = (fb_bytes + 4095) / 4096;

                for i in 0..pages {
                    paging_map_page_in(cr3,
                                       UI_FB_VA + i * 4096,
                                       fp + i * 4096,
                                       0x1 | 0x2 | 0x4 | 0x8);
                }

                /* font: RO + user + NX */
                let font_phys = kvirt_to_phys(&font8x8);

                paging_map_page_in(cr3, UI_FONT_VA, font_phys,
                                   0x1 | 0x4 | 0x8);

                r.arg0 = ((w as u64) << 16) | h as u64;
                r.arg1 = s as u64;
                r.arg2 = UI_FB_VA;

                0
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