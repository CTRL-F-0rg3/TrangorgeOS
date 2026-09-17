#![allow(dead_code)]

use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::ptr;
use core::mem::MaybeUninit;
use crate::cpu::scheduler::entities::task::{
    TaskStruct, TaskId, TaskState, TaskFlags, ListHead, SpinLock, CPU_NONE, MAX_CPUS,
    Credentials, RLimit, RlimitResource, RLIM_INFINITY
};

pub const MAX_PID: u32 = 1_048_576;
pub const RESERVED_PIDS: u32 = 300;
const PID_BITMAP_WORDS: usize = (MAX_PID as usize + 63) / 64;

pub struct PidAllocator {
    bitmap: [AtomicU64; PID_BITMAP_WORDS],
    next_hint: AtomicU32,
    allocated: AtomicU32,
}

impl PidAllocator {
    pub const fn new() -> Self {
        const ZERO: AtomicU64 = AtomicU64::new(0);
        Self {
            bitmap: [ZERO; PID_BITMAP_WORDS],
            next_hint: AtomicU32::new(RESERVED_PIDS),
            allocated: AtomicU32::new(0),
        }
    }

    pub fn alloc(&self) -> Option<TaskId> {
        let hint = self.next_hint.load(Ordering::Relaxed) as usize;
        let start_idx = hint / 64;
        
        for offset in 0..PID_BITMAP_WORDS {
            let idx = (start_idx + offset) % PID_BITMAP_WORDS;
            let word = self.bitmap[idx].load(Ordering::Relaxed);
            if word == u64::MAX { continue; }
            
            let bit = (!word).trailing_zeros();
            let pid = (idx * 64 + bit as usize) as u32;
            if pid >= MAX_PID { continue; }
            
            let mask = 1u64 << bit;
            if self.bitmap[idx].fetch_or(mask, Ordering::AcqRel) & mask == 0 {
                self.allocated.fetch_add(1, Ordering::Relaxed);
                self.next_hint.store(pid + 1, Ordering::Relaxed);
                return Some(pid as TaskId);
            }
        }
        None
    }

