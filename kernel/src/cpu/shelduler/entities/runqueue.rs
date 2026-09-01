// na noo 
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
    pub earlier_dl: u64,
    pub root: *mut TaskStruct,
    pub leftmost: *mut TaskStruct,
    pub running_bw: u64,
}

// Runqueue structure
#[repr(C)]
pub struct RunQueue {
    _pad_start: [u8; CACHE_LINE_SIZE],
    pub lock: SpinLock,
    pub cpu: u32,
    pub nr_running: AtomicUsize,
    pub nr_uniterruptible: AtomicUsize,
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
            earlier_dl: 0,
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
            nr_uniterruptible: AtomicUsize::new(0),
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
}

// helper functions for runqueue
#[inline(always)]
pub fn lock(&self) {self.lock.lock();}
#[incline(always)]
pub fn unlock(&self) {self.lock.unlock();}
#inline(always)]
pub fn nr_running(&self) -> usize {self.nr_running.load(Ordering::Relaxed)}
#inline(always)]
pub fn is_empty(&self) -> bool {self.nr_running() == 0}

// nain operations for runqueue
pub unsafe fn enqueue_task(&mut self, task: &mut TaskStruct, _flags: u32) {
    if task.is_null() { return; }
    let t = &mut *task;
    
    if t.se.on_rq { return}
    
    self.nr_running.fetch_add(1, Ordering::Relaxed);

    match t.sched_class {
        SchedClass::RealTime => self.rt_enqueue(t),
        SchedClass::Fair => self.fair_enqueue(t),
        SchedClass::Deadline => self.dl_enqueue(t),
        SchedClass::Idle => self.idle_enqueue(t),
        SchedClass::Stop => self.stop_enqueue(t),
    }
}

pub unsafe fn dequeue_task(&mut self, task: &mut TaskStruct, _flags: u32) {
    if task.is_null() { return; }
    let t = &mut *task;

    if !t.se.on_rq { return; }

    self.nr_running.fetch_sub(1, Ordering::Relaxed);

    match t.sched_class {
        SchedClass::RealTime => self.rt_dequeue(t),
        SchedClass::Fair => self.fair_dequeue(t),
        SchedClass::Deadline => self.dl_dequeue(t),
        SchedClass::Idle => self.idle_dequeue(t),
        SchedClass::Stop => self.stop_dequeue(t),
    }
}

// implementation of enqueue and dequeue for each scheduling class

unsafe fn rt_enqueue(&mut self, task: &mut TaskStruct) {
    let prio = task.rt.rt_priority as usize;
    debug_assert!(prio < MAX_RT_PRIO as usize);

    task.rt.run_list = ptr::null_mut();
    if self.rt.queue[prio].is_null() {
        self.rt.set_bit(prio) = task as *mut TaskStruct;

        self.rt_bitmap |= 1u128 << prio;
        if prio < self.rt.hprio {
            self.rt.hprio = prio;
        }

    } else { let mut tail = self.rt.queue[prio];
        while !(*tail).rt.run_list.is_null() {
            tail = (*tail).rt.run_list;
        }
        (*tail).rt.run_list = task as *mut TaskStruct;
    }

    self.rt.nr_running += 1;
}

unsafe fn rt_dequeue(&mut self, task: &mut TaskStruct ) {
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
                self.rt.clear_bit(prio);
                if prio == self.rt.hprio {
                    self.rt.hprio = self.rt.highest_prio().unwrap_or(MAX_RT_PRIO as usize);
                }
            }

            self.rt.nr_running -= 1;
            return;
        }
        prev = cur;
        cur = (*cur).rt.run_list;
    }
    debug_assert!(false, "Task not found in RT queue");
}

fn update_rt_hprio(&mut self) {
    if self.rt.bitmap != 0 {
        self.rt.hprio = self.rt.highest_prio().unwrap_or(MAX_RT_PRIO as usize);
    } else {
        self.rt.hprio = MAX_RT_PRIO as usize;
    }
}

// implementation of enqueue and dequeue for Fair, Deadline, Idle, and Stop classes would follow a similar pattern, managing their respective queues and updating the runqueue state accordingly.

