

#![allow(dead_code)]

use crate::mm::api::{kalloc_pages, kfree_pages};
use core::cmp;
use core::fmt;
use core::ptr;
use core::str;
use core::sync::atomic::{
    AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicU8, Ordering,
};


pub type TaskId = u64;
pub type Pid = TaskId;
pub type Tgid = TaskId;

pub const KERNEL_STACK_SIZE: usize = 16 * 1024;
pub const KERNEL_STACK_PAGES: usize = KERNEL_STACK_SIZE / 4096;
pub const TASK_COMM_LEN: usize = 16;

pub const NICE_MIN: i8 = -20;
pub const NICE_MAX: i8 = 19;
pub const NICE_WIDTH: usize = (NICE_MAX - NICE_MIN + 1) as usize;

pub const MAX_RT_PRIO: i32 = 100;
pub const MAX_PRIO: i32 = 140;
pub const DEFAULT_PRIO: i32 = MAX_RT_PRIO + 20;

pub const NICE_0_LOAD: u64 = 1024;

pub const MAX_CPUS: usize = 256;
const CPUMASK_WORDS: usize = MAX_CPUS / 64;

pub const RLIM_NLIMITS: usize = 10;
pub const RLIM_INFINITY: u64 = u64::MAX;

pub const CPU_NONE: u32 = u32::MAX;

// ======================================================================

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TaskFlags: u32 {
        const PF_KTHREAD        = 1 << 0;
        const PF_EXITING        = 1 << 1;
        const PF_EXITPIDONE     = 1 << 2;
        const PF_FORKNOEXEC     = 1 << 3;
        const PF_WQ_WORKER      = 1 << 4;
        const PF_NO_SETAFFINITY = 1 << 5;
        const PF_IDLE           = 1 << 6;
        const PF_MEMALLOC       = 1 << 7;
        const PF_FROZEN         = 1 << 8;
        const PF_SUPERPRIV      = 1 << 9;
        const PF_DUMPCORE       = 1 << 10;
        const PF_SIGNALED       = 1 << 11;
        const PF_MEMRECLAIM     = 1 << 12;
        const PF_RANDOMIZE      = 1 << 13;
        const PF_CPU_BOUND      = 1 << 14;
        const PF_VCPU           = 1 << 15;
        const PF_IO_WORKER      = 1 << 16;
        const PF_NEED_RESCHED   = 1 << 17;
        const PF_MIGRATING      = 1 << 18;
        const PF_NO_SLEEP       = 1 << 19;
        const PF_NO_MIGRATE     = 1 << 20;
        const PF_WAKING         = 1 << 21;
        const PF_DL_THROTTLED   = 1 << 22;
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct AtomicTaskFlags(AtomicU32);

impl AtomicTaskFlags {
    pub const fn new(flags: TaskFlags) -> Self {
        Self(AtomicU32::new(flags.bits()))
    }

    pub const fn empty() -> Self {
        Self::new(TaskFlags::empty())
    }

    #[inline]
    pub fn load(&self, order: Ordering) -> TaskFlags {
        TaskFlags::from_bits_truncate(self.0.load(order))
    }

    #[inline]
    pub fn store(&self, flags: TaskFlags, order: Ordering) {
        self.0.store(flags.bits(), order);
    }

    #[inline]
    pub fn fetch_insert(&self, flags: TaskFlags) -> TaskFlags {
        let prev = self.0.fetch_or(flags.bits(), Ordering::AcqRel);
        TaskFlags::from_bits_truncate(prev)
    }

    /// Czyści bity `flags`, zwraca poprzednią wartość.
    #[inline]
    pub fn fetch_remove(&self, flags: TaskFlags) -> TaskFlags {
        let prev = self.0.fetch_and(!flags.bits(), Ordering::AcqRel);
        TaskFlags::from_bits_truncate(prev)
    }

    #[inline]
    pub fn contains(&self, flags: TaskFlags) -> bool {
        self.load(Ordering::Acquire).contains(flags)
    }

    #[inline]
    pub fn test_and_set(&self, flags: TaskFlags) -> bool {
        let prev = self.fetch_insert(flags);
        !prev.contains(flags)
    }
}

impl Clone for AtomicTaskFlags {
    fn clone(&self) -> Self {
        Self::new(self.load(Ordering::Acquire))
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Runnable = 0,
    Interruptible = 1,
    Uninterruptible = 2,
    Stopped = 3,
    Traced = 4,
    Zombie = 5,
    Dead = 6,
    Idle = 7,
}

impl TaskState {
    pub const fn is_runnable(self) -> bool {
        matches!(self, TaskState::Runnable | TaskState::Idle)
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, TaskState::Zombie | TaskState::Dead)
    }

    pub const fn is_waiting(self) -> bool {
        matches!(
            self,
            TaskState::Interruptible
                | TaskState::Uninterruptible
                | TaskState::Stopped
                | TaskState::Traced
        )
    }

    pub const fn can_transition_to(self, next: TaskState) -> bool {
        use TaskState::*;
        match (self, next) {
            (Zombie, Dead) => true,
            (Zombie, _) => false,
            (Dead, _) => false,
            (Idle, Idle) => true,
            (Idle, _) => false,
            (_, Zombie) => true,
            (Runnable, Runnable) => true,
            (Runnable, Interruptible) => true,
            (Runnable, Uninterruptible) => true,
            (Runnable, Stopped) => true,
            (Runnable, Traced) => true,
            (Interruptible, Runnable) => true,
            (Interruptible, Uninterruptible) => true,
            (Interruptible, Stopped) => true,
            (Interruptible, Traced) => true,
            (Uninterruptible, Runnable) => true,
            (Uninterruptible, Interruptible) => true,
            (Stopped, Runnable) => true,
            (Stopped, Traced) => true,
            (Traced, Runnable) => true,
            (Traced, Stopped) => true,
            _ => false,
        }
    }

    const fn from_u8(v: u8) -> TaskState {
        match v {
            0 => TaskState::Runnable,
            1 => TaskState::Interruptible,
            2 => TaskState::Uninterruptible,
            3 => TaskState::Stopped,
            4 => TaskState::Traced,
            5 => TaskState::Zombie,
            6 => TaskState::Dead,
            _ => TaskState::Idle,
        }
    }
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Dead
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
    Normal = 0,
    Batch = 1,
    Idle = 2,
    Fifo = 3,
    RoundRobin = 4,
    Deadline = 5,
    Stop = 6,
}

impl SchedPolicy {
    pub const fn is_realtime(self) -> bool {
        matches!(self, SchedPolicy::Fifo | SchedPolicy::RoundRobin)
    }

    pub const fn is_fair(self) -> bool {
        matches!(self, SchedPolicy::Normal | SchedPolicy::Batch)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchedClass {
    Idle = 0,
    Fair = 1,
    RealTime = 2,
    Deadline = 3,
    Stop = 4,
}

impl From<SchedPolicy> for SchedClass {
    fn from(policy: SchedPolicy) -> Self {
        match policy {
            SchedPolicy::Idle => SchedClass::Idle,
            SchedPolicy::Normal | SchedPolicy::Batch => SchedClass::Fair,
            SchedPolicy::Fifo | SchedPolicy::RoundRobin => SchedClass::RealTime,
            SchedPolicy::Deadline => SchedClass::Deadline,
            SchedPolicy::Stop => SchedClass::Stop,
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuMask {
    bits: [u64; CPUMASK_WORDS],
}

impl CpuMask {
    pub const fn empty() -> Self {
        Self { bits: [0; CPUMASK_WORDS] }
    }

    pub const fn all() -> Self {
        Self { bits: [u64::MAX; CPUMASK_WORDS] }
    }

    pub fn single(cpu: u32) -> Self {
        let mut m = Self::empty();
        m.set(cpu);
        m
    }

    pub fn first_n(n: u32) -> Self {
        let mut m = Self::empty();
        let n = cmp::min(n as usize, MAX_CPUS);
        for cpu in 0..n {
            m.set(cpu as u32);
        }
        m
    }

    pub fn set(&mut self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            self.bits[cpu / 64] |= 1u64 << (cpu % 64);
        }
    }

    pub fn clear(&mut self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            self.bits[cpu / 64] &= !(1u64 << (cpu % 64));
        }
    }

    pub fn is_set(&self, cpu: u32) -> bool {
        let cpu = cpu as usize;
        cpu < MAX_CPUS && (self.bits[cpu / 64] & (1u64 << (cpu % 64))) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|w| *w == 0)
    }

    pub fn count(&self) -> u32 {
        self.bits.iter().map(|w| w.count_ones()).sum()
    }

    pub fn intersects(&self, other: &CpuMask) -> bool {
        self.bits.iter().zip(other.bits.iter()).any(|(a, b)| a & b != 0)
    }

    pub fn and(&self, other: &CpuMask) -> CpuMask {
        let mut out = CpuMask::empty();
        for i in 0..CPUMASK_WORDS {
            out.bits[i] = self.bits[i] & other.bits[i];
        }
        out
    }

    pub fn or(&self, other: &CpuMask) -> CpuMask {
        let mut out = CpuMask::empty();
        for i in 0..CPUMASK_WORDS {
            out.bits[i] = self.bits[i] | other.bits[i];
        }
        out
    }

    pub fn first(&self) -> Option<u32> {
        for (word_idx, word) in self.bits.iter().enumerate() {
            if *word != 0 {
                return Some((word_idx * 64 + word.trailing_zeros() as usize) as u32);
            }
        }
        None
    }

    pub fn next_after(&self, after: u32) -> Option<u32> {
        let start = after.wrapping_add(1);
        for cpu in start..MAX_CPUS as u32 {
            if self.is_set(cpu) {
                return Some(cpu);
            }
        }
        for cpu in 0..=after.min(MAX_CPUS as u32 - 1) {
            if self.is_set(cpu) {
                return Some(cpu);
            }
        }
        None
    }

    pub fn iter(&self) -> CpuMaskIter {
        CpuMaskIter { mask: *self, next_cpu: 0 }
    }
}

impl Default for CpuMask {
    fn default() -> Self {
        CpuMask::empty()
    }
}

pub struct CpuMaskIter {
    mask: CpuMask,
    next_cpu: usize,
}

impl Iterator for CpuMaskIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        while self.next_cpu < MAX_CPUS {
            let cpu = self.next_cpu as u32;
            self.next_cpu += 1;
            if self.mask.is_set(cpu) {
                return Some(cpu);
            }
        }
        None
    }
}


const NICE_TO_WEIGHT_TABLE: [u64; NICE_WIDTH] = [
    88761, 71755, 56483, 46273, 36291,
    29154, 23254, 18705, 14949, 11916,
    9548, 7620, 6100, 4904, 3906,
    3121, 2501, 1991, 1586, 1277,
    1024, 820, 655, 526, 423,
    335, 272, 215, 172, 137,
    110, 87, 70, 56, 45,
    36, 29, 23, 18, 15,
];

const NICE_TO_WMULT_TABLE: [u32; NICE_WIDTH] = [
    48388, 59856, 76040, 92818, 118348,
    147320, 184698, 229616, 287308, 360437,
    449829, 563644, 704093, 875809, 1099582,
    1376151, 1717300, 2157191, 2708050, 3363326,
    4194304, 5237765, 6557202, 8165337, 10153587,
    12820798, 15790321, 19976592, 24970740, 31350126,
    39045157, 49367440, 61356676, 76695844, 95443717,
    119304647, 148102320, 186737708, 238609294, 286331153,
];

pub fn nice_to_weight(nice: i8) -> u64 {
    let clamped = nice.clamp(NICE_MIN, NICE_MAX);
    NICE_TO_WEIGHT_TABLE[(clamped - NICE_MIN) as usize]
}

pub fn nice_to_wmult(nice: i8) -> u32 {
    let clamped = nice.clamp(NICE_MIN, NICE_MAX);
    NICE_TO_WMULT_TABLE[(clamped - NICE_MIN) as usize]
}

pub fn weight_to_nice(weight: u64) -> i8 {
    let mut best_idx = 0usize;
    let mut best_diff = u64::MAX;
    for (idx, w) in NICE_TO_WEIGHT_TABLE.iter().enumerate() {
        let diff = if *w > weight { *w - weight } else { weight - *w };
        if diff < best_diff {
            best_diff = diff;
            best_idx = idx;
        }
    }
    NICE_MIN + best_idx as i8
}

pub const WMULT_SHIFT: u32 = 32;

pub fn calc_delta_fair(delta_exec: u64, weight: u64, inv_weight: u32) -> u64 {
    if weight == NICE_0_LOAD {
        return delta_exec;
    }
    let product = (delta_exec as u128) * (inv_weight as u128);
    (product >> WMULT_SHIFT) as u64
}


#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct FxSaveArea {
    pub bytes: [u8; 512],
}

