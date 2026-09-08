use crate::cpu::scheduler::entities::task::{TaskStruct, ListHead};
use core::ptr;

#[inline(always)]
unsafe fn plist_prio_slot(node: *mut TaskStruct) -> *mut i32 {
    (node as *mut u8).add(TaskStruct::PLIST_PRIO_OFFSET) as *mut i32
}

#[inline(always)]
unsafe fn plist_same_prio(node: *mut TaskStruct) -> *mut ListHead {
    (node as *mut u8).add(TaskStruct::PLIST_SAME_PRIO_OFFSET) as *mut ListHead
}

#[inline(always)]
unsafe fn plist_node(node: *mut TaskStruct) -> *mut ListHead {
    (node as *mut u8).add(TaskStruct::PLIST_NODE_OFFSET) as *mut ListHead
}

#[inline(always)]
fn plist_prio(node: *mut TaskStruct) -> i32 {
    unsafe { *plist_prio_slot(node) }
}

/// Odtwarza `TaskStruct` z węzła `plist_node`.
///
/// Przechodzi przez `TaskStruct::container_of`, więc w trybie debug
/// wynik jest zweryfikowany round-tripem — błędny `PLIST_NODE_OFFSET`
/// (np. po zmianie layoutu `TaskStruct` bez aktualizacji stałej)
/// wywala się jako panika w testach, a nie jako ciche uszkodzenie
/// pamięci na produkcji.
#[inline(always)]
unsafe fn task_from_node(node: *mut ListHead) -> *mut TaskStruct {
    TaskStruct::container_of(node, TaskStruct::PLIST_NODE_OFFSET)
}

/// Jak `task_from_node`, ale dla węzła `plist_same_prio`.
#[inline(always)]
unsafe fn task_from_same_prio(node: *mut ListHead) -> *mut TaskStruct {
    TaskStruct::container_of(node, TaskStruct::PLIST_SAME_PRIO_OFFSET)
}

pub struct PList {
    pub head: ListHead,
}

impl PList {
    pub const fn new() -> Self {
        Self { head: ListHead::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_empty()
    }

    /// Wstawia `task` z priorytetem `priority`, zachowując porządek
    /// rosnący po poziomach oraz FIFO w obrębie tego samego priorytetu.
    ///
    /// # Bezpieczeństwo
    /// `task` musi być przypięty (nie przenoszony w pamięci) przez
    /// cały czas obecności na liście, i nie może już być na niej
    /// obecny (podwójny insert bez remove uszkodzi listę — patrz
    /// `debug_assert` poniżej, który to wyłapuje w testach).
    pub unsafe fn insert(&mut self, task: *mut TaskStruct, priority: i32) {
        debug_assert!(!task.is_null(), "insert: task nie może być null");
        debug_assert!(
            (*plist_node(task)).is_empty() && (*plist_same_prio(task)).is_empty(),
            "insert: task jest już wpięty w listę (podwójny insert bez remove)"
        );

        *plist_prio_slot(task) = priority;
        *plist_same_prio(task) = ListHead::new();
        *plist_node(task) = ListHead::new();

        if self.head.is_empty() {
            self.head.insert_before(plist_node(task));
            return;
        }

        let head_ptr = &mut self.head as *mut ListHead;
        let mut iter = self.head.next;

        while iter != head_ptr {
            let iter_task = task_from_node(iter);
            let iter_prio = plist_prio(iter_task);

            if iter_prio == priority {
                // Istniejący poziom: dopisz do FIFO tego poziomu.
                // Head poziomu (`iter_task`) trzyma w `same_prio.prev`
                // wskaźnik na ogon FIFO — to właśnie czyni `last()`
                // (patrz niżej) operacją O(1) zamiast liniowego
                // przeszukania, na które narzekał kod przed poprawką.
                let level_head_same_prio = plist_same_prio(iter_task);
                let tail = if (*level_head_same_prio).is_empty() {
                    level_head_same_prio
                } else {
                    (*level_head_same_prio).prev
                };
                (*tail).insert_before(plist_same_prio(task));
                return;
            }
            if iter_prio > priority {
                break;
            }
            iter = (*iter).next;
        }

        (*iter).insert_before(plist_node(task));
    }

    /// Usuwa `task` z listy w czasie O(1).
    ///
    /// # Bezpieczeństwo
    /// `task` musi aktualnie znajdować się na tej liście.
    pub unsafe fn remove(&mut self, task: *mut TaskStruct) {
        debug_assert!(!task.is_null(), "remove: task nie może być null");
        debug_assert!(
            !(*plist_node(task)).is_empty() || !(*plist_same_prio(task)).is_empty(),
            "remove: task nie jest obecny na żadnej liście (double-remove albo brak insert)"
        );

        let same_prio = plist_same_prio(task);
        let node = plist_node(task);

        if !(*same_prio).is_empty() {
            let next_same_prio_node = (*same_prio).next;
            let next_task = task_from_same_prio(next_same_prio_node);

            if !(*node).is_empty() {
                // `task` był head'em poziomu (miał wpis w `plist_node`,
                // czyli był wpięty w listę poziomów) — awansuj kolejny
                // element FIFO na jego miejsce, zarówno w liście
                // poziomów, jak i jako nowy head `same_prio`.
                let list_next = (*node).next;
                let list_prev = (*node).prev;
                ListHead::remove(node);
                (*plist_node(next_task)).next = list_next;
                (*plist_node(next_task)).prev = list_prev;
                if !list_next.is_null() { (*list_next).prev = plist_node(next_task); }
                if !list_prev.is_null() { (*list_prev).next = plist_node(next_task); }
            }
            ListHead::remove(same_prio);
            return;
        }

        ListHead::remove(node);
    }

    /// Zadanie o najwyższym priorytecie (head listy poziomów),
    /// pierwsze w kolejności FIFO na tym poziomie. O(1).
    pub fn first(&self) -> *mut TaskStruct {
        if self.head.is_empty() { return ptr::null_mut(); }
        unsafe { task_from_node(self.head.next) }
    }

    /// Zadanie o najniższym priorytecie (ostatni poziom), ostatnie
    /// w kolejności FIFO na tym poziomie.
    ///
    /// O(1) — poprzednio ta funkcja mogła degenerować się do O(N)
    /// przy wielu zadaniach na tym samym (najniższym) priorytecie,
    /// bo szła liniowo przez `same_prio.prev` licząc na to, że
    /// wskaźnik "prev" head'a wskazuje bezpośrednio na ogon (co nie
    /// było gwarantowane przez ówczesny `insert`). Teraz `insert`
    /// utrzymuje ten inwariant jawnie: `same_prio.prev` head'a
    /// zawsze wskazuje ogon FIFO, więc odczyt jest bezpośredni.
    pub fn last(&self) -> *mut TaskStruct {
        if self.head.is_empty() { return ptr::null_mut(); }
        unsafe {
            let last_level = task_from_node(self.head.prev);
            let same_prio = plist_same_prio(last_level);
            if (*same_prio).is_empty() {
                last_level
            } else {
                task_from_same_prio((*same_prio).prev)
            }
        }
    }
}