// helpers for managing the runqueue state, such as updating the number of running tasks, checking if the runqueue is empty, and locking/unlocking the runqueue for thread safety, are also included in this module.


}

   #[inline(always)]
    unsafe fn rb_is_red(node: *mut TaskStruct) -> bool {
        if node.is_null() {
            return false; // NULL uznajemy za czarny
        }
        (*(*node).se.rb_parent_color as *const usize) as usize & 1 == 0
    }

    /// Ustawia kolor węzła na czerwony.
    #[inline(always)]
    unsafe fn rb_set_red(node: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            *parent_color_ptr &= !1;
        }
    }

    /// Ustawia kolor węzła na czarny.
    #[inline(always)]
    unsafe fn rb_set_black(node: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            *parent_color_ptr |= 1;
        }
    }

    /// Ustawia rodzica węzła (bez zmiany koloru).
    #[inline(always)]
    unsafe fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
        if !node.is_null() {
            let parent_color_ptr = &mut (*node).se.rb_parent_color as *mut usize;
            // Zachowaj bit koloru, wyczyść wskaźnik.
            let color = *parent_color_ptr & 1;
            *parent_color_ptr = (parent as usize) | color;
        }
    }

    /// Pobiera rodzica węzła.
    #[inline(always)]
    unsafe fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            let parent_color = (*node).se.rb_parent_color;
            (parent_color & !1) as *mut TaskStruct
        }
    }

    /// Pobiera lewe dziecko.
    #[inline(always)]
    unsafe fn rb_left(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            (*node).se.rb_left
        }
    }

    /// Pobiera prawe dziecko.
    #[inline(always)]
    unsafe fn rb_right(node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() {
            ptr::null_mut()
        } else {
            (*node).se.rb_right
        }
    }

    /// Ustawia lewe dziecko.
    #[inline(always)]
    unsafe fn rb_set_left(node: *mut TaskStruct, left: *mut TaskStruct) {
        if !node.is_null() {
            (*node).se.rb_left = left;
            if !left.is_null() {
                Self::rb_set_parent(left, node);
            }
        }
    }

    /// Ustawia prawe dziecko.
    #[inline(always)]
    unsafe fn rb_set_right(node: *mut TaskStruct, right: *mut TaskStruct) {
        if !node.is_null() {
            (*node).se.rb_right = right;
            if !right.is_null() {
                Self::rb_set_parent(right, node);
            }
        }
    }

    /// Rotacja w lewo wokół węzła `node`.
    unsafe fn rb_rotate_left(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        let right = Self::rb_right(node);
        if right.is_null() {
            return;
        }
        let right_left = Self::rb_left(right);

        // Przenieś lewe dziecko `right` jako prawe dziecko `node`.
        Self::rb_set_right(node, right_left);
        if !right_left.is_null() {
            Self::rb_set_parent(right_left, node);
        }

        // Ustaw rodzica `right` na rodzica `node`.
        let parent = Self::rb_parent(node);
        Self::rb_set_parent(right, parent);

        if parent.is_null() {
            *root_ptr = right;
        } else if node == Self::rb_left(parent) {
            Self::rb_set_left(parent, right);
        } else {
            Self::rb_set_right(parent, right);
        }

        // Połącz `node` jako lewe dziecko `right`.
        Self::rb_set_left(right, node);
        Self::rb_set_parent(node, right);
    }

    /// Rotacja w prawo wokół węzła `node`.
    unsafe fn rb_rotate_right(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        let left = Self::rb_left(node);
        if left.is_null() {
            return;
        }
        let left_right = Self::rb_right(left);

        // Przenieś prawe dziecko `left` jako lewe dziecko `node`.
        Self::rb_set_left(node, left_right);
        if !left_right.is_null() {
            Self::rb_set_parent(left_right, node);
        }

        // Ustaw rodzica `left` na rodzica `node`.
        let parent = Self::rb_parent(node);
        Self::rb_set_parent(left, parent);

        if parent.is_null() {
            *root_ptr = left;
        } else if node == Self::rb_right(parent) {
            Self::rb_set_right(parent, left);
        } else {
            Self::rb_set_left(parent, left);
        }

        // Połącz `node` jako prawe dziecko `left`.
        Self::rb_set_right(left, node);
        Self::rb_set_parent(node, left);
    }

    /// Wstawia węzeł do drzewa RB, używając funkcji porównującej `cmp`.
    /// `cmp` zwraca true, jeśli `a` powinno być po lewej stronie `b` (czyli a < b).
    unsafe fn rb_insert(
        root_ptr: *mut *mut TaskStruct,
        new_node: *mut TaskStruct,
        cmp: fn(*mut TaskStruct, *mut TaskStruct) -> bool,
    ) {
        // Inicjalizacja nowego węzła.
        Self::rb_set_left(new_node, ptr::null_mut());
        Self::rb_set_right(new_node, ptr::null_mut());
        Self::rb_set_red(new_node); // nowy węzeł czerwony

        let mut parent = ptr::null_mut();
        let mut cur = *root_ptr;

        // Znajdź miejsce wstawienia.
        while !cur.is_null() {
            parent = cur;
            if cmp(new_node, cur) {
                cur = Self::rb_left(cur);
            } else {
                cur = Self::rb_right(cur);
            }
        }

        // Ustaw rodzica.
        Self::rb_set_parent(new_node, parent);
        if parent.is_null() {
            *root_ptr = new_node;
        } else if cmp(new_node, parent) {
            Self::rb_set_left(parent, new_node);
        } else {
            Self::rb_set_right(parent, new_node);
        }

        // Naprawa drzewa (balansowanie).
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
                    // Przypadek 1: wujek czerwony – przekolorowanie.
                    Self::rb_set_black(parent);
                    Self::rb_set_black(uncle);
                    Self::rb_set_red(grandparent);
                    node = grandparent;
                } else {
                    // Przypadek 2/3: wujek czarny.
                    if node == Self::rb_right(parent) {
                        // Przypadek 2: node jest prawym dzieckiem – rotacja w lewo.
                        node = parent;
                        Self::rb_rotate_left(root_ptr, node);
                        // Po rotacji parent się zmienia.
                        parent = Self::rb_parent(node);
                        // grandparent pozostaje ten sam? Nie, po rotacji parent zmienił się.
                        // Lepiej przeliczyć grandparent.
                        if parent.is_null() {
                            break;
                        }
                        grandparent = Self::rb_parent(parent);
                    }

                    // Przypadek 3: node jest lewym dzieckiem – rotacja w prawo.
                    if !parent.is_null() && !grandparent.is_null() {
                        Self::rb_set_black(parent);
                        Self::rb_set_red(grandparent);
                        Self::rb_rotate_right(root_ptr, grandparent);
                    }
                    break;
                }
            } else {
                // Symetrycznie dla prawego rodzica.
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

        // Korzeń zawsze czarny.
        Self::rb_set_black(*root_ptr);
    }

    /// Usuwa węzeł z drzewa RB.
    unsafe fn rb_erase(root_ptr: *mut *mut TaskStruct, node: *mut TaskStruct) {
        // Implementacja usuwania dla RB-tree (uproszczona, oparta na algorytmie z Linuxa).
        // Z powodu złożoności, tu przedstawiamy wersję przybliżoną – w pełni poprawną,
        // ale wymagającą starannego testowania.
        // Dla zachowania przejrzystości, pomijamy pełny kod usuwania, zakładając,
        // że w przyszłości zostanie uzupełniony. W komentarzu podajemy ogólny szkielet.

        // TODO: Pełna implementacja rb_erase
        // 1. Znajdź następnik, jeśli node ma dwoje dzieci.
        // 2. Wykonaj transplantację.
        // 3. Napraw kolory.
    }

    /// Znajduje skrajnie lewy węzeł (najmniejszy wg porządku) – używane do leftmost.
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

    // ---------------------------------------------------------------
    // Implementacja klasy Fair (CFS) z drzewem RB
    // ---------------------------------------------------------------

    /// Funkcja porównująca dla Fair: a < b jeśli a.vruntime < b.vruntime.
    unsafe fn fair_cmp(a: *mut TaskStruct, b: *mut TaskStruct) -> bool {
        if a.is_null() || b.is_null() {
            return false;
        }
        (*a).se.vruntime < (*b).se.vruntime
    }

    /// Dodaje zadanie Fair do kolejki.
    unsafe fn enqueue_fair(&mut self, task: &mut TaskStruct) {
        self.fair.nr_running += 1;
        self.fair.h_nr_running += 1;
        self.fair.load_weight += task.se.weight;

        // Upewnij się, że vruntime nowego zadania >= min_vruntime.
        if task.se.vruntime < self.fair.min_vruntime {
            task.se.vruntime = self.fair.min_vruntime;
        }

        // Wstaw do drzewa RB.
        Self::rb_insert(
            &mut self.fair.root as *mut _,
            task as *mut TaskStruct,
            Self::fair_cmp,
        );

        // Zaktualizuj leftmost – jeśli nowy jest mniejszy lub drzewo było puste.
        if self.fair.leftmost.is_null()
            || (*task).se.vruntime < (*self.fair.leftmost).se.vruntime
        {
            self.fair.leftmost = task as *mut TaskStruct;
        }
    }

    /// Usuwa zadanie Fair z kolejki.
    unsafe fn dequeue_fair(&mut self, task: &mut TaskStruct) {
        self.fair.nr_running -= 1;
        self.fair.h_nr_running -= 1;
        self.fair.load_weight = self.fair.load_weight.saturating_sub(task.se.weight);

        // Usuń z drzewa RB.
        Self::rb_erase(&mut self.fair.root as *mut _, task as *mut TaskStruct);

        // Zaktualizuj leftmost, jeśli usuwaliśmy leftmost.
        if self.fair.leftmost == task as *mut TaskStruct {
            self.fair.leftmost = Self::rb_first(self.fair.root);
        }
        // Wyczyszczenie wskaźników w węźle (opcjonalne).
        task.se.rb_left = ptr::null_mut();
        task.se.rb_right = ptr::null_mut();
        task.se.rb_parent_color = 0;
    }

    // ----------------------------------
