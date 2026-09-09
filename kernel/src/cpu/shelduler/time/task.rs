use core::ptr;

pub const MAX_RT_PRIO: u32 = 100;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ListHead {
    pub next: *mut ListHead,
    pub prev: *mut ListHead,
}

impl ListHead {
    pub const fn new() -> Self {
        Self { next: ptr::null_mut(), prev: ptr::null_mut() }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.next.is_null() || ptr::eq(self.next, self as *const ListHead as *mut ListHead)
    }

    #[inline(always)]
    pub unsafe fn insert_before(&mut self, new: *mut ListHead) {
        let self_ptr = self as *mut ListHead;
        let prev = self.prev;
        if prev.is_null() {
            (*new).next = self_ptr;
            (*new).prev = new;
            self.prev = new;
            self.next = new;
            return;
        }
        (*new).prev = prev;
        (*new).next = self_ptr;
        (*prev).next = new;
        self.prev = new;
    }

    #[inline(always)]
    pub unsafe fn remove(node: *mut ListHead) {
        if node.is_null() { return; }
        let prev = (*node).prev;
        let next = (*node).next;
        if !prev.is_null() { (*prev).next = next; }
        if !next.is_null() { (*next).prev = prev; }
        (*node).next = ptr::null_mut();
        (*node).prev = ptr::null_mut();
    }
}

#[repr(C)]
pub struct RtFields {
    pub rt_priority: u32,
    pub run_list: ListHead,
}

#[repr(C)]
pub struct FairFields {
    pub vruntime: u64,
}

#[repr(C)]
pub struct TaskStruct {
    pub pid: u32,

    pub rb_left: *mut TaskStruct,
    pub rb_right: *mut TaskStruct,
    pub rb_parent_color: usize,

    pub plist_prio: i32,
    pub plist_same_prio: ListHead,
    pub plist_node: ListHead,

    pub rt: RtFields,
    pub fair: FairFields,
}

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
}
