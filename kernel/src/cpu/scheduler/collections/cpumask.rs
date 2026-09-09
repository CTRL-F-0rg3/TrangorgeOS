#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

pub mod cpumask {
    use core::sync::atomic::{AtomicU64, Ordering};

    pub const MAX_CPUS: usize = 1024;
    pub const WORDS: usize = MAX_CPUS / 64;

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CpuMask {
        bits: [u64; WORDS],
    }

    impl CpuMask {
        #[inline]
        pub const fn empty() -> Self {
            Self { bits: [0; WORDS] }
        }

        #[inline]
        pub const fn full() -> Self {
            Self { bits: [u64::MAX; WORDS] }
        }

        #[inline]
        pub const fn single(cpu: u32) -> Self {
            let mut bits = [0u64; WORDS];
            if (cpu as usize) < MAX_CPUS {
                bits[cpu as usize / 64] = 1u64 << (cpu as usize % 64);
            }
            Self { bits }
        }

        #[inline]
        pub fn set(&mut self, cpu: u32) {
            let idx = cpu as usize;
            if idx < MAX_CPUS {
                self.bits[idx / 64] |= 1u64 << (idx % 64);
            }
        }

        #[inline]
        pub fn clear(&mut self, cpu: u32) {
            let idx = cpu as usize;
            if idx < MAX_CPUS {
                self.bits[idx / 64] &= !(1u64 << (idx % 64));
            }
        }

        #[inline]
        pub fn test(&self, cpu: u32) -> bool {
            let idx = cpu as usize;
            idx < MAX_CPUS && (self.bits[idx / 64] & (1u64 << (idx % 64))) != 0
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.bits.iter().all(|&w| w == 0)
        }

        #[inline]
        pub fn weight(&self) -> u32 {
            self.bits.iter().map(|w| w.count_ones()).sum()
        }

        #[inline]
        pub fn intersects(&self, other: &Self) -> bool {
            self.bits.iter().zip(other.bits.iter()).any(|(a, b)| (a & b) != 0)
        }

        #[inline]
        pub fn is_subset(&self, other: &Self) -> bool {
            self.bits.iter().zip(other.bits.iter()).all(|(a, b)| (a & !b) == 0)
        }

        #[inline]
        pub fn andnot(&self, other: &Self) -> Self {
            let mut bits = [0u64; WORDS];
            for i in 0..WORDS {
                bits[i] = self.bits[i] & !other.bits[i];
            }
            Self { bits }
        }

        #[inline]
        pub fn first(&self) -> Option<u32> {
            for (i, &w) in self.bits.iter().enumerate() {
                if w != 0 {
                    return Some((i * 64 + w.trailing_zeros() as usize) as u32);
                }
            }
            None
        }

        #[inline]
        pub fn next_after(&self, cpu: u32) -> Option<u32> {
            let start = cpu.wrapping_add(1) as usize;
            for i in (start / 64)..WORDS {
                let w = self.bits[i];
                if w == 0 { continue; }
                let offset = if i * 64 == start { start % 64 } else { 0 };
                let masked = w & (!0u64 << offset);
                if masked != 0 {
                    return Some((i * 64 + masked.trailing_zeros() as usize) as u32);
                }
            }
            for i in 0..=(cpu as usize / 64) {
                let w = self.bits[i];
                if w == 0 { continue; }
                let masked = if i == (cpu as usize / 64) {
                    w & (!0u64 << ((cpu as usize % 64) + 1))
                } else {
                    w
                };
                if masked != 0 {
                    return Some((i * 64 + masked.trailing_zeros() as usize) as u32);
                }
            }
            None
        }

        #[inline]
        pub fn iter(&self) -> CpuMaskIter {
            CpuMaskIter { mask: *self, word_idx: 0, current_word: self.bits[0] }
        }
    }

    impl core::ops::BitAnd for CpuMask {
        type Output = Self;
        #[inline]
        fn bitand(self, rhs: Self) -> Self {
            let mut bits = [0u64; WORDS];
            for i in 0..WORDS { bits[i] = self.bits[i] & rhs.bits[i]; }
            Self { bits }
        }
    }

    impl core::ops::BitOr for CpuMask {
        type Output = Self;
        #[inline]
        fn bitor(self, rhs: Self) -> Self {
            let mut bits = [0u64; WORDS];
            for i in 0..WORDS { bits[i] = self.bits[i] | rhs.bits[i]; }
            Self { bits }
        }
    }

    impl core::ops::Not for CpuMask {
        type Output = Self;
        #[inline]
        fn not(self) -> Self {
            let mut bits = [0u64; WORDS];
            for i in 0..WORDS { bits[i] = !self.bits[i]; }
            Self { bits }
        }
    }

    pub struct CpuMaskIter {
        mask: CpuMask,
        word_idx: usize,
        current_word: u64,
    }

    impl Iterator for CpuMaskIter {
        type Item = u32;
        #[inline]
        fn next(&mut self) -> Option<u32> {
            while self.word_idx < WORDS {
                if self.current_word != 0 {
                    let bit = self.current_word.trailing_zeros() as usize;
                    self.current_word &= self.current_word - 1;
                    return Some((self.word_idx * 64 + bit) as u32);
                }
                self.word_idx += 1;
                if self.word_idx < WORDS {
                    self.current_word = self.mask.bits[self.word_idx];
                }
            }
            None
        }
    }

    #[repr(C)]
    pub struct AtomicCpuMask {
        bits: [AtomicU64; WORDS],
    }

    impl AtomicCpuMask {
        #[inline]
        pub const fn new() -> Self {
            const INIT: AtomicU64 = AtomicU64::new(0);
            Self { bits: [INIT; WORDS] }
        }

        #[inline]
        pub fn set(&self, cpu: u32) {
            let idx = cpu as usize;
            if idx < MAX_CPUS {
                self.bits[idx / 64].fetch_or(1u64 << (idx % 64), Ordering::Relaxed);
            }
        }

        #[inline]
        pub fn clear(&self, cpu: u32) {
            let idx = cpu as usize;
            if idx < MAX_CPUS {
                self.bits[idx / 64].fetch_and(!(1u64 << (idx % 64)), Ordering::Relaxed);
            }
        }

