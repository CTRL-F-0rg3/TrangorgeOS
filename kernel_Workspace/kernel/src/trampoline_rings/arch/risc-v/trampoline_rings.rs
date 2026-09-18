use core::arch::asm;

pub const HYPER_YIELD: u64 = 1;
pub const HYPER_LOG: u64 = 2;
pub const HYPER_TICK: u64 = 3;

pub const RING_KERNEL: u8 = 0;
pub const RING_DRIVER: u8 = 1;
pub const RING_USER: u8 = 3;


pub const SSTATUS_WORLD: u64 = 1 << 5;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CpuCtx {
    pub x: [u64; 32],
    pub sp: u64,
    pub epc: u64,
    pub sstatus: u64,
    pub pad: u64,
}

extern "C" {
    fn tr_init(kernel_stack_top: u64);
    pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
}

pub struct World {
    pub ring: u8,
    pub satp: u64,
    pub ctx: CpuCtx,
    pub alive: bool,
}

const MAX_WORLDS: usize = 4;

static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
static mut CURRENT: Option<usize> = None;
static mut TICK: u64 = 0;

static mut KSTACK: [u64; 4096] = [0; 4096];

pub fn init() {
    unsafe {
        let ktop = KSTACK.as_ptr() as u64 + 4096 * 8;
        tr_init(ktop);
    }
}

pub fn add_world(ring: u8, satp: u64, entry: u64,
                 stack: u64, arg: u64) -> Option<usize> {
    unsafe {
        for i in 0..MAX_WORLDS {
            if WORLDS[i].is_none() {
                let mut ctx = CpuCtx::default();

                ctx.x[10] = arg;          
                ctx.epc = entry;
                ctx.sp = stack;
                ctx.sstatus = SSTATUS_WORLD;

                WORLDS[i] = Some(World { ring, satp, ctx, alive: true });

                return Some(i);
            }
        }
    }

    None
}

fn write_satp(v: u64) {
    unsafe {
        asm!("csrw satp, {}", "sfence.vma", in(reg) v);
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

        write_satp(w.satp);
        tr_restore_ctx(&mut w.ctx);
    }
}

extern "C" fn tr_hyper(ctx: *mut CpuCtx) {
    unsafe {
        let c = &mut *ctx;
        let cur = CURRENT.unwrap();

        match c.x[10] {
            HYPER_YIELD => {
                crate::driverspaceinit::init::service::poll();

                TICK += 1;

                WORLDS[cur].as_mut().unwrap().ctx = *c;

                let next = pick_next(Some(cur));
                CURRENT = Some(next);

                let w = WORLDS[next].as_mut().unwrap();

                write_satp(w.satp);
                tr_restore_ctx(&mut w.ctx);
            }

            HYPER_LOG => {
                extern "C" { fn kprintf(fmt: *const u8, ...); }
                kprintf(b"[u-world ring%d] %s\n\0".as_ptr(),
                        WORLDS[cur].as_ref().unwrap().ring as u32,
                        c.x[11] as *const u8);
            }

            HYPER_TICK => {
                c.x[10] = TICK;
            }

            _ => {
                c.x[10] = u64::MAX;
            }
        }
    }
}

