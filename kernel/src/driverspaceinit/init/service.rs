use super::super::abi::abi::*;
use super::super::abi::src::RingView;
use super::init;
use super::initabi::*;
use crate::fs::driver::registry;
use crate::mm::phys;
use crate::mm::space;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

const GRANT_RAM: u8 = 1;
const GRANT_MMIO: u8 = 2;

#[derive(Clone, Copy)]
struct Grant {
    va: u64,
    phys: u64,
    pages: u64,
    kind: u8,
    used: bool,
}

static mut GRANTS: [Grant; 32] = [Grant {
    va: 0, phys: 0, pages: 0, kind: 0, used: false,
}; 32];

static mut NEXT_VA: u64 = 0x4100_0000;

static mut AC97_BARS: (u64, u64) = (0, 0);

fn ac97_bars() -> (u64, u64) {
    unsafe {
        if AC97_BARS.0 != 0 {
            return AC97_BARS;
        }

        if let Some(d) = crate::drivers::pci::find_class(0x04, 0x01, 0x00) {
            d.enable_mmio();
            AC97_BARS = (d.bar(2), d.bar(3));
        }

        AC97_BARS
    }
}

fn grant_add(va: u64, phys: u64, pages: u64, kind: u8) -> bool {
    unsafe {
        for g in GRANTS.iter_mut() {
            if !g.used {
                *g = Grant { va, phys, pages, kind, used: true };
                return true;
            }
        }
    }

    false
}

fn grant_take(va: u64) -> Option<Grant> {
    unsafe {
        for g in GRANTS.iter_mut() {
            if g.used && g.va == va {
                g.used = false;
                return Some(*g);
            }
        }
    }

    None
}

fn grant_phys(va: u64) -> Option<(u64, u64)> {
    unsafe {
        for g in GRANTS.iter() {
            if g.used && va >= g.va && va < g.va + g.pages * 4096 {
                return Some((g.phys + (va - g.va), g.pages));
            }
        }
    }

    None
}

pub fn video_call(op: u32, m: &DsMsg, r: &mut DsMsg) -> i32 {
    match op {
        VID_FB_INFO => {
            let (w, h, s, phys) = crate::gfx::console::fb_info();

            r.arg0 = ((w as u64) << 16) | h as u64;
            r.arg1 = s as u64;
            r.arg2 = phys;
            0
        }
        VID_FB_TAKEOVER => {
            crate::gfx::console::set_enabled(false);
            0
        }
        VID_FB_RELEASE => {
            crate::gfx::console::set_enabled(true);
            crate::gfx::console::refresh();
            0
        }
        _ => -1,
    }
}

