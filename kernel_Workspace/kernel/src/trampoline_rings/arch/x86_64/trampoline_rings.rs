use core::arch::asm;

pub const SEL_KCODE: u16 = 0x08;
pub const SEL_KDATA: u16 = 0x10;
pub const SEL_R1CODE: u16 = 0x18 | 1;
pub const SEL_R1DATA: u16 = 0x20 | 1;
pub const SEL_R3CODE: u16 = 0x28 | 3;
pub const SEL_R3DATA: u16 = 0x30 | 3;

pub const HYPER_YIELD: u64 = 1;
pub const HYPER_LOG: u64 = 2;
pub const HYPER_TICK: u64 = 3;

pub const RING_KERNEL: u8 = 0;
pub const RING_DRIVER: u8 = 1;
pub const RING_USER: u8 = 3;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CpuCtx {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9: u64, pub r8: u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    pub rip: u64, pub cs: u64, pub rflags: u64, pub rsp: u64, pub ss: u64,
}

extern "C" {
    fn tr_init(rsp0: u64);
    pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
    static mut tr_tss_desc: [u64; 2];
    static mut isr_hypercall: u8;
    static mut isr_default: u8;
}

#[repr(C, packed)]
struct IdtGate {
    low: u16,
    sel: u16,
    ist: u8,
    type_attr: u8,
    mid: u16,
    high: u32,
    reserved: u32,
}

static mut IDT: [IdtGate; 256] = [IdtGate {
    low: 0, sel: 0, ist: 0, type_attr: 0, mid: 0, high: 0, reserved: 0,
}; 256];

fn set_gate(i: usize, handler: u64, dpl: u8) {
    unsafe {
        IDT[i] = IdtGate {
            low: handler as u16,
            sel: SEL_KCODE,
            ist: 0,
            type_attr: 0x8E | (dpl << 5),
            mid: (handler >> 16) as u16,
            high: (handler >> 32) as u32,
            reserved: 0,
        };
    }
}

pub struct World {
    pub ring: u8,
    pub cr3: u64,
    pub ctx: CpuCtx,
    pub alive: bool,
}

const MAX_WORLDS: usize = 4;

static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
static mut CURRENT: Option<usize> = None;
static mut TICK: u64 = 0;

static mut KSTACK: [u64; 4096] = [0; 4096];

fn sel_for(ring: u8) -> (u64, u64) {
    match ring {
        1 => (SEL_R1CODE as u64, SEL_R1DATA as u64),
        _ => (SEL_R3CODE as u64, SEL_R3DATA as u64),
    }
}

pub fn init() {
    unsafe {
        let tss = &mut tr_tss_desc;
        let base = &tr_tss as *const _ as u64;

        let mut d0: u64 = 0;
        d0 |= (base & 0xFFFF) << 16;
        d0 |= ((base >> 16) & 0xFF) << 32;
        d0 |= 0x89 << 40;
        d0 |= ((base >> 24) & 0xFF) << 56;

        let mut d1: u64 = (base >> 32) & 0xFF_FFFF;
        d1 |= 103 << 32;

        tss[0] = d0;
        tss[1] = d1;

        let ktop = KSTACK.as_ptr() as u64 + 4096 * 8;

        tr_init(ktop);

        let def = &isr_default as *const _ as u64;
        let hyp = &isr_hypercall as *const _ as u64;

        for i in 0..256 {
            set_gate(i, def, 0);
        }

        set_gate(0x80, hyp, 3);

        asm!("lidt {}", in(reg) &IDT_PTR);
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

pub fn add_world(ring: u8, cr3: u64, entry: u64,
                 stack: u64, arg: u64) -> Option<usize> {
    unsafe {
        for i in 0..MAX_WORLDS {
            if WORLDS[i].is_none() {
                let (cs, ss) = sel_for(ring);

                let mut ctx = CpuCtx::default();
                ctx.rip = entry;
                ctx.cs = cs;
                ctx.rflags = 0x202;
                ctx.rsp = stack;
                ctx.ss = ss;
                ctx.rdi = arg;

                WORLDS[i] = Some(World { ring, cr3, ctx, alive: true });

                return Some(i);
            }
        }
    }

    None
}

fn write_cr3(v: u64) {
    unsafe { asm!("mov {}, cr3", in(reg) v) };
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

        write_cr3(w.cr3);
        tr_restore_ctx(&mut w.ctx);
    }
}

extern "C" fn tr_hyper(ctx: *mut CpuCtx) {
    unsafe {
        let c = &mut *ctx;
        let cur = CURRENT.unwrap();

        match c.rax {
            HYPER_YIELD => {
                crate::driverspaceinit::init::service::poll();

                TICK += 1;

                *WORLDS[cur].as_mut().unwrap().ctx_mut() = *c;

                let next = pick_next(Some(cur));
                CURRENT = Some(next);

                let w = WORLDS[next].as_mut().unwrap();

                write_cr3(w.cr3);
                tr_restore_ctx(&mut w.ctx);
            }

            HYPER_LOG => {
                extern "C" { fn kprintf(fmt: *const u8, ...); }
                kprintf(b"[ring%d] %s\n\0".as_ptr(),
                        WORLDS[cur].as_ref().unwrap().ring as u32,
                        c.rdi as *const u8);
            }

            HYPER_TICK => {
                c.rax = TICK;
            }

            _ => {
                c.rax = u64::MAX;
            }
        }
    }
}

impl World {
    fn ctx_mut(&mut self) -> &mut CpuCtx {
        &mut self.ctx
    }
}

// // yield = oddaj kwant
// unsafe { asm!("int 0x80", in("rax") HYPER_YIELD); }

// // log
// unsafe { asm!("int 0x80", in("rax") HYPER_LOG, in("rdi") msg.as_ptr()); }

// __asm__ volatile("int $0x80" :: "a"(1));

// trampoline_rings::init();

// trampoline_rings::add_world(
//     RING_DRIVER,
//     ds_app_cr3,          // aspace driverspace
//     DS_CODE_VA,          // entry
//     DS_STACK_TOP,
//     DS_INIT_PARAMS_VA,
// );

// trampoline_rings::add_world(
//     RING_USER,
//     user_cr3,            // aspace userspace
//     USER_ENTRY,
//     USER_STACK_TOP,
//     0,
// );

// trampoline_rings::start();   // kernel od teraz żyje w tr_hyper