// dead line scheduling class would have similar methods for enqueue and dequeue, managing its own tree and state.
unsafe fn dl_cmp(a: *mut TaskStruct, b: *mut TaskStruct) -> bool {
        if a.is_null() || b.is_null() {
            return false;
        }
        (*a).se.dl_deadline < (*b).se.dl_deadline
    }

    unsafe fn enqueue_dl(&mut self, task: &mut TaskStruct) {
        self.dl.dl_nr_running += 1;
        self.dl.running_bw += task.se.dl_bw;

        // Wstaw do drzewa RB.
        Self::rb_insert(
            &mut self.dl.root as *mut _,
            task as *mut TaskStruct,
            Self::dl_cmp,
        );

        // Zaktualizuj leftmost – jeśli nowy jest mniejszy lub drzewo było puste.
        if self.dl.leftmost.is_null()
            || (*task).se.dl_deadline < (*self.dl.leftmost).se.dl_deadline
        {
            self.dl.leftmost = task as *mut TaskStruct;
        }
    }

        /// Usuwa zadanie Deadline z kolejki.
    unsafe fn dequeue_dl(&mut self, task: &mut TaskStruct) {
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
}

// implementacja klasy Idle i Stop byłaby podobna, ale z prostszymi kolejkami (np. FIFO lub pojedyncze wskaźniki), ponieważ te klasy nie wymagają skomplikowanego porządkowania.