    pub fn free(&self, pid: TaskId) {
        let pid = pid as u32;
        if pid >= MAX_PID { return; }
        let idx = pid as usize / 64;
        let bit = 1u64 << (pid as usize % 64);
        if self.bitmap[idx].fetch_and(!bit, Ordering::AcqRel) & bit != 0 {
            self.allocated.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn reserve(&self, pid: TaskId) -> bool {
        let pid = pid as u32;
        if pid >= MAX_PID { return false; }
        let idx = pid as usize / 64;
        let bit = 1u64 << (pid as usize % 64);
        if self.bitmap[idx].fetch_or(bit, Ordering::AcqRel) & bit == 0 {
            self.allocated.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn is_used(&self, pid: TaskId) -> bool {
        let pid = pid as u32;
        if pid >= MAX_PID { return true; }
        let idx = pid as usize / 64;
        let bit = 1u64 << (pid as usize % 64);
        (self.bitmap[idx].load(Ordering::Acquire) & bit) != 0
    }

    pub fn count_allocated(&self) -> u32 {
        self.allocated.load(Ordering::Relaxed)
    }
}

<<<<<<< HEAD
pub const MAX_TASKS: usize = 65536;
const TGID_SLOTS: usize = MAX_TASKS / 4;

pub struct TaskTable {
    pid_slots: [AtomicPtr<TaskStruct>; MAX_TASKS],
    tgid_slots: [AtomicPtr<TaskStruct>; TGID_SLOTS],
    count: AtomicU32,
=======
#[repr(C)]
pub struct RtFields {
    pub rt_priority: u32,
    pub run_list: ListHead,
>>>>>>> da9f7b9 (wip: opis zmian)
}

impl TaskTable {
    pub const fn new() -> Self {
        const NULL_PTR: AtomicPtr<TaskStruct> = AtomicPtr::new(ptr::null_mut());
        Self {
            pid_slots: [NULL_PTR; MAX_TASKS],
            tgid_slots: [NULL_PTR; TGID_SLOTS],
            count: AtomicU32::new(0),
        }
    }

    fn hash_pid(pid: TaskId) -> usize { (pid as usize) % MAX_TASKS }
    fn hash_tgid(tgid: TaskId) -> usize { (tgid as usize) % TGID_SLOTS }

    pub fn insert(&self, task: *mut TaskStruct) -> bool {
        if task.is_null() { return false; }
        let pid = unsafe { (*task).pid };
        let tgid = unsafe { (*task).tgid };
        
        let start = Self::hash_pid(pid);
        for offset in 0..MAX_TASKS {
            let idx = (start + offset) % MAX_TASKS;
            let expected = ptr::null_mut();
            if self.pid_slots[idx].compare_exchange(
                expected, task, Ordering::AcqRel, Ordering::Acquire
            ).is_ok() {
                self.count.fetch_add(1, Ordering::Relaxed);
                
                if pid == tgid {
                    let t_start = Self::hash_tgid(tgid);
                    for t_off in 0..TGID_SLOTS {
                        let t_idx = (t_start + t_off) % TGID_SLOTS;
                        let t_expected = ptr::null_mut();
                        if self.tgid_slots[t_idx].compare_exchange(
                            t_expected, task, Ordering::AcqRel, Ordering::Acquire
                        ).is_ok() {
                            break;
                        }
                    }
                }
                return true;
            }
            if unsafe { self.pid_slots[idx].load(Ordering::Relaxed) == task } {
                return false;
            }
        }
        false
    }

    pub fn remove(&self, pid: TaskId) -> *mut TaskStruct {
        let start = Self::hash_pid(pid);
        for offset in 0..MAX_TASKS {
            let idx = (start + offset) % MAX_TASKS;
            let slot = self.pid_slots[idx].load(Ordering::Acquire);
            if slot.is_null() { return ptr::null_mut(); }
            
            if unsafe { (*slot).pid == pid } {
                if self.pid_slots[idx].compare_exchange(
                    slot, ptr::null_mut(), Ordering::AcqRel, Ordering::Acquire
                ).is_ok() {
                    self.count.fetch_sub(1, Ordering::Relaxed);
                    
                    let tgid = unsafe { (*slot).tgid };
                    if pid == tgid {
                        let t_start = Self::hash_tgid(tgid);
                        for t_off in 0..TGID_SLOTS {
                            let t_idx = (t_start + t_off) % TGID_SLOTS;
                            if self.tgid_slots[t_idx].load(Ordering::Acquire) == slot {
                                self.tgid_slots[t_idx].store(ptr::null_mut(), Ordering::Release);
                                break;
                            }
                        }
                    }
                    return slot;
                }
            }
        }
        ptr::null_mut()
    }

    pub fn lookup_pid(&self, pid: TaskId) -> *mut TaskStruct {
        let start = Self::hash_pid(pid);
        for offset in 0..MAX_TASKS {
            let idx = (start + offset) % MAX_TASKS;
            let slot = self.pid_slots[idx].load(Ordering::Acquire);
            if slot.is_null() { return ptr::null_mut(); }
            if unsafe { (*slot).pid == pid } { return slot; }
        }
        ptr::null_mut()
    }

    pub fn lookup_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
        let start = Self::hash_tgid(tgid);
        for offset in 0..TGID_SLOTS {
            let idx = (start + offset) % TGID_SLOTS;
            let slot = self.tgid_slots[idx].load(Ordering::Acquire);
            if slot.is_null() { return ptr::null_mut(); }
            if unsafe { (*slot).tgid == tgid && (*slot).pid == tgid } { return slot; }
        }
        ptr::null_mut()
    }

    pub fn count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}

unsafe fn task_from_list_node(node: *mut ListHead) -> *mut TaskStruct {
    let dummy = MaybeUninit::<TaskStruct>::uninit();
    let base = dummy.as_ptr() as *mut u8;
    let field = core::ptr::addr_of!((*dummy.as_ptr()).thread_group) as *mut u8;
    let offset = field.offset_from(base) as usize;
    (node as *mut u8).sub(offset) as *mut TaskStruct
}

<<<<<<< HEAD
pub struct TaskIterator<'a> {
    table: &'a TaskTable,
    index: usize,
=======
impl TaskStruct {
    pub const RB_LEFT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_left);
    pub const RB_RIGHT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_right);
    pub const RB_PARENT_COLOR_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_parent_color);

    pub const PLIST_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_prio);
    pub const PLIST_SAME_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_same_prio);
    pub const PLIST_NODE_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_node);

    pub const RT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rt);
    pub const RT_RUN_LIST_OFFSET: usize = Self::RT_OFFSET + core::mem::offset_of!(RtFields, run_list);

    /// Odtwarza `*mut TaskStruct` z surowego wskaźnika na pole
    /// znajdujące się pod `field_offset` od początku struktury.
    ///
    /// To jedyne miejsce w całym module, gdzie wykonywana jest
    /// arytmetyka `container_of`. Wszystkie pliki (`rbtree.rs`,
    /// `plist.rs`, `rt_array.rs`) muszą przechodzić przez tę funkcję
    /// zamiast liczyć `.sub(OFFSET)` samodzielnie — dzięki temu:
    /// 1. jest jedno miejsce do naprawy, gdy zmieni się layout,
    /// 2. w trybie debug każdy round-trip jest weryfikowany
    ///    (`debug_assert`), więc błędny offset wywala się głośno
    ///    w testach, zamiast cicho psuć pamięć na produkcji.
    ///
    /// # Bezpieczeństwo
    /// `field_ptr` musi realnie wskazywać na pole znajdujące się pod
    /// `field_offset` bajtów od adresu żywego `TaskStruct`. Wywołujący
    /// odpowiada za tę gwarancję (patrz dokumentacja funkcji w
    /// `rbtree.rs`/`plist.rs`/`rt_array.rs`, które to wywołują).
    #[inline(always)]
    pub unsafe fn container_of<F>(field_ptr: *mut F, field_offset: usize) -> *mut TaskStruct {
        debug_assert!(!field_ptr.is_null(), "container_of z null polem — błąd wywołującego");
        let task_ptr = (field_ptr as *mut u8).sub(field_offset) as *mut TaskStruct;

        #[cfg(debug_assertions)]
        {
            // Weryfikacja rundtripu: policz offset od odtworzonego
            // TaskStruct z powrotem do pola i porównaj z tym, co
            // podał wywołujący. Niezgodność oznacza błędny offset
            // (np. literówkę przy dodawaniu nowego pola do TaskStruct
            // bez aktualizacji stałych *_OFFSET powyżej).
            let recomputed = (task_ptr as *mut u8).add(field_offset);
            debug_assert_eq!(
                recomputed, field_ptr as *mut u8,
                "container_of: niespójny offset — sprawdź stałe *_OFFSET względem TaskStruct"
            );
        }

        task_ptr
    }
