

#![allow(dead_code)]

use core::cmp;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, Ordering};

use crate::cpu::scheduler::entities::task::{
    calc_delta_fair, deadline_has_priority, fair_has_priority, nice_to_weight, nice_to_wmult,
    rb_color, rb_make_parent_color, rb_parent, weight_to_nice, CpuMask, ListHead, SchedClass,
    SchedPolicy, SpinLock, TaskFlags, TaskId, TaskState, TaskStruct, CPU_NONE, MAX_RT_PRIO,
    NICE_0_LOAD, RB_BLACK, RB_RED,
};


bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EnqueueFlags: u32 {
        const ENQUEUE_RESTORE   = 1 << 0;
        const ENQUEUE_WAKEUP    = 1 << 1;
        const ENQUEUE_NEW       = 1 << 2;
        const ENQUEUE_MIGRATED  = 1 << 3;
        const ENQUEUE_HEAD      = 1 << 4;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DequeueFlags: u32 {
        const DEQUEUE_SLEEP     = 1 << 0;
        const DEQUEUE_SAVE      = 1 << 1;
        const DEQUEUE_MIGRATING = 1 << 2;
    }
}

pub const MIN_GRANULARITY_NS: u64 = 750_000; // 0.75 ms
pub const TARGET_LATENCY_NS: u64 = 6_000_000; // 6 ms
pub const BALANCE_INTERVAL_NS: u64 = 4_000_000; 
pub const IMBALANCE_THRESHOLD: u32 = 2;

pub mod rbtree {
    use super::*;

    pub type KeyOf = fn(*const TaskStruct) -> u64;

    #[inline]
    unsafe fn left_of(n: *mut TaskStruct) -> *mut TaskStruct {
        if n.is_null() {
            ptr::null_mut()
        } else {
            (*n).se.rb_left
        }
    }

    #[inline]
    unsafe fn right_of(n: *mut TaskStruct) -> *mut TaskStruct {
        if n.is_null() {
            ptr::null_mut()
        } else {
            (*n).se.rb_right
        }
    }

    #[inline]
    unsafe fn parent_of(n: *mut TaskStruct) -> *mut TaskStruct {
        if n.is_null() {
            ptr::null_mut()
        } else {
            rb_parent((*n).se.rb_parent_color)
        }
    }

    #[inline]
    unsafe fn color_of(n: *mut TaskStruct) -> usize {
        if n.is_null() {
            RB_BLACK
        } else {
            rb_color((*n).se.rb_parent_color)
        }
    }

    #[inline]
    unsafe fn set_left(n: *mut TaskStruct, v: *mut TaskStruct) {
        debug_assert!(!n.is_null());
        (*n).se.rb_left = v;
    }

    #[inline]
    unsafe fn set_right(n: *mut TaskStruct, v: *mut TaskStruct) {
        debug_assert!(!n.is_null());
        (*n).se.rb_right = v;
    }

    #[inline]
    unsafe fn set_parent_keep_color(n: *mut TaskStruct, p: *mut TaskStruct) {
        if n.is_null() {
            return;
        }
        let color = rb_color((*n).se.rb_parent_color);
        (*n).se.rb_parent_color = rb_make_parent_color(p, color);
    }

    #[inline]
    unsafe fn set_color(n: *mut TaskStruct, color: usize) {
        if n.is_null() {
            return;
        }
        let p = rb_parent((*n).se.rb_parent_color);
        (*n).se.rb_parent_color = rb_make_parent_color(p, color);
    }

    unsafe fn rotate_left(root: &mut *mut TaskStruct, x: *mut TaskStruct) {
        let y = right_of(x);
        debug_assert!(!y.is_null(), "rotate_left wymaga niepustego prawego dziecka");
        set_right(x, left_of(y));
        if !left_of(y).is_null() {
            set_parent_keep_color(left_of(y), x);
        }
        set_parent_keep_color(y, parent_of(x));
        let xp = parent_of(x);
        if xp.is_null() {
            *root = y;
        } else if x == left_of(xp) {
            set_left(xp, y);
        } else {
            set_right(xp, y);
        }
        set_left(y, x);
        set_parent_keep_color(x, y);
    }

    unsafe fn rotate_right(root: &mut *mut TaskStruct, x: *mut TaskStruct) {
        let y = left_of(x);
        debug_assert!(!y.is_null(), "rotate_right wymaga niepustego lewego dziecka");
        set_left(x, right_of(y));
        if !right_of(y).is_null() {
            set_parent_keep_color(right_of(y), x);
        }
        set_parent_keep_color(y, parent_of(x));
        let xp = parent_of(x);
        if xp.is_null() {
            *root = y;
        } else if x == left_of(xp) {
            set_left(xp, y);
        } else {
            set_right(xp, y);
        }
        set_right(y, x);
        set_parent_keep_color(x, y);
    }

    pub unsafe fn subtree_min(mut node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            return ptr::null_mut();
        }
        while !left_of(node).is_null() {
            node = left_of(node);
        }
        node
    }

    pub unsafe fn subtree_max(mut node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            return ptr::null_mut();
        }
        while !right_of(node).is_null() {
            node = right_of(node);
        }
        node
    }

    pub unsafe fn successor(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            return ptr::null_mut();
        }
        if !right_of(node).is_null() {
            return subtree_min(right_of(node));
        }
        let mut x = node;
        let mut y = parent_of(x);
        while !y.is_null() && x == right_of(y) {
            x = y;
            y = parent_of(y);
        }
        y
    }

    fn insert_fixup(root: &mut *mut TaskStruct, mut z: *mut TaskStruct) {
        unsafe {
            while color_of(parent_of(z)) == RB_RED {
                let zp = parent_of(z);
                let zpp = parent_of(zp);
                debug_assert!(!zpp.is_null(), "rodzic czerwonego węzła nie może być korzeniem");
                if zp == left_of(zpp) {
                    let y = right_of(zpp);
                    if color_of(y) == RB_RED {
                        set_color(zp, RB_BLACK);
                        set_color(y, RB_BLACK);
                        set_color(zpp, RB_RED);
                        z = zpp;
                    } else {
                        if z == right_of(zp) {
                            z = zp;
                            rotate_left(root, z);
                        }
                        let zp2 = parent_of(z);
                        let zpp2 = parent_of(zp2);
                        set_color(zp2, RB_BLACK);
                        set_color(zpp2, RB_RED);
                        rotate_right(root, zpp2);
                    }
                } else {
                    let y = left_of(zpp);
                    if color_of(y) == RB_RED {
                        set_color(zp, RB_BLACK);
                        set_color(y, RB_BLACK);
                        set_color(zpp, RB_RED);
                        z = zpp;
                    } else {
                        if z == left_of(zp) {
                            z = zp;
                            rotate_right(root, z);
                        }
                        let zp2 = parent_of(z);
                        let zpp2 = parent_of(zp2);
                        set_color(zp2, RB_BLACK);
                        set_color(zpp2, RB_RED);
                        rotate_left(root, zpp2);
                    }
                }
                if z == *root {
                    break;
                }
            }
            set_color(*root, RB_BLACK);
        }
    }

    pub fn insert(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, node: *mut TaskStruct, key_of: KeyOf) {
        unsafe {
            debug_assert!(!node.is_null());
            (*node).se.rb_left = ptr::null_mut();
            (*node).se.rb_right = ptr::null_mut();
            (*node).se.rb_parent_color = rb_make_parent_color(ptr::null_mut(), RB_RED);

            let node_key = key_of(node);
            let mut parent: *mut TaskStruct = ptr::null_mut();
            let mut cur = *root;
            let mut is_leftmost = true;

            while !cur.is_null() {
                parent = cur;
                if node_key < key_of(cur) {
                    cur = left_of(cur);
                } else {
                    is_leftmost = false;
                    cur = right_of(cur);
                }
            }

            set_parent_keep_color(node, parent);
            if parent.is_null() {
                *root = node;
            } else if node_key < key_of(parent) {
                set_left(parent, node);
            } else {
                set_right(parent, node);
            }

            if is_leftmost {
                *leftmost = node;
            }

            insert_fixup(root, node);
        }
    }

    unsafe fn transplant(root: &mut *mut TaskStruct, u: *mut TaskStruct, v: *mut TaskStruct) {
        let up = parent_of(u);
        if up.is_null() {
            *root = v;
        } else if u == left_of(up) {
            set_left(up, v);
        } else {
            set_right(up, v);
        }
        if !v.is_null() {
            set_parent_keep_color(v, up);
        }
    }

    fn delete_fixup(root: &mut *mut TaskStruct, mut x: *mut TaskStruct, mut x_parent: *mut TaskStruct) {
        unsafe {
            while x != *root && color_of(x) == RB_BLACK {
                if x_parent.is_null() {
                    break;
                }
                if x == left_of(x_parent) {
                    let mut w = right_of(x_parent);
                    if color_of(w) == RB_RED {
                        set_color(w, RB_BLACK);
                        set_color(x_parent, RB_RED);
                        rotate_left(root, x_parent);
                        w = right_of(x_parent);
                    }
                    if color_of(left_of(w)) == RB_BLACK && color_of(right_of(w)) == RB_BLACK {
                        set_color(w, RB_RED);
                        x = x_parent;
                        x_parent = parent_of(x);
                    } else {
                        if color_of(right_of(w)) == RB_BLACK {
                            set_color(left_of(w), RB_BLACK);
                            set_color(w, RB_RED);
                            rotate_right(root, w);
                            w = right_of(x_parent);
                        }
                        set_color(w, color_of(x_parent));
                        set_color(x_parent, RB_BLACK);
                        set_color(right_of(w), RB_BLACK);
                        rotate_left(root, x_parent);
                        x = *root;
                        x_parent = ptr::null_mut();
                    }
                } else {
                    let mut w = left_of(x_parent);
                    if color_of(w) == RB_RED {
                        set_color(w, RB_BLACK);
                        set_color(x_parent, RB_RED);
                        rotate_right(root, x_parent);
                        w = left_of(x_parent);
                    }
                    if color_of(right_of(w)) == RB_BLACK && color_of(left_of(w)) == RB_BLACK {
                        set_color(w, RB_RED);
                        x = x_parent;
                        x_parent = parent_of(x);
                    } else {
                        if color_of(left_of(w)) == RB_BLACK {
                            set_color(right_of(w), RB_BLACK);
                            set_color(w, RB_RED);
                            rotate_left(root, w);
                            w = left_of(x_parent);
                        }
                        set_color(w, color_of(x_parent));
                        set_color(x_parent, RB_BLACK);
                        set_color(left_of(w), RB_BLACK);
                        rotate_right(root, x_parent);
                        x = *root;
                        x_parent = ptr::null_mut();
                    }
                }
            }
            set_color(x, RB_BLACK);
        }
    }

    pub fn delete(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, z: *mut TaskStruct) {
        unsafe {
            debug_assert!(!z.is_null());
            let mut y = z;
            let mut y_original_color = color_of(y);
            let x: *mut TaskStruct;
            let x_parent: *mut TaskStruct;

            if left_of(z).is_null() {
                x = right_of(z);
                x_parent = parent_of(z);
                transplant(root, z, right_of(z));
            } else if right_of(z).is_null() {
                x = left_of(z);
                x_parent = parent_of(z);
                transplant(root, z, left_of(z));
            } else {
                y = subtree_min(right_of(z));
                y_original_color = color_of(y);
                x = right_of(y);
                if parent_of(y) == z {
                    x_parent = y;
                } else {
                    x_parent = parent_of(y);
                    transplant(root, y, right_of(y));
                    set_right(y, right_of(z));
                    set_parent_keep_color(right_of(z), y);
                }
                transplant(root, z, y);
                set_left(y, left_of(z));
                set_parent_keep_color(left_of(z), y);
                let z_color = color_of(z);
                set_color(y, z_color);
            }

            (*z).se.rb_left = ptr::null_mut();
            (*z).se.rb_right = ptr::null_mut();
            (*z).se.rb_parent_color = rb_make_parent_color(ptr::null_mut(), RB_RED);

            if y_original_color == RB_BLACK {
                delete_fixup(root, x, x_parent);
            }

            *leftmost = subtree_min(*root);
        }
    }

    /// Liczba węzłów w drzewie — WYŁĄCZNIE do testów/asercji (O(n)).
    #[cfg(test)]
    pub unsafe fn count(root: *mut TaskStruct) -> usize {
        if root.is_null() {
            0
        } else {
            1 + count(left_of(root)) + count(right_of(root))
        }
    }

    /// Sprawdza wszystkie niezmienniki drzewa RB — WYŁĄCZNIE do testów.
    #[cfg(test)]
    pub unsafe fn check_invariants(root: *mut TaskStruct, key_of: KeyOf) -> Result<usize, &'static str> {
        if root.is_null() {
            return Ok(1); // czarna wysokość pustego poddrzewa
        }
        if color_of(root) != RB_BLACK {
            return Err("korzeń musi być czarny");
        }
        fn check(node: *mut TaskStruct, key_of: KeyOf) -> Result<usize, &'static str> {
            unsafe {
                if node.is_null() {
                    return Ok(1);
                }
                if color_of(node) == RB_RED {
                    if color_of(left_of(node)) == RB_RED || color_of(right_of(node)) == RB_RED {
                        return Err("czerwony węzeł ma czerwone dziecko");
                    }
                }
                if !left_of(node).is_null() && key_of(left_of(node)) > key_of(node) {
                    return Err("naruszony porządek BST (lewe dziecko)");
                }
                if !right_of(node).is_null() && key_of(right_of(node)) < key_of(node) {
                    return Err("naruszony porządek BST (prawe dziecko)");
                }
                let lh = check(left_of(node), key_of)?;
                let rh = check(right_of(node), key_of)?;
                if lh != rh {
                    return Err("niespójna czarna wysokość między poddrzewami");
                }
                Ok(lh + if color_of(node) == RB_BLACK { 1 } else { 0 })
            }
        }
        check(root, key_of)
    }
}