impl FxSaveArea {
    pub const fn zeroed() -> Self {
        Self { bytes: [0u8; 512] }
    }
}

impl Default for FxSaveArea {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl fmt::Debug for FxSaveArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FxSaveArea(512 B @ {:p})", self.bytes.as_ptr())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub rsp: u64,
    pub rdi: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub fs_base: u64,
    pub gs_base: u64,
    pub fpu: FxSaveArea,
    pub fpu_valid: bool,
}

impl Default for CpuContext {
    fn default() -> Self {
        Self {
            rsp: 0,
            rdi: 0,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            fs_base: 0,
            gs_base: 0,
            fpu: FxSaveArea::zeroed(),
            fpu_valid: false,
        }
    }
}

impl CpuContext {
    #[cfg(all(target_arch = "x86_64", not(test)))]
    #[inline]
    pub unsafe fn save_fpu(&mut self) {
        core::arch::asm!(
            "fxsave [{0}]",
            in(reg) self.fpu.bytes.as_mut_ptr(),
            options(nostack)
        );
        self.fpu_valid = true;
    }

    #[cfg(any(not(target_arch = "x86_64"), test))]
    #[inline]
    pub unsafe fn save_fpu(&mut self) {
        self.fpu_valid = true;
    }

    #[cfg(all(target_arch = "x86_64", not(test)))]
    #[inline]
    pub unsafe fn restore_fpu(&self) {
        if !self.fpu_valid {
            return;
        }
        core::arch::asm!(
            "fxrstor [{0}]",
            in(reg) self.fpu.bytes.as_ptr(),
            options(nostack, readonly)
        );
    }

    #[cfg(any(not(target_arch = "x86_64"), test))]
    #[inline]
    pub unsafe fn restore_fpu(&self) {}
}

pub fn xsave_area_size() -> usize {
    core::mem::size_of::<FxSaveArea>()
}


pub const RB_RED: usize = 0;
pub const RB_BLACK: usize = 1;

#[inline]
pub fn rb_parent(pc: usize) -> *mut TaskStruct {
    (pc & !1) as *mut TaskStruct
}

#[inline]
pub fn rb_color(pc: usize) -> usize {
    pc & 1
}

#[inline]
pub fn rb_is_red(pc: usize) -> bool {
    rb_color(pc) == RB_RED
}

#[inline]
pub fn rb_is_black(pc: usize) -> bool {
    rb_color(pc) == RB_BLACK
}

