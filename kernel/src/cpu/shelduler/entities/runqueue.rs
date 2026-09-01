use crate::cpu::scheduler::entities::task::{
    TaskStruct, TaskState, SchedClass, SchedPolicy, MAX_RT_PRIO, NICE_0_LOAD
};
use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub const CACHE_LINE_SIZE: usize = 64;

#[repr(C)]
pub struct RqRt {
    pub nr_running: usize,
    pub hprio: usize,
    pub overloaded: bool,
    pub active_mask: u128,
    pub queue: [*mut TaskStruct; MAX_RT_PRIO as usize],
}

#[repr(C)]
pub struct RqFair {
    pub nr_running: usize,
    pub h_nr_running: usize,
    pub min_vruntime: u64,
    pub root: *mut TaskStruct,
    pub leftmost: *mut TaskStruct,
    pub load_weight: u64,
}

#[repr(C)]
pub struct RqDl {
    pub dl_nr_running: usize,
    pub earliest_dl: u64,
    pub root: *mut TaskStruct,
    pub leftmost: *mut TaskStruct,
    pub running_bw: u64,
}

#[repr(C)]
pub struct RunQueue {
    _pad_start: [u8; CACHE_LINE_SIZE],
    
    pub lock: crate::cpu::scheduler::task::SpinLock,
    pub cpu: u32,
    pub nr_running: AtomicUsize,
    pub nr_uninterruptible: AtomicUsize,
    pub next_balance: u64,
    
    pub curr: *mut TaskStruct,
    pub idle: *mut TaskStruct,
    pub stop: *mut TaskStruct,
    
    pub rt: RqRt,
    pub fair: RqFair,
    pub dl: RqDl,
    
    pub clock_task: u64,
    pub clock: u64,
    
    _pad_end: [u8; CACHE_LINE_SIZE],
}

impl RqRt {
    pub const fn new() -> Self {
        Self {
            nr_running: 0,
            hprio: MAX_RT_PRIO as usize,
            overloaded: false,
            active_mask: 0,
            queue: [ptr::null_mut(); MAX_RT_PRIO as usize],
        }
    }
}

impl RqFair {
    pub const fn new() -> Self {
        Self {
            nr_running: 0,
            h_nr_running: 0,
            min_vruntime: 0,
            root: ptr::null_mut(),
            leftmost: ptr::null_mut(),
            load_weight: 0,
        }
    }
}

impl RqDl {
    pub const fn new() -> Self {
        Self {
            dl_nr_running: 0,
            earliest_dl: u64::MAX,
            root: ptr::null_mut(),
            leftmost: ptr::null_mut(),
            running_bw: 0,
        }
    }
}

impl RunQueue {
    pub const fn new(cpu: u32) -> Self {
        Self {
            _pad_start: [0; CACHE_LINE_SIZE],
            lock: crate::cpu::scheduler::task::SpinLock::new(),
            cpu,
            nr_running: AtomicUsize::new(0),
            nr_uninterruptible: AtomicUsize::new(0),
            next_balance: 0,
            curr: ptr::null_mut(),
            idle: ptr::null_mut(),
            stop: ptr::null_mut(),
            rt: RqRt::new(),
            fair: RqFair::new(),
            dl: RqDl::new(),
            clock_task: 0,
            clock: 0,
            _pad_end: [0; CACHE_LINE_SIZE],
        }
    }

    #[inline(always)]
    pub fn lock(&self) {
        self.lock.lock();
    }

    #[inline(always)]
    pub fn unlock(&self) {
        self.lock.unlock();
    }

    #[inline(always)]
    pub fn nr_running(&self) -> usize {
        self.nr_running.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.nr_running() == 0
    }