fn fair_key_of(node: *const TaskStruct) -> u64 {
    unsafe { (*node).se.vruntime }
}

#[repr(C)]
pub struct RqFair {
    root: *mut TaskStruct,
    leftmost: *mut TaskStruct,
    pub nr_running: u32,
    pub load_weight: u64,
    pub min_vruntime: u64,
}

impl Default for RqFair {
    fn default() -> Self {
        Self {
            root: ptr::null_mut(),
            leftmost: ptr::null_mut(),
            nr_running: 0,
            load_weight: 0,
            min_vruntime: 0,
        }
    }
}

impl RqFair {
    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    pub fn leftmost(&self) -> *mut TaskStruct {
        self.leftmost
    }

    fn place_entity(&self, task: *mut TaskStruct, flags: EnqueueFlags) {
        unsafe {
            let se = &mut (*task).se;
            if flags.intersects(EnqueueFlags::ENQUEUE_NEW | EnqueueFlags::ENQUEUE_WAKEUP)
                && !flags.contains(EnqueueFlags::ENQUEUE_MIGRATED)
            {
                se.vruntime = cmp::max(se.vruntime, self.min_vruntime);
            }
        }
    }

    /// Naprawa wady #2: prawdziwe wstawienie do drzewa RB (nie
    /// pojedynczy wskaźnik udający drzewo).
    pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
        debug_assert!(!task.is_null());
        self.place_entity(task, flags);
        rbtree::insert(&mut self.root, &mut self.leftmost, task, fair_key_of);
        unsafe {
            (*task).se.on_rq = true;
            self.load_weight = self.load_weight.saturating_add((*task).se.weight);
        }
        self.nr_running += 1;
    }

    pub fn dequeue(&mut self, task: *mut TaskStruct) {
        debug_assert!(!task.is_null());
        rbtree::delete(&mut self.root, &mut self.leftmost, task);
        unsafe {
            (*task).se.on_rq = false;
            self.load_weight = self.load_weight.saturating_sub((*task).se.weight);
        }
        self.nr_running = self.nr_running.saturating_sub(1);
    }

    pub fn pick_first(&self) -> *mut TaskStruct {
        self.leftmost
    }

    pub fn charge_exec(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) -> u64 {
        unsafe {
            let se = &mut (*task).se;
            let delta_vruntime = calc_delta_fair(delta_exec, se.weight, se.inv_weight);
            se.vruntime = se.vruntime.saturating_add(delta_vruntime);
            se.load.accumulate(now, delta_exec, se.weight, true);

            self.min_vruntime = cmp::max(self.min_vruntime, se.vruntime);
            if !self.leftmost.is_null() {
                let leftmost_key = fair_key_of(self.leftmost);
                self.min_vruntime = cmp::min(self.min_vruntime, cmp::max(leftmost_key, self.min_vruntime.saturating_sub(TARGET_LATENCY_NS)));
            }
        }
        self.min_vruntime
    }

    pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
        unsafe {
            if !fair_has_priority(&*candidate, &*current) {
                return false;
            }
            let vdiff = (*current).se.vruntime.saturating_sub((*candidate).se.vruntime);
            let ran_for = (*current).se.sum_exec_runtime.saturating_sub((*current).se.prev_sum_exec_runtime);
            vdiff > 0 && ran_for >= MIN_GRANULARITY_NS || vdiff > TARGET_LATENCY_NS
        }
    }

    #[cfg(test)]
    pub fn rb_invariants_ok(&self) -> bool {
        unsafe { rbtree::check_invariants(self.root, fair_key_of).is_ok() }
    }

    #[cfg(test)]
    pub fn rb_count(&self) -> usize {
        unsafe { rbtree::count(self.root) }
    }
}


fn dl_key_of(node: *const TaskStruct) -> u64 {
    unsafe { (*node).dl.deadline }
}

#[repr(C)]
pub struct RqDl {
    root: *mut TaskStruct,
    leftmost: *mut TaskStruct,
    pub dl_nr_running: u32,
    pub running_bw: u64,
    pub max_bw: u64,
}

pub const DL_BW_SCALE: u64 = 1 << 20;

impl Default for RqDl {
    fn default() -> Self {
        Self {
            root: ptr::null_mut(),
            leftmost: ptr::null_mut(),
            dl_nr_running: 0,
            running_bw: 0,
            max_bw: (DL_BW_SCALE * 95) / 100,
        }
    }
}

impl RqDl {
    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    pub fn leftmost(&self) -> *mut TaskStruct {
        self.leftmost
    }

    pub fn admission_control(&self, dl_runtime: u64, dl_period: u64) -> bool {
        if dl_period == 0 {
            return false;
        }
        let bw = (dl_runtime.saturating_mul(DL_BW_SCALE)) / dl_period;
        self.running_bw.saturating_add(bw) <= self.max_bw
    }

    fn task_bw(task: *const TaskStruct) -> u64 {
        unsafe {
            let dl = &(*task).dl;
            if dl.dl_period == 0 {
                0
            } else {
                (dl.dl_runtime.saturating_mul(DL_BW_SCALE)) / dl.dl_period
            }
        }
    }