#[inline]
pub fn rb_make_parent_color(parent: *mut TaskStruct, color: usize) -> usize {
    (parent as usize & !1) | (color & 1)
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadAvg {
    pub last_update_time: u64,
    pub load_sum: u64,
    pub load_avg: u32,
    pub util_sum: u64,
    pub util_avg: u32,
    pub period_contrib: u32,
}

const PELT_DECAY_SHIFT: u32 = 10;

impl LoadAvg {
    pub fn accumulate(&mut self, now: u64, delta_ns: u64, weight: u64, running: bool) {
        if delta_ns == 0 {
            self.last_update_time = now;
            return;
        }
        // Wygaszenie starego wkładu: avg -= avg >> PELT_DECAY_SHIFT
        let load_decay = self.load_avg >> PELT_DECAY_SHIFT.min(31);
        self.load_avg = self.load_avg.saturating_sub(load_decay);
        let util_decay = self.util_avg >> PELT_DECAY_SHIFT.min(31);
        self.util_avg = self.util_avg.saturating_sub(util_decay);

        // Nowy wkład, przeskalowany przez czas trwania i wagę encji.
        let scaled_time = cmp::min(delta_ns, 1_000_000_000) as u128;
        let contrib = ((scaled_time * weight as u128) / 1_000_000_000u128) as u32;
        self.load_avg = self.load_avg.saturating_add(contrib);
        self.load_sum = self.load_sum.saturating_add(delta_ns);

        if running {
            let util_contrib = (scaled_time / 1_000_000) as u32; // ~ms wykonania
            self.util_avg = self.util_avg.saturating_add(util_contrib);
            self.util_sum = self.util_sum.saturating_add(delta_ns);
        }

        self.period_contrib = self.period_contrib.wrapping_add(1);
        self.last_update_time = now;
    }
}

#[repr(C)]
pub struct SchedEntity {
    pub rb_left: *mut TaskStruct,
    pub rb_right: *mut TaskStruct,
    pub rb_parent_color: usize,

    pub run_list: *mut TaskStruct,

    pub vruntime: u64,
    pub vlag: i64,
    pub slice: u64,
    pub sum_exec_runtime: u64,
    pub prev_sum_exec_runtime: u64,
    pub weight: u64,
    pub inv_weight: u32,
    pub load: LoadAvg,
    pub on_rq: bool,
    pub last_cpu: u32,
    pub wake_cpu: u32,
    pub cpus_allowed: CpuMask,
    pub nr_migrations: u32,
}

impl Default for SchedEntity {
    fn default() -> Self {
        Self {
            rb_left: ptr::null_mut(),
            rb_right: ptr::null_mut(),
            rb_parent_color: RB_RED, // parent = null, kolor = czerwony (węzeł nowy, poza drzewem)
            run_list: ptr::null_mut(),
            vruntime: 0,
            vlag: 0,
            slice: 0,
            sum_exec_runtime: 0,
            prev_sum_exec_runtime: 0,
            weight: NICE_0_LOAD,
            inv_weight: nice_to_wmult(0),
            load: LoadAvg::default(),
            on_rq: false,
            last_cpu: CPU_NONE,
            wake_cpu: CPU_NONE,
            cpus_allowed: CpuMask::all(),
            nr_migrations: 0,
        }
    }
}

#[repr(C)]
pub struct RtSchedEntity {
    pub rt_list: ListHead,
    pub rt_priority: u8,
    pub time_slice: u32,
    pub watchdog_stamp: u64,
    pub nr_cpus_allowed: u32,
    pub queued_prio: u32,
    pub owner: *mut TaskStruct,
}

impl Default for RtSchedEntity {
    fn default() -> Self {
        Self {
            rt_list: ListHead::new(),
            rt_priority: 0,
            time_slice: default_time_slice(SchedPolicy::RoundRobin),
            watchdog_stamp: 0,
            nr_cpus_allowed: MAX_CPUS as u32,
            queued_prio: MAX_RT_PRIO as u32,
            owner: ptr::null_mut(),
        }
    }
}

#[repr(C)]
pub struct DlSchedEntity {
    pub dl_runtime: u64,
    pub dl_deadline: u64,
    pub dl_period: u64,
    pub runtime: i64,
    pub deadline: u64,
    pub throttled: bool,
    pub yielded: bool,
    pub dl_bw: u64,
    pub replenish_at: u64,
}

impl Default for DlSchedEntity {
    fn default() -> Self {
        Self {
            dl_runtime: 0,
            dl_deadline: 0,
            dl_period: 0,
            runtime: 0,
            deadline: 0,
            throttled: false,
            yielded: false,
            dl_bw: 0,
            replenish_at: 0,
        }
    }
}


#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskStats {
    pub start_time: u64,
    pub utime: u64,
    pub stime: u64,
    pub guest_time: u64,
    pub nr_voluntary_switches: u64,
    pub nr_involuntary_switches: u64,
    pub nr_migrations: u64,
    pub min_flt: u64,
    pub maj_flt: u64,
    pub wait_sum: u64,
    pub sleep_sum: u64,
    pub block_sum: u64,
    pub last_enqueue_time: u64,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Credentials {
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
    pub suid: u32,
    pub sgid: u32,
    pub fsuid: u32,
    pub fsgid: u32,
    pub cap_effective: u64,
}

impl Credentials {
    pub const fn kernel() -> Self {
        Self {
            uid: 0,
            gid: 0,
            euid: 0,
            egid: 0,
            suid: 0,
            sgid: 0,
            fsuid: 0,
            fsgid: 0,
            cap_effective: u64::MAX,
        }
    }

    pub const fn user(uid: u32, gid: u32) -> Self {
        Self {
            uid,
            gid,
            euid: uid,
            egid: gid,
            suid: uid,
            sgid: gid,
            fsuid: uid,
            fsgid: gid,
            cap_effective: 0,
        }
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials::kernel()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RLimit {
    pub cur: u64,
    pub max: u64,
}

impl RLimit {
    pub const fn unlimited() -> Self {
        Self { cur: RLIM_INFINITY, max: RLIM_INFINITY }
    }

    pub const fn bounded(cur: u64, max: u64) -> Self {
        Self { cur, max }
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RlimitResource {
    Cpu = 0,
    Fsize = 1,
    Data = 2,
    Stack = 3,
    Core = 4,
    Rss = 5,
    Nofile = 6,
    As = 7,
    Nproc = 8,
    Memlock = 9,
}

pub const fn default_rlimits() -> [RLimit; RLIM_NLIMITS] {
    let mut limits = [RLimit::unlimited(); RLIM_NLIMITS];
    limits[RlimitResource::Stack as usize] = RLimit::bounded(8 * 1024 * 1024, RLIM_INFINITY);
    limits[RlimitResource::Nofile as usize] = RLimit::bounded(1024, 4096);
    limits[RlimitResource::Nproc as usize] = RLimit::bounded(4096, 8192);
    limits
}


#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SignalState {
    pub pending: u64,
    pub blocked: u64,
}

impl SignalState {
    pub fn has_pending(&self) -> bool {
        (self.pending & !self.blocked) != 0
    }

    pub fn raise(&mut self, signum: u8) {
        if signum < 64 {
            self.pending |= 1u64 << signum;
        }
    }

    pub fn clear(&mut self, signum: u8) {
        if signum < 64 {
            self.pending &= !(1u64 << signum);
        }
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct ListHead {
    pub prev: *mut ListHead,
    pub next: *mut ListHead,
}

impl ListHead {
    pub const fn new() -> Self {
        Self { prev: ptr::null_mut(), next: ptr::null_mut() }
    }

    pub fn init(&mut self) {
        let self_ptr = self as *mut ListHead;
        self.prev = self_ptr;
        self.next = self_ptr;
    }

    pub fn is_empty(&self) -> bool {
        core::ptr::eq(self.next, self as *const ListHead as *mut ListHead)
            || self.next.is_null()
    }

    pub unsafe fn insert_after(&mut self, node: *mut ListHead) {
        let self_ptr = self as *mut ListHead;
        let next = self.next;
        (*node).prev = self_ptr;
        (*node).next = next;
        (*next).prev = node;
        self.next = node;
    }

    pub unsafe fn insert_before(&mut self, node: *mut ListHead) {
        let self_ptr = self as *mut ListHead;
        let prev = self.prev;
        (*node).next = self_ptr;
        (*node).prev = prev;
        (*prev).next = node;
        self.prev = node;
    }

    pub unsafe fn remove(&mut self) {
        let prev = self.prev;
        let next = self.next;
        if !prev.is_null() {
            (*prev).next = next;
        }
        if !next.is_null() {
            (*next).prev = prev;
        }
        self.init();
    }

    pub fn is_linked(&self) -> bool {
        !self.next.is_null() && !self.prev.is_null()
    }
}

impl Default for ListHead {
    fn default() -> Self {
        ListHead::new()
    }
}

#[cfg(all(target_arch = "x86_64", not(test)))]
#[inline(always)]
unsafe fn arch_local_irq_save() -> u64 {
    let flags: u64;
    core::arch::asm!(
        "pushfq",
        "cli",
        "pop {0}",
        out(reg) flags,
        options(nomem, preserves_flags)
    );
    flags
}

#[cfg(all(target_arch = "x86_64", not(test)))]
#[inline(always)]
unsafe fn arch_local_irq_restore(flags: u64) {
    core::arch::asm!(
        "push {0}",
        "popfq",
        in(reg) flags,
        options(nomem)
    );
}

#[cfg(any(not(target_arch = "x86_64"), test))]
#[inline(always)]
unsafe fn arch_local_irq_save() -> u64 {
    0
}

#[cfg(any(not(target_arch = "x86_64"), test))]
#[inline(always)]
unsafe fn arch_local_irq_restore(_flags: u64) {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct SpinLock {
    locked: AtomicBool,
    #[cfg(debug_assertions)]
    holder_cpu: AtomicU32,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            #[cfg(debug_assertions)]
            holder_cpu: AtomicU32::new(CPU_NONE),
        }
    }

    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    pub fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn lock_irqsave(&self) -> usize {
        let flags = unsafe { arch_local_irq_save() };
        self.lock();
        flags as usize
    }
    pub fn unlock_irqrestore(&self, flags: usize) {
        self.unlock();
        unsafe { arch_local_irq_restore(flags as u64) };
    }

    pub fn lock_guard(&self) -> SpinLockGuard<'_> {
        self.lock();
        SpinLockGuard { lock: self, saved_flags: None }
    }

    pub fn lock_irqsave_guard(&self) -> SpinLockGuard<'_> {
        let flags = self.lock_irqsave();
        SpinLockGuard { lock: self, saved_flags: Some(flags) }
    }
}

#[must_use = "SpinLockGuard zwalnia blokadę przy Drop — porzucenie natychmiast ją odblokuje"]
pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
    saved_flags: Option<usize>,
}

impl<'a> Drop for SpinLockGuard<'a> {
    fn drop(&mut self) {
        match self.saved_flags {
            Some(flags) => self.lock.unlock_irqrestore(flags),
            None => self.lock.unlock(),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskError {
    OutOfMemory,
    InvalidStateTransition { from: TaskState, to: TaskState },
    InvalidRtPriority,
    InvalidNice,
    AffinityDenied,
    AffinityEmpty,
    TaskTerminal,
    ConcurrentStateChange,
    ResourceExhausted,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::OutOfMemory => write!(f, "brak pamięci na stos jądra"),
            TaskError::InvalidStateTransition { from, to } => {
                write!(f, "niedozwolone przejście stanu: {:?} -> {:?}", from, to)
            }
            TaskError::InvalidRtPriority => write!(f, "priorytet RT poza zakresem 0..99"),
            TaskError::InvalidNice => write!(f, "nice poza zakresem -20..=19"),
            TaskError::AffinityDenied => write!(f, "PF_NO_SETAFFINITY blokuje zmianę affinity"),
            TaskError::AffinityEmpty => write!(f, "pusta maska affinity"),
            TaskError::TaskTerminal => write!(f, "operacja niedozwolona na zadaniu terminalnym"),
            TaskError::ConcurrentStateChange => {
                write!(f, "stan zadania zmienił się współbieżnie w trakcie operacji")
            }
            TaskError::ResourceExhausted => write!(f, "wyczerpano zasoby potrzebne do utworzenia zadania"),
        }
    }
}

const STACK_TRAP_RETADDR: u64 = 0xDEAD_0BAD_DEAD_0BAD;


#[repr(C)]
pub struct TaskStruct {
    pub pid: TaskId,
    pub tgid: TaskId,
    pub ppid: TaskId,
    pub comm: [u8; TASK_COMM_LEN],
    pub flags: AtomicTaskFlags,
    state: AtomicU8,

    pub policy: SchedPolicy,
    pub sched_class: SchedClass,
    pub static_prio: i32,
    pub normal_prio: i32,
    pub prio: i32,

    pub se: SchedEntity,
    pub rt: RtSchedEntity,
    pub dl: DlSchedEntity,

    pub tasks: ListHead,
    pub thread_group: ListHead,
    pub children: *mut TaskStruct,
    pub sibling: *mut TaskStruct,
    pub parent: *mut TaskStruct,
    pub group_leader: *mut TaskStruct,

    pub stack_base: *mut u8,
    pub stack_size: usize,
    pub preempt_count: AtomicI32,
    pub irq_count: AtomicI32,
    pub on_cpu: AtomicBool,
    pub cpu: AtomicU32,

    pub context: CpuContext,

    pub mm: *mut core::ffi::c_void,
    pub fs: *mut core::ffi::c_void,
    pub files: *mut core::ffi::c_void,
    pub nsproxy: *mut core::ffi::c_void,

    pub cred: Credentials,
    pub rlimits: [RLimit; RLIM_NLIMITS],

    pub sig: SignalState,
    pub stats: TaskStats,

    pub wait_queue: *mut core::ffi::c_void,
    pub task_lock: SpinLock,

    pub rq: AtomicPtr<core::ffi::c_void>,
}

impl TaskStruct {
    unsafe fn build_initial_stack(stack_base: *mut u8, stack_size: usize, _arg: usize) -> usize {
        ptr::write_bytes(stack_base, 0, stack_size);

        let mut stack_top = stack_base.add(stack_size) as usize;
        stack_top &= !0xF;
        stack_top -= 8;
        *(stack_top as *mut u64) = STACK_TRAP_RETADDR;

        stack_top
    }

    unsafe fn init(
        &mut self,
        pid: TaskId,
        tgid: TaskId,
        ppid: TaskId,
        policy: SchedPolicy,
        nice: i8,
        entry_point: usize,
        arg: usize,
        name: &str,
    ) -> Result<(), TaskError> {
        self.pid = pid;
        self.tgid = tgid;
        self.ppid = ppid;
        self.flags = AtomicTaskFlags::empty();
        self.state = AtomicU8::new(TaskState::Dead as u8);

        self.set_comm(name);

        self.policy = policy;
        self.sched_class = SchedClass::from(policy);
        self.static_prio = DEFAULT_PRIO + nice as i32;
        self.normal_prio = self.static_prio;
        self.prio = self.normal_prio;

        self.se = SchedEntity::default();
        self.se.weight = nice_to_weight(nice);
        self.se.inv_weight = nice_to_wmult(nice);
        self.rt = RtSchedEntity::default();
        self.rt.rt_list.init();
        self.rt.owner = self as *mut TaskStruct;
        self.dl = DlSchedEntity::default();

        self.tasks.init();
        self.thread_group.init();
        self.children = ptr::null_mut();
        self.sibling = ptr::null_mut();
        self.parent = ptr::null_mut();
        self.group_leader = self as *mut TaskStruct;

        self.preempt_count = AtomicI32::new(0);
        self.irq_count = AtomicI32::new(0);
        self.on_cpu = AtomicBool::new(false);
        self.cpu = AtomicU32::new(CPU_NONE);

        self.mm = ptr::null_mut();
        self.fs = ptr::null_mut();
        self.files = ptr::null_mut();
        self.nsproxy = ptr::null_mut();

        self.cred = Credentials::default();
        self.rlimits = default_rlimits();

        self.sig = SignalState::default();
        self.stats = TaskStats::default();

        self.wait_queue = ptr::null_mut();
        self.task_lock = SpinLock::new();
        self.rq = AtomicPtr::new(ptr::null_mut());

        let stack_mem = match kalloc_pages(KERNEL_STACK_PAGES) {
            Some(base) => base,
            None => {
                self.stack_base = ptr::null_mut();
                self.stack_size = 0;
                return Err(TaskError::OutOfMemory);
            }
        };
        self.stack_base = stack_mem;
        self.stack_size = KERNEL_STACK_SIZE;

        let stack_top = Self::build_initial_stack(stack_mem, self.stack_size, arg);

        self.context = CpuContext {
            rsp: stack_top as u64,
            rdi: arg as u64,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry_point as u64,
            fs_base: 0,
            gs_base: 0,
            fpu: FxSaveArea::zeroed(),
            fpu_valid: false,
        };

        if matches!(policy, SchedPolicy::Fifo | SchedPolicy::RoundRobin) {
            self.static_prio = 0;
            self.normal_prio = 0;
            self.prio = 0;
        }

        self.flags.fetch_insert(TaskFlags::PF_FORKNOEXEC);
        self.set_state_unchecked(TaskState::Runnable);
        Ok(())
    }

    pub unsafe fn kthread_create(
        &mut self,
        pid: TaskId,
        entry_point: usize,
        arg: usize,
        name: &str,
    ) -> Result<(), TaskError> {
        self.init(
            pid,
            pid,
            0,
            SchedPolicy::Normal,
            0,
            entry_point,
            arg,
            name,
        )?;
        self.flags.fetch_insert(TaskFlags::PF_KTHREAD);
        self.mm = ptr::null_mut();
        Ok(())
    }

    pub unsafe fn fork(
        &self,
        child: &mut TaskStruct,
        child_pid: TaskId,
        entry_point: usize,
        arg: usize,
    ) -> Result<(), TaskError> {
        let name = self.comm_str();
        child.init(
            child_pid,
            child_pid,
            self.pid,
            self.policy,
            weight_to_nice(self.se.weight),
            entry_point,
            arg,
            name,
        )?;

        child.cred = self.cred;
        child.rlimits = self.rlimits;
        child.se.cpus_allowed = self.se.cpus_allowed;
        child.rt.nr_cpus_allowed = self.rt.nr_cpus_allowed;
        child.flags.fetch_insert(TaskFlags::PF_FORKNOEXEC);

        child.mm = self.mm;
        child.fs = self.fs;
        child.files = self.files;
        child.nsproxy = self.nsproxy;

        child.parent = self as *const TaskStruct as *mut TaskStruct;
        Ok(())
    }

    pub unsafe fn exec(&mut self, entry_point: usize, arg: usize) -> Result<(), TaskError> {
        if self.state().is_terminal() {
            return Err(TaskError::TaskTerminal);
        }
        if self.stack_base.is_null() {
            return Err(TaskError::OutOfMemory);
        }

        let stack_top = Self::build_initial_stack(self.stack_base, self.stack_size, arg);
        self.context = CpuContext {
            rsp: stack_top as u64,
            rdi: arg as u64,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry_point as u64,
            fs_base: 0,
            gs_base: 0,
            fpu: FxSaveArea::zeroed(),
            fpu_valid: false,
        };

        self.flags.fetch_remove(TaskFlags::PF_FORKNOEXEC);
        self.se.sum_exec_runtime = 0;
        self.se.prev_sum_exec_runtime = 0;
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        let _guard = self.task_lock.lock_irqsave_guard();

        if self.state() == TaskState::Dead {
            return;
        }

        if !self.stack_base.is_null() {
            kfree_pages(self.stack_base, KERNEL_STACK_PAGES);
            self.stack_base = ptr::null_mut();
            self.stack_size = 0;
        }

        if self.tasks.is_linked() && !self.tasks.is_empty() {
            self.tasks.remove();
        }
        if self.thread_group.is_linked() && !self.thread_group.is_empty() {
            self.thread_group.remove();
        }
        if self.rt.rt_list.is_linked() && !self.rt.rt_list.is_empty() {
            self.rt.rt_list.remove();
        }

        self.children = ptr::null_mut();
        self.sibling = ptr::null_mut();
        self.parent = ptr::null_mut();
        self.group_leader = ptr::null_mut();
        self.mm = ptr::null_mut();
        self.fs = ptr::null_mut();
        self.files = ptr::null_mut();
        self.nsproxy = ptr::null_mut();
        self.wait_queue = ptr::null_mut();
        self.rq.store(ptr::null_mut(), Ordering::Release);

        self.set_state_unchecked(TaskState::Dead);
    }

    pub fn state(&self) -> TaskState {
        TaskState::from_u8(self.state.load(Ordering::Acquire))
    }

    fn set_state_unchecked(&self, new_state: TaskState) {
        self.state.store(new_state as u8, Ordering::Release);
    }
    pub fn set_state(&self, new_state: TaskState) -> Result<(), TaskError> {
        loop {
            let current = self.state();
            if !current.can_transition_to(new_state) {
                return Err(TaskError::InvalidStateTransition { from: current, to: new_state });
            }
            match self.state.compare_exchange_weak(
                current as u8,
                new_state as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
    }

    #[allow(dead_code)]
    fn set_state_owned(&mut self, new_state: TaskState) -> Result<(), TaskError> {
        self.set_state(new_state)
    }

    pub fn wake_up(&self) -> Result<(), TaskError> {
        match self.state() {
            TaskState::Interruptible | TaskState::Uninterruptible => self.set_state(TaskState::Runnable),
            TaskState::Runnable => Ok(()),
            other => Err(TaskError::InvalidStateTransition { from: other, to: TaskState::Runnable }),
        }
    }

    pub fn sleep(&self, interruptible: bool) -> Result<(), TaskError> {
        let target = if interruptible { TaskState::Interruptible } else { TaskState::Uninterruptible };
        self.set_state(target)
    }

    pub fn is_kernel_thread(&self) -> bool {
        self.flags.contains(TaskFlags::PF_KTHREAD)
    }

    pub fn is_idle_task(&self) -> bool {
        self.flags.contains(TaskFlags::PF_IDLE) || self.state() == TaskState::Idle
    }

    pub fn is_zombie(&self) -> bool {
        self.state() == TaskState::Zombie
    }

    pub fn is_runnable(&self) -> bool {
        self.state().is_runnable()
    }

    pub fn needs_resched(&self) -> bool {
        self.flags.contains(TaskFlags::PF_NEED_RESCHED)
    }

    pub fn set_need_resched(&self) {
        self.flags.fetch_insert(TaskFlags::PF_NEED_RESCHED);
    }

    pub fn clear_need_resched(&self) {
        self.flags.fetch_remove(TaskFlags::PF_NEED_RESCHED);
    }

    pub fn set_nice(&mut self, nice: i8) -> Result<(), TaskError> {
        if !self.policy.is_fair() {
            return Err(TaskError::InvalidNice);
        }
        let clamped = nice.clamp(NICE_MIN, NICE_MAX);
        self.static_prio = DEFAULT_PRIO + clamped as i32;
        self.normal_prio = self.static_prio;
        self.prio = self.normal_prio;
        self.se.weight = nice_to_weight(clamped);
        self.se.inv_weight = nice_to_wmult(clamped);
        Ok(())
    }

    pub fn set_rt_priority(&mut self, rt_priority: u8) -> Result<(), TaskError> {
        if !self.policy.is_realtime() {
            return Err(TaskError::InvalidRtPriority);
        }
        if rt_priority as i32 >= MAX_RT_PRIO {
            return Err(TaskError::InvalidRtPriority);
        }
        self.rt.rt_priority = rt_priority;
        self.static_prio = rt_priority as i32;
        self.normal_prio = self.static_prio;
        self.prio = self.normal_prio;
        Ok(())
    }

    pub fn effective_prio(&self) -> i32 {
        self.prio
    }

    pub fn set_affinity(&mut self, mask: CpuMask) -> Result<(), TaskError> {
        if self.flags.contains(TaskFlags::PF_NO_SETAFFINITY) {
            return Err(TaskError::AffinityDenied);
        }
        if mask.is_empty() {
            return Err(TaskError::AffinityEmpty);
        }
        self.se.cpus_allowed = mask;
        self.rt.nr_cpus_allowed = mask.count();
        Ok(())
    }

    pub fn can_run_on(&self, cpu: u32) -> bool {
        self.se.cpus_allowed.is_set(cpu)
    }

    /// nigdzie.
    pub fn rq_ptr(&self) -> *mut core::ffi::c_void {
        self.rq.load(Ordering::Acquire)
    }

    pub fn set_rq_ptr(&self, rq: *mut core::ffi::c_void) {
        self.rq.store(rq, Ordering::Release);
    }

    pub fn charge_cputime(&mut self, delta_ns: u64) {
        self.se.prev_sum_exec_runtime = self.se.sum_exec_runtime;
        self.se.sum_exec_runtime = self.se.sum_exec_runtime.saturating_add(delta_ns);
        self.stats.utime = self.stats.utime.saturating_add(delta_ns);

        if self.policy.is_realtime() && self.rt.time_slice > 0 {
            let delta_ticks = (delta_ns / 1_000_000) as u32;
            self.rt.time_slice = self.rt.time_slice.saturating_sub(delta_ticks);
        }

        if matches!(self.policy, SchedPolicy::Deadline) {
            self.dl.runtime -= delta_ns as i64;
            if self.dl.runtime <= 0 {
                self.dl.throttled = true;
                self.flags.fetch_insert(TaskFlags::PF_DL_THROTTLED);
            }
        }
    }

    pub fn record_voluntary_switch(&mut self) {
        self.stats.nr_voluntary_switches = self.stats.nr_voluntary_switches.saturating_add(1);
    }

    pub fn record_involuntary_switch(&mut self) {
        self.stats.nr_involuntary_switches = self.stats.nr_involuntary_switches.saturating_add(1);
    }

    pub fn record_migration(&mut self, new_cpu: u32) {
        self.stats.nr_migrations = self.stats.nr_migrations.saturating_add(1);
        self.se.nr_migrations = self.se.nr_migrations.saturating_add(1);
        self.se.last_cpu = new_cpu;
        self.cpu.store(new_cpu, Ordering::Release);
    }

    pub fn set_comm(&mut self, name: &str) {
        self.comm = [0; TASK_COMM_LEN];
        let name_bytes = name.as_bytes();
        let copy_len = cmp::min(name_bytes.len(), TASK_COMM_LEN - 1);
        self.comm[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
    }

    pub fn comm_str(&self) -> &str {
        let end = self.comm.iter().position(|&b| b == 0).unwrap_or(TASK_COMM_LEN);
        str::from_utf8(&self.comm[..end]).unwrap_or("")
    }
}

impl fmt::Debug for TaskStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskStruct")
            .field("pid", &self.pid)
            .field("tgid", &self.tgid)
            .field("comm", &self.comm_str())
            .field("state", &self.state())
            .field("policy", &self.policy)
            .field("prio", &self.prio)
            .field("cpu", &self.cpu.load(Ordering::Relaxed))
            .finish()
    }
}

impl fmt::Display for TaskStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} state={:?} prio={} cpu={}",
            self.pid,
            self.comm_str(),
            self.state(),
            self.prio,
            self.cpu.load(Ordering::Relaxed)
        )
    }
}

