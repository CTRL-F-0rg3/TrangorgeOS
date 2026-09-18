use crate::cpu::scheduler::entities::task::TaskStruct;
use core::ptr;

const RB_RED: usize = 0;
const RB_BLACK: usize = 1;

#[inline(always)]
unsafe fn rb_left(node: *mut TaskStruct) -> *mut *mut TaskStruct {
    (node as *mut u8).add(TaskStruct::RB_LEFT_OFFSET) as *mut *mut TaskStruct
}

#[inline(always)]
unsafe fn rb_right(node: *mut TaskStruct) -> *mut *mut TaskStruct {
    (node as *mut u8).add(TaskStruct::RB_RIGHT_OFFSET) as *mut *mut TaskStruct
}

#[inline(always)]
unsafe fn rb_parent_color_slot(node: *mut TaskStruct) -> *mut usize {
    (node as *mut u8).add(TaskStruct::RB_PARENT_COLOR_OFFSET) as *mut usize
}

#[inline(always)]
fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
    if node.is_null() { return ptr::null_mut(); }
    unsafe { (*rb_parent_color_slot(node) & !1) as *mut TaskStruct }
}

#[inline(always)]
fn rb_color(node: *mut TaskStruct) -> usize {
    if node.is_null() { return RB_BLACK; }
    unsafe { *rb_parent_color_slot(node) & 1 }
}

#[inline(always)]
fn rb_is_red(node: *mut TaskStruct) -> bool {
    !node.is_null() && rb_color(node) == RB_RED
}

#[inline(always)]
fn rb_set_parent_color(node: *mut TaskStruct, parent: *mut TaskStruct, color: usize) {
    if node.is_null() { return; }
    unsafe {
        *rb_parent_color_slot(node) = (parent as usize & !1) | color;
    }
}

#[inline(always)]
fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
    let color = rb_color(node);
    rb_set_parent_color(node, parent, color);
}

#[inline(always)]
fn rb_set_color(node: *mut TaskStruct, color: usize) {
    if node.is_null() { return; }
    unsafe {
        let slot = rb_parent_color_slot(node);
        *slot = (*slot & !1) | color;
    }
}

pub struct RbTree {
    pub root: *mut TaskStruct,
    pub leftmost: *mut TaskStruct,
}

impl RbTree {
    pub const fn new() -> Self {
        Self { root: ptr::null_mut(), leftmost: ptr::null_mut() }
    }

    pub unsafe fn insert<F>(&mut self, node: *mut TaskStruct, mut less: F)
    where
        F: FnMut(*mut TaskStruct, *mut TaskStruct) -> bool,
    {
        if node.is_null() { return; }
        debug_assert!(
            !self.contains(node),
            "insert: node jest już częścią tego drzewa (podwójny insert bez remove)"
        );

        let mut parent: *mut TaskStruct = ptr::null_mut();
        let mut current = self.root;
        let mut went_left = true;

        while !current.is_null() {
            parent = current;
            if less(node, current) {
                current = *rb_left(current);
                went_left = true;
            } else {
                current = *rb_right(current);
                went_left = false;
            }
        }

        *rb_left(node) = ptr::null_mut();
        *rb_right(node) = ptr::null_mut();
        rb_set_parent_color(node, parent, RB_RED);

        if parent.is_null() {
            self.root = node;
            self.leftmost = node;
        } else if went_left {
            *rb_left(parent) = node;
            if parent == self.leftmost {
                self.leftmost = node;
            }
        } else {
            *rb_right(parent) = node;
        }

        self.rb_insert_fixup(node);
    }

    fn rb_insert_fixup(&mut self, mut node: *mut TaskStruct) {
        unsafe {
            while rb_is_red(rb_parent(node)) {
                let parent = rb_parent(node);
                let grandparent = rb_parent(parent);
                debug_assert!(!grandparent.is_null(), "czerwony rodzic implikuje istnienie czarnego dziadka");

                if parent == *rb_left(grandparent) {
                    let uncle = *rb_right(grandparent);
                    if rb_is_red(uncle) {
                        rb_set_color(parent, RB_BLACK);
                        rb_set_color(uncle, RB_BLACK);
                        rb_set_color(grandparent, RB_RED);
                        node = grandparent;
                        continue;
                    }
                    if node == *rb_right(parent) {
                        node = parent;
                        self.rotate_left(node);
                    }
                    let parent = rb_parent(node);
                    let grandparent = rb_parent(parent);
                    rb_set_color(parent, RB_BLACK);
                    rb_set_color(grandparent, RB_RED);
                    self.rotate_right(grandparent);
                } else {
                    let uncle = *rb_left(grandparent);
                    if rb_is_red(uncle) {
                        rb_set_color(parent, RB_BLACK);
                        rb_set_color(uncle, RB_BLACK);
                        rb_set_color(grandparent, RB_RED);
                        node = grandparent;
                        continue;
                    }
                    if node == *rb_left(parent) {
                        node = parent;
                        self.rotate_right(node);
                    }
                    let parent = rb_parent(node);
                    let grandparent = rb_parent(parent);
                    rb_set_color(parent, RB_BLACK);
                    rb_set_color(grandparent, RB_RED);
                    self.rotate_left(grandparent);
                }
            }
        }
        rb_set_color(self.root, RB_BLACK);
    }

