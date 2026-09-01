use crate::cpu::scheduler::task::{
    SchedClass, SchedEntity, SchedPolicy, SpinLock, TaskStruct, MAX_RT_PRIO, NICE_0_LOAD,
};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

pub const CACHE_LINE_SIZE: usize = 64;

#[repr(C)]
pub struct RqRt {
    pub nr_running: usize,
    pub hprio: usize,
    pub bitmap: u128,
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
    pub lock: SpinLock,
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
    pub clock: u64,
    pub clock_task: u64,
    _pad_end: [u8; CACHE_LINE_SIZE],
}

impl RqRt {
    pub const fn new() -> Self {
        Self {
            nr_running: 0,
            hprio: 0,
            bitmap: 0,
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
            earliest_dl: 0,
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
            lock: SpinLock::new(),
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
            clock: 0,
            clock_task: 0,
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

    pub unsafe fn enqueue_task(&mut self, task: *mut TaskStruct, _flags: u32) {
        if task.is_null() {
            return;
        }
        let t = &mut *task;
        if t.se.on_rq {
            return;
        }
        t.se.on_rq = true;
        self.nr_running.fetch_add(1, Ordering::Relaxed);

        match t.sched_class {
            SchedClass::RealTime => self.rt_enqueue(t),
            SchedClass::Fair => self.fair_enqueue(t),
            SchedClass::Deadline => self.dl_enqueue(t),
            SchedClass::Stop => self.stop_enqueue(t),
            SchedClass::Idle => self.idle_enqueue(t),
        }
    }

    pub unsafe fn dequeue_task(&mut self, task: *mut TaskStruct, _flags: u32) {
        if task.is_null() {
            return;
        }
        let t = &mut *task;
        if !t.se.on_rq {
            return;
        }
        t.se.on_rq = false;
        self.nr_running.fetch_sub(1, Ordering::Relaxed);

        match t.sched_class {
            SchedClass::RealTime => self.rt_dequeue(t),
            SchedClass::Fair => self.fair_dequeue(t),
            SchedClass::Deadline => self.dl_dequeue(t),
            SchedClass::Stop => self.stop_dequeue(t),
            SchedClass::Idle => self.idle_dequeue(t),
        }
    }

    unsafe fn rt_enqueue(&mut self, task: &mut TaskStruct) {
        let prio = task.rt.rt_priority as usize;
        debug_assert!(prio < MAX_RT_PRIO as usize);

        task.rt.run_list = ptr::null_mut();

        if self.rt.queue[prio].is_null() {
            self.rt.queue[prio] = task as *mut TaskStruct;
            self.rt.bitmap |= 1u128 << prio;
            if prio < self.rt.hprio || self.rt.hprio == 0 {
                self.rt.hprio = prio;
            }
        } else {
            let mut tail = self.rt.queue[prio];
            while !(*tail).rt.run_list.is_null() {
                tail = (*tail).rt.run_list;
            }
            (*tail).rt.run_list = task as *mut TaskStruct;
        }

        self.rt.nr_running += 1;
    }

    unsafe fn rt_dequeue(&mut self, task: &mut TaskStruct) {
        let prio = task.rt.rt_priority as usize;
        let mut cur = self.rt.queue[prio];
        let mut prev: *mut TaskStruct = ptr::null_mut();

        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.rt.queue[prio] = (*cur).rt.run_list;
                } else {
                    (*prev).rt.run_list = (*cur).rt.run_list;
                }

                if self.rt.queue[prio].is_null() {
                    self.rt.bitmap &= !(1u128 << prio);
                    if prio == self.rt.hprio {
                        self.rt.hprio = if self.rt.bitmap == 0 {
                            MAX_RT_PRIO as usize
                        } else {
                            self.rt.bitmap.trailing_zeros() as usize
                        };
                    }
                }

                task.rt.run_list = ptr::null_mut();
                self.rt.nr_running -= 1;
                return;
            }
            prev = cur;
            cur = (*cur).rt.run_list;
        }
        debug_assert!(false, "rt_dequeue: task not found");
    }

    fn update_rt_hprio(&mut self) {
        self.rt.hprio = if self.rt.bitmap == 0 {
            MAX_RT_PRIO as usize
        } else {
            self.rt.bitmap.trailing_zeros() as usize
        };
    }

    #[inline(always)]
    unsafe fn rb_is_red(node: *mut TaskStruct) -> bool {
        if node.is_null() {
            return false;
        }
        (*(*node).se.rb_parent_color as *const usize) as usize & 1 == 0
    }

    #[inline(always)]
    unsafe fn rb_set_red(node: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            *parent_color_ptr &= !1;
        }
    }

    #[inline(always)]
    unsafe fn rb_set_black(node: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            *parent_color_ptr |= 1;
        }
    }

    #[inline(always)]
    unsafe fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            let color = *parent_color_ptr & 1;
            *parent_color_ptr = (parent as usize) | color;
        }
    }

    #[inline(always)]
    unsafe fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            let parent_color = (*node).se.rb_parent_color;
            (parent_color & !1) as *mut TaskStruct
        }
    }

    #[inline(always)]
    unsafe fn rb_left(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            (*node).se.rb_left
        }
    }

    #[inline(always)]
    unsafe fn rb_right(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            (*node).se.rb_right
        }
    }

    #[inline(always)]
    unsafe fn rb_set_left(node: *mut TaskStruct, left: *mut TaskStruct) {
        if !node.is_null() {
            (*node).se.rb_left = left;
            if !left.is_null() {
                Self::rb_set_parent(left, node);
            }
        }
    }

    #[inline(always)]
    unsafe fn rb_set_right(node: *mut TaskStruct, right: *mut TaskStruct) {
        if !node.is_null() {
            (*node).se.rb_right = right;
            if !right.is_null() {
                Self::rb_set_parent(right, node);
            }
        }
    }

    unsafe fn rb_rotate_left(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        let right = Self::rb_right(node);
        if right.is_null() {
            return;
        }
        let right_left = Self::rb_left(right);

        Self::rb_set_right(node, right_left);
        if !right_left.is_null() {
            Self::rb_set_parent(right_left, node);
        }

        let parent = Self::rb_parent(node);
        Self::rb_set_parent(right, parent);

        if parent.is_null() {
            *root_ptr = right;
        } else if node == Self::rb_left(parent) {
            Self::rb_set_left(parent, right);
        } else {
            Self::rb_set_right(parent, right);
        }

        Self::rb_set_left(right, node);
        Self::rb_set_parent(node, right);
    }

    unsafe fn rb_rotate_right(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        let left = Self::rb_left(node);
        if left.is_null() {
            return;
        }
        let left_right = Self::rb_right(left);

        Self::rb_set_left(node, left_right);
        if !left_right.is_null() {
            Self::rb_set_parent(left_right, node);
        }

        let parent = Self::rb_parent(node);
        Self::rb_set_parent(left, parent);

        if parent.is_null() {
            *root_ptr = left;
        } else if node == Self::rb_right(parent) {
            Self::rb_set_right(parent, left);
        } else {
            Self::rb_set_left(parent, left);
        }

        Self::rb_set_right(left, node);
        Self::rb_set_parent(node, left);
    }

    unsafe fn rb_insert(
        root_ptr: *mut *mut TaskStruct,
        new_node: *mut TaskStruct,
        cmp: fn(*mut TaskStruct, *mut TaskStruct) -> bool,
    ) {
        Self::rb_set_left(new_node, ptr::null_mut());
        Self::rb_set_right(new_node, ptr::null_mut());
        Self::rb_set_red(new_node);

        let mut parent = ptr::null_mut();
        let mut cur = *root_ptr;

        while !cur.is_null() {
            parent = cur;
            if cmp(new_node, cur) {
                cur = Self::rb_left(cur);
            } else {
                cur = Self::rb_right(cur);
            }
        }

        Self::rb_set_parent(new_node, parent);
        if parent.is_null() {
            *root_ptr = new_node;
        } else if cmp(new_node, parent) {
            Self::rb_set_left(parent, new_node);
        } else {
            Self::rb_set_right(parent, new_node);
        }

        let mut node = new_node;
        while !node.is_null() && node != *root_ptr && Self::rb_is_red(Self::rb_parent(node)) {
            let parent = Self::rb_parent(node);
            let grandparent = Self::rb_parent(parent);
            if grandparent.is_null() {
                break;
            }

            if parent == Self::rb_left(grandparent) {
                let uncle = Self::rb_right(grandparent);
                if Self::rb_is_red(uncle) {
                    Self::rb_set_black(parent);
                    Self::rb_set_black(uncle);
                    Self::rb_set_red(grandparent);
                    node = grandparent;
                } else {
                    if node == Self::rb_right(parent) {
                        node = parent;
                        Self::rb_rotate_left(root_ptr, node);
                        parent = Self::rb_parent(node);
                        if parent.is_null() {
                            break;
                        }
                        grandparent = Self::rb_parent(parent);
                    }
                    if !parent.is_null() && !grandparent.is_null() {
                        Self::rb_set_black(parent);
                        Self::rb_set_red(grandparent);
                        Self::rb_rotate_right(root_ptr, grandparent);
                    }
                    break;
                }
            } else {
                let uncle = Self::rb_left(grandparent);
                if Self::rb_is_red(uncle) {
                    Self::rb_set_black(parent);
                    Self::rb_set_black(uncle);
                    Self::rb_set_red(grandparent);
                    node = grandparent;
                } else {
                    if node == Self::rb_left(parent) {
                        node = parent;
                        Self::rb_rotate_right(root_ptr, node);
                        parent = Self::rb_parent(node);
                        if parent.is_null() {
                            break;
                        }
                        grandparent = Self::rb_parent(parent);
                    }
                    if !parent.is_null() && !grandparent.is_null() {
                        Self::rb_set_black(parent);
                        Self::rb_set_red(grandparent);
                        Self::rb_rotate_left(root_ptr, grandparent);
                    }
                    break;
                }
            }
        }

        Self::rb_set_black(*root_ptr);
    }

    unsafe fn rb_erase(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        let mut node = node;
        let mut child: *mut TaskStruct;
        let mut parent: *mut TaskStruct;
        let color: bool;

        if !Self::rb_left(node).is_null() && !Self::rb_right(node).is_null() {
            let mut old = node;
            node = Self::rb_right(node);
            while !Self::rb_left(node).is_null() {
                node = Self::rb_left(node);
            }
            child = Self::rb_right(node);
            parent = Self::rb_parent(node);
            color = Self::rb_is_red(node);
            if !child.is_null() {
                Self::rb_set_parent(child, parent);
            }
            if parent.is_null() {
                *root_ptr = child;
            } else if node == Self::rb_left(parent) {
                Self::rb_set_left(parent, child);
            } else {
                Self::rb_set_right(parent, child);
            }
            if node == Self::rb_parent(old) {
                parent = node;
            }
            Self::rb_set_parent(node, Self::rb_parent(old));
            Self::rb_set_left(node, Self::rb_left(old));
            Self::rb_set_right(node, Self::rb_right(old));
            Self::rb_set_parent(Self::rb_left(old), node);
            Self::rb_set_parent(Self::rb_right(old), node);
            if Self::rb_parent(old).is_null() {
                *root_ptr = node;
            } else if old == Self::rb_left(Self::rb_parent(old)) {
                Self::rb_set_left(Self::rb_parent(old), node);
            } else {
                Self::rb_set_right(Self::rb_parent(old), node);
            }
            if !color {
                Self::rb_set_black(node);
            }
        } else {
            if Self::rb_left(node).is_null() {
                child = Self::rb_right(node);
            } else {
                child = Self::rb_left(node);
            }
            parent = Self::rb_parent(node);
            color = Self::rb_is_red(node);
            if !child.is_null() {
                Self::rb_set_parent(child, parent);
            }
            if parent.is_null() {
                *root_ptr = child;
            } else if node == Self::rb_left(parent) {
                Self::rb_set_left(parent, child);
            } else {
                Self::rb_set_right(parent, child);
            }
            if !color {
                Self::rb_erase_fixup(root_ptr, child, parent);
            }
        }
    }

    unsafe fn rb_erase_fixup(
        root_ptr: *mut *mut TaskStruct,
        mut node: *mut TaskStruct,
        mut parent: *mut TaskStruct,
    ) {
        while node.is_null() || !Self::rb_is_red(node) {
            if node == Self::rb_left(parent) {
                let mut sibling = Self::rb_right(parent);
                if Self::rb_is_red(sibling) {
                    Self::rb_set_black(sibling);
                    Self::rb_set_red(parent);
                    Self::rb_rotate_left(root_ptr, parent);
                    sibling = Self::rb_right(parent);
                }
                if (!sibling.is_null() && !Self::rb_is_red(Self::rb_left(sibling))) &&
                   (!sibling.is_null() && !Self::rb_is_red(Self::rb_right(sibling)))
                {
                    Self::rb_set_red(sibling);
                    node = parent;
                    parent = Self::rb_parent(node);
                } else {
                    if !sibling.is_null() && !Self::rb_is_red(Self::rb_right(sibling)) {
                        Self::rb_set_black(Self::rb_left(sibling));
                        Self::rb_set_red(sibling);
                        Self::rb_rotate_right(root_ptr, sibling);
                        sibling = Self::rb_right(parent);
                    }
                    if !sibling.is_null() {
                        if Self::rb_is_red(parent) {
                            Self::rb_set_red(sibling);
                        } else {
                            Self::rb_set_black(sibling);
                        }
                        Self::rb_set_black(parent);
                        Self::rb_set_black(Self::rb_right(sibling));
                        Self::rb_rotate_left(root_ptr, parent);
                    }
                    node = *root_ptr;
                    parent = ptr::null_mut();
                }
            } else {
                let mut sibling = Self::rb_left(parent);
                if Self::rb_is_red(sibling) {
                    Self::rb_set_black(sibling);
                    Self::rb_set_red(parent);
                    Self::rb_rotate_right(root_ptr, parent);
                    sibling = Self::rb_left(parent);
                }
                if (!sibling.is_null() && !Self::rb_is_red(Self::rb_left(sibling))) &&
                   (!sibling.is_null() && !Self::rb_is_red(Self::rb_right(sibling)))
                {
                    Self::rb_set_red(sibling);
                    node = parent;
                    parent = Self::rb_parent(node);
                } else {
                    if !sibling.is_null() && !Self::rb_is_red(Self::rb_left(sibling)) {
                        Self::rb_set_black(Self::rb_right(sibling));
                        Self::rb_set_red(sibling);
                        Self::rb_rotate_left(root_ptr, sibling);
                        sibling = Self::rb_left(parent);
                    }
                    if !sibling.is_null() {
                        if Self::rb_is_red(parent) {
                            Self::rb_set_red(sibling);
                        } else {
                            Self::rb_set_black(sibling);
                        }
                        Self::rb_set_black(parent);
                        Self::rb_set_black(Self::rb_left(sibling));
                        Self::rb_rotate_right(root_ptr, parent);
                    }
                    node = *root_ptr;
                    parent = ptr::null_mut();
                }
            }
        }
        if !node.is_null() {
            Self::rb_set_black(node);
        }
    }

    unsafe fn rb_first(root: *mut TaskStruct) -> *mut TaskStruct {
        let mut node = root;
        if node.is_null() {
            return ptr::null_mut();
        }
        while !Self::rb_left(node).is_null() {
            node = Self::rb_left(node);
        }
        node
    }

    unsafe fn fair_cmp(a: *mut TaskStruct, b: *mut TaskStruct) -> bool {
        if a.is_null() || b.is_null() {
            return false;
        }
        (*a).se.vruntime < (*b).se.vruntime
    }

    unsafe fn fair_enqueue(&mut self, task: &mut TaskStruct) {
        self.fair.nr_running += 1;
        self.fair.h_nr_running += 1;
        self.fair.load_weight += task.se.weight;

        if task.se.vruntime < self.fair.min_vruntime {
            task.se.vruntime = self.fair.min_vruntime;
        }

        Self::rb_insert(
            &mut self.fair.root as *mut _,
            task as *mut TaskStruct,
            Self::fair_cmp,
        );

        if self.fair.leftmost.is_null()
            || (*task).se.vruntime < (*self.fair.leftmost).se.vruntime
        {
            self.fair.leftmost = task as *mut TaskStruct;
        }
    }

    unsafe fn fair_dequeue(&mut self, task: &mut TaskStruct) {
        self.fair.nr_running -= 1;
        self.fair.h_nr_running -= 1;
        self.fair.load_weight = self.fair.load_weight.saturating_sub(task.se.weight);

        Self::rb_erase(&mut self.fair.root as *mut _, task as *mut TaskStruct);

        if self.fair.leftmost == task as *mut TaskStruct {
            self.fair.leftmost = Self::rb_first(self.fair.root);
        }
        task.se.rb_left = ptr::null_mut();
        task.se.rb_right = ptr::null_mut();
        task.se.rb_parent_color = 0;
    }

    unsafe fn dl_cmp(a: *mut TaskStruct, b: *mut TaskStruct) -> bool {
        if a.is_null() || b.is_null() {
            return false;
        }
        (*a).dl.deadline < (*b).dl.deadline
    }

    unsafe fn dl_enqueue(&mut self, task: &mut TaskStruct) {
        self.dl.dl_nr_running += 1;
        self.dl.running_bw += task.dl.dl_bw;

        Self::rb_insert(
            &mut self.dl.root as *mut _,
            task as *mut TaskStruct,
            Self::dl_cmp,
        );

        if self.dl.leftmost.is_null()
            || (*task).dl.deadline < (*self.dl.leftmost).dl.deadline
        {
            self.dl.leftmost = task as *mut TaskStruct;
            self.dl.earliest_dl = (*task).dl.deadline;
        }
    }

    unsafe fn dl_dequeue(&mut self, task: &mut TaskStruct) {
        self.dl.dl_nr_running -= 1;
        if task.dl.dl_period > 0 {
            self.dl.running_bw = self.dl.running_bw.saturating_sub(
                task.dl.dl_runtime / task.dl.dl_period,
            );
        }

        Self::rb_erase(&mut self.dl.root as *mut _, task as *mut TaskStruct);

        if self.dl.leftmost == task as *mut TaskStruct {
            self.dl.leftmost = Self::rb_first(self.dl.root);
            self.dl.earliest_dl = if self.dl.leftmost.is_null() {
                u64::MAX
            } else {
                (*self.dl.leftmost).dl.deadline
            };
        }
        task.se.rb_left = ptr::null_mut();
        task.se.rb_right = ptr::null_mut();
        task.se.rb_parent_color = 0;
    }

    unsafe fn stop_enqueue(&mut self, task: &mut TaskStruct) {
        task.se.run_list = self.stop;
        self.stop = task as *mut TaskStruct;
    }

    unsafe fn stop_dequeue(&mut self, task: &mut TaskStruct) {
        let mut cur = self.stop;
        let mut prev: *mut TaskStruct = ptr::null_mut();
        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.stop = (*cur).se.run_list;
                } else {
                    (*prev).se.run_list = (*cur).se.run_list;
                }
                task.se.run_list = ptr::null_mut();
                return;
            }
            prev = cur;
            cur = (*cur).se.run_list;
        }
    }

    unsafe fn idle_enqueue(&mut self, task: &mut TaskStruct) {
        task.se.run_list = self.idle;
        self.idle = task as *mut TaskStruct;
    }

    unsafe fn idle_dequeue(&mut self, task: &mut TaskStruct) {
        let mut cur = self.idle;
        let mut prev: *mut TaskStruct = ptr::null_mut();
        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.idle = (*cur).se.run_list;
                } else {
                    (*prev).se.run_list = (*cur).se.run_list;
                }
                task.se.run_list = ptr::null_mut();
                return;
            }
            prev = cur;
            cur = (*cur).se.run_list;
        }
    }

    pub unsafe fn pick_next_task(&mut self) -> *mut TaskStruct {
        if self.is_empty() {
            return self.idle;
        }

        if !self.stop.is_null() {
            return self.stop;
        }

        if self.dl.dl_nr_running > 0 {
            return self.dl.leftmost;
        }

        if self.rt.nr_running > 0 {
            let prio = self.rt.hprio;
            if prio < MAX_RT_PRIO as usize {
                return self.rt.queue[prio];
            }
        }

        if self.fair.nr_running > 0 {
            return self.fair.leftmost;
        }

        self.idle
    }

    pub unsafe fn update_curr(&mut self, delta_exec: u64) {
        if self.curr.is_null() {
            return;
        }
        let curr = &mut *self.curr;

        self.clock += delta_exec;
        self.clock_task += delta_exec;

        curr.charge_cputime(delta_exec);

        if curr.sched_class == SchedClass::Fair {
            let delta_vruntime = (delta_exec * NICE_0_LOAD) / curr.se.weight;
            curr.se.vruntime = curr.se.vruntime.saturating_add(delta_vruntime);
            if curr.se.vruntime > self.fair.min_vruntime {
                self.fair.min_vruntime = curr.se.vruntime;
            }
        }

        if curr.sched_class == SchedClass::Deadline {
            curr.dl.runtime -= delta_exec as i64;
            if curr.dl.runtime <= 0 {
                curr.dl.throttled = true;
            }
        }
    }

    pub unsafe fn check_preempt_curr(&mut self, task: *mut TaskStruct, _flags: u32) -> bool {
        if self.curr.is_null() || task.is_null() {
            return false;
        }
        let curr = &*self.curr;
        let new = &*task;

        if (new.sched_class as u8) > (curr.sched_class as u8) {
            return true;
        }
        match new.sched_class {
            SchedClass::RealTime => new.rt.rt_priority < curr.rt.rt_priority,
            SchedClass::Fair => new.se.vruntime < curr.se.vruntime,
            SchedClass::Deadline => new.dl.deadline < curr.dl.deadline,
            _ => false,
        }
    }

    pub unsafe fn resched_curr(&mut self) {
        if !self.curr.is_null() {
        }
    }
}

unsafe impl Send for RunQueue {}
unsafe impl Sync for RunQueue {}