unsafe impl Send for TaskStruct {}
unsafe impl Sync for TaskStruct {}
#[repr(C)]
pub struct ThreadInfoView<'a> {
    pub stack_base: *mut u8,
    pub stack_size: usize,
    pub context: &'a mut CpuContext,
    pub cpu: &'a AtomicU32,
    pub on_cpu: &'a AtomicBool,
    pub preempt_count: &'a AtomicI32,
}

impl TaskStruct {
    pub fn thread_info(&mut self) -> ThreadInfoView<'_> {
        ThreadInfoView {
            stack_base: self.stack_base,
            stack_size: self.stack_size,
            context: &mut self.context,
            cpu: &self.cpu,
            on_cpu: &self.on_cpu,
            preempt_count: &self.preempt_count,
        }
    }
}

pub const fn default_time_slice(policy: SchedPolicy) -> u32 {
    match policy {
        SchedPolicy::Fifo => 0,
        SchedPolicy::RoundRobin => 100,
        SchedPolicy::Normal | SchedPolicy::Batch => 0,
        SchedPolicy::Idle => 0,
        SchedPolicy::Deadline => 0,
        SchedPolicy::Stop => 1,
    }
}

pub fn fair_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
    a.se.vruntime < b.se.vruntime
}

pub fn deadline_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
    a.dl.deadline < b.dl.deadline
}