fn handle(m: &DsMsg, r: &mut DsMsg) -> i32 {
    let class = m.cmd >> 8;
    let op = m.cmd & 0xFF;

    match class {
        SVC_VIDEO => return video_call(op, &m, r),
        SVC_INPUT => return input_call(op, &m, r),
        SVC_AUDIO => return audio_call(op, &m, r),
        SVC_BLOCK => return block_call(op, &m, r),
        _ => {}
    }

    match m.cmd {
        x if x == DsCmd::Log as u32 => {
            let scratch = match init::scratch_view() {
                Some(s) => s,
                None => return -1,
            };

            let len = (m.arg0 as usize).min(DS_SCRATCH_SIZE - 1);

            unsafe {
                let mut tmp = [0u8; DS_SCRATCH_SIZE];
                core::ptr::copy_nonoverlapping(scratch, tmp.as_mut_ptr(), len);
                kprintf(b"[driverspace] %s\n\0".as_ptr(), tmp.as_ptr());
            }

            0
        }

        x if x == DsCmd::AllocPages as u32 => {
            let pages = m.arg0.max(1);

            let phys = match phys::alloc_frames(pages as usize) {
                Some(p) => p,
                None => return -1,
            };

            let va = unsafe {
                let v = NEXT_VA;
                NEXT_VA += pages * 4096;
                v
            };

            let prot = space::PROT_READ | space::PROT_WRITE | space::PROT_USER;

            if !init::map_into_ds(va, phys, pages as usize * 4096, prot) {
                return -1;
            }

            grant_add(va, phys, pages, GRANT_RAM);

            r.arg0 = va;
            0
        }

        x if x == DsCmd::FreePages as u32 => {
            match grant_take(m.arg0) {
                Some(g) if g.kind == GRANT_RAM => {
                    init::unmap_from_ds(g.va, g.pages as usize * 4096);
                    phys::free_frames(g.phys, g.pages as usize);
                    0
                }
                _ => -1,
            }
        }

        x if x == DsCmd::JackQuery as u32 => {
            r.arg0 = crate::audio::jack::query();
            0
        }

        x if x == DsCmd::JackSetAmp as u32 => {
            crate::audio::jack::set_amp(m.arg0 != 0);
            0
        }

        x if x == DsCmd::AudioPlay as u32 => {
            match grant_phys(m.arg0) {
                Some((phys, _)) => {
                    if crate::audio::jack::play_phys(phys, m.arg1 as u32) { 0 } else { -1 }
                }
                None => -1,
            }
        }

        x if x == DsCmd::AudioStop as u32 => {
            crate::audio::jack::stop();
            0
        }

        x if x == DsCmd::MapMmio as u32 => {
            let phys = m.arg0;
            let len = m.arg1;
            let va = m.arg2;

            let prot = space::PROT_READ | space::PROT_WRITE
                     | space::PROT_USER | space::PROT_DEVICE;

            if !init::map_into_ds(va, phys, len as usize, prot) {
                return -1;
            }

            grant_add(va, phys, (len + 4095) / 4096, GRANT_MMIO);
            0
        }

        x if x == DsCmd::GetDeviceCount as u32 => {
            r.arg0 = registry::count() as u64;
            0
        }

        x if x == DsCmd::BlockRead as u32 => {
            let disk = match registry::get(m.arg0 as usize) {
                Some(d) => d,
                None => return -1,
            };

            let scratch = match init::scratch_view() {
                Some(s) => s,
                None => return -1,
            };

            let mut buf = [0u8; 512];

            if disk.read_block(m.arg1, &mut buf).is_err() {
                return -1;
            }

            unsafe {
                core::ptr::copy_nonoverlapping(buf.as_ptr(), scratch, 512);
            }

            0
        }

        x if x == DsCmd::BlockWrite as u32 => {
            let disk = match registry::get(m.arg0 as usize) {
                Some(d) => d,
                None => return -1,
            };

            let scratch = match init::scratch_view() {
                Some(s) => s,
                None => return -1,
            };

            let mut buf = [0u8; 512];

            unsafe {
                core::ptr::copy_nonoverlapping(scratch, buf.as_mut_ptr(), 512);
            }

            match disk.write_block(m.arg1, &buf) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }

        x if x == DsCmd::AudioInfo as u32 => {
            let (nam, bm) = ac97_bars();
            r.arg0 = nam;
            r.arg1 = bm;
            if nam == 0 { -1 } else { 0 }
        }

        x if x == DsCmd::PagePhys as u32 => {
            match grant_phys(m.arg0) {
                Some((p, _)) => { r.arg0 = p; 0 }
                None => -1,
            }
        }

        _ => -1,
    }
}

pub fn poll() {
    let d2k = match init::d2k_view() {
        Some(v) => v,
        None => return,
    };

    let k2d = match init::k2d_view() {
        Some(v) => v,
        None => return,
    };

    while let Some(m) = d2k.pop() {
        let mut r = DsMsg {
            id: m.id,
            cmd: m.cmd,
            flags: 0,
            arg0: 0,
            arg1: 0,
            arg2: 0,
            status: 0,
            pad: 0,
        };

        r.status = handle(&m, &mut r);

        k2d.push(&r);
    }
}

pub fn post_event(cmd: DsCmd, a0: u64, a1: u64) {
    if let Some(k2d) = init::k2d_view() {
        let _ = k2d.push(&DsMsg {
            id: u64::MAX,
            cmd: cmd as u32,
            flags: 0,
            arg0: a0,
            arg1: a1,
            arg2: 0,
            status: 0,
            pad: 0,
        });
    }
}