    unsafe fn rotate_left(&mut self, x: *mut TaskStruct) {
        let y = *rb_right(x);
        *rb_right(x) = *rb_left(y);
        if !(*rb_left(y)).is_null() {
            rb_set_parent(*rb_left(y), x);
        }
        rb_set_parent(y, rb_parent(x));
        let parent = rb_parent(x);
        if parent.is_null() {
            self.root = y;
        } else if x == *rb_left(parent) {
            *rb_left(parent) = y;
        } else {
            *rb_right(parent) = y;
        }
        *rb_left(y) = x;
        rb_set_parent(x, y);
    }

    unsafe fn rotate_right(&mut self, x: *mut TaskStruct) {
        let y = *rb_left(x);
        *rb_left(x) = *rb_right(y);
        if !(*rb_right(y)).is_null() {
            rb_set_parent(*rb_right(y), x);
        }
        rb_set_parent(y, rb_parent(x));
        let parent = rb_parent(x);
        if parent.is_null() {
            self.root = y;
        } else if x == *rb_left(parent) {
            *rb_left(parent) = y;
        } else {
            *rb_right(parent) = y;
        }
        *rb_right(y) = x;
        rb_set_parent(x, y);
    }

    unsafe fn transplant(&mut self, old: *mut TaskStruct, new: *mut TaskStruct) {
        let parent = rb_parent(old);
        if parent.is_null() {
            self.root = new;
        } else if old == *rb_left(parent) {
            *rb_left(parent) = new;
        } else {
            *rb_right(parent) = new;
        }
        if !new.is_null() {
            rb_set_parent(new, parent);
        }
    }

    unsafe fn minimum(mut node: *mut TaskStruct) -> *mut TaskStruct {
        while !(*rb_left(node)).is_null() {
            node = *rb_left(node);
        }
        node
    }

    pub unsafe fn remove(&mut self, node: *mut TaskStruct) {
        if node.is_null() || self.root.is_null() { return; }
        debug_assert!(
            self.contains(node),
            "remove: node nie należy do tego drzewa (usuwanie z cudzego runqueue?)"
        );

        if self.leftmost == node {
            self.leftmost = if !(*rb_right(node)).is_null() {
                Self::minimum(*rb_right(node))
            } else {
                rb_parent(node)
            };
        }

        let mut y = node;
        let mut y_original_color = rb_color(y);
        let x: *mut TaskStruct;
        let x_parent: *mut TaskStruct;

        if (*rb_left(node)).is_null() {
            x = *rb_right(node);
            x_parent = rb_parent(node);
            self.transplant(node, x);
        } else if (*rb_right(node)).is_null() {
            x = *rb_left(node);
            x_parent = rb_parent(node);
            self.transplant(node, x);
        } else {
            y = Self::minimum(*rb_right(node));
            y_original_color = rb_color(y);
            x = *rb_right(y);

            if rb_parent(y) == node {
                x_parent = y;
            } else {
                x_parent = rb_parent(y);
                self.transplant(y, x);
                *rb_right(y) = *rb_right(node);
                rb_set_parent(*rb_right(y), y);
            }

            self.transplant(node, y);
            *rb_left(y) = *rb_left(node);
            rb_set_parent(*rb_left(y), y);
            rb_set_color(y, rb_color(node));
        }

        if y_original_color == RB_BLACK {
            self.remove_fixup(x, x_parent);
        }
    }

    unsafe fn remove_fixup(&mut self, mut x: *mut TaskStruct, mut x_parent: *mut TaskStruct) {
        while x != self.root && !rb_is_red(x) {
            if x_parent.is_null() { break; }

            if x == *rb_left(x_parent) {
                let mut sibling = *rb_right(x_parent);

                if rb_is_red(sibling) {
                    rb_set_color(sibling, RB_BLACK);
                    rb_set_color(x_parent, RB_RED);
                    self.rotate_left(x_parent);
                    sibling = *rb_right(x_parent);
                }

                if !rb_is_red(*rb_left(sibling)) && !rb_is_red(*rb_right(sibling)) {
                    rb_set_color(sibling, RB_RED);
                    x = x_parent;
                    x_parent = rb_parent(x);
                    continue;
                }

                if !rb_is_red(*rb_right(sibling)) {
                    rb_set_color(*rb_left(sibling), RB_BLACK);
                    rb_set_color(sibling, RB_RED);
                    self.rotate_right(sibling);
                    sibling = *rb_right(x_parent);
                }

                rb_set_color(sibling, rb_color(x_parent));
                rb_set_color(x_parent, RB_BLACK);
                rb_set_color(*rb_right(sibling), RB_BLACK);
                self.rotate_left(x_parent);
                x = self.root;
                x_parent = ptr::null_mut();
            } else {
                let mut sibling = *rb_left(x_parent);

                if rb_is_red(sibling) {
                    rb_set_color(sibling, RB_BLACK);
                    rb_set_color(x_parent, RB_RED);
                    self.rotate_right(x_parent);
                    sibling = *rb_left(x_parent);
                }

                if !rb_is_red(*rb_right(sibling)) && !rb_is_red(*rb_left(sibling)) {
                    rb_set_color(sibling, RB_RED);
                    x = x_parent;
                    x_parent = rb_parent(x);
                    continue;
                }

                if !rb_is_red(*rb_left(sibling)) {
                    rb_set_color(*rb_right(sibling), RB_BLACK);
                    rb_set_color(sibling, RB_RED);
                    self.rotate_left(sibling);
                    sibling = *rb_left(x_parent);
                }

                rb_set_color(sibling, rb_color(x_parent));
                rb_set_color(x_parent, RB_BLACK);
                rb_set_color(*rb_left(sibling), RB_BLACK);
                self.rotate_right(x_parent);
                x = self.root;
                x_parent = ptr::null_mut();
            }
        }
        rb_set_color(x, RB_BLACK);
    }