        #[inline]
        pub fn test(&self, cpu: u32) -> bool {
            let idx = cpu as usize;
            idx < MAX_CPUS && (self.bits[idx / 64].load(Ordering::Acquire) & (1u64 << (idx % 64))) != 0
        }

        #[inline]
        pub fn snapshot(&self) -> CpuMask {
            let mut bits = [0u64; WORDS];
            for (i, w) in self.bits.iter().enumerate() {
                bits[i] = w.load(Ordering::Relaxed);
            }
            CpuMask { bits }
        }
    }

    static POSSIBLE: AtomicCpuMask = AtomicCpuMask::new();
    static PRESENT: AtomicCpuMask = AtomicCpuMask::new();
    static ONLINE: AtomicCpuMask = AtomicCpuMask::new();
    static ACTIVE: AtomicCpuMask = AtomicCpuMask::new();

    const NO_TOPO: u32 = u32::MAX;
    static PACKAGE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] = [const { core::sync::atomic::AtomicU32::new(NO_TOPO) }; MAX_CPUS];
    static CORE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] = [const { core::sync::atomic::AtomicU32::new(NO_TOPO) }; MAX_CPUS];

    #[inline] pub fn mark_possible(cpu: u32) { POSSIBLE.set(cpu); }
    #[inline] pub fn mark_present(cpu: u32) { PRESENT.set(cpu); }
    #[inline] pub fn mark_online(cpu: u32) { ONLINE.set(cpu); ACTIVE.set(cpu); }
    #[inline] pub fn mark_offline(cpu: u32) { ACTIVE.clear(cpu); ONLINE.clear(cpu); }
    #[inline] pub fn is_online(cpu: u32) -> bool { ONLINE.test(cpu) }
    #[inline] pub fn online_mask() -> CpuMask { ONLINE.snapshot() }
    #[inline] pub fn active_mask() -> CpuMask { ACTIVE.snapshot() }

    #[inline]
    pub fn set_topology(cpu: u32, pkg: u32, core: u32) {
        if (cpu as usize) < MAX_CPUS {
            PACKAGE_ID[cpu as usize].store(pkg, Ordering::Release);
            CORE_ID[cpu as usize].store(core, Ordering::Release);
        }
    }

    #[inline]
    pub fn sibling_mask(cpu: u32) -> CpuMask {
        let mut out = CpuMask::empty();
        let pkg = PACKAGE_ID[cpu as usize].load(Ordering::Acquire);
        let core = CORE_ID[cpu as usize].load(Ordering::Acquire);
        if pkg == NO_TOPO || core == NO_TOPO { return out; }
        
        for c in online_mask().iter() {
            let c_idx = c as usize;
            if PACKAGE_ID[c_idx].load(Ordering::Acquire) == pkg && CORE_ID[c_idx].load(Ordering::Acquire) == core {
                out.set(c);
            }
        }
        out
    }
}