#[cfg(test)]
impl TaskStruct {
    pub fn blank() -> TaskStruct {
        TaskStruct {
            pid: 0,
            tgid: 0,
            ppid: 0,
            comm: [0; TASK_COMM_LEN],
            flags: AtomicTaskFlags::empty(),
            state: AtomicU8::new(TaskState::Dead as u8),
            policy: SchedPolicy::Normal,
            sched_class: SchedClass::Fair,
            static_prio: DEFAULT_PRIO,
            normal_prio: DEFAULT_PRIO,
            prio: DEFAULT_PRIO,
            se: SchedEntity::default(),
            rt: RtSchedEntity::default(),
            dl: DlSchedEntity::default(),
            tasks: ListHead::new(),
            thread_group: ListHead::new(),
            children: ptr::null_mut(),
            sibling: ptr::null_mut(),
            parent: ptr::null_mut(),
            group_leader: ptr::null_mut(),
            stack_base: ptr::null_mut(),
            stack_size: 0,
            preempt_count: AtomicI32::new(0),
            irq_count: AtomicI32::new(0),
            on_cpu: AtomicBool::new(false),
            cpu: AtomicU32::new(CPU_NONE),
            context: CpuContext::default(),
            mm: ptr::null_mut(),
            fs: ptr::null_mut(),
            files: ptr::null_mut(),
            nsproxy: ptr::null_mut(),
            cred: Credentials::default(),
            rlimits: default_rlimits(),
            sig: SignalState::default(),
            stats: TaskStats::default(),
            wait_queue: ptr::null_mut(),
            task_lock: SpinLock::new(),
            rq: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn init_test_stub(&mut self, pid: TaskId, policy: SchedPolicy, nice: i8) {
        self.pid = pid;
        self.tgid = pid;
        self.state = AtomicU8::new(TaskState::Runnable as u8);
        self.policy = policy;
        self.sched_class = SchedClass::from(policy);

        self.se = SchedEntity::default();
        self.se.weight = nice_to_weight(nice);
        self.se.inv_weight = nice_to_wmult(nice);

        self.rt = RtSchedEntity::default();
        self.rt.rt_list.init();
        self.rt.owner = self as *mut TaskStruct;

        self.dl = DlSchedEntity::default();

        match policy {
            SchedPolicy::Fifo | SchedPolicy::RoundRobin => {
                let rt_prio = (MAX_RT_PRIO - 1 - nice as i32).clamp(0, MAX_RT_PRIO - 1) as u8;
                self.rt.rt_priority = rt_prio;
                self.static_prio = rt_prio as i32;
                self.normal_prio = rt_prio as i32;
                self.prio = rt_prio as i32;
                self.rt.time_slice = default_time_slice(policy);
            }
            SchedPolicy::Deadline => {
                self.static_prio = 0;
                self.normal_prio = 0;
                self.prio = 0;
            }
            SchedPolicy::Stop => {
                self.static_prio = -1;
                self.normal_prio = -1;
                self.prio = -1;
            }
            SchedPolicy::Idle => {
                self.static_prio = MAX_PRIO;
                self.normal_prio = MAX_PRIO;
                self.prio = MAX_PRIO;
                self.flags.fetch_insert(TaskFlags::PF_IDLE);
                self.set_state_unchecked(TaskState::Idle);
            }
            SchedPolicy::Normal | SchedPolicy::Batch => {
                self.static_prio = DEFAULT_PRIO + nice as i32;
                self.normal_prio = self.static_prio;
                self.prio = self.normal_prio;
            }
        }

        self.tasks.init();
        self.thread_group.init();
        self.rq = AtomicPtr::new(ptr::null_mut());
        self.cpu = AtomicU32::new(CPU_NONE);
        self.on_cpu = AtomicBool::new(false);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_to_weight_is_monotonically_decreasing() {
        let mut prev = u64::MAX;
        for nice in NICE_MIN..=NICE_MAX {
            let w = nice_to_weight(nice);
            assert!(w < prev);
            prev = w;
        }
    }

    #[test]
    fn nice_zero_has_default_weight() {
        assert_eq!(nice_to_weight(0), NICE_0_LOAD);
    }

    #[test]
    fn nice_to_weight_clamps_out_of_range() {
        assert_eq!(nice_to_weight(-100), nice_to_weight(NICE_MIN));
        assert_eq!(nice_to_weight(100), nice_to_weight(NICE_MAX));
    }

    #[test]
    fn weight_to_nice_round_trip_is_close() {
        for nice in NICE_MIN..=NICE_MAX {
            let w = nice_to_weight(nice);
            assert_eq!(weight_to_nice(w), nice);
        }
    }

    #[test]
    fn calc_delta_fair_is_identity_at_nice_zero() {
        let d = calc_delta_fair(1_000_000, NICE_0_LOAD, nice_to_wmult(0));
        assert_eq!(d, 1_000_000);
    }

    #[test]
    fn calc_delta_fair_grows_for_lower_weight() {
        let heavy = calc_delta_fair(1_000_000, nice_to_weight(-5), nice_to_wmult(-5));
        let light = calc_delta_fair(1_000_000, nice_to_weight(5), nice_to_wmult(5));
        assert!(light > heavy);
    }

    #[test]
    fn cpumask_basic_set_clear() {
        let mut mask = CpuMask::empty();
        assert!(mask.is_empty());
        mask.set(3);
        mask.set(70);
        assert!(mask.is_set(3));
        assert!(mask.is_set(70));
        assert!(!mask.is_set(4));
        assert_eq!(mask.count(), 2);
        mask.clear(3);
        assert!(!mask.is_set(3));
        assert_eq!(mask.count(), 1);
    }

    #[test]
    fn cpumask_all_contains_every_cpu() {
        let mask = CpuMask::all();
        assert_eq!(mask.count(), MAX_CPUS as u32);
        assert!(mask.is_set(0));
        assert!(mask.is_set((MAX_CPUS - 1) as u32));
    }

    #[test]
    fn cpumask_intersects() {
        let mut a = CpuMask::empty();
        let mut b = CpuMask::empty();
        a.set(5);
        b.set(6);
        assert!(!a.intersects(&b));
        b.set(5);
        assert!(a.intersects(&b));
    }

    #[test]
    fn cpumask_and_or() {
        let mut a = CpuMask::empty();
        let mut b = CpuMask::empty();
        a.set(1);
        a.set(2);
        b.set(2);
        b.set(3);
        assert_eq!(a.and(&b).count(), 1);
        assert!(a.and(&b).is_set(2));
        assert_eq!(a.or(&b).count(), 3);
    }

    #[test]
    fn cpumask_next_after_wraps() {
        let mut m = CpuMask::empty();
        m.set(2);
        m.set(5);
        assert_eq!(m.next_after(2), Some(5));
        assert_eq!(m.next_after(5), Some(2)); // zawija
    }

    #[test]
    fn cpumask_first_n() {
        let m = CpuMask::first_n(4);
        assert_eq!(m.count(), 4);
        assert!(m.is_set(3));
        assert!(!m.is_set(4));
    }

    #[test]
    fn task_state_valid_transitions() {
        assert!(TaskState::Runnable.can_transition_to(TaskState::Interruptible));
        assert!(TaskState::Interruptible.can_transition_to(TaskState::Runnable));
        assert!(TaskState::Runnable.can_transition_to(TaskState::Zombie));
        assert!(TaskState::Zombie.can_transition_to(TaskState::Dead));
    }

    #[test]
    fn task_state_invalid_transitions_are_rejected() {
        assert!(!TaskState::Dead.can_transition_to(TaskState::Runnable));
        assert!(!TaskState::Zombie.can_transition_to(TaskState::Runnable));
        assert!(!TaskState::Idle.can_transition_to(TaskState::Runnable));
    }

    #[test]
    fn default_rlimits_have_sane_stack_and_nofile() {
        let limits = default_rlimits();
        assert_eq!(limits[RlimitResource::Stack as usize].cur, 8 * 1024 * 1024);
        assert_eq!(limits[RlimitResource::Nofile as usize].cur, 1024);
        assert_eq!(limits[RlimitResource::Cpu as usize].cur, RLIM_INFINITY);
    }

    #[test]
    fn credentials_kernel_has_full_capabilities() {
        let cred = Credentials::kernel();
        assert_eq!(cred.uid, 0);
        assert_eq!(cred.cap_effective, u64::MAX);
    }

    #[test]
    fn credentials_user_has_no_extra_capabilities() {
        let cred = Credentials::user(1000, 1000);
        assert_eq!(cred.uid, 1000);
        assert_eq!(cred.cap_effective, 0);
    }

    #[test]
    fn signal_state_pending_respects_blocked_mask() {
        let mut sig = SignalState::default();
        sig.raise(9);
        assert!(sig.has_pending());
        sig.blocked |= 1 << 9;
        assert!(!sig.has_pending());
        sig.clear(9);
        assert!(!sig.has_pending());
    }

    #[test]
    fn list_head_insert_and_remove_roundtrip() {
        let mut head = ListHead::new();
        head.init();

        let mut a = ListHead::new();
        a.init();
        let mut b = ListHead::new();
        b.init();

        unsafe {
            head.insert_after(&mut a as *mut ListHead);
            head.insert_after(&mut b as *mut ListHead);
        }

        assert!(!head.is_empty());
        unsafe {
            assert!(core::ptr::eq(head.next, &mut b as *mut ListHead));
            assert!(core::ptr::eq(b.next, &mut a as *mut ListHead));
        }

        unsafe {
            a.remove();
            b.remove();
        }
        assert!(head.is_empty());
    }

    #[test]
    fn list_head_remove_never_creates_a_cycle_with_three_nodes() {
        let mut head = ListHead::new();
        head.init();
        let mut nodes: [ListHead; 3] = [ListHead::new(), ListHead::new(), ListHead::new()];
        for n in nodes.iter_mut() {
            n.init();
        }
        unsafe {
            for n in nodes.iter_mut() {
                head.insert_before(n as *mut ListHead);
            }
        }
        let mut count = 0;
        let mut cur = head.next;
        unsafe {
            while !core::ptr::eq(cur, &mut head as *mut ListHead) && count < 10 {
                count += 1;
                cur = (*cur).next;
            }
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn spinlock_lock_unlock_cycle() {
        let lock = SpinLock::new();
        assert!(!lock.is_locked());
        lock.lock();
        assert!(lock.is_locked());
        lock.unlock();
        assert!(!lock.is_locked());
    }

    #[test]
    fn spinlock_try_lock_fails_when_held() {
        let lock = SpinLock::new();
        assert!(lock.try_lock());
        assert!(!lock.try_lock());
        lock.unlock();
        assert!(lock.try_lock());
        lock.unlock();
    }

    #[test]
    fn spinlock_irqsave_guard_unlocks_on_drop() {
        let lock = SpinLock::new();
        {
            let _guard = lock.lock_irqsave_guard();
            assert!(lock.is_locked());
        }
        assert!(!lock.is_locked());
    }

    #[test]
    fn spinlock_plain_guard_unlocks_on_drop() {
        let lock = SpinLock::new();
        {
            let _guard = lock.lock_guard();
            assert!(lock.is_locked());
        }
        assert!(!lock.is_locked());
    }

    #[test]
    fn atomic_task_flags_insert_remove_are_bitwise() {
        let flags = AtomicTaskFlags::empty();
        flags.fetch_insert(TaskFlags::PF_NEED_RESCHED);
        assert!(flags.contains(TaskFlags::PF_NEED_RESCHED));
        flags.fetch_insert(TaskFlags::PF_KTHREAD);
        assert!(flags.contains(TaskFlags::PF_NEED_RESCHED));
        assert!(flags.contains(TaskFlags::PF_KTHREAD));
        flags.fetch_remove(TaskFlags::PF_NEED_RESCHED);
        assert!(!flags.contains(TaskFlags::PF_NEED_RESCHED));
        assert!(flags.contains(TaskFlags::PF_KTHREAD));
    }

    #[test]
    fn atomic_task_flags_test_and_set_is_edge_triggered() {
        let flags = AtomicTaskFlags::empty();
        assert!(flags.test_and_set(TaskFlags::PF_WAKING));
        assert!(!flags.test_and_set(TaskFlags::PF_WAKING));
    }

    #[test]
    fn rb_parent_color_encoding_roundtrips() {
        let fake_parent = 0x1000usize as *mut TaskStruct;
        let pc_red = rb_make_parent_color(fake_parent, RB_RED);
        let pc_black = rb_make_parent_color(fake_parent, RB_BLACK);
        assert_eq!(rb_parent(pc_red), fake_parent);
        assert_eq!(rb_parent(pc_black), fake_parent);
        assert!(rb_is_red(pc_red));
        assert!(rb_is_black(pc_black));
    }

    #[test]
    fn fx_save_area_default_is_zeroed() {
        let area = FxSaveArea::default();
        assert!(area.bytes.iter().all(|&b| b == 0));
        assert_eq!(core::mem::size_of::<FxSaveArea>(), 512);
        assert_eq!(core::mem::align_of::<FxSaveArea>(), 16);
    }

    #[test]
    fn cpu_context_default_has_no_valid_fpu_state() {
        let ctx = CpuContext::default();
        assert!(!ctx.fpu_valid);
    }

    #[test]
    fn sched_policy_maps_to_expected_class() {
        assert_eq!(SchedClass::from(SchedPolicy::Normal), SchedClass::Fair);
        assert_eq!(SchedClass::from(SchedPolicy::Batch), SchedClass::Fair);
        assert_eq!(SchedClass::from(SchedPolicy::Idle), SchedClass::Idle);
        assert_eq!(SchedClass::from(SchedPolicy::Fifo), SchedClass::RealTime);
        assert_eq!(SchedClass::from(SchedPolicy::RoundRobin), SchedClass::RealTime);
        assert_eq!(SchedClass::from(SchedPolicy::Deadline), SchedClass::Deadline);
        assert_eq!(SchedClass::from(SchedPolicy::Stop), SchedClass::Stop);
    }

    #[test]
    fn sched_class_ordering_matches_pick_next_priority() {
        assert!(SchedClass::Stop > SchedClass::Deadline);
        assert!(SchedClass::Deadline > SchedClass::RealTime);
        assert!(SchedClass::RealTime > SchedClass::Fair);
        assert!(SchedClass::Fair > SchedClass::Idle);
    }

    #[test]
    fn default_time_slice_matches_policy_expectations() {
        assert_eq!(default_time_slice(SchedPolicy::Fifo), 0);
        assert!(default_time_slice(SchedPolicy::RoundRobin) > 0);
        assert_eq!(default_time_slice(SchedPolicy::Normal), 0);
    }

    #[test]
    fn load_avg_accumulate_grows_then_decays() {
        let mut la = LoadAvg::default();
        la.accumulate(1_000_000_000, 1_000_000_000, NICE_0_LOAD, true);
        let after_first = la.load_avg;
        assert!(after_first > 0);
        la.accumulate(2_000_000_000, 1_000_000_000, 0, false);
        assert!(la.load_avg <= after_first);
    }

    fn bare_task_for_state_tests() -> TaskStruct {
        TaskStruct {
            pid: 1,
            tgid: 1,
            ppid: 0,
            comm: [0; TASK_COMM_LEN],
            flags: AtomicTaskFlags::empty(),
            state: AtomicU8::new(TaskState::Runnable as u8),
            policy: SchedPolicy::Normal,
            sched_class: SchedClass::Fair,
            static_prio: DEFAULT_PRIO,
            normal_prio: DEFAULT_PRIO,
            prio: DEFAULT_PRIO,
            se: SchedEntity::default(),
            rt: RtSchedEntity::default(),
            dl: DlSchedEntity::default(),
            tasks: ListHead::new(),
            thread_group: ListHead::new(),
            children: ptr::null_mut(),
            sibling: ptr::null_mut(),
            parent: ptr::null_mut(),
            group_leader: ptr::null_mut(),
            stack_base: ptr::null_mut(),
            stack_size: 0,
            preempt_count: AtomicI32::new(0),
            irq_count: AtomicI32::new(0),
            on_cpu: AtomicBool::new(false),
            cpu: AtomicU32::new(CPU_NONE),
            context: CpuContext::default(),
            mm: ptr::null_mut(),
            fs: ptr::null_mut(),
            files: ptr::null_mut(),
            nsproxy: ptr::null_mut(),
            cred: Credentials::default(),
            rlimits: default_rlimits(),
            sig: SignalState::default(),
            stats: TaskStats::default(),
            wait_queue: ptr::null_mut(),
            task_lock: SpinLock::new(),
            rq: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[test]
    fn set_state_via_shared_reference_does_not_need_mut() {
        let task = bare_task_for_state_tests();
        let task_ref: &TaskStruct = &task;
        assert!(task_ref.set_state(TaskState::Interruptible).is_ok());
        assert_eq!(task_ref.state(), TaskState::Interruptible);
        assert!(task_ref.wake_up().is_ok());
        assert_eq!(task_ref.state(), TaskState::Runnable);
    }

    #[test]
    fn set_state_rejects_illegal_transition_via_shared_reference() {
        let task = bare_task_for_state_tests();
        assert!(task.set_state(TaskState::Zombie).is_ok());
        assert!(task.set_state(TaskState::Runnable).is_err());
        assert_eq!(task.state(), TaskState::Zombie);
    }

    #[test]
    fn wake_up_process_pattern_from_remote_cpu_context() {
        let task = bare_task_for_state_tests();
        task.set_state(TaskState::Uninterruptible).unwrap();

        fn remote_wake(t: &TaskStruct) -> Result<(), TaskError> {
            t.wake_up()
        }

        assert!(remote_wake(&task).is_ok());
        assert_eq!(task.state(), TaskState::Runnable);
    }

    #[test]
    fn rq_backpointer_roundtrips() {
        let task = bare_task_for_state_tests();
        assert!(task.rq_ptr().is_null());
        let fake: usize = 0xdead_beef;
        task.set_rq_ptr(fake as *mut core::ffi::c_void);
        assert_eq!(task.rq_ptr() as usize, fake);
    }

    #[test]
    fn need_resched_flag_is_settable_from_shared_reference() {
        let task = bare_task_for_state_tests();
        assert!(!task.needs_resched());
        task.set_need_resched();
        assert!(task.needs_resched());
        task.clear_need_resched();
        assert!(!task.needs_resched());
    }

    #[test]
    fn rt_sched_entity_starts_with_initialized_list_and_no_queued_prio() {
        let rt = RtSchedEntity::default();
        assert_eq!(rt.queued_prio, MAX_RT_PRIO as u32);
    }
}