    pub unsafe fn next(&self, node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() { return ptr::null_mut(); }
        if !(*rb_right(node)).is_null() {
            return Self::minimum(*rb_right(node));
        }
        let mut cur = node;
        let mut parent = rb_parent(cur);
        while !parent.is_null() && cur == *rb_right(parent) {
            cur = parent;
            parent = rb_parent(cur);
        }
        parent
    }

    pub unsafe fn prev(&self, node: *mut TaskStruct) -> *mut TaskStruct {
        if node.is_null() { return ptr::null_mut(); }
        if !(*rb_left(node)).is_null() {
            let mut n = *rb_left(node);
            while !(*rb_right(n)).is_null() {
                n = *rb_right(n);
            }
            return n;
        }
        let mut cur = node;
        let mut parent = rb_parent(cur);
        while !parent.is_null() && cur == *rb_left(parent) {
            cur = parent;
            parent = rb_parent(cur);
        }
        parent
    }

    /// Sprawdza, czy `node` jest aktualnie członkiem TEGO drzewa,
    /// przechodząc od `node` do korzenia i porównując z `self.root`.
    ///
    /// O(log n) — nie ma tańszego sposobu bez dodatkowego pola
    /// "właściciel drzewa" w węźle. Używane głównie w
    /// `debug_assert!` przy `remove`, żeby wyłapać próbę usunięcia
    /// węzła z cudzego drzewa (klasyczny błąd przy wielu
    /// runqueue na różnych CPU, gdzie task został przeniesiony,
    /// a stary wskaźnik do drzewa nie został zaktualizowany).
    pub unsafe fn contains(&self, node: *mut TaskStruct) -> bool {
        if node.is_null() || self.root.is_null() { return false; }
        let mut cur = node;
        while !rb_parent(cur).is_null() {
            cur = rb_parent(cur);
        }
        cur == self.root
    }

    /// Waliduje inwarianty czerwono-czarnego drzewa:
    /// 1. korzeń jest czarny,
    /// 2. czerwony węzeł nie ma czerwonego dziecka,
    /// 3. każda ścieżka od węzła do liścia ma tę samą liczbę
    ///    czarnych węzłów (czarna wysokość),
    /// 4. `leftmost` faktycznie wskazuje najmniejszy węzeł.
    ///
    /// To NIE jest funkcja do wywoływania na hot-pathcie (koszt
    /// O(n)) — służy do `debug_assert!` w testach i po większych
    /// zmianach w `insert_fixup`/`remove_fixup`, gdzie łatwo o
    /// subtelny błąd psujący balans drzewa bez widocznego panicu.
    /// Kolega słusznie zauważył, że plik "spełniał samo minimum" —
    /// bez tej funkcji nie ma jak automatycznie zweryfikować, że
    /// fixupy faktycznie utrzymują własności RB po serii insert/remove.
    pub unsafe fn is_valid(&self) -> bool {
        if self.root.is_null() {
            return self.leftmost.is_null();
        }
        if rb_color(self.root) != RB_BLACK {
            return false;
        }
        if Self::black_height(self.root).is_none() {
            return false;
        }
        let computed_leftmost = Self::minimum(self.root);
        computed_leftmost == self.leftmost
    }

    /// Zwraca czarną wysokość poddrzewa zakorzenionego w `node`,
    /// albo `None`, jeśli inwarianty RB są złamane w tym poddrzewie
    /// (czerwony węzeł z czerwonym dzieckiem, albo niezgodna czarna
    /// wysokość między lewym a prawym poddrzewem).
    unsafe fn black_height(node: *mut TaskStruct) -> Option<usize> {
        if node.is_null() {
            return Some(0);
        }
        if rb_is_red(node) {
            let left = *rb_left(node);
            let right = *rb_right(node);
            if rb_is_red(left) || rb_is_red(right) {
                return None;
            }
        }
        let left_height = Self::black_height(*rb_left(node))?;
        let right_height = Self::black_height(*rb_right(node))?;
        if left_height != right_height {
            return None;
        }
        let own = if rb_color(node) == RB_BLACK { 1 } else { 0 };
        Some(left_height + own)
    }
}
