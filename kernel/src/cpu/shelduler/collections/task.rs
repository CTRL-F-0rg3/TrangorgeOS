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
    pub run_list: *mut ListHead,
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
}