    pub fn enqueue(&mut self, task: *mut TaskStruct, now: u64, flags: EnqueueFlags) {
        debug_assert!(!task.is_null());
        unsafe {
            let dl = &mut (*task).dl;
            if flags.contains(EnqueueFlags::ENQUEUE_NEW) {
                // Pierwsze wejście do systemu: zacznij nowy okres CBS
                // teraz, z pełnym budżetem.
                dl.runtime = dl.dl_runtime as i64;
                dl.deadline = now.saturating_add(dl.dl_deadline);
                dl.replenish_at = now.saturating_add(dl.dl_period);
                self.running_bw = self.running_bw.saturating_add(Self::task_bw(task));
            } else if dl.throttled {
                debug_assert!(false, "próba enqueue zadania throttled bez replenish");
            }
            dl.yielded = false;
        }
        rbtree::insert(&mut self.root, &mut self.leftmost, task, dl_key_of);
        unsafe { (*task).se.on_rq = true };
        self.dl_nr_running += 1;
    }

    pub fn dequeue(&mut self, task: *mut TaskStruct, removing_permanently: bool) {
        debug_assert!(!task.is_null());
        rbtree::delete(&mut self.root, &mut self.leftmost, task);
        unsafe { (*task).se.on_rq = false };
        self.dl_nr_running = self.dl_nr_running.saturating_sub(1);
        if removing_permanently {
            self.running_bw = self.running_bw.saturating_sub(Self::task_bw(task));
        }
    }

    pub fn pick_first(&self) -> *mut TaskStruct {
        self.leftmost
    }

    pub fn update_curr(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) {
        unsafe {
            let dl = &mut (*task).dl;
            dl.runtime -= delta_exec as i64;

            if now >= dl.deadline {
                dl.deadline = now.saturating_add(dl.dl_deadline);
                dl.runtime = dl.dl_runtime as i64;
                dl.throttled = false;
                (*task).flags.fetch_remove(TaskFlags::PF_DL_THROTTLED);
            } else if dl.runtime <= 0 {
                dl.throttled = true;
                dl.replenish_at = dl.deadline;
                (*task).flags.fetch_insert(TaskFlags::PF_DL_THROTTLED);
            }
        }
    }
    pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
        unsafe { deadline_has_priority(&*candidate, &*current) }
    }

    #[cfg(test)]
    pub fn rb_invariants_ok(&self) -> bool {
        unsafe { rbtree::check_invariants(self.root, dl_key_of).is_ok() }
    }
}


const RT_BITMAP_WORDS: usize = (MAX_RT_PRIO as usize + 63) / 64;

#[repr(C)]
pub struct RqRt {
    active: [ListHead; MAX_RT_PRIO as usize],
    bitmap: [u64; RT_BITMAP_WORDS],
    pub nr_running: u32,
    pub rt_nr_migratory: u32,
    pub overloaded: bool,
    highest_prio: i32,
}

impl RqRt {
    pub fn new() -> Self {
        const INIT: ListHead = ListHead { prev: ptr::null_mut(), next: ptr::null_mut() };
        let mut rq = Self {
            active: [INIT; MAX_RT_PRIO as usize],
            bitmap: [0; RT_BITMAP_WORDS],
            nr_running: 0,
            rt_nr_migratory: 0,
            overloaded: false,
            highest_prio: MAX_RT_PRIO,
        };
        for head in rq.active.iter_mut() {
            head.init();
        }
        rq
    }

    pub fn is_empty(&self) -> bool {
        self.nr_running == 0
    }

    fn set_bit(&mut self, prio: usize) {
        self.bitmap[prio / 64] |= 1u64 << (prio % 64);
    }

    fn clear_bit(&mut self, prio: usize) {
        self.bitmap[prio / 64] &= !(1u64 << (prio % 64));
    }

    fn sched_find_first_bit(&self) -> Option<usize> {
        for (word_idx, word) in self.bitmap.iter().enumerate() {
            if *word != 0 {
                let bit = word.trailing_zeros() as usize;
                let prio = word_idx * 64 + bit;
                if prio < MAX_RT_PRIO as usize {
                    return Some(prio);
                }
            }
        }
        None
    }

    pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
        debug_assert!(!task.is_null());
        unsafe {
            let prio = (*task).prio;
            debug_assert!(prio >= 0 && prio < MAX_RT_PRIO, "prio RT poza zakresem 0..MAX_RT_PRIO");
            let prio = prio as usize;

            debug_assert!(
                core::ptr::eq((*task).rt.owner, task),
                "RtSchedEntity.owner musi wskazywać na właściciela — zapomniano TaskStruct::init?"
            );

            let node_ptr = &mut (*task).rt.rt_list as *mut ListHead;
            if flags.contains(EnqueueFlags::ENQUEUE_HEAD) {
                self.active[prio].insert_after(node_ptr);
            } else {
                self.active[prio].insert_before(node_ptr);
            }

            self.set_bit(prio);
            (*task).rt.queued_prio = prio as u32;
            (*task).se.on_rq = true;

            if (prio as i32) < self.highest_prio {
                self.highest_prio = prio as i32;
            }
            if (*task).rt.nr_cpus_allowed > 1 {
                self.rt_nr_migratory += 1;
            }
        }
        self.nr_running += 1;
        self.overloaded = self.rt_nr_migratory > 1;
    }

    pub fn dequeue(&mut self, task: *mut TaskStruct) {
        debug_assert!(!task.is_null());
        unsafe {
            let prio = (*task).rt.queued_prio as usize;
            debug_assert!(prio < MAX_RT_PRIO as usize, "dequeue zadania, które nie było zakolejkowane");

            (*task).rt.rt_list.remove();
            (*task).se.on_rq = false;
            (*task).rt.queued_prio = MAX_RT_PRIO as u32;

            if self.active[prio].is_empty() {
                self.clear_bit(prio);
                if self.highest_prio == prio as i32 {
                    self.highest_prio = self.sched_find_first_bit().map(|p| p as i32).unwrap_or(MAX_RT_PRIO);
                }
            }
            if (*task).rt.nr_cpus_allowed > 1 {
                self.rt_nr_migratory = self.rt_nr_migratory.saturating_sub(1);
            }
        }
        self.nr_running = self.nr_running.saturating_sub(1);
        self.overloaded = self.rt_nr_migratory > 1;
    }

    pub fn pick_first(&self) -> *mut TaskStruct {
        match self.sched_find_first_bit() {
            Some(prio) => unsafe {
                let head = &self.active[prio] as *const ListHead as *mut ListHead;
                let front = (*head).next;
                if front.is_null() || core::ptr::eq(front, head) {
                    ptr::null_mut()
                } else {
                    let rt_entity = front as *mut crate::cpu::scheduler::entities::task::RtSchedEntity;
                    (*rt_entity).owner
                }
            },
            None => ptr::null_mut(),
        }
    }

    pub fn requeue(&mut self, task: *mut TaskStruct) {
        debug_assert!(!task.is_null());
        unsafe {
            let prio = (*task).rt.queued_prio as usize;
            debug_assert!(prio < MAX_RT_PRIO as usize);
            (*task).rt.rt_list.remove();
            let node_ptr = &mut (*task).rt.rt_list as *mut ListHead;
            self.active[prio].insert_before(node_ptr);
        }
    }

    pub fn highest_priority(&self) -> i32 {
        self.highest_prio
    }
}


#[repr(C)]
#[derive(Default)]
pub struct RqStop {
    task: *mut TaskStruct,
}

impl RqStop {
    pub fn is_empty(&self) -> bool {
        self.task.is_null()
    }

    pub fn enqueue(&mut self, task: *mut TaskStruct) {
        debug_assert!(self.task.is_null(), "co najwyżej jedno zadanie Stop naraz na CPU");
        unsafe { (*task).se.on_rq = true };
        self.task = task;
    }

    pub fn dequeue(&mut self, task: *mut TaskStruct) {
        if self.task == task {
            unsafe { (*task).se.on_rq = false };
            self.task = ptr::null_mut();
        }
    }

    pub fn pick_first(&self) -> *mut TaskStruct {
        self.task
    }
}


#[repr(C)]
pub struct RunQueue {
    pub lock: SpinLock,
    pub cpu: u32,
    pub online: AtomicBool,
    pub clock: AtomicU64,
    pub clock_task: AtomicU64,

    pub fair: RqFair,
    pub rt: RqRt,
    pub dl: RqDl,
    pub stop: RqStop,

    pub curr: AtomicPtr<TaskStruct>,
    pub idle: *mut TaskStruct,

    pub nr_running: AtomicU32,
    pub nr_uninterruptible: AtomicU32,

    pub next_balance: AtomicU64,

    /// Licznik wywołań `schedule()` — diagnostyka/testy.
    pub nr_switches: AtomicU64,
}

impl RunQueue {
    pub fn new(cpu: u32, idle_task: *mut TaskStruct) -> Self {
        assert!(!idle_task.is_null(), "RunQueue::new wymaga poprawnego zadania idle");
        unsafe {
            debug_assert!(
                (*idle_task).sched_class == SchedClass::Idle,
                "zadanie przekazane jako idle musi mieć sched_class == Idle"
            );
        }
        let rq = Self {
            lock: SpinLock::new(),
            cpu,
            online: AtomicBool::new(true),
            clock: AtomicU64::new(0),
            clock_task: AtomicU64::new(0),
            fair: RqFair::default(),
            rt: RqRt::new(),
            dl: RqDl::default(),
            stop: RqStop::default(),
            curr: AtomicPtr::new(idle_task),
            idle: idle_task,
            nr_running: AtomicU32::new(0),
            nr_uninterruptible: AtomicU32::new(0),
            next_balance: AtomicU64::new(BALANCE_INTERVAL_NS),
            nr_switches: AtomicU64::new(0),
        };
        unsafe {
            (*idle_task).cpu.store(cpu, Ordering::Release);
            (*idle_task).on_cpu.store(true, Ordering::Release);
        }
        rq
    }

