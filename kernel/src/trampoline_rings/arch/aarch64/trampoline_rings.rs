use core::arch::asm;

pub const HYPER_YIELD: u64 = 1;
pub const HYPER_LOG: u64 = 2;
pub const HYPER_TICK: u64 = 3;

pub const RING_KERNEL: u8 = 0;
pub const RING_DRIVER: u8 = 1;
pub const RING_USER: u8 = 3;

pub const SPSR_EL0T: u64 = 0x0;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CpuCtx {
    pub x: [u64; 31],
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
}

extern "C" {
    fn tr_init();
    pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
}

pub struct World {
    pub ring: u8,
    pub ttbr0: u64,
    pub ctx: CpuCtx,
    pub alive: bool,
}

const MAX_WORLDS: usize = 4;

static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
static mut CURRENT: Option<usize> = None;
static mut TICK: u64 = 0;

pub fn init() {
    unsafe {
        tr_init();
    }
}

pub fn add_world(ring: u8, ttbr0: u64, entry: u64,
                 stack: u64, arg: u64) -> Option<usize> {
    unsafe {
        for i in 0..MAX_WORLDS {
            if WORLDS[i].is_none() {
                let mut ctx = CpuCtx::default();

                ctx.x[0] = arg;
                ctx.elr = entry;
                ctx.sp = stack;
                ctx.spsr = SPSR_EL0T;

                WORLDS[i] = Some(World { ring, ttbr0, ctx, alive: true });

                return Some(i);
            }
        }
    }

    None
}

fn write_ttbr0(v: u64) {
    unsafe {
        asm!("msr ttbr0_el1, {}", "isb", in(reg) v);
    }
}

fn pick_next(from: Option<usize>) -> usize {
    unsafe {
        let start = from.map(|f| f + 1).unwrap_or(0) % MAX_WORLDS;

        for i in 0..MAX_WORLDS {
            let idx = (start + i) % MAX_WORLDS;

            if WORLDS[idx].as_ref().map(|w| w.alive).unwrap_or(false) {
                return idx;
            }
        }
    }

    0
}

pub fn start() -> ! {
    unsafe {
        let next = pick_next(None);
        CURRENT = Some(next);

        let w = WORLDS[next].as_mut().unwrap();

        write_ttbr0(w.ttbr0);
        tr_restore_ctx(&mut w.ctx);
    }
}

extern "C" fn tr_hyper(ctx: *mut CpuCtx) {
    unsafe {
        let c = &mut *ctx;
        let cur = CURRENT.unwrap();

        match c.x[0] {
            HYPER_YIELD => {
                crate::driverspaceinit::init::service::poll();

                TICK += 1;

                WORLDS[cur].as_mut().unwrap().ctx = *c;

                let next = pick_next(Some(cur));
                CURRENT = Some(next);

                let w = WORLDS[next].as_mut().unwrap();

                write_ttbr0(w.ttbr0);
                tr_restore_ctx(&mut w.ctx);
            }

            HYPER_LOG => {
                extern "C" { fn kprintf(fmt: *const u8, ...); }
                kprintf(b"[el0 world ring%d] %s\n\0".as_ptr(),
                        WORLDS[cur].as_ref().unwrap().ring as u32,
                        c.x[1] as *const u8);
            }

            HYPER_TICK => {
                c.x[0] = TICK;
            }

            _ => {
                c.x[0] = u64::MAX;
            }
        }
    }
}

// unsafe { core::arch::asm!("svc #0", in("x0") 1u64); }          // yield
// unsafe { core::arch::asm!("svc #0", in("x0") 2u64, in("x1") msg.as_ptr()); } // log

// register long x0 __asm__("x0") = 1;
// __asm__ volatile("svc #0" : "+r"(x0));

// foreign arm {
// 	@(link_name="ds_hyper") hyper :: proc(call: u64, arg: u64) ---
// }

// trampoline_rings::init();

// trampoline_rings::add_world(
//     RING_DRIVER,
//     ds_ttbr0,          // dolna połówka aspace driverspace
//     DS_CODE_VA,
//     DS_STACK_TOP,
//     DS_INIT_PARAMS_VA,
// );

// trampoline_rings::add_world(
//     RING_USER,
//     user_ttbr0,
//     USER_ENTRY,
//     USER_STACK_TOP,
//     0,
// );

// trampoline_rings::start();