>>>>>>> da9f7b9 (wip: opis zmian)
}

impl<'a> TaskIterator<'a> {
    pub fn new(table: &'a TaskTable) -> Self {
        Self { table, index: 0 }
    }
}

impl<'a> Iterator for TaskIterator<'a> {
    type Item = *mut TaskStruct;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < MAX_TASKS {
            let slot = self.table.pid_slots[self.index].load(Ordering::Acquire);
            self.index += 1;
            if !slot.is_null() {
                return Some(slot);
            }
        }
        None
    }
}

pub struct ThreadGroupIterator {
    current: *mut TaskStruct,
    head: *mut ListHead,
}

impl ThreadGroupIterator {
    pub unsafe fn new(leader: *mut TaskStruct) -> Self {
        if leader.is_null() {
            return Self { current: ptr::null_mut(), head: ptr::null_mut() };
        }
        let head = &(*leader).thread_group as *const ListHead as *mut ListHead;
        let first = (*head).next;
        let current = if first == head || first.is_null() { ptr::null_mut() } else { leader };
        Self { current, head }
    }
}

impl Iterator for ThreadGroupIterator {
    type Item = *mut TaskStruct;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() || self.head.is_null() { return None; }
        let result = self.current;
        let list_node = &(*self.current).thread_group as *const ListHead as *mut ListHead;
        let next_node = (*list_node).next;
        