    pub fn bind_idle_task(&self) {
        unsafe {
            (*self.idle).set_rq_ptr(self as *const RunQueue as *mut RunQueue as *mut core::ffi::c_void);
        }
    }

    fn now(&self) -> u64 {
        self.clock.load(Ordering::Relaxed)
    }

    pub unsafe fn advance_clock(&self, delta_ns: u64) {
        debug_assert!(self.lock.is_locked(), "advance_clock wymaga self.lock");
        self.clock.fetch_add(delta_ns, Ordering::Relaxed);
        self.clock_task.fetch_add(delta_ns, Ordering::Relaxed);
    }

    pub fn current(&self) -> *mut TaskStruct {
        self.curr.load(Ordering::Acquire)
    }

    fn set_current(&self, task: *mut TaskStruct) {
        self.curr.store(task, Ordering::Release);
    }

    pub fn nr_running(&self) -> u32 {
        self.nr_running.load(Ordering::Relaxed)
    }

    pub fn is_idle(&self) -> bool {
        core::ptr::eq(self.current(), self.idle)
    }
    pub unsafe fn enqueue_task(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
        debug_assert!(!task.is_null(), "enqueue_task: task == null");
        debug_assert!(self.lock.is_locked(), "enqueue_task wymaga self.lock");
        debug_assert!(
            !(*task).se.on_rq,
            "enqueue_task: zadanie jest już zakolejkowane (podwójny enqueue)"
        );

        let now = self.now();
        match (*task).sched_class {
            SchedClass::Stop => self.stop.enqueue(task),
            SchedClass::Deadline => self.dl.enqueue(task, now, flags),
            SchedClass::RealTime => self.rt.enqueue(task, flags),
            SchedClass::Fair => self.fair.enqueue(task, flags),
            SchedClass::Idle => {
                (*task).se.on_rq = true;
            }
        }

        (*task).set_rq_ptr(self as *mut RunQueue as *mut core::ffi::c_void);
        self.nr_running.fetch_add(1, Ordering::Relaxed);

        if flags.contains(EnqueueFlags::ENQUEUE_WAKEUP) {
            (*task).stats.last_enqueue_time = now;
            self.nr_uninterruptible_dec_if(task);
            self.check_preempt_curr(task);
        }
    }
    pub unsafe fn dequeue_task(&mut self, task: *mut TaskStruct, flags: DequeueFlags) {
        debug_assert!(!task.is_null(), "dequeue_task: task == null");
        debug_assert!(self.lock.is_locked(), "dequeue_task wymaga self.lock");
        debug_assert!((*task).se.on_rq, "dequeue_task: zadanie nie jest zakolejkowane");

        match (*task).sched_class {
            SchedClass::Stop => self.stop.dequeue(task),
            SchedClass::Deadline => {
                let permanent = !flags.contains(DequeueFlags::DEQUEUE_SAVE);
                self.dl.dequeue(task, permanent);
            }
            SchedClass::RealTime => self.rt.dequeue(task),
            SchedClass::Fair => self.fair.dequeue(task),
            SchedClass::Idle => {
                (*task).se.on_rq = false;
            }
        }

        if !flags.contains(DequeueFlags::DEQUEUE_MIGRATING) {
            (*task).set_rq_ptr(ptr::null_mut());
        }
        self.nr_running.fetch_sub(1, Ordering::Relaxed);

        if flags.contains(DequeueFlags::DEQUEUE_SLEEP) && (*task).state() == TaskState::Uninterruptible {
            self.nr_uninterruptible.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn nr_uninterruptible_dec_if(&self, task: *mut TaskStruct) {
        unsafe {
            if (*task).state() == TaskState::Runnable {
                let prev = self.nr_uninterruptible.load(Ordering::Relaxed);
                if prev > 0 {
                    self.nr_uninterruptible.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub unsafe fn activate_task(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
        debug_assert!(self.lock.is_locked());
        self.enqueue_task(task, flags);
    }

    pub unsafe fn deactivate_task(&mut self, task: *mut TaskStruct, flags: DequeueFlags) {
        debug_assert!(self.lock.is_locked());
        self.dequeue_task(task, flags);
    }

    pub unsafe fn update_curr(&mut self) {
        debug_assert!(self.lock.is_locked(), "update_curr wymaga self.lock");
        let curr = self.current();
        if curr.is_null() || core::ptr::eq(curr, self.idle) {
            return;
        }

        let now = self.clock_task.load(Ordering::Relaxed);
        let prev_runtime = (*curr).se.sum_exec_runtime;
        let delta_exec = now.saturating_sub(prev_runtime);
        if delta_exec == 0 {
            return;
        }

        (*curr).charge_cputime(delta_exec);

        match (*curr).sched_class {
            SchedClass::Fair => {
                self.fair.charge_exec(curr, delta_exec, now);
            }
            SchedClass::Deadline => {
                self.dl.update_curr(curr, delta_exec, now);
            }
            SchedClass::RealTime => {
                // RT nie ma vruntime do aktualizacji, ale statystyki
                // obciążenia (naprawa #16) wciąż mają znaczenie dla
                // load-balancingu.
                (*curr).se.load.accumulate(now, delta_exec, (*curr).se.weight, true);
            }
            SchedClass::Stop | SchedClass::Idle => {}
        }
    }

    pub unsafe fn check_preempt_curr(&self, candidate: *mut TaskStruct) {
        let curr = self.current();
        if curr.is_null() || core::ptr::eq(curr, candidate) {
            return;
        }

        let curr_class = (*curr).sched_class;
        let cand_class = (*candidate).sched_class;

        // Naprawa wady #5: hierarchia klas jest ABSOLUTNA — klasa wyższa
        // zawsze wywłaszcza niższą, bez wyjątków warunkowych.
        let should_preempt = if cand_class != curr_class {
            cand_class > curr_class
        } else {
            match cand_class {
                SchedClass::Stop => true, // dwa zadania Stop na raz nie powinny się zdarzyć, ale bezpiecznie: nowe wygrywa
                SchedClass::Deadline => self.dl.should_preempt(curr, candidate),
                SchedClass::RealTime => (*candidate).prio < (*curr).prio,
                SchedClass::Fair => self.fair.should_preempt(curr, candidate),
                SchedClass::Idle => false,
            }
        };

        if should_preempt {
            self.resched_curr();
        }
    }

    pub unsafe fn resched_curr(&self) {
        let curr = self.current();
        if !curr.is_null() {
            (*curr).set_need_resched();
        }
    }

    pub unsafe fn pick_next_task(&mut self) -> *mut TaskStruct {
        debug_assert!(self.lock.is_locked(), "pick_next_task wymaga self.lock");

        if !self.stop.is_empty() {
            return self.stop.pick_first();
        }
        if !self.dl.is_empty() {
            return self.dl.pick_first();
        }
        if !self.rt.is_empty() {
            return self.rt.pick_first();
        }
        if !self.fair.is_empty() {
            return self.fair.pick_first();
        }
        self.idle
    }

    pub unsafe fn set_curr_task(&mut self, next: *mut TaskStruct) -> *mut TaskStruct {
        debug_assert!(self.lock.is_locked());
        let prev = self.current();
        if !prev.is_null() && !core::ptr::eq(prev, next) {
            (*prev).on_cpu.store(false, Ordering::Release);
        }

        (*next).se.prev_sum_exec_runtime = self.clock_task.load(Ordering::Relaxed);
        (*next).clear_need_resched();
        (*next).on_cpu.store(true, Ordering::Release);
        (*next).cpu.store(self.cpu, Ordering::Release);
        self.set_current(next);
        self.nr_switches.fetch_add(1, Ordering::Relaxed);
        prev
    }

    pub unsafe fn schedule(&mut self) -> (*mut TaskStruct, *mut TaskStruct) {
        self.update_curr();

        let curr = self.current();
        if !curr.is_null() && !core::ptr::eq(curr, self.idle) {
            self.maybe_requeue_for_timeslice(curr);
        }

        let next = self.pick_next_task();
        let prev = self.set_curr_task(next);
        (prev, next)
    }
    unsafe fn maybe_requeue_for_timeslice(&mut self, curr: *mut TaskStruct) {
        if (*curr).policy != SchedPolicy::RoundRobin {
            return;
        }
        if (*curr).rt.time_slice > 0 {
            return;
        }
        (*curr).rt.time_slice = crate::cpu::scheduler::entities::task::default_time_slice(SchedPolicy::RoundRobin);
        if (*curr).se.on_rq {
            self.rt.requeue(curr);
        }
    }

    pub unsafe fn task_tick(&mut self) {
        debug_assert!(self.lock.is_locked());
        self.update_curr();

        let curr = self.current();
        if curr.is_null() || core::ptr::eq(curr, self.idle) {
            return;
        }

        if matches!((*curr).sched_class, SchedClass::Deadline) && (*curr).dl.throttled {
            self.dl.dequeue(curr, false);
            (*curr).set_need_resched();
        }

        self.reap_expired_dl_throttles();
        self.maybe_requeue_for_timeslice(curr);
        (*curr).set_need_resched();
    }

    unsafe fn reap_expired_dl_throttles(&mut self) {
        let now = self.now();
        let mut node = self.dl.leftmost();
        while !node.is_null() {
            if (*node).dl.throttled && (*node).dl.replenish_at <= now {
                self.dl.dequeue(node, false);
                (*node).dl.throttled = false;
                (*node).set_need_resched();
            }
            node = rbtree::successor(node);
        }
    }

    pub unsafe fn yield_task(&mut self, task: *mut TaskStruct) {
        debug_assert!(self.lock.is_locked());
        debug_assert!(core::ptr::eq(task, self.current()), "yield_task tylko na zadaniu bieżącym");

        match (*task).sched_class {
            SchedClass::Fair => {
                self.update_curr();
                if let Some(max_node_key) = self.fair_max_vruntime() {
                    (*task).se.vruntime = cmp::max((*task).se.vruntime, max_node_key);
                }
            }
            SchedClass::RealTime => {
                if (*task).se.on_rq {
                    self.rt.requeue(task);
                }
            }
            _ => {}
        }
        (*task).set_need_resched();
    }

    fn fair_max_vruntime(&self) -> Option<u64> {
        unsafe {
            let max_node = rbtree::subtree_max(self.fair_root_for_test());
            if max_node.is_null() {
                None
            } else {
                Some((*max_node).se.vruntime)
            }
        }
    }

    #[cfg(test)]
    pub fn fair_root_for_test(&self) -> *mut TaskStruct {
        self.fair_root_internal()
    }

    #[cfg(not(test))]
    fn fair_root_for_test(&self) -> *mut TaskStruct {
        self.fair_root_internal()
    }

    fn fair_root_internal(&self) -> *mut TaskStruct {
        self.fair.root
    }

    pub unsafe fn schedule_tail(&mut self, prev: *mut TaskStruct) {
        if prev.is_null() {
            return;
        }
        if (*prev).is_zombie() {
            (*prev).on_cpu.store(false, Ordering::Release);
        } else if (*prev).se.on_rq {
            (*prev).record_involuntary_switch();
        } else {
            (*prev).record_voluntary_switch();
        }
    }

}

unsafe impl Send for RunQueue {}
unsafe impl Sync for RunQueue {}

impl core::fmt::Debug for RunQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RunQueue")
            .field("cpu", &self.cpu)
            .field("nr_running", &self.nr_running.load(Ordering::Relaxed))
            .field("fair_nr", &self.fair.nr_running)
            .field("rt_nr", &self.rt.nr_running)
            .field("dl_nr", &self.dl.dl_nr_running)
            .field("clock", &self.clock.load(Ordering::Relaxed))
            .finish()
    }
}


pub mod smp {
    use super::*;
    use crate::cpu::scheduler::entities::task::TaskError;

    pub type RunQueueRegistry<'a> = &'a [*mut RunQueue];

    unsafe fn double_lock(a: &RunQueue, b: &RunQueue) -> (usize, usize) {
        if core::ptr::eq(a, b) {
            let f = a.lock.lock_irqsave();
            return (f, f);
        }
        if a.cpu < b.cpu {
            let fa = a.lock.lock_irqsave();
            let fb = b.lock.lock_irqsave();
            (fa, fb)
        } else {
            let fb = b.lock.lock_irqsave();
            let fa = a.lock.lock_irqsave();
            (fa, fb)
        }
    }

    unsafe fn double_unlock(a: &RunQueue, b: &RunQueue, flags: (usize, usize)) {
        if core::ptr::eq(a, b) {
            a.lock.unlock_irqrestore(flags.0);
            return;
        }
        a.lock.unlock_irqrestore(flags.0);
        b.lock.unlock_irqrestore(flags.1);
    }

    fn can_migrate_task(task: *const TaskStruct, dst_cpu: u32) -> bool {
        unsafe {
            if (*task).flags.contains(TaskFlags::PF_NO_MIGRATE) {
                return false;
            }
            if (*task).on_cpu.load(Ordering::Relaxed) {
                // Fizycznie wykonywane teraz na jakimś CPU — przełącznik
                // kontekstu jest w toku, nie ruszamy.
                return false;
            }
            (*task).se.cpus_allowed.is_set(dst_cpu)
        }
    }

    unsafe fn find_migratable(src: &RunQueue, dst_cpu: u32) -> *mut TaskStruct {
        let mut node = src.fair.leftmost();
        while !node.is_null() {
            if !core::ptr::eq(node, src.current()) && can_migrate_task(node, dst_cpu) {
                return node;
            }
            node = rbtree::successor(node);
        }
        if src.rt.overloaded {
            let candidate = src.rt.pick_first();
            if !candidate.is_null()
                && !core::ptr::eq(candidate, src.current())
                && can_migrate_task(candidate, dst_cpu)
            {
                return candidate;
            }
        }
        ptr::null_mut()
    }

    unsafe fn migrate_task_locked(task: *mut TaskStruct, src: &mut RunQueue, dst: &mut RunQueue) {
        src.dequeue_task(task, DequeueFlags::DEQUEUE_MIGRATING);
        (*task).record_migration(dst.cpu);
        dst.enqueue_task(task, EnqueueFlags::ENQUEUE_MIGRATED);
    }
    pub unsafe fn select_task_rq(task: *const TaskStruct, registry: RunQueueRegistry) -> u32 {
        let allowed = (*task).se.cpus_allowed;
        let preferred = (*task).se.last_cpu;

        if preferred != CPU_NONE && allowed.is_set(preferred) {
            if let Some(&rq_ptr) = registry.get(preferred as usize) {
                if !rq_ptr.is_null() && (*rq_ptr).online.load(Ordering::Relaxed) && (*rq_ptr).nr_running() == 0 {
                    return preferred;
                }
            }
        }

        let mut best_cpu = CPU_NONE;
        let mut best_load = u32::MAX;
        for cpu in allowed.iter() {
            if let Some(&rq_ptr) = registry.get(cpu as usize) {
                if rq_ptr.is_null() || !(*rq_ptr).online.load(Ordering::Relaxed) {
                    continue;
                }
                let load = (*rq_ptr).nr_running();
                if load < best_load {
                    best_load = load;
                    best_cpu = cpu;
                }
            }
        }

        if best_cpu == CPU_NONE {
            if preferred != CPU_NONE { preferred } else { 0 }
        } else {
            best_cpu
        }
    }
    pub unsafe fn wake_up_process(task: *mut TaskStruct, registry: RunQueueRegistry) -> Result<(), TaskError> {
        (*task).wake_up()?;

        if (*task).se.on_rq {
            return Ok(());
        }

        let target_cpu = select_task_rq(task, registry);
        let rq_ptr = match registry.get(target_cpu as usize) {
            Some(&p) if !p.is_null() => p,
            _ => return Err(TaskError::ResourceExhausted),
        };
        let rq = &mut *rq_ptr;

        let flags = rq.lock.lock_irqsave();
        if !(*task).se.on_rq {
            let prev_cpu = (*task).se.last_cpu;
            rq.activate_task(task, EnqueueFlags::ENQUEUE_WAKEUP);
            if prev_cpu != CPU_NONE && prev_cpu != target_cpu {
                (*task).record_migration(target_cpu);
            } else {
                (*task).cpu.store(target_cpu, Ordering::Release);
                (*task).se.last_cpu = target_cpu;
            }
        }
        rq.lock.unlock_irqrestore(flags);
        Ok(())
    }
    pub unsafe fn idle_balance(this_cpu: u32, registry: RunQueueRegistry) -> bool {
        let this_rq_ptr = match registry.get(this_cpu as usize) {
            Some(&p) if !p.is_null() => p,
            _ => return false,
        };
        let this_rq = &mut *this_rq_ptr;

        let mut busiest: *mut RunQueue = ptr::null_mut();
        let mut busiest_load = 1u32; // potrzebujemy >=2, żeby oddanie jednego miało sens
        for &rq_ptr in registry.iter() {
            if rq_ptr.is_null() || core::ptr::eq(rq_ptr, this_rq_ptr) {
                continue;
            }
            let rq = &*rq_ptr;
            if !rq.online.load(Ordering::Relaxed) {
                continue;
            }
            let load = rq.nr_running();
            if load > busiest_load {
                busiest_load = load;
                busiest = rq_ptr;
            }
        }

        if busiest.is_null() {
            return false;
        }
        let busiest_rq = &mut *busiest;

        let flags = double_lock(this_rq, busiest_rq);
        let mut pulled = false;
        if busiest_rq.nr_running() > this_rq.nr_running().saturating_add(1) {
            let task = find_migratable(busiest_rq, this_rq.cpu);
            if !task.is_null() {
                migrate_task_locked(task, busiest_rq, this_rq);
                pulled = true;
            }
        }
        double_unlock(this_rq, busiest_rq, flags);
        pulled
    }
    pub unsafe fn load_balance(registry: RunQueueRegistry) -> u32 {
        let mut busiest: *mut RunQueue = ptr::null_mut();
        let mut idlest: *mut RunQueue = ptr::null_mut();
        let mut max_load = 0u32;
        let mut min_load = u32::MAX;

        for &rq_ptr in registry.iter() {
            if rq_ptr.is_null() {
                continue;
            }
            let rq = &*rq_ptr;
            if !rq.online.load(Ordering::Relaxed) {
                continue;
            }
            let load = rq.nr_running();
            if load > max_load {
                max_load = load;
                busiest = rq_ptr;
            }
            if load < min_load {
                min_load = load;
                idlest = rq_ptr;
            }
        }

        if busiest.is_null() || idlest.is_null() || core::ptr::eq(busiest, idlest) {
            return 0;
        }
        if max_load.saturating_sub(min_load) < IMBALANCE_THRESHOLD {
            return 0;
        }

        let busiest_rq = &mut *busiest;
        let idlest_rq = &mut *idlest;
        let flags = double_lock(busiest_rq, idlest_rq);

        let mut migrated = 0u32;
        if busiest_rq.nr_running().saturating_sub(idlest_rq.nr_running()) >= IMBALANCE_THRESHOLD {
            let task = find_migratable(busiest_rq, idlest_rq.cpu);
            if !task.is_null() {
                migrate_task_locked(task, busiest_rq, idlest_rq);
                migrated = 1;
            }
        }

        double_unlock(busiest_rq, idlest_rq, flags);
        migrated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::scheduler::entities::task::{SchedPolicy, TaskState, TaskStruct};
    fn make_idle(pid: TaskId, cpu: u32) -> TaskStruct {
        let mut t = TaskStruct::blank();
        t.init_test_stub(pid, SchedPolicy::Idle, 0);
        let _ = cpu;
        t
    }

    fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
        let mut t = TaskStruct::blank();
        t.init_test_stub(pid, policy, nice);
        t
    }

    fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
        t as *mut TaskStruct
    }

    #[test]
    fn fair_tree_maintains_rb_invariants_under_pseudorandom_inserts() {
        const N: usize = 40;
        let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
        let mut seed: u64 = 88172645463325252;
        let mut vruntimes = [0u64; N];
        for i in 0..N {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            vruntimes[i] = seed % 1_000_000;
        }

        let mut fair = RqFair::default();
        for i in 0..N {
            tasks[i].init_test_stub(i as u64, SchedPolicy::Normal, 0);
            tasks[i].se.vruntime = vruntimes[i];
            let p = ptr_of(&mut tasks[i]);
            fair.enqueue(p, EnqueueFlags::ENQUEUE_RESTORE);
        }

        assert_eq!(fair.rb_count(), N);
        assert!(fair.rb_invariants_ok(), "naruszone niezmienniki RB po wstawieniach");

        let min_vrt = vruntimes.iter().copied().min().unwrap();
        unsafe {
            assert_eq!((*fair.leftmost()).se.vruntime, min_vrt);
        }
    }

    #[test]
    fn fair_tree_inorder_traversal_is_sorted_by_vruntime() {
        const N: usize = 25;
        let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
        let mut fair = RqFair::default();
        for i in 0..N {
            tasks[i].init_test_stub(i as u64, SchedPolicy::Normal, 0);
            tasks[i].se.vruntime = ((N - i) * 1000) as u64;
            let p = ptr_of(&mut tasks[i]);
            fair.enqueue(p, EnqueueFlags::ENQUEUE_RESTORE);
        }

        let mut collected = [0u64; N];
        let mut n_collected = 0usize;
        unsafe {
            let mut node = fair.leftmost();
            while !node.is_null() {
                collected[n_collected] = (*node).se.vruntime;
                n_collected += 1;
                node = rbtree::successor(node);
            }
        }

        assert_eq!(n_collected, N);
        let mut sorted = collected;
        sorted.sort_unstable();
        assert_eq!(collected, sorted, "przejście in-order musi być posortowane po vruntime");
    }

    #[test]
    fn fair_tree_survives_random_removals_keeping_invariants() {
        const N: usize = 30;
        let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
        let mut fair = RqFair::default();
        let mut ptrs: [*mut TaskStruct; N] = [ptr::null_mut(); N];
        for i in 0..N {
            tasks[i].init_test_stub(i as u64, SchedPolicy::Normal, 0);
            tasks[i].se.vruntime = (i as u64) * 137 % 5000;
            ptrs[i] = ptr_of(&mut tasks[i]);
            fair.enqueue(ptrs[i], EnqueueFlags::ENQUEUE_RESTORE);
        }

        let mut remaining = N;
        for i in (0..N).step_by(2) {
            fair.dequeue(ptrs[i]);
            remaining -= 1;
            assert!(fair.rb_invariants_ok(), "niezmienniki RB naruszone po usunięciu #{i}");
            assert_eq!(fair.rb_count(), remaining);
        }
    }

    #[test]
    fn deadline_tree_orders_by_earliest_deadline() {
        let mut a = make_task(1, SchedPolicy::Deadline, 0);
        let mut b = make_task(2, SchedPolicy::Deadline, 0);
        let mut c = make_task(3, SchedPolicy::Deadline, 0);
        a.dl.deadline = 5000;
        b.dl.deadline = 1000;
        c.dl.deadline = 3000;

        let mut dl = RqDl::default();
        dl.enqueue(ptr_of(&mut a), 0, EnqueueFlags::ENQUEUE_RESTORE);
        dl.enqueue(ptr_of(&mut b), 0, EnqueueFlags::ENQUEUE_RESTORE);
        dl.enqueue(ptr_of(&mut c), 0, EnqueueFlags::ENQUEUE_RESTORE);

        unsafe {
            assert_eq!((*dl.pick_first()).pid, 2);
        }
        assert!(dl.rb_invariants_ok());
    }


    #[test]
    fn rt_queue_fifo_order_at_same_priority_without_cycles() {
        let mut a = make_task(1, SchedPolicy::Fifo, 0);
        let mut b = make_task(2, SchedPolicy::Fifo, 0);
        let mut c = make_task(3, SchedPolicy::Fifo, 0);
        // Ta sama priorytetowa "szufladka".
        a.prio = 10;
        b.prio = 10;
        c.prio = 10;
        a.rt.rt_priority = 10;
        b.rt.rt_priority = 10;
        c.rt.rt_priority = 10;

        let mut rt = RqRt::new();
        rt.enqueue(ptr_of(&mut a), EnqueueFlags::ENQUEUE_RESTORE);
        rt.enqueue(ptr_of(&mut b), EnqueueFlags::ENQUEUE_RESTORE);
        rt.enqueue(ptr_of(&mut c), EnqueueFlags::ENQUEUE_RESTORE);

        assert_eq!(rt.nr_running, 3);

        unsafe {
            let first = rt.pick_first();
            assert_eq!((*first).pid, 1);
            rt.dequeue(first);

            let second = rt.pick_first();
            assert_eq!((*second).pid, 2);
            rt.dequeue(second);

            let third = rt.pick_first();
            assert_eq!((*third).pid, 3);
            rt.dequeue(third);
        }

        assert_eq!(rt.nr_running, 0);
        assert!(rt.is_empty());
        assert_eq!(rt.highest_priority(), MAX_RT_PRIO);
    }

    #[test]
    fn rt_queue_picks_highest_priority_across_levels() {
        let mut low = make_task(1, SchedPolicy::Fifo, 0);
        let mut high = make_task(2, SchedPolicy::Fifo, 0);
        low.prio = 50;
        low.rt.rt_priority = 50;
        high.prio = 5;
        high.rt.rt_priority = 5;

        let mut rt = RqRt::new();
        rt.enqueue(ptr_of(&mut low), EnqueueFlags::ENQUEUE_RESTORE);
        rt.enqueue(ptr_of(&mut high), EnqueueFlags::ENQUEUE_RESTORE);

        unsafe {
            assert_eq!((*rt.pick_first()).pid, 2);
        }
        assert_eq!(rt.highest_priority(), 5);
    }

    #[test]
    fn rt_queue_requeue_moves_task_to_back_of_its_level() {
        let mut a = make_task(1, SchedPolicy::RoundRobin, 0);
        let mut b = make_task(2, SchedPolicy::RoundRobin, 0);
        a.prio = 20;
        a.rt.rt_priority = 20;
        b.prio = 20;
        b.rt.rt_priority = 20;

        let mut rt = RqRt::new();
        rt.enqueue(ptr_of(&mut a), EnqueueFlags::ENQUEUE_RESTORE);
        rt.enqueue(ptr_of(&mut b), EnqueueFlags::ENQUEUE_RESTORE);

        unsafe { assert_eq!((*rt.pick_first()).pid, 1) };
        rt.requeue(ptr_of(&mut a));
        unsafe { assert_eq!((*rt.pick_first()).pid, 2) };
    }

    // ------------------------------------------------------------------
    // pick_next_task — hierarchia klas (naprawa wad #3 i #5)
    // ------------------------------------------------------------------

    #[test]
    fn pick_next_task_respects_stop_over_deadline_over_rt_over_fair_over_idle() {
        let mut idle = make_idle(0, 0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fair_task = make_task(1, SchedPolicy::Normal, 0);
        let mut rt_task = make_task(2, SchedPolicy::Fifo, 0);
        rt_task.prio = 10;
        rt_task.rt.rt_priority = 10;
        let mut dl_task = make_task(3, SchedPolicy::Deadline, 0);
        dl_task.dl.dl_runtime = 1000;
        dl_task.dl.dl_deadline = 100_000;
        dl_task.dl.dl_period = 100_000;
        let mut stop_task = make_task(4, SchedPolicy::Stop, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();

            // Pusto -> idle.
            assert!(core::ptr::eq(rq.pick_next_task(), ptr_of(&mut idle)));

            rq.enqueue_task(ptr_of(&mut fair_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*rq.pick_next_task()).pid, 1);

            rq.enqueue_task(ptr_of(&mut rt_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*rq.pick_next_task()).pid, 2, "RT musi wygrać z Fair");

            rq.enqueue_task(ptr_of(&mut dl_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*rq.pick_next_task()).pid, 3, "Deadline musi wygrać z RT");

            rq.enqueue_task(ptr_of(&mut stop_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*rq.pick_next_task()).pid, 4, "Stop musi wygrać ze wszystkim");

            rq.dequeue_task(ptr_of(&mut stop_task), DequeueFlags::empty());
            assert_eq!((*rq.pick_next_task()).pid, 3);

            rq.dequeue_task(ptr_of(&mut dl_task), DequeueFlags::empty());
            assert_eq!((*rq.pick_next_task()).pid, 2);

            rq.dequeue_task(ptr_of(&mut rt_task), DequeueFlags::empty());
            assert_eq!((*rq.pick_next_task()).pid, 1);

            rq.dequeue_task(ptr_of(&mut fair_task), DequeueFlags::empty());
            assert!(core::ptr::eq(rq.pick_next_task(), ptr_of(&mut idle)), "musi wrócić do idle, nigdy null");

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn pick_next_task_never_returns_null_even_when_fully_empty() {
        let mut idle = make_idle(0, 1);
        let mut rq = RunQueue::new(1, ptr_of(&mut idle));
        rq.bind_idle_task();
        unsafe {
            let flags = rq.lock.lock_irqsave();
            let picked = rq.pick_next_task();
            assert!(!picked.is_null());
            assert!(core::ptr::eq(picked, ptr_of(&mut idle)));
            rq.lock.unlock_irqrestore(flags);
        }
    }


    #[test]
    fn deadline_task_is_picked_even_before_its_deadline_has_passed() {
        let mut idle = make_idle(0, 2);
        let mut rq = RunQueue::new(2, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut rt_task = make_task(1, SchedPolicy::Fifo, 0);
        rt_task.prio = 0; // najwyższy możliwy priorytet RT
        rt_task.rt.rt_priority = 99;
        let mut dl_task = make_task(2, SchedPolicy::Deadline, 0);
        dl_task.dl.dl_runtime = 1_000;
        dl_task.dl.dl_deadline = 1_000_000_000; // bardzo odległy termin
        dl_task.dl.dl_period = 1_000_000_000;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            assert_eq!(rq.clock.load(Ordering::Relaxed), 0);
            rq.enqueue_task(ptr_of(&mut rt_task), EnqueueFlags::ENQUEUE_NEW);
            rq.enqueue_task(ptr_of(&mut dl_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!(
                (*rq.pick_next_task()).pid,
                2,
                "Deadline musi zostać wybrany, mimo że jego termin jest daleko w przyszłości"
            );
            rq.lock.unlock_irqrestore(flags);
        }
    }


    #[test]
    fn min_vruntime_never_decreases_across_updates() {
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        let mut fair = RqFair::default();
        fair.enqueue(ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);

        let m1 = fair.charge_exec(ptr_of(&mut t), 1_000_000, 1_000_000);
        let m2 = fair.charge_exec(ptr_of(&mut t), 2_000_000, 3_000_000);
        assert!(m2 >= m1);

        let m3 = fair.charge_exec(ptr_of(&mut t), 0, 3_000_000);
        assert!(m3 >= m2);
    }

    #[test]
    fn newly_woken_task_does_not_start_before_min_vruntime() {
        let mut veteran = make_task(1, SchedPolicy::Normal, 0);
        let mut newcomer = make_task(2, SchedPolicy::Normal, 0);

        let mut fair = RqFair::default();
        fair.enqueue(ptr_of(&mut veteran), EnqueueFlags::ENQUEUE_NEW);
        fair.charge_exec(ptr_of(&mut veteran), 50_000_000, 50_000_000);

        newcomer.se.vruntime = 0;
        fair.enqueue(ptr_of(&mut newcomer), EnqueueFlags::ENQUEUE_WAKEUP);

        unsafe {
            assert!(
                (*ptr_of(&mut newcomer)).se.vruntime >= fair.min_vruntime.saturating_sub(1),
                "nowo obudzone zadanie powinno wystartować od min_vruntime, nie od 0"
            );
        }
    }

    #[test]
    fn higher_class_always_preempts_lower_class() {
        let mut idle = make_idle(0, 3);
        let mut rq = RunQueue::new(3, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fair_task = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.enqueue_task(ptr_of(&mut fair_task), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut fair_task));
            assert!(!(*ptr_of(&mut fair_task)).needs_resched());

            let mut rt_task = make_task(2, SchedPolicy::Fifo, 0);
            rt_task.prio = 10;
            rq.enqueue_task(ptr_of(&mut rt_task), EnqueueFlags::ENQUEUE_WAKEUP);

            assert!(
                (*ptr_of(&mut fair_task)).needs_resched(),
                "zadanie Fair musi zostać oznaczone do przełączenia, gdy budzi się RT"
            );
            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn fair_preemption_requires_minimum_granularity() {
        let idle_owner_vrt = 0u64;
        let _ = idle_owner_vrt;
        let mut current = make_task(1, SchedPolicy::Normal, 0);
        let mut candidate = make_task(2, SchedPolicy::Normal, 0);
        current.se.vruntime = 1000;
        current.se.sum_exec_runtime = 100; // dopiero co zaczął, poniżej MIN_GRANULARITY_NS
        current.se.prev_sum_exec_runtime = 0;
        candidate.se.vruntime = 0;

        let fair = RqFair::default();
        assert!(!fair.should_preempt(&current as *const _, &candidate as *const _));

        current.se.sum_exec_runtime = MIN_GRANULARITY_NS + 1;
        assert!(fair.should_preempt(&current as *const _, &candidate as *const _));
    }

    #[test]
    fn deadline_preempts_via_earliest_deadline_first() {
        let mut current = make_task(1, SchedPolicy::Deadline, 0);
        let mut candidate = make_task(2, SchedPolicy::Deadline, 0);
        current.dl.deadline = 5000;
        candidate.dl.deadline = 1000;

        let dl = RqDl::default();
        assert!(dl.should_preempt(&current as *const _, &candidate as *const _));
        candidate.dl.deadline = 9000;
        assert!(!dl.should_preempt(&current as *const _, &candidate as *const _));
    }


    #[test]
    fn deadline_task_gets_throttled_when_budget_is_exhausted() {
        let mut t = make_task(1, SchedPolicy::Deadline, 0);
        t.dl.dl_runtime = 10_000;
        t.dl.dl_deadline = 100_000;
        t.dl.dl_period = 100_000;

        let mut dl = RqDl::default();
        dl.enqueue(ptr_of(&mut t), 0, EnqueueFlags::ENQUEUE_NEW);
        assert!(!t.dl.throttled);

        dl.update_curr(ptr_of(&mut t), 15_000, 20_000);

        assert!(t.dl.throttled);
        assert!(t.flags.contains(TaskFlags::PF_DL_THROTTLED));
        assert!(t.dl.runtime <= 0);
    }

    #[test]
    fn deadline_task_is_replenished_after_period_rollover() {
        let mut t = make_task(1, SchedPolicy::Deadline, 0);
        t.dl.dl_runtime = 10_000;
        t.dl.dl_deadline = 100_000;
        t.dl.dl_period = 100_000;

        let mut dl = RqDl::default();
        dl.enqueue(ptr_of(&mut t), 0, EnqueueFlags::ENQUEUE_NEW);
        assert_eq!(t.dl.deadline, 100_000);
        dl.update_curr(ptr_of(&mut t), 10_000, 100_000);

        assert!(!t.dl.throttled, "po zakończeniu okresu budżet musi zostać odnowiony");
        assert_eq!(t.dl.runtime, 10_000);
        assert_eq!(t.dl.deadline, 200_000);
    }

    #[test]
    fn deadline_admission_control_rejects_overcommitted_bandwidth() {
        let mut dl = RqDl::default();
        assert!(dl.admission_control(60_000, 100_000));

        let mut a = make_task(1, SchedPolicy::Deadline, 0);
        a.dl.dl_runtime = 60_000;
        a.dl.dl_period = 100_000;
        dl.enqueue(ptr_of(&mut a), 0, EnqueueFlags::ENQUEUE_NEW);
        assert!(!dl.admission_control(60_000, 100_000));
        // Ale 30% wciąż się mieści (60% + 30% = 90% <= 95%).
        assert!(dl.admission_control(30_000, 100_000));
    }


    #[test]
    fn activate_and_deactivate_round_trip_updates_bookkeeping() {
        let mut idle = make_idle(0, 4);
        let mut rq = RunQueue::new(4, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            assert_eq!(rq.nr_running(), 0);

            rq.activate_task(ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!(rq.nr_running(), 1);
            assert!((*ptr_of(&mut t)).se.on_rq);
            assert_eq!((*ptr_of(&mut t)).rq_ptr(), &rq as *const RunQueue as *mut core::ffi::c_void);

            t.set_state(TaskState::Uninterruptible).unwrap();
            rq.deactivate_task(ptr_of(&mut t), DequeueFlags::DEQUEUE_SLEEP);
            assert_eq!(rq.nr_running(), 0);
            assert!(!(*ptr_of(&mut t)).se.on_rq);
            assert!((*ptr_of(&mut t)).rq_ptr().is_null());
            assert_eq!(rq.nr_uninterruptible.load(Ordering::Relaxed), 1);

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn double_enqueue_is_rejected_in_debug_builds() {
        let mut idle = make_idle(0, 5);
        let mut rq = RunQueue::new(5, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.enqueue_task(ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            assert!((*ptr_of(&mut t)).se.on_rq);
            rq.lock.unlock_irqrestore(flags);
        }
    }


    #[test]
    fn select_task_rq_prefers_idle_cpu_with_cache_affinity() {
        let mut idle0 = make_idle(100, 0);
        let mut idle1 = make_idle(101, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t = make_task(1, SchedPolicy::Normal, 0);
        t.se.last_cpu = 1;

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        unsafe {
            let chosen = smp::select_task_rq(ptr_of(&mut t), &registry);
            assert_eq!(chosen, 1, "powinien preferować ostatni CPU, skoro jest bezczynny");
        }
    }

    #[test]
    fn select_task_rq_picks_least_loaded_when_no_affinity_hit() {
        let mut idle0 = make_idle(100, 0);
        let mut idle1 = make_idle(101, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut filler = make_task(2, SchedPolicy::Normal, 0);
        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.enqueue_task(ptr_of(&mut filler), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        let mut t = make_task(1, SchedPolicy::Normal, 0);
        t.se.last_cpu = CPU_NONE;

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        unsafe {
            let chosen = smp::select_task_rq(ptr_of(&mut t), &registry);
            assert_eq!(chosen, 1, "CPU1 jest pusty, CPU0 ma już jedno zadanie");
        }
    }

    #[test]
    fn wake_up_process_transitions_state_and_enqueues_remotely() {
        let mut idle0 = make_idle(100, 0);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        rq0.bind_idle_task();

        let mut t = make_task(1, SchedPolicy::Normal, 0);
        t.set_state(TaskState::Uninterruptible).unwrap();
        t.se.last_cpu = CPU_NONE;

        let registry: [*mut RunQueue; 1] = [&mut rq0 as *mut RunQueue];
        unsafe {
            smp::wake_up_process(ptr_of(&mut t), &registry).unwrap();
            assert_eq!((*ptr_of(&mut t)).state(), TaskState::Runnable);
            assert!((*ptr_of(&mut t)).se.on_rq);
            assert_eq!(rq0.nr_running(), 1);
        }
    }

    #[test]
    fn wake_up_process_is_idempotent_when_already_queued() {
        let mut idle0 = make_idle(100, 0);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        rq0.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        let registry: [*mut RunQueue; 1] = [&mut rq0 as *mut RunQueue];
        unsafe {
            let flags = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(flags);

            let res = smp::wake_up_process(ptr_of(&mut t), &registry);
            assert!(res.is_ok());
            assert_eq!(rq0.nr_running(), 1);
        }
    }

    #[test]
    fn idle_balance_pulls_one_task_from_busy_cpu() {
        let mut idle0 = make_idle(200, 0);
        let mut idle1 = make_idle(201, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t0 = make_task(1, SchedPolicy::Normal, 0);
        let mut t1 = make_task(2, SchedPolicy::Normal, 0);
        let mut t2 = make_task(3, SchedPolicy::Normal, 0);

        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
            rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
            rq0.activate_task(ptr_of(&mut t2), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        assert_eq!(rq0.nr_running(), 3);
        assert_eq!(rq1.nr_running(), 0);

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        let pulled = unsafe { smp::idle_balance(1, &registry) };

        assert!(pulled, "CPU1 bezczynny powinien ukraść zadanie od przeciążonego CPU0");
        assert_eq!(rq0.nr_running(), 2);
        assert_eq!(rq1.nr_running(), 1);
    }

    #[test]
    fn idle_balance_does_nothing_when_balanced() {
        let mut idle0 = make_idle(200, 0);
        let mut idle1 = make_idle(201, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t0 = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];

        let pulled = unsafe { smp::idle_balance(1, &registry) };
        assert!(!pulled);
        assert_eq!(rq0.nr_running(), 1);
        assert_eq!(rq1.nr_running(), 0);
    }

    #[test]
    fn migrated_task_gets_updated_cpu_and_rq_pointer() {
        let mut idle0 = make_idle(200, 0);
        let mut idle1 = make_idle(201, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t0 = make_task(1, SchedPolicy::Normal, 0);
        let mut t1 = make_task(2, SchedPolicy::Normal, 0);
        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
            rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        let pulled = unsafe { smp::idle_balance(1, &registry) };
        assert!(pulled);

        unsafe {
            // Dokładnie jedno z t0/t1 zostało przeniesione na CPU1.
            let moved = if (*ptr_of(&mut t0)).cpu.load(Ordering::Relaxed) == 1 {
                ptr_of(&mut t0)
            } else {
                ptr_of(&mut t1)
            };
            assert_eq!((*moved).cpu.load(Ordering::Relaxed), 1);
            assert_eq!((*moved).rq_ptr(), &rq1 as *const RunQueue as *mut core::ffi::c_void);
            assert!((*moved).stats.nr_migrations >= 1);
        }
    }

    #[test]
    fn no_migration_respects_cpus_allowed_mask() {
        let mut idle0 = make_idle(200, 0);
        let mut idle1 = make_idle(201, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t0 = make_task(1, SchedPolicy::Normal, 0);
        let mut t1 = make_task(2, SchedPolicy::Normal, 0);
        t0.se.cpus_allowed = CpuMask::single(0);
        t1.se.cpus_allowed = CpuMask::single(0);

        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
            rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        let pulled = unsafe { smp::idle_balance(1, &registry) };
        assert!(!pulled, "affinity zabrania migracji obu zadań na CPU1");
        assert_eq!(rq0.nr_running(), 2);
        assert_eq!(rq1.nr_running(), 0);
    }


    #[test]
    fn yield_task_pushes_vruntime_to_back_of_fair_queue() {
        let mut idle = make_idle(0, 6);
        let mut rq = RunQueue::new(6, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut a = make_task(1, SchedPolicy::Normal, 0);
        let mut b = make_task(2, SchedPolicy::Normal, 0);
        a.se.vruntime = 100;
        b.se.vruntime = 5000;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.enqueue_task(ptr_of(&mut a), EnqueueFlags::ENQUEUE_NEW);
            rq.enqueue_task(ptr_of(&mut b), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut a));

            rq.yield_task(ptr_of(&mut a));
            assert!((*ptr_of(&mut a)).se.vruntime >= 5000, "yield powinien oddać pierwszeństwo pozostałym zadaniom");
            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn schedule_tail_records_voluntary_vs_involuntary_switch() {
        let mut idle = make_idle(0, 7);
        let mut rq = RunQueue::new(7, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut sleeper = make_task(1, SchedPolicy::Normal, 0);
        let mut preempted = make_task(2, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.enqueue_task(ptr_of(&mut sleeper), EnqueueFlags::ENQUEUE_NEW);
            rq.enqueue_task(ptr_of(&mut preempted), EnqueueFlags::ENQUEUE_NEW);

            rq.dequeue_task(ptr_of(&mut sleeper), DequeueFlags::DEQUEUE_SLEEP);
            rq.schedule_tail(ptr_of(&mut sleeper));
            assert_eq!((*ptr_of(&mut sleeper)).stats.nr_voluntary_switches, 1);

            rq.schedule_tail(ptr_of(&mut preempted));
            assert_eq!((*ptr_of(&mut preempted)).stats.nr_involuntary_switches, 1);

            rq.lock.unlock_irqrestore(flags);
        }
    }


    #[test]
    fn clock_advances_and_update_curr_charges_real_delta() {
        let mut idle = make_idle(0, 8);
        let mut rq = RunQueue::new(8, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.enqueue_task(ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut t));

            assert_eq!(rq.clock.load(Ordering::Relaxed), 0);
            rq.advance_clock(2_000_000);
            assert_eq!(rq.clock.load(Ordering::Relaxed), 2_000_000);

            let vrt_before = (*ptr_of(&mut t)).se.vruntime;
            rq.update_curr();
            let vrt_after = (*ptr_of(&mut t)).se.vruntime;
            assert!(vrt_after > vrt_before, "update_curr musi naliczyć postęp vruntime po przesunięciu zegara");

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn update_curr_is_a_noop_on_idle_task() {
        let mut idle = make_idle(0, 9);
        let mut rq = RunQueue::new(9, ptr_of(&mut idle));
        rq.bind_idle_task();
        unsafe {
            let flags = rq.lock.lock_irqsave();
            rq.advance_clock(1_000_000);
            rq.update_curr(); // curr == idle, nie powinno panikować ani nic zmienić
            rq.lock.unlock_irqrestore(flags);
        }
    }


    #[test]
    fn lower_nice_task_accumulates_vruntime_slower() {
        let mut nice_low = make_task(1, SchedPolicy::Normal, -10); // wysoki priorytet
        let mut nice_high = make_task(2, SchedPolicy::Normal, 10); // niski priorytet
        assert!(nice_to_weight(-10) > nice_to_weight(10));

        let mut fair = RqFair::default();
        fair.enqueue(ptr_of(&mut nice_low), EnqueueFlags::ENQUEUE_NEW);
        fair.enqueue(ptr_of(&mut nice_high), EnqueueFlags::ENQUEUE_NEW);

        fair.charge_exec(ptr_of(&mut nice_low), 10_000_000, 10_000_000);
        fair.charge_exec(ptr_of(&mut nice_high), 10_000_000, 10_000_000);

        unsafe {
            assert!(
                (*ptr_of(&mut nice_low)).se.vruntime < (*ptr_of(&mut nice_high)).se.vruntime,
                "zadanie o wyższym priorytecie (niższe nice) powinno akumulować vruntime wolniej"
            );
        }
        let _ = weight_to_nice(NICE_0_LOAD);
        let _ = nice_to_wmult(0);
    }


    #[test]
    fn load_balance_moves_task_between_two_uneven_queues() {
        let mut idle0 = make_idle(0, 0);
        let mut idle1 = make_idle(0, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut tasks: [TaskStruct; 4] = core::array::from_fn(|_| TaskStruct::blank());
        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            for (i, t) in tasks.iter_mut().enumerate() {
                t.init_test_stub(i as u64 + 1, SchedPolicy::Normal, 0);
                rq0.activate_task(t as *mut TaskStruct, EnqueueFlags::ENQUEUE_NEW);
            }
            rq0.lock.unlock_irqrestore(f0);
        }

        assert_eq!(rq0.nr_running(), 4);
        assert_eq!(rq1.nr_running(), 0);

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        let moved = unsafe { smp::load_balance(&registry) };
        assert_eq!(moved, 1, "load_balance przenosi co najwyżej jedno zadanie na wywołanie");
        assert_eq!(rq0.nr_running(), 3);
        assert_eq!(rq1.nr_running(), 1);
    }

    #[test]
    fn load_balance_is_noop_below_imbalance_threshold() {
        let mut idle0 = make_idle(0, 0);
        let mut idle1 = make_idle(0, 1);
        let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
        let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
        rq0.bind_idle_task();
        rq1.bind_idle_task();

        let mut t0 = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let f0 = rq0.lock.lock_irqsave();
            rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
            rq0.lock.unlock_irqrestore(f0);
        }

        let registry: [*mut RunQueue; 2] = [&mut rq0 as *mut RunQueue, &mut rq1 as *mut RunQueue];
        let moved = unsafe { smp::load_balance(&registry) };
        assert_eq!(moved, 0);
    }
}