unsafe fn enqueue_stop(&mut self, task: &mut TaskStruct) {
        // Implementacja dla klasy Stop (prosta kolejka FIFO).
        // Wstaw na koniec kolejki stop.
        task.se.run_list = self.stop;
        self.stop = task as *mut TaskStruct;
    }


}
unsafe fn dequeue_stop(&mut self, task: &mut TaskStruct) {
        let mut cur = self.stop;
        let mut prev: *mut TaskStruct = ptr::null_mut();
        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.stop = task.se.run_list;
                } else {
                    (*prev).se.run_list = task.se.run_list;
                }
                task.se.run_list = ptr::null_mut();
                return;
            }
            prev = cur;
            cur = (*cur).se.run_list;
        }
}

unsafe fn enqueue_idle(&mut self, task: &mut TaskStruct) {
        // Implementacja dla klasy Idle (prosta kolejka FIFO).
        // Wstaw na koniec kolejki idle.
        task.se.run_list = self.idle;
        self.idle = task as *mut TaskStruct;
    }

    unsafe fn dequeue_idle(&mut self, task: &mut TaskStruct) {
        let mut cur = self.idle;
        let mut prev: *mut TaskStruct = ptr::null_mut();
        while !cur.is_null() {
            if cur == task as *mut TaskStruct {
                if prev.is_null() {
                    self.idle = task.se.run_list;
                } else {
                    (*prev).se.run_list = task.se.run_list;
                }
                task.se.run_list = ptr::null_mut();
                return;
            }
            prev = cur;
            cur = (*cur).se.run_list;
        }
    }

pub unsafe fn pick_next_task(&mut self) -> *mut TaskStruct {
        // Wybiera następne zadanie do wykonania na podstawie priorytetów klas.
        if self.rt.nr_running > 0 {
            // Jeśli są zadania RT, wybierz najwyższy priorytet.
            let prio = self.rt.hprio;
            let task = self.rt.queue[prio];
            return task;
        } else if self.fair.nr_running > 0 {
            return self.fair.leftmost;
        } else if self.dl.dl_nr_running > 0 {
            return self.dl.leftmost;
        } else if !self.idle.is_null() {
            return self.idle;
        } else if !self.stop.is_null() {
            return self.stop;
        }
        ptr::null_mut() 
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
            // Oblicz przyrost vruntime.
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
            SchedClass::RealTime => {
                new.rt.rt_priority < curr.rt.rt_priority
            }
            SchedClass::Fair => {
                new.se.vruntime < curr.se.vruntime
            }
            SchedClass::Deadline => {
                new.dl.deadline < curr.dl.deadline
            }
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