    pub unsafe fn enqueue_task(&mut self, task: *mut TaskStruct, flags: u32) {
        if task.is_null() { return; }
        let t = &mut *task;
        
        if t.se.on_rq {
            return;
        }

        t.se.on_rq = true;
        t.se.wake_cpu = self.cpu;
        self.nr_running.fetch_add(1, Ordering::Relaxed);

        match t.sched_class {
            SchedClass::RealTime => self.enqueue_rt(t),
            SchedClass::Fair => self.enqueue_fair(t, flags),
            SchedClass::Deadline => self.enqueue_dl(t),
            SchedClass::Stop => self.enqueue_stop(t),
            SchedClass::Idle => self.enqueue_idle(t),
        }
    }

    pub unsafe fn dequeue_task(&mut self, task: *mut TaskStruct, flags: u32) {
        if task.is_null() { return; }
        let t = &mut *task;
        
        if !t.se.on_rq {
            return;
        }

        t.se.on_rq = false;
        self.nr_running.fetch_sub(1, Ordering::Relaxed);

        match t.sched_class {
            SchedClass::RealTime => self.dequeue_rt(t),
            SchedClass::Fair => self.dequeue_fair(t, flags),
            SchedClass::Deadline => self.dequeue_dl(t),
            SchedClass::Stop => self.dequeue_stop(t),
            SchedClass::Idle => self.dequeue_idle(t),
        }
    }

    unsafe fn enqueue_rt(&mut self, task: &mut TaskStruct) {
        let prio = task.rt.rt_priority as usize;
        task.rt.run_list = self.rt.queue[prio];
        
        if self.rt.queue[prio].is_null() {
            self.rt.active_mask |= 1u128 << prio;
            if prio < self.rt.hprio {
                self.rt.hprio = prio;
            }
        } else {
            let mut tail = self.rt.queue[prio];
            while !(*tail).rt.run_list.is_null() {
                tail = (*tail).rt.run_list;
            }
            (*tail).rt.run_list = task as *mut TaskStruct;
        }
        
        self.rt.queue[prio] = task as *mut TaskStruct;
        self.rt.nr_running += 1;
    }