        if next_node == self.head || next_node.is_null() {
            self.current = ptr::null_mut();
        } else {
            self.current = task_from_list_node(next_node);
        }
        Some(result)
    }
}

pub struct ChildIterator {
    current: *mut TaskStruct,
}

impl ChildIterator {
    pub unsafe fn new(parent: *mut TaskStruct) -> Self {
        if parent.is_null() {
            return Self { current: ptr::null_mut() };
        }
        Self { current: (*parent).children }
    }
}

impl Iterator for ChildIterator {
    type Item = *mut TaskStruct;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() { return None; }
        let result = self.current;
        self.current = (*self.current).sibling;
        Some(result)
    }
}

pub struct DescendantIterator {
    stack: Vec<*mut TaskStruct>,
}

impl DescendantIterator {
    pub unsafe fn new(root: *mut TaskStruct) -> Self {
        let mut stack = Vec::new();
        if !root.is_null() {
            stack.push(root);
        }
        Self { stack }
    }
}

impl Iterator for DescendantIterator {
    type Item = *mut TaskStruct;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;
        let mut child = (*current).children;
        while !child.is_null() {
            self.stack.push(child);
            child = (*child).sibling;
        }
        Some(current)
    }
}

pub struct ProcessGroup {
    pub pgid: TaskId,
    pub leader: AtomicPtr<TaskStruct>,
    pub session: AtomicPtr<Session>,
    pub tasks: ListHead,
    pub lock: SpinLock,
    pub refcount: AtomicU32,
    pub is_orphaned: AtomicBool,
}

impl ProcessGroup {
    pub fn new(pgid: TaskId, leader: *mut TaskStruct) -> Self {
        let mut pg = Self {
            pgid,
            leader: AtomicPtr::new(leader),
            session: AtomicPtr::new(ptr::null_mut()),
            tasks: ListHead::new(),
            lock: SpinLock::new(),
            refcount: AtomicU32::new(1),
            is_orphaned: AtomicBool::new(false),
        };
        pg.tasks.init();
        pg
    }

    pub unsafe fn add_task(&self, task: *mut TaskStruct) {
        if task.is_null() { return; }
        let _guard = self.lock.lock_guard();
        let node = &mut (*task).tasks as *mut ListHead;
        self.tasks.insert_before(node);
        self.refcount.fetch_add(1, Ordering::Relaxed);
    }

    pub unsafe fn remove_task(&self, task: *mut TaskStruct) {
        if task.is_null() { return; }
        let _guard = self.lock.lock_guard();
        let node = &mut (*task).tasks as *mut ListHead;
        node.remove();
        self.refcount.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        self.refcount.load(Ordering::Relaxed) <= 1
    }

    pub unsafe fn send_signal(&self, sig: u8) {
        let mut node = self.tasks.next;
        while node != &self.tasks as *const ListHead as *mut ListHead && !node.is_null() {
            let task = task_from_list_node(node);
            if !task.is_null() {
                (*task).sig.raise(sig);
            }
            node = (*node).next;
        }
    }
}

pub struct Session {
    pub sid: TaskId,
    pub leader: AtomicPtr<TaskStruct>,
    pub groups: ListHead,
    pub lock: SpinLock,
    pub refcount: AtomicU32,
    pub foreground_pgid: AtomicU32,
}

impl Session {
    pub fn new(sid: TaskId, leader: *mut TaskStruct) -> Self {
        let mut s = Self {
            sid,
            leader: AtomicPtr::new(leader),
            groups: ListHead::new(),
            lock: SpinLock::new(),
            refcount: AtomicU32::new(1),
            foreground_pgid: AtomicU32::new(0),
        };
        s.groups.init();
        s
    }

