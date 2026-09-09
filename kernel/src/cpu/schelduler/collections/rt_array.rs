use crate::cpu::scheduler::entities::task::{TaskStruct, ListHead, MAX_RT_PRIO};
use core::ptr;

pub struct RtArray {
    pub queue: [ListHead; MAX_RT_PRIO as usize],
    pub bitmap: [u64; 2],
    pub nr_running: usize,
}

impl RtArray {
    pub const fn new() -> Self {
        Self {
            queue: [ListHead::new(); MAX_RT_PRIO as usize],
            bitmap: [0, 0],
            nr_running: 0,
        }
    }

    #[inline(always)]
    pub fn set_bit(&mut self, prio: usize) {
        let word = prio / 64;
        let bit = prio % 64;
        self.bitmap[word] |= 1u64 << bit;
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, prio: usize) {
        let word = prio / 64;
        let bit = prio % 64;
        self.bitmap[word] &= !(1u64 << bit);
    }

    #[inline(always)]
    fn bit_is_set(&self, prio: usize) -> bool {
        let word = prio / 64;
        let bit = prio % 64;
        (self.bitmap[word] & (1u64 << bit)) != 0
    }

    #[inline(always)]
    pub fn highest_prio(&self) -> Option<usize> {
        if self.bitmap[0] != 0 {
            Some(self.bitmap[0].trailing_zeros() as usize)
        } else if self.bitmap[1] != 0 {
            Some(64 + self.bitmap[1].trailing_zeros() as usize)
        } else {
            None
        }
    }

    /// Dodaje `task` do kolejki jego priorytetu i ustawia bit
    /// w bitmapie. O(1).
    ///
    /// # Bezpieczeństwo
    /// `task` musi być przypięty przez cały czas obecności w tablicy
    /// i nie może być już zakolejkowany (podwójny enqueue bez
    /// dequeue uszkodzi listę — wyłapane przez `debug_assert` poniżej).
    ///
    /// # Panika
    /// Panikuje, gdy `(*task).rt.rt_priority >= MAX_RT_PRIO` — to
    /// błąd wywołującego (priorytet spoza zakresu nigdy nie powinien
    /// dotrzeć tak nisko w planiście), sprawdzany też w trybie
    /// release przez `assert!`, bo indeksowanie poza `self.queue`
    /// byłoby UB, nie tylko błędem logicznym.
    pub unsafe fn enqueue(&mut self, task: *mut TaskStruct) {
        let prio = (*task).rt.rt_priority as usize;
        assert!(prio < MAX_RT_PRIO as usize, "rt_priority poza zakresem RT (0..{MAX_RT_PRIO})");
        debug_assert!(
            (*task).rt.run_list.is_empty(),
            "enqueue: task jest już zakolejkowany (podwójny enqueue bez dequeue)"
        );

        let list = &mut self.queue[prio];
        if list.is_empty() {
            self.set_bit(prio);
        }

        let run_list_ptr = &mut (*task).rt.run_list as *mut ListHead;
        list.insert_before(run_list_ptr);
        self.nr_running += 1;
    }

    /// Usuwa `task` z kolejki jego priorytetu. Jeśli kolejka
    /// staje się pusta, czyści bit w bitmapie. O(1).
    ///
    /// # Bezpieczeństwo
    /// `task` musi aktualnie być zakolejkowany w tej tablicy.
    ///
    /// # Panika
    /// Jak w `enqueue` — nieprawidłowy priorytet panikuje zawsze,
    /// nie tylko w debug, bo prowadziłoby do UB przy indeksowaniu.
    pub unsafe fn dequeue(&mut self, task: *mut TaskStruct) {
        let prio = (*task).rt.rt_priority as usize;
        assert!(prio < MAX_RT_PRIO as usize, "rt_priority poza zakresem RT (0..{MAX_RT_PRIO})");
        debug_assert!(
            !(*task).rt.run_list.is_empty(),
            "dequeue: task nie jest zakolejkowany (double-dequeue albo brak enqueue)"
        );

        let run_list_ptr = &mut (*task).rt.run_list as *mut ListHead;
        ListHead::remove(run_list_ptr);

        if self.queue[prio].is_empty() {
            debug_assert!(
                self.bit_is_set(prio),
                "dequeue: kolejka pusta, ale bit w bitmapie nie był ustawiony — bitmapa i kolejki rozjechały się wcześniej"
            );
            self.clear_bit(prio);
        }
        self.nr_running -= 1;
    }

    /// Zwraca zadanie na czele kolejki o najwyższym priorytecie,
    /// przesuwając je na koniec tej samej kolejki (round-robin
    /// w obrębie priorytetu) — standardowe zachowanie SCHED_RR.
    /// O(1): pierwszy ustawiony bit przez `trailing_zeros`
    /// (kompiluje się do `TZCNT`/`BSF`), niezależnie od liczby
    /// zakolejkowanych zadań.
    pub unsafe fn pick_next(&mut self) -> *mut TaskStruct {
        let prio = match self.highest_prio() {
            Some(p) => p,
            None => return ptr::null_mut(),
        };
        let list = &mut self.queue[prio];
        if list.is_empty() {
            return ptr::null_mut();
        }

        let next = list.next;
        let task = task_from_run_list(next);

        // Round-robin: przesuń wybrany węzeł na koniec kolejki tego
        // samego priorytetu, żeby kolejne wywołanie pick_next zwróciło
        // następne zadanie z tego poziomu, a nie zawsze to samo.
        ListHead::remove(next);
        list.insert_before(next);

        task
    }

    /// Liczba unikalnych aktywnych poziomów priorytetu (nie mylić
    /// z `nr_running`, które liczy wszystkie zakolejkowane zadania).
    ///
    /// Dopisane, bo bez tego nie da się odróżnić "jeden proces o
    /// wysokim priorytecie" od "wiele procesów rozłożonych na wiele
    /// priorytetów" bez przechodzenia całej bitmapy ręcznie za każdym
    /// razem, gdy taka informacja jest potrzebna (np. w heurystykach
    /// load-balancingu między CPU).
    pub fn active_levels(&self) -> u32 {
        self.bitmap[0].count_ones() + self.bitmap[1].count_ones()
    }
}

/// Odtwarza `TaskStruct` z węzła `rt.run_list`.
///
/// Przechodzi przez `TaskStruct::container_of`, więc w trybie debug
/// wynik jest zweryfikowany round-tripem (patrz dokumentacja
/// `container_of` w `task.rs`) — spójnie z tym, jak `plist.rs`
/// odtwarza `TaskStruct` ze swoich węzłów.
#[inline(always)]
unsafe fn task_from_run_list(node: *mut ListHead) -> *mut TaskStruct {
    TaskStruct::container_of(node, TaskStruct::RT_RUN_LIST_OFFSET)
}