    unsafe fn dequeue_rt(&mut self, task: &mut TaskStruct) {
        let prio = task.rt.rt_priority as usize;
        let mut cur = self.rt.queue[prio];
        let mut prev: *mut TaskStruct = ptr::null_mut();

        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.rt.queue[prio] = task.rt.run_list;
                    if self.rt.queue[prio].is_null() {
                        self.rt.active_mask &= !(1u128 << prio);
                        self.update_rt_hprio();
                    }
                } else {
                    (*prev).rt.run_list = task.rt.run_list;
                }
                task.rt.run_list = ptr::null_mut();
                self.rt.nr_running -= 1;
                return;
            }
            prev = cur;
            cur = (*cur).rt.run_list;
        }
    }

    fn update_rt_hprio(&mut self) {
        self.rt.hprio = if self.rt.active_mask == 0 {
            MAX_RT_PRIO as usize
        } else {
            self.rt.active_mask.trailing_zeros() as usize
        };
    }

    unsafe fn enqueue_fair(&mut self, task: &mut TaskStruct, _flags: u32) {
        self.fair.nr_running += 1;
        self.fair.h_nr_running += 1;
        self.fair.load_weight += task.se.weight;
        
        if self.fair.root.is_null() {
            self.fair.root = task as *mut TaskStruct;
            self.fair.leftmost = task as *mut TaskStruct;
            self.fair.min_vruntime = task.se.vruntime;
        } else {
            self.fair_insert(task as *mut TaskStruct);
        }
    }

    unsafe fn dequeue_fair(&mut self, task: &mut TaskStruct, _flags: u32) {
        self.fair.nr_running -= 1;
        self.fair.h_nr_running -= 1;
        self.fair.load_weight = self.fair.load_weight.saturating_sub(task.se.weight);
        
        if self.fair.root == task as *mut TaskStruct {
            self.fair.root = ptr::null_mut();
            self.fair.leftmost = ptr::null_mut();
        } else {
            self.fair_remove(task as *mut TaskStruct);
        }
    }

    unsafe fn fair_insert(&mut self, task: *mut TaskStruct) {
        let mut node = self.fair.root;
        let mut parent: *mut TaskStruct = ptr::null_mut();
        let mut leftmost = self.fair.leftmost;

        while !node.is_null() {
            parent = node;
            if (*task).se.vruntime < (*node).se.vruntime {
                leftmost = node;
                node = (*node).se.run_list; 
            } else {
                node = (*node).se.run_list; 
            }
        }

        (*task).se.run_list = ptr::null_mut();
        if parent.is_null() {
            self.fair.root = task;
        } else if (*task).se.vruntime < (*parent).se.vruntime {
            (*parent).se.run_list = task;
        }

        if leftmost == self.fair.leftmost || self.fair.leftmost.is_null() {
            self.fair.leftmost = task;
        }
    }

    unsafe fn fair_remove(&mut self, task: *mut TaskStruct) {
        let mut node = self.fair.root;
        let mut parent: *mut TaskStruct = ptr::null_mut();
        let mut is_left = false;

        while !node.is_null() {
            if node == task {
                break;
            }
            parent = node;
            if (*task).se.vruntime < (*node).se.vruntime {
                is_left = true;
                node = (*node).se.run_list;
            } else {
                is_left = false;
                node = (*node).se.run_list;
            }
        }

        if node.is_null() { return; }

        let mut child = (*node).se.run_list;
        if !child.is_null() {
            let mut right = (*child).se.run_list;
            while !right.is_null() {
                child = right;
                right = (*right).se.run_list;
            }
            
            if parent.is_null() {
                self.fair.root = child;
            } else if is_left {
                (*parent).se.run_list = child;
            } else {
                (*parent).se.run_list = child;
            }
            
            if child != node {
                (*child).se.run_list = (*node).se.run_list;
            }
        } else {
            if parent.is_null() {
                self.fair.root = ptr::null_mut();
            } else if is_left {
                (*parent).se.run_list = ptr::null_mut();
            } else {
                (*parent).se.run_list = ptr::null_mut();
            }
        }

        if self.fair.leftmost == task {
            self.fair.leftmost = self.fair_get_leftmost();
        }
        
        (*task).se.run_list = ptr::null_mut();
    }

    fn fair_get_leftmost(&self) -> *mut TaskStruct {
        let mut node = self.fair.root;
        let mut leftmost = ptr::null_mut();
        while !node.is_null() {
            leftmost = node;
            node = (*node).se.run_list;
        }
        leftmost
    }

    unsafe fn enqueue_dl(&mut self, task: &mut TaskStruct) {
        self.dl.dl_nr_running += 1;
        if self.dl.root.is_null() {
            self.dl.root = task as *mut TaskStruct;
            self.dl.leftmost = task as *mut TaskStruct;
            self.dl.earliest_dl = task.dl.deadline;
        } else {
            self.dl_insert(task as *mut TaskStruct);
        }
    }

    unsafe fn dequeue_dl(&mut self, task: &mut TaskStruct) {
        self.dl.dl_nr_running -= 1;
        if self.dl.root == task as *mut TaskStruct {
            self.dl.root = ptr::null_mut();
            self.dl.leftmost = ptr::null_mut();
            self.dl.earliest_dl = u64::MAX;
        } else {
            self.dl_remove(task as *mut TaskStruct);
        }
    }

    unsafe fn dl_insert(&mut self, task: *mut TaskStruct) {
        let mut node = self.dl.root;
        let mut parent: *mut TaskStruct = ptr::null_mut();

        while !node.is_null() {
            parent = node;
            if (*task).dl.deadline < (*node).dl.deadline {
                node = (*node).se.run_list;
            } else {
                node = (*node).se.run_list;
            }
        }

        (*task).se.run_list = ptr::null_mut();
        if parent.is_null() {
            self.dl.root = task;
        } else if (*task).dl.deadline < (*parent).dl.deadline {
            (*parent).se.run_list = task;
        }

        if self.dl.leftmost.is_null() || (*task).dl.deadline < (*self.dl.leftmost).dl.deadline {
            self.dl.leftmost = task;
            self.dl.earliest_dl = (*task).dl.deadline;
        }
    }

    unsafe fn dl_remove(&mut self, task: *mut TaskStruct) {
        let mut node = self.dl.root;
        let mut parent: *mut TaskStruct = ptr::null_mut();
        let mut is_left = false;

        while !node.is_null() {
            if node == task { break; }
            parent = node;
            if (*task).dl.deadline < (*node).dl.deadline {
                is_left = true;
                node = (*node).se.run_list;
            } else {
                is_left = false;
                node = (*node).se.run_list;
            }
        }

        if node.is_null() { return; }

        let mut child = (*node).se.run_list;
        if !child.is_null() {
            let mut right = (*child).se.run_list;
            while !right.is_null() {
                child = right;
                right = (*right).se.run_list;
            }
            if parent.is_null() { self.dl.root = child; }
            else if is_left { (*parent).se.run_list = child; }
            else { (*parent).se.run_list = child; }
            
            if child != node {
                (*child).se.run_list = (*node).se.run_list;
            }
        } else {
            if parent.is_null() { self.dl.root = ptr::null_mut(); }
            else if is_left { (*parent).se.run_list = ptr::null_mut(); }
            else { (*parent).se.run_list = ptr::null_mut(); }
        }

        if self.dl.leftmost == task {
            self.dl.leftmost = self.dl_get_leftmost();
            self.dl.earliest_dl = if self.dl.leftmost.is_null() { u64::MAX } else { (*self.dl.leftmost).dl.deadline };
        }
        (*task).se.run_list = ptr::null_mut();
    }

    fn dl_get_leftmost(&self) -> *mut TaskStruct {
        let mut node = self.dl.root;
        let mut leftmost = ptr::null_mut();
        while !node.is_null() {
            leftmost = node;
            node = (*node).se.run_list;
        }
        leftmost
    }

    unsafe fn enqueue_stop(&mut self, task: &mut TaskStruct) {
        task.se.run_list = self.stop;
        self.stop = task as *mut TaskStruct;
    }

    unsafe fn dequeue_stop(&mut self, task: &mut TaskStruct) {
        let mut cur = self.stop;
        let mut prev: *mut TaskStruct = ptr::null_mut();
        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() { self.stop = task.se.run_list; }
                else { (*prev).se.run_list = task.se.run_list; }
                task.se.run_list = ptr::null_mut();
                return;
            }
            prev = cur;
            cur = (*cur).se.run_list;
        }
    }

    unsafe fn enqueue_idle(&mut self, task: &mut TaskStruct) {
        self.idle = task as *mut TaskStruct;
    }

    unsafe fn dequeue_idle(&mut self, task: &mut TaskStruct) {
        if self.idle == task as *mut TaskStruct {
            self.idle = ptr::null_mut();
        }
    }

    pub unsafe fn pick_next_task(&mut self) -> *mut TaskStruct {
        if self.is_empty() {
            return self.idle;
        }

        if !self.stop.is_null() {
            return self.stop;
        }

        if self.dl.dl_nr_running > 0 && self.dl.earliest_dl < self.clock {
            return self.dl.leftmost;
        }

        if self.rt.nr_running > 0 {
            return self.rt.queue[self.rt.hprio];
        }

        if self.fair.nr_running > 0 {
            return self.fair.leftmost;
        }

        self.idle
    }

    pub unsafe fn update_curr(&mut self, delta_exec: u64) {
        let curr = self.curr;
        if curr.is_null() { return; }
        
        let t = &mut *curr;
        t.charge_cputime(delta_exec);
        
        if t.sched_class == SchedClass::Fair {
            t.se.vruntime = t.se.vruntime.saturating_add(
                (delta_exec * NICE_0_LOAD) / t.se.weight
            );
            if t.se.vruntime < self.fair.min_vruntime {
                t.se.vruntime = self.fair.min_vruntime;
            }
        }
    }
}