pub mod task {
    use super::cpumask::{CpuMask, MAX_CPUS};
    use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicU64, AtomicU8, Ordering};
    use core::ptr;

    pub type TaskId = u64;
    pub const KERNEL_STACK_SIZE: usize = 16 * 1024;
    pub const MAX_RT_PRIO: i32 = 100;
    pub const NICE_0_LOAD: u64 = 1024;
    pub const CPU_NONE: u32 = u32::MAX;

    pub const RB_RED: usize = 0;
    pub const RB_BLACK: usize = 1;

    #[inline]
    pub fn rb_parent(pc: usize) -> *mut TaskStruct { (pc & !1) as *mut TaskStruct }
    #[inline]
    pub fn rb_color(pc: usize) -> usize { pc & 1 }
    #[inline]
    pub fn rb_make_parent_color(p: *mut TaskStruct, c: usize) -> usize { (p as usize & !1) | (c & 1) }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TaskState {
        Runnable = 0,
        Interruptible = 1,
        Uninterruptible = 2,
        Stopped = 3,
        Zombie = 5,
        Dead = 6,
        Idle = 7,
    }

    impl TaskState {
        #[inline]
        pub const fn is_runnable(self) -> bool { matches!(self, Self::Runnable | Self::Idle) }
        #[inline]
        pub const fn is_terminal(self) -> bool { matches!(self, Self::Zombie | Self::Dead) }
        
        #[inline]
        pub const fn can_transition_to(self, next: Self) -> bool {
            matches!(
                (self, next),
                (Self::Runnable, Self::Interruptible) |
                (Self::Runnable, Self::Uninterruptible) |
                (Self::Runnable, Self::Stopped) |
                (Self::Runnable, Self::Zombie) |
                (Self::Interruptible, Self::Runnable) |
                (Self::Uninterruptible, Self::Runnable) |
                (Self::Stopped, Self::Runnable) |
                (Self::Zombie, Self::Dead) |
                (Self::Idle, Self::Idle) |
                (s, n) if s == n
            )
        }

        #[inline]
        const fn from_u8(v: u8) -> Self {
            match v {
                0 => Self::Runnable, 1 => Self::Interruptible, 2 => Self::Uninterruptible,
                3 => Self::Stopped, 5 => Self::Zombie, 6 => Self::Dead, 7 => Self::Idle,
                _ => Self::Dead,
            }
        }
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SchedPolicy { Normal, Batch, Idle, Fifo, RoundRobin, Deadline, Stop }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum SchedClass { Idle = 0, Fair = 1, RealTime = 2, Deadline = 3, Stop = 4 }

    impl From<SchedPolicy> for SchedClass {
        #[inline]
        fn from(p: SchedPolicy) -> Self {
            match p {
                SchedPolicy::Idle => Self::Idle,
                SchedPolicy::Normal | SchedPolicy::Batch => Self::Fair,
                SchedPolicy::Fifo | SchedPolicy::RoundRobin => Self::RealTime,
                SchedPolicy::Deadline => Self::Deadline,
                SchedPolicy::Stop => Self::Stop,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct LoadAvg {
        pub last_update: u64,
        pub load_sum: u64,
        pub load_avg: u32,
        pub util_sum: u64,
        pub util_avg: u32,
    }

    impl LoadAvg {
        #[inline]
        pub fn accumulate(&mut self, now: u64, delta_ns: u64, weight: u64, running: bool) {
            if delta_ns == 0 { self.last_update = now; return; }
            let decay = self.load_avg >> 10;
            self.load_avg = self.load_avg.saturating_sub(decay);
            let contrib = (core::cmp::min(delta_ns, 1_000_000_000) as u128 * weight as u128 / 1_000_000_000) as u32;
            self.load_avg = self.load_avg.saturating_add(contrib);
            self.load_sum = self.load_sum.saturating_add(delta_ns);
            if running {
                self.util_avg = self.util_avg.saturating_sub(self.util_avg >> 10);
                self.util_avg = self.util_avg.saturating_add((delta_ns / 1_000_000) as u32);
                self.util_sum = self.util_sum.saturating_add(delta_ns);
            }
            self.last_update = now;
        }
    }

    #[repr(C)]
    pub struct SchedEntity {
        pub rb_left: *mut TaskStruct,
        pub rb_right: *mut TaskStruct,
        pub rb_parent_color: usize,
        pub vruntime: u64,
        pub sum_exec_runtime: u64,
        pub prev_sum_exec_runtime: u64,
        pub weight: u64,
        pub inv_weight: u32,
        pub load: LoadAvg,
        pub on_rq: bool,
        pub last_cpu: u32,
        pub cpus_allowed: CpuMask,
        pub nr_migrations: u32,
    }

    impl Default for SchedEntity {
        fn default() -> Self {
            Self {
                rb_left: ptr::null_mut(), rb_right: ptr::null_mut(), rb_parent_color: RB_RED,
                vruntime: 0, sum_exec_runtime: 0, prev_sum_exec_runtime: 0,
                weight: NICE_0_LOAD, inv_weight: 4194304, load: LoadAvg::default(),
                on_rq: false, last_cpu: CPU_NONE, cpus_allowed: CpuMask::full(), nr_migrations: 0,
            }
        }
    }

    #[repr(C)]
    pub struct RtEntity {
        pub list_prev: *mut TaskStruct,
        pub list_next: *mut TaskStruct,
        pub time_slice: u32,
        pub queued_prio: u32,
    }

    impl Default for RtEntity {
        fn default() -> Self {
            Self { list_prev: ptr::null_mut(), list_next: ptr::null_mut(), time_slice: 100, queued_prio: MAX_RT_PRIO as u32 }
        }
    }

    #[repr(C)]
    pub struct DlEntity {
        pub rb_left: *mut TaskStruct,
        pub rb_right: *mut TaskStruct,
        pub rb_parent_color: usize,
        pub dl_runtime: u64,
        pub dl_deadline: u64,
        pub dl_period: u64,
        pub runtime: i64,
        pub deadline: u64,
        pub throttled: bool,
        pub replenish_at: u64,
    }

    impl Default for DlEntity {
        fn default() -> Self {
            Self {
                rb_left: ptr::null_mut(), rb_right: ptr::null_mut(), rb_parent_color: RB_RED,
                dl_runtime: 0, dl_deadline: 0, dl_period: 0, runtime: 0, deadline: 0,
                throttled: false, replenish_at: 0,
            }
        }
    }

    #[repr(C, align(64))]
    pub struct TaskStruct {
        pub pid: TaskId,
        pub tgid: TaskId,
        pub state: AtomicU8,
        pub flags: AtomicU32,
        pub policy: SchedPolicy,
        pub sched_class: SchedClass,
        pub prio: i32,
        pub se: SchedEntity,
        pub rt: RtEntity,
        pub dl: DlEntity,
        pub on_cpu: AtomicBool,
        pub cpu: AtomicU32,
        pub rq: AtomicPtr<core::ffi::c_void>,
        pub stack_base: *mut u8,
        pub stack_size: usize,
        pub context: CpuContext,
        pub preempt_count: AtomicI32,
        pub stats: TaskStats,
        pub parent: *mut TaskStruct,
        pub children: *mut TaskStruct,
        pub sibling: *mut TaskStruct,
        pub comm: [u8; 16],
        pub task_lock: SpinLock,
    }

    #[repr(C)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct TaskStats {
        pub utime: u64,
        pub stime: u64,
        pub nr_voluntary: u64,
        pub nr_involuntary: u64,
        pub nr_migrations: u64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CpuContext {
        pub rsp: u64, pub rip: u64, pub rflags: u64,
        pub rbx: u64, pub rbp: u64, pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
        pub fs_base: u64, pub gs_base: u64,
        pub fpu: [u8; 512],
    }

    impl Default for CpuContext {
        fn default() -> Self {
            Self {
                rsp: 0, rip: 0, rflags: 0x202, rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
                fs_base: 0, gs_base: 0, fpu: [0; 512],
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Default)]
    pub struct SpinLock {
        locked: AtomicBool,
    }

    impl SpinLock {
        #[inline]
        pub const fn new() -> Self { Self { locked: AtomicBool::new(false) } }

        #[inline]
        pub fn lock(&self) {
            while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
                while self.locked.load(Ordering::Relaxed) { core::hint::spin_loop(); }
            }
        }

        #[inline]
        pub fn unlock(&self) { self.locked.store(false, Ordering::Release); }
        
        #[inline]
        pub fn is_locked(&self) -> bool { self.locked.load(Ordering::Relaxed) }
    }

    impl TaskStruct {
        #[inline]
        pub fn state(&self) -> TaskState { TaskState::from_u8(self.state.load(Ordering::Acquire)) }

        #[inline]
        pub fn set_state(&self, new: TaskState) -> bool {
            let mut current = self.state.load(Ordering::Acquire);
            loop {
                let curr_state = TaskState::from_u8(current);
                if !curr_state.can_transition_to(new) { return false; }
                match self.state.compare_exchange_weak(current, new as u8, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => return true,
                    Err(c) => current = c,
                }
            }
        }

        #[inline]
        pub fn set_need_resched(&self) { self.flags.fetch_or(1 << 17, Ordering::Relaxed); }
        
        #[inline]
        pub fn clear_need_resched(&self) { self.flags.fetch_and(!(1 << 17), Ordering::Relaxed); }

        #[inline]
        pub fn needs_resched(&self) -> bool { (self.flags.load(Ordering::Relaxed) & (1 << 17)) != 0 }

        #[inline]
        pub fn rq_ptr(&self) -> *mut core::ffi::c_void { self.rq.load(Ordering::Acquire) }

        #[inline]
        pub fn set_rq_ptr(&self, rq: *mut core::ffi::c_void) { self.rq.store(rq, Ordering::Release); }
    }

    unsafe impl Send for TaskStruct {}
    unsafe impl Sync for TaskStruct {}
}

pub mod runqueue {
    use super::task::*;
    use super::cpumask::*;
    use core::ptr;
    use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

    pub const MIN_GRANULARITY_NS: u64 = 750_000;
    pub const TARGET_LATENCY_NS: u64 = 6_000_000;

    mod rbtree {
        use super::*;

        #[inline] unsafe fn left(n: *mut TaskStruct) -> *mut TaskStruct { if n.is_null() { ptr::null_mut() } else { (*n).se.rb_left } }
        #[inline] unsafe fn right(n: *mut TaskStruct) -> *mut TaskStruct { if n.is_null() { ptr::null_mut() } else { (*n).se.rb_right } }
        #[inline] unsafe fn parent(n: *mut TaskStruct) -> *mut TaskStruct { if n.is_null() { ptr::null_mut() } else { rb_parent((*n).se.rb_parent_color) } }
        #[inline] unsafe fn color(n: *mut TaskStruct) -> usize { if n.is_null() { RB_BLACK } else { rb_color((*n).se.rb_parent_color) } }
        
        #[inline] unsafe fn set_left(n: *mut TaskStruct, v: *mut TaskStruct) { (*n).se.rb_left = v; }
        #[inline] unsafe fn set_right(n: *mut TaskStruct, v: *mut TaskStruct) { (*n).se.rb_right = v; }
        
        #[inline] unsafe fn set_parent_color(n: *mut TaskStruct, p: *mut TaskStruct, c: usize) {
            if !n.is_null() { (*n).se.rb_parent_color = rb_make_parent_color(p, c); }
        }

        unsafe fn rotate_left(root: &mut *mut TaskStruct, x: *mut TaskStruct) {
            let y = right(x);
            set_right(x, left(y));
            if !left(y).is_null() { set_parent_color(left(y), x, color(left(y))); }
            set_parent_color(y, parent(x), color(y));
            let xp = parent(x);
            if xp.is_null() { *root = y; } else if x == left(xp) { set_left(xp, y); } else { set_right(xp, y); }
            set_left(y, x);
            set_parent_color(x, y, color(x));
        }

        unsafe fn rotate_right(root: &mut *mut TaskStruct, x: *mut TaskStruct) {
            let y = left(x);
            set_left(x, right(y));
            if !right(y).is_null() { set_parent_color(right(y), x, color(right(y))); }
            set_parent_color(y, parent(x), color(y));
            let xp = parent(x);
            if xp.is_null() { *root = y; } else if x == left(xp) { set_left(xp, y); } else { set_right(xp, y); }
            set_right(y, x);
            set_parent_color(x, y, color(x));
        }

        unsafe fn insert_fixup(root: &mut *mut TaskStruct, mut z: *mut TaskStruct) {
            while color(parent(z)) == RB_RED {
                let zp = parent(z);
                let zpp = parent(zp);
                if zp == left(zpp) {
                    let y = right(zpp);
                    if color(y) == RB_RED {
                        set_parent_color(zp, parent(zp), RB_BLACK);
                        set_parent_color(y, parent(y), RB_BLACK);
                        set_parent_color(zpp, parent(zpp), RB_RED);
                        z = zpp;
                    } else {
                        if z == right(zp) { z = zp; rotate_left(root, z); }
                        let zp2 = parent(z); let zpp2 = parent(zp2);
                        set_parent_color(zp2, parent(zp2), RB_BLACK);
                        set_parent_color(zpp2, parent(zpp2), RB_RED);
                        rotate_right(root, zpp2);
                    }
                } else {
                    let y = left(zpp);
                    if color(y) == RB_RED {
                        set_parent_color(zp, parent(zp), RB_BLACK);
                        set_parent_color(y, parent(y), RB_BLACK);
                        set_parent_color(zpp, parent(zpp), RB_RED);
                        z = zpp;
                    } else {
                        if z == left(zp) { z = zp; rotate_right(root, z); }
                        let zp2 = parent(z); let zpp2 = parent(zp2);
                        set_parent_color(zp2, parent(zp2), RB_BLACK);
                        set_parent_color(zpp2, parent(zpp2), RB_RED);
                        rotate_left(root, zpp2);
                    }
                }
                if z == *root { break; }
            }
            set_parent_color(*root, ptr::null_mut(), RB_BLACK);
        }

        pub unsafe fn insert(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, node: *mut TaskStruct, key_of: fn(*const TaskStruct) -> u64) {
            (*node).se.rb_left = ptr::null_mut();
            (*node).se.rb_right = ptr::null_mut();
            (*node).se.rb_parent_color = rb_make_parent_color(ptr::null_mut(), RB_RED);
            
            let node_key = key_of(node);
            let mut parent = ptr::null_mut();
            let mut cur = *root;
            let mut is_leftmost = true;
            
            while !cur.is_null() {
                parent = cur;
                if node_key < key_of(cur) { cur = left(cur); } 
                else { is_leftmost = false; cur = right(cur); }
            }
            
            set_parent_color(node, parent, RB_RED);
            if parent.is_null() { *root = node; } 
            else if node_key < key_of(parent) { set_left(parent, node); } 
            else { set_right(parent, node); }
            
            if is_leftmost { *leftmost = node; }
            insert_fixup(root, node);
        }

        unsafe fn transplant(root: &mut *mut TaskStruct, u: *mut TaskStruct, v: *mut TaskStruct) {
            let up = parent(u);
            if up.is_null() { *root = v; } 
            else if u == left(up) { set_left(up, v); } 
            else { set_right(up, v); }
            if !v.is_null() { set_parent_color(v, up, color(v)); }
        }

        unsafe fn delete_fixup(root: &mut *mut TaskStruct, mut x: *mut TaskStruct, mut xp: *mut TaskStruct) {
            while x != *root && color(x) == RB_BLACK {
                if x == left(xp) {
                    let mut w = right(xp);
                    if color(w) == RB_RED {
                        set_parent_color(w, parent(w), RB_BLACK);
                        set_parent_color(xp, parent(xp), RB_RED);
                        rotate_left(root, xp);
                        w = right(xp);
                    }
                    if color(left(w)) == RB_BLACK && color(right(w)) == RB_BLACK {
                        set_parent_color(w, parent(w), RB_RED);
                        x = xp; xp = parent(x);
                    } else {
                        if color(right(w)) == RB_BLACK {
                            set_parent_color(left(w), parent(left(w)), RB_BLACK);
                            set_parent_color(w, parent(w), RB_RED);
                            rotate_right(root, w);
                            w = right(xp);
                        }
                        set_parent_color(w, parent(xp), color(xp));
                        set_parent_color(xp, parent(xp), RB_BLACK);
                        set_parent_color(right(w), parent(right(w)), RB_BLACK);
                        rotate_left(root, xp);
                        x = *root; xp = ptr::null_mut();
                    }
                } else {
                    let mut w = left(xp);
                    if color(w) == RB_RED {
                        set_parent_color(w, parent(w), RB_BLACK);
                        set_parent_color(xp, parent(xp), RB_RED);
                        rotate_right(root, xp);
                        w = left(xp);
                    }
                    if color(right(w)) == RB_BLACK && color(left(w)) == RB_BLACK {
                        set_parent_color(w, parent(w), RB_RED);
                        x = xp; xp = parent(x);
                    } else {
                        if color(left(w)) == RB_BLACK {
                            set_parent_color(right(w), parent(right(w)), RB_BLACK);
                            set_parent_color(w, parent(w), RB_RED);
                            rotate_left(root, w);
                            w = left(xp);
                        }
                        set_parent_color(w, parent(xp), color(xp));
                        set_parent_color(xp, parent(xp), RB_BLACK);
                        set_parent_color(left(w), parent(left(w)), RB_BLACK);
                        rotate_right(root, xp);
                        x = *root; xp = ptr::null_mut();
                    }
                }
            }
            set_parent_color(x, parent(x), RB_BLACK);
        }

        pub unsafe fn delete(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, z: *mut TaskStruct) {
            let mut y = z;
            let mut y_orig_color = color(y);
            let x: *mut TaskStruct;
            let xp: *mut TaskStruct;

            if left(z).is_null() {
                x = right(z); xp = parent(z);
                transplant(root, z, right(z));
            } else if right(z).is_null() {
                x = left(z); xp = parent(z);
                transplant(root, z, left(z));
            } else {
                y = subtree_min(right(z));
                y_orig_color = color(y);
                x = right(y);
                if parent(y) == z { xp = y; } 
                else {
                    xp = parent(y);
                    transplant(root, y, right(y));
                    set_right(y, right(z));
                    set_parent_color(right(z), y, color(right(z)));
                }
                transplant(root, z, y);
                set_left(y, left(z));
                set_parent_color(left(z), y, color(left(z)));
                set_parent_color(y, parent(y), color(z));
            }
            
            (*z).se.rb_left = ptr::null_mut();
            (*z).se.rb_right = ptr::null_mut();
            (*z).se.rb_parent_color = rb_make_parent_color(ptr::null_mut(), RB_RED);

            if y_orig_color == RB_BLACK { delete_fixup(root, x, xp); }
            *leftmost = subtree_min(*root);
        }

        pub unsafe fn subtree_min(mut n: *mut TaskStruct) -> *mut TaskStruct {
            if n.is_null() { return ptr::null_mut(); }
            while !left(n).is_null() { n = left(n); }
            n
        }
        
        pub unsafe fn successor(mut n: *mut TaskStruct) -> *mut TaskStruct {
            if n.is_null() { return ptr::null_mut(); }
            if !right(n).is_null() { return subtree_min(right(n)); }
            let mut p = parent(n);
            while !p.is_null() && n == right(p) { n = p; p = parent(p); }
            p
        }
    }

    #[inline] fn fair_key(n: *const TaskStruct) -> u64 { unsafe { (*n).se.vruntime } }
    #[inline] fn dl_key(n: *const TaskStruct) -> u64 { unsafe { (*n).dl.deadline } }

    #[repr(C)]
    pub struct CfsRq {
        root: *mut TaskStruct,
        leftmost: *mut TaskStruct,
        pub nr_running: u32,
        pub load_weight: u64,
        pub min_vruntime: u64,
    }

    impl Default for CfsRq {
        fn default() -> Self {
            Self { root: ptr::null_mut(), leftmost: ptr::null_mut(), nr_running: 0, load_weight: 0, min_vruntime: 0 }
        }
    }

    impl CfsRq {
        #[inline] pub fn is_empty(&self) -> bool { self.root.is_null() }
        #[inline] pub fn pick_first(&self) -> *mut TaskStruct { self.leftmost }

        #[inline]
        pub unsafe fn enqueue(&mut self, task: *mut TaskStruct) {
            let se = &mut (*task).se;
            se.vruntime = core::cmp::max(se.vruntime, self.min_vruntime);
            rbtree::insert(&mut self.root, &mut self.leftmost, task, fair_key);
            se.on_rq = true;
            self.load_weight = self.load_weight.saturating_add(se.weight);
            self.nr_running += 1;
        }

        #[inline]
        pub unsafe fn dequeue(&mut self, task: *mut TaskStruct) {
            rbtree::delete(&mut self.root, &mut self.leftmost, task);
            (*task).se.on_rq = false;
            self.load_weight = self.load_weight.saturating_sub((*task).se.weight);
            self.nr_running = self.nr_running.saturating_sub(1);
        }

        #[inline]
        pub unsafe fn charge_exec(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) {
            let se = &mut (*task).se;
            let delta_vrt = if se.weight == NICE_0_LOAD { delta_exec } 
                            else { ((delta_exec as u128 * se.inv_weight as u128) >> 32) as u64 };
            se.vruntime = se.vruntime.saturating_add(delta_vrt);
            se.load.accumulate(now, delta_exec, se.weight, true);
            self.min_vruntime = core::cmp::max(self.min_vruntime, se.vruntime);
            if !self.leftmost.is_null() {
                let lm_key = fair_key(self.leftmost);
                self.min_vruntime = core::cmp::min(self.min_vruntime, core::cmp::max(lm_key, self.min_vruntime.saturating_sub(TARGET_LATENCY_NS)));
            }
        }

        #[inline]
        pub unsafe fn should_preempt(&self, curr: *const TaskStruct, cand: *const TaskStruct) -> bool {
            if (*cand).se.vruntime >= (*curr).se.vruntime { return false; }
            let vdiff = (*curr).se.vruntime - (*cand).se.vruntime;
            let ran_for = (*curr).se.sum_exec_runtime.saturating_sub((*curr).se.prev_sum_exec_runtime);
            vdiff > 0 && ran_for >= MIN_GRANULARITY_NS || vdiff > TARGET_LATENCY_NS
        }
    }

    const RT_BITMAP_WORDS: usize = (MAX_RT_PRIO as usize + 63) / 64;

    #[repr(C)]
    pub struct RtRq {
        active_head: [*mut TaskStruct; MAX_RT_PRIO as usize],
        active_tail: [*mut TaskStruct; MAX_RT_PRIO as usize],
        bitmap: [u64; RT_BITMAP_WORDS],
        pub nr_running: u32,
        highest_prio: i32,
    }

    impl RtRq {
        #[inline]
        pub fn new() -> Self {
            Self {
                active_head: [ptr::null_mut(); MAX_RT_PRIO as usize],
                active_tail: [ptr::null_mut(); MAX_RT_PRIO as usize],
                bitmap: [0; RT_BITMAP_WORDS],
                nr_running: 0,
                highest_prio: MAX_RT_PRIO,
            }
        }

        #[inline] pub fn is_empty(&self) -> bool { self.nr_running == 0 }

        #[inline]
        fn set_bit(&mut self, prio: usize) { self.bitmap[prio / 64] |= 1u64 << (prio % 64); }
        
        #[inline]
        fn clear_bit(&mut self, prio: usize) { 
            self.bitmap[prio / 64] &= !(1u64 << (prio % 64)); 
        }

        #[inline]
        fn find_first_bit(&self) -> Option<usize> {
            for (i, &w) in self.bitmap.iter().enumerate() {
                if w != 0 {
                    let prio = i * 64 + w.trailing_zeros() as usize;
                    if prio < MAX_RT_PRIO as usize { return Some(prio); }
                }
            }
            None
        }

        #[inline]
        pub unsafe fn enqueue(&mut self, task: *mut TaskStruct) {
            let prio = (*task).prio as usize;
            let rt = &mut (*task).rt;
            
            if self.active_head[prio].is_null() {
                self.active_head[prio] = task;
                self.active_tail[prio] = task;
                rt.list_prev = ptr::null_mut();
                rt.list_next = ptr::null_mut();
            } else {
                let tail = self.active_tail[prio];
                (*tail).rt.list_next = task;
                rt.list_prev = tail;
                rt.list_next = ptr::null_mut();
                self.active_tail[prio] = task;
            }
            
            self.set_bit(prio);
            rt.queued_prio = prio as u32;
            (*task).se.on_rq = true;
            if (prio as i32) < self.highest_prio { self.highest_prio = prio as i32; }
            self.nr_running += 1;
        }

        #[inline]
        pub unsafe fn dequeue(&mut self, task: *mut TaskStruct) {
            let prio = (*task).rt.queued_prio as usize;
            let rt = &(*task).rt;
            let prev = rt.list_prev;
            let next = rt.list_next;

            if !prev.is_null() { (*prev).rt.list_next = next; } else { self.active_head[prio] = next; }
            if !next.is_null() { (*next).rt.list_prev = prev; } else { self.active_tail[prio] = prev; }

            (*task).se.on_rq = false;
            (*task).rt.queued_prio = MAX_RT_PRIO as u32;
            
            if self.active_head[prio].is_null() {
                self.clear_bit(prio);
                if self.highest_prio == prio as i32 {
                    self.highest_prio = self.find_first_bit().map(|p| p as i32).unwrap_or(MAX_RT_PRIO);
                }
            }
            self.nr_running = self.nr_running.saturating_sub(1);
        }

        #[inline]
        pub fn pick_first(&self) -> *mut TaskStruct {
            match self.find_first_bit() {
                Some(prio) => self.active_head[prio],
                None => ptr::null_mut(),
            }
        }
        
        #[inline]
        pub unsafe fn requeue(&mut self, task: *mut TaskStruct) {
            self.dequeue(task);
            self.enqueue(task);
        }
    }

    #[repr(C)]
    pub struct DlRq {
        root: *mut TaskStruct,
        leftmost: *mut TaskStruct,
        pub nr_running: u32,
        pub running_bw: u64,
    }

    impl Default for DlRq {
        fn default() -> Self {
            Self { root: ptr::null_mut(), leftmost: ptr::null_mut(), nr_running: 0, running_bw: 0 }
        }
    }

    impl DlRq {
        #[inline] pub fn is_empty(&self) -> bool { self.root.is_null() }
        #[inline] pub fn pick_first(&self) -> *mut TaskStruct { self.leftmost }

        #[inline]
        pub unsafe fn enqueue(&mut self, task: *mut TaskStruct, now: u64) {
            let dl = &mut (*task).dl;
            if dl.dl_period > 0 && dl.deadline == 0 {
                dl.runtime = dl.dl_runtime as i64;
                dl.deadline = now.saturating_add(dl.dl_deadline);
                dl.replenish_at = now.saturating_add(dl.dl_period);
                let bw = (dl.dl_runtime << 20) / dl.dl_period;
                self.running_bw = self.running_bw.saturating_add(bw);
            }
            rbtree::insert(&mut self.root, &mut self.leftmost, task, dl_key);
            (*task).se.on_rq = true;
            self.nr_running += 1;
        }

        #[inline]
        pub unsafe fn dequeue(&mut self, task: *mut TaskStruct, permanent: bool) {
            rbtree::delete(&mut self.root, &mut self.leftmost, task);
            (*task).se.on_rq = false;
            self.nr_running = self.nr_running.saturating_sub(1);
            if permanent {
                let dl = &(*task).dl;
                if dl.dl_period > 0 {
                    let bw = (dl.dl_runtime << 20) / dl.dl_period;
                    self.running_bw = self.running_bw.saturating_sub(bw);
                }
            }
        }

        #[inline]
        pub unsafe fn update_curr(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) {
            let dl = &mut (*task).dl;
            dl.runtime -= delta_exec as i64;
            if now >= dl.deadline {
                dl.deadline = now.saturating_add(dl.dl_deadline);
                dl.runtime = dl.dl_runtime as i64;
                dl.throttled = false;
            } else if dl.runtime <= 0 {
                dl.throttled = true;
                dl.replenish_at = dl.deadline;
            }
        }
    }

    #[repr(C)]
    pub struct RunQueue {
        pub lock: SpinLock,
        pub cpu: u32,
        pub online: AtomicBool,
        pub clock: AtomicU64,
        pub clock_task: AtomicU64,
        pub fair: CfsRq,
        pub rt: RtRq,
        pub dl: DlRq,
        pub curr: AtomicPtr<TaskStruct>,
        pub idle: *mut TaskStruct,
        pub nr_running: AtomicU32,
        pub stats: super::stats::PerCpuStats,
    }

    impl RunQueue {
        #[inline]
        pub fn new(cpu: u32, idle: *mut TaskStruct) -> Self {
            Self {
                lock: SpinLock::new(), cpu, online: AtomicBool::new(true),
                clock: AtomicU64::new(0), clock_task: AtomicU64::new(0),
                fair: CfsRq::default(), rt: RtRq::new(), dl: DlRq::default(),
                curr: AtomicPtr::new(idle), idle,
                nr_running: AtomicU32::new(0),
                stats: super::stats::PerCpuStats::new(),
            }
        }

        #[inline] pub fn now(&self) -> u64 { self.clock.load(Ordering::Relaxed) }
        
        #[inline]
        pub unsafe fn advance_clock(&self, delta: u64) {
            self.clock.fetch_add(delta, Ordering::Relaxed);
            self.clock_task.fetch_add(delta, Ordering::Relaxed);
        }

        #[inline] pub fn current(&self) -> *mut TaskStruct { self.curr.load(Ordering::Acquire) }
        
        #[inline]
        fn set_current(&self, task: *mut TaskStruct) { self.curr.store(task, Ordering::Release); }

        #[inline]
        pub unsafe fn enqueue_task(&mut self, task: *mut TaskStruct) {
            let now = self.now();
            match (*task).sched_class {
                SchedClass::Deadline => self.dl.enqueue(task, now),
                SchedClass::RealTime => self.rt.enqueue(task),
                SchedClass::Fair => self.fair.enqueue(task),
                _ => { (*task).se.on_rq = true; }
            }
            (*task).set_rq_ptr(self as *mut _ as *mut core::ffi::c_void);
            self.nr_running.fetch_add(1, Ordering::Relaxed);
        }

        #[inline]
        pub unsafe fn dequeue_task(&mut self, task: *mut TaskStruct) {
            match (*task).sched_class {
                SchedClass::Deadline => self.dl.dequeue(task, true),
                SchedClass::RealTime => self.rt.dequeue(task),
                SchedClass::Fair => self.fair.dequeue(task),
                _ => { (*task).se.on_rq = false; }
            }
            (*task).set_rq_ptr(ptr::null_mut());
            self.nr_running.fetch_sub(1, Ordering::Relaxed);
        }

        #[inline]
        pub unsafe fn update_curr(&mut self) {
            let curr = self.current();
            if curr.is_null() || core::ptr::eq(curr, self.idle) { return; }
            let now = self.clock_task.load(Ordering::Relaxed);
            let delta = now.saturating_sub((*curr).se.sum_exec_runtime);
            if delta == 0 { return; }
            
            (*curr).se.prev_sum_exec_runtime = (*curr).se.sum_exec_runtime;
            (*curr).se.sum_exec_runtime = (*curr).se.sum_exec_runtime.saturating_add(delta);
            
            match (*curr).sched_class {
                SchedClass::Fair => self.fair.charge_exec(curr, delta, now),
                SchedClass::Deadline => self.dl.update_curr(curr, delta, now),
                _ => {}
            }
        }

        #[inline]
        pub unsafe fn pick_next_task(&mut self) -> *mut TaskStruct {
            if !self.dl.is_empty() { return self.dl.pick_first(); }
            if !self.rt.is_empty() { return self.rt.pick_first(); }
            if !self.fair.is_empty() { return self.fair.pick_first(); }
            self.idle
        }

        #[inline]
        pub unsafe fn set_curr_task(&mut self, next: *mut TaskStruct) -> *mut TaskStruct {
            let prev = self.current();
            if !prev.is_null() && !core::ptr::eq(prev, next) { (*prev).on_cpu.store(false, Ordering::Release); }
            (*next).se.prev_sum_exec_runtime = self.clock_task.load(Ordering::Relaxed);
            (*next).clear_need_resched();
            (*next).on_cpu.store(true, Ordering::Release);
            (*next).cpu.store(self.cpu, Ordering::Release);
            self.set_current(next);
            prev
        }

        #[inline]
        pub unsafe fn schedule(&mut self) -> (*mut TaskStruct, *mut TaskStruct) {
            self.update_curr();
            let next = self.pick_next_task();
            let prev = self.set_curr_task(next);
            (prev, next)
        }
        
        #[inline]
        pub unsafe fn check_preempt(&self, cand: *mut TaskStruct) {
            let curr = self.current();
            if curr.is_null() || core::ptr::eq(curr, cand) { return; }
            let cc = (*curr).sched_class;
            let nc = (*cand).sched_class;
            let preempt = if nc != cc { nc > cc } 
            else {
                match nc {
                    SchedClass::Deadline => (*cand).dl.deadline < (*curr).dl.deadline,
                    SchedClass::RealTime => (*cand).prio < (*curr).prio,
                    SchedClass::Fair => self.fair.should_preempt(curr, cand),
                    _ => false,
                }
            };
            if preempt { curr.set_need_resched(); }
        }
    }

    pub mod smp {
        use super::*;

        #[inline]
        unsafe fn double_lock(a: &RunQueue, b: &RunQueue) {
            if core::ptr::eq(a, b) { a.lock.lock(); return; }
            if (a as *const _ as usize) < (b as *const _ as usize) {
                a.lock.lock(); b.lock.lock();
            } else {
                b.lock.lock(); a.lock.lock();
            }
        }

        #[inline]
        unsafe fn double_unlock(a: &RunQueue, b: &RunQueue) {
            a.lock.unlock();
            if !core::ptr::eq(a, b) { b.lock.unlock(); }
        }

        #[inline]
        pub unsafe fn select_task_rq(task: *const TaskStruct, registry: &[*mut RunQueue]) -> u32 {
            let allowed = &(*task).se.cpus_allowed;
            let preferred = (*task).se.last_cpu;
            
            if preferred != CPU_NONE && allowed.test(preferred) {
                if let Some(&rq_ptr) = registry.get(preferred as usize) {
                    if !rq_ptr.is_null() && (*rq_ptr).online.load(Ordering::Relaxed) && (*rq_ptr).nr_running.load(Ordering::Relaxed) == 0 {
                        return preferred;
                    }
                }
            }

            let mut best_cpu = CPU_NONE;
            let mut best_load = u32::MAX;
            
            for cpu in allowed.iter() {
                if let Some(&rq_ptr) = registry.get(cpu as usize) {
                    if rq_ptr.is_null() || !(*rq_ptr).online.load(Ordering::Relaxed) { continue; }
                    let load = (*rq_ptr).nr_running.load(Ordering::Relaxed);
                    if load < best_load { best_load = load; best_cpu = cpu; }
                }
            }
            
            if best_cpu == CPU_NONE { if preferred != CPU_NONE { preferred } else { 0 } } else { best_cpu }
        }

        #[inline]
        pub unsafe fn idle_balance(this_cpu: u32, registry: &[*mut RunQueue]) -> bool {
            let this_rq = &mut *registry[this_cpu as usize];
            let mut busiest: *mut RunQueue = ptr::null_mut();
            let mut busiest_load = 1u32;

            for &rq_ptr in registry {
                if rq_ptr.is_null() || core::ptr::eq(rq_ptr, this_rq as *mut _) { continue; }
                let rq = &*rq_ptr;
                if !rq.online.load(Ordering::Relaxed) { continue; }
                let load = rq.nr_running.load(Ordering::Relaxed);
                if load > busiest_load { busiest_load = load; busiest = rq_ptr; }
            }

            if busiest.is_null() { return false; }
            let busiest_rq = &mut *busiest;

            double_lock(this_rq, busiest_rq);
            let mut pulled = false;
            
            if busiest_rq.nr_running.load(Ordering::Relaxed) > this_rq.nr_running.load(Ordering::Relaxed).saturating_add(1) {
                let mut node = busiest_rq.fair.leftmost;
                while !node.is_null() {
                    if !core::ptr::eq(node, busiest_rq.current()) && (*node).se.cpus_allowed.test(this_cpu) {
                        busiest_rq.fair.dequeue(node);
                        (*node).se.last_cpu = this_cpu;
                        (*node).se.nr_migrations += 1;
                        this_rq.fair.enqueue(node);
                        pulled = true;
                        break;
                    }
                    node = rbtree::successor(node);
                }
            }
            
            double_unlock(this_rq, busiest_rq);
            pulled
        }
    }

    unsafe impl Send for RunQueue {}
    unsafe impl Sync for RunQueue {}
}

pub mod stats {
    use core::sync::atomic::{AtomicU64, Ordering};

    #[repr(C, align(64))]
    pub struct PerCpuStats {
        pub switches: AtomicU64,
        pub voluntary: AtomicU64,
        pub involuntary: AtomicU64,
        pub migrations: AtomicU64,
        pub user_time: AtomicU64,
        pub sys_time: AtomicU64,
        pub idle_time: AtomicU64,
        pub io_wait: AtomicU64,
    }

    impl PerCpuStats {
        #[inline]
        pub const fn new() -> Self {
            Self {
                switches: AtomicU64::new(0), voluntary: AtomicU64::new(0),
                involuntary: AtomicU64::new(0), migrations: AtomicU64::new(0),
                user_time: AtomicU64::new(0), sys_time: AtomicU64::new(0),
                idle_time: AtomicU64::new(0), io_wait: AtomicU64::new(0),
            }
        }

        #[inline] pub fn record_switch(&self, vol: bool) {
            self.switches.fetch_add(1, Ordering::Relaxed);
            if vol { self.voluntary.fetch_add(1, Ordering::Relaxed); } 
            else { self.involuntary.fetch_add(1, Ordering::Relaxed); }
        }

        #[inline] pub fn record_migration(&self) { self.migrations.fetch_add(1, Ordering::Relaxed); }
        
        #[inline] pub fn add_user(&self, ns: u64) { self.user_time.fetch_add(ns, Ordering::Relaxed); }
        #[inline] pub fn add_sys(&self, ns: u64) { self.sys_time.fetch_add(ns, Ordering::Relaxed); }
        #[inline] pub fn add_idle(&self, ns: u64) { self.idle_time.fetch_add(ns, Ordering::Relaxed); }
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct GlobalStatsSnapshot {
        pub total_switches: u64,
        pub total_voluntary: u64,
        pub total_involuntary: u64,
        pub total_migrations: u64,
        pub total_user: u64,
        pub total_sys: u64,
        pub total_idle: u64,
    }

    #[inline]
    pub fn collect_global(registry: &[super::runqueue::RunQueue]) -> GlobalStatsSnapshot {
        let mut snap = GlobalStatsSnapshot::default();
        for rq in registry {
            let s = &rq.stats;
            snap.total_switches += s.switches.load(Ordering::Relaxed);
            snap.total_voluntary += s.voluntary.load(Ordering::Relaxed);
            snap.total_involuntary += s.involuntary.load(Ordering::Relaxed);
            snap.total_migrations += s.migrations.load(Ordering::Relaxed);
            snap.total_user += s.user_time.load(Ordering::Relaxed);
            snap.total_sys += s.sys_time.load(Ordering::Relaxed);
            snap.total_idle += s.idle_time.load(Ordering::Relaxed);
        }
        snap
    }
}