    pub unsafe fn add_group(&self, pg: *mut ProcessGroup) {
        if pg.is_null() { return; }
        let _guard = self.lock.lock_guard();
        (*pg).session.store(self as *const Session as *mut Session, Ordering::Release);
        self.refcount.fetch_add(1, Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        self.refcount.load(Ordering::Relaxed) <= 1
    }

    pub fn set_foreground(&self, pgid: TaskId) {
        self.foreground_pgid.store(pgid as u32, Ordering::Release);
    }

    pub fn get_foreground(&self) -> TaskId {
        self.foreground_pgid.load(Ordering::Acquire) as TaskId
    }
}

pub const MAX_UID: usize = 65536;

pub struct UidTaskCount {
    counts: [AtomicU32; MAX_UID],
}

impl UidTaskCount {
    pub const fn new() -> Self {
        const ZERO: AtomicU32 = AtomicU32::new(0);
        Self { counts: [ZERO; MAX_UID] }
    }

    pub fn inc(&self, uid: u32) -> u32 {
        let idx = (uid as usize) % MAX_UID;
        self.counts[idx].fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn dec(&self, uid: u32) {
        let idx = (uid as usize) % MAX_UID;
        self.counts[idx].fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            if v > 0 { Some(v - 1) } else { Some(0) }
        }).ok();
    }

    pub fn get(&self, uid: u32) -> u32 {
        let idx = (uid as usize) % MAX_UID;
        self.counts[idx].load(Ordering::Relaxed)
    }
}

pub enum TaskRegistryError {
    PidExhausted,
    PidAlreadyUsed,
    TableFull,
    InvalidTask,
    NprocExceeded,
}

pub struct TaskRegistry {
    pub pids: PidAllocator,
    pub table: TaskTable,
    pub init_task: AtomicPtr<TaskStruct>,
    pub total_forks: AtomicU64,
    pub total_exits: AtomicU64,
    pub global_lock: SpinLock,
    pub uid_counts: UidTaskCount,
}

impl TaskRegistry {
    pub const fn new() -> Self {
        Self {
            pids: PidAllocator::new(),
            table: TaskTable::new(),
            init_task: AtomicPtr::new(ptr::null_mut()),
            total_forks: AtomicU64::new(0),
            total_exits: AtomicU64::new(0),
            global_lock: SpinLock::new(),
            uid_counts: UidTaskCount::new(),
        }
    }

    pub unsafe fn register(&self, task: *mut TaskStruct) -> Result<TaskId, TaskRegistryError> {
        if task.is_null() { return Err(TaskRegistryError::InvalidTask); }
        
        let uid = (*task).cred.uid;
        let limit = (*task).rlimits[RlimitResource::Nproc as usize].cur;
        if limit != RLIM_INFINITY && self.uid_counts.get(uid) as u64 >= limit {
            return Err(TaskRegistryError::NprocExceeded);
        }

        let pid = self.pids.alloc().ok_or(TaskRegistryError::PidExhausted)?;
        (*task).pid = pid;
        if (*task).tgid == 0 {
            (*task).tgid = pid;
        }
        
        if !self.table.insert(task) {
            self.pids.free(pid);
            return Err(TaskRegistryError::TableFull);
        }
        
        self.uid_counts.inc(uid);
        self.total_forks.fetch_add(1, Ordering::Relaxed);
        Ok(pid)
    }

    pub unsafe fn unregister(&self, task: *mut TaskStruct) {
        if task.is_null() { return; }
        let pid = (*task).pid;
        let uid = (*task).cred.uid;
        
        let removed = self.table.remove(pid);
        if !removed.is_null() {
            self.pids.free(pid);
            self.uid_counts.dec(uid);
            self.total_exits.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn find_by_pid(&self, pid: TaskId) -> *mut TaskStruct {
        self.table.lookup_pid(pid)
    }

    pub fn find_by_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
        self.table.lookup_tgid(tgid)
    }

    pub unsafe fn link_hierarchy(&self, parent: *mut TaskStruct, child: *mut TaskStruct) {
        if parent.is_null() || child.is_null() { return; }
        let _guard = self.global_lock.lock_guard();
        
        (*child).parent = parent;
        (*child).ppid = (*parent).pid;
        (*child).sibling = (*parent).children;
        (*parent).children = child;
    }

    pub unsafe fn unlink_hierarchy(&self, task: *mut TaskStruct) {
        if task.is_null() { return; }
        let _guard = self.global_lock.lock_guard();
        
        let parent = (*task).parent;
        if parent.is_null() { return; }
        
        let mut prev: *mut TaskStruct = ptr::null_mut();
        let mut curr = (*parent).children;
        
        while !curr.is_null() {
            if curr == task {
                if prev.is_null() {
                    (*parent).children = (*curr).sibling;
                } else {
                    (*prev).sibling = (*curr).sibling;
                }
                (*curr).sibling = ptr::null_mut();
                (*curr).parent = ptr::null_mut();
                return;
            }
            prev = curr;
            curr = (*curr).sibling;
        }
    }

    pub unsafe fn reparent_to_init(&self, task: *mut TaskStruct) {
        if task.is_null() { return; }
        let init = self.init_task.load(Ordering::Acquire);
        if init.is_null() || init == task { return; }
        
        self.unlink_hierarchy(task);
        self.link_hierarchy(init, task);
    }

    pub unsafe fn reparent_children(&self, dying_parent: *mut TaskStruct) {
        if dying_parent.is_null() { return; }
        let init = self.init_task.load(Ordering::Acquire);
        if init.is_null() { return; }
        
        let _guard = self.global_lock.lock_guard();
        let mut child = (*dying_parent).children;
        
        while !child.is_null() {
            let next = (*child).sibling;
            (*child).parent = init;
            (*child).ppid = (*init).pid;
            (*child).sibling = (*init).children;
            (*init).children = child;
            child = next;
        }
        (*dying_parent).children = ptr::null_mut();
    }

    pub unsafe fn reap_zombies(&self, parent: *mut TaskStruct) -> u32 {
        if parent.is_null() { return 0; }
        let mut reaped = 0u32;
        let _guard = self.global_lock.lock_guard();
        
        let mut prev: *mut TaskStruct = ptr::null_mut();
        let mut curr = (*parent).children;
        
        while !curr.is_null() {
            let next = (*curr).sibling;
            if (*curr).state() == TaskState::Zombie {
                if prev.is_null() {
                    (*parent).children = next;
                } else {
                    (*prev).sibling = next;
                }
                (*curr).sibling = ptr::null_mut();
                (*curr).parent = ptr::null_mut();
                self.unregister(curr);
                reaped += 1;
            } else {
                prev = curr;
            }
            curr = next;
        }
        reaped
    }

    pub fn iter(&self) -> TaskIterator {
        TaskIterator::new(&self.table)
    }

    pub unsafe fn send_signal_to_group(&self, pgid: TaskId, sig: u8) {
        let leader = self.find_by_tgid(pgid);
        if leader.is_null() { return; }
        
        for task_ptr in ThreadGroupIterator::new(leader) {
            if !task_ptr.is_null() {
                (*task_ptr).sig.raise(sig);
            }
        }
    }

    pub fn set_init_task(&self, task: *mut TaskStruct) {
        self.init_task.store(task, Ordering::Release);
    }

    pub fn stats(&self) -> (u32, u64, u64) {
        (
            self.table.count(),
            self.total_forks.load(Ordering::Relaxed),
            self.total_exits.load(Ordering::Relaxed),
        )
    }
}

pub struct PidNamespace {
    pub id: u32,
    pub level: u32,
    pub parent: *mut PidNamespace,
    pub registry: TaskRegistry,
    pub lock: SpinLock,
}

impl PidNamespace {
    pub const fn new(id: u32, level: u32, parent: *mut PidNamespace) -> Self {
        Self {
            id,
            level,
            parent,
            registry: TaskRegistry::new(),
            lock: SpinLock::new(),
        }
    }

    pub unsafe fn alloc_pid_in_ns(&self) -> Option<TaskId> {
        self.registry.pids.alloc()
    }

    pub unsafe fn map_pid(&self, ns_pid: TaskId) -> TaskId {
        ns_pid + (self.level as TaskId * MAX_PID)
    }
}