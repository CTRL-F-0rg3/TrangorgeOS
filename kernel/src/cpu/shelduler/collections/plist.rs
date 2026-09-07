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

    pub unsafe fn insert(&mut self, task: *mut TaskStruct, priority: i32) {
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
            let iter_task = Self::task_from_node(iter);
            let iter_prio = plist_prio(iter_task);

            if iter_prio == priority {
                (*plist_same_prio(iter_task)).insert_before(plist_same_prio(task));
                return;
            }
            if iter_prio > priority {
                break;
            }
            iter = (*iter).next;
        }

        (*iter).insert_before(plist_node(task));
    }

    pub unsafe fn remove(&mut self, task: *mut TaskStruct) {
        let same_prio = plist_same_prio(task);
        let node = plist_node(task);

        if !(*same_prio).is_empty() {
            let next_same_prio_node = (*same_prio).next;
            let next_task = Self::task_from_same_prio(next_same_prio_node);

            if !(*node).is_empty() {
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

    pub fn first(&self) -> *mut TaskStruct {
        if self.head.is_empty() { return ptr::null_mut(); }
        unsafe { Self::task_from_node(self.head.next) }
    }

    pub fn last(&self) -> *mut TaskStruct {
        if self.head.is_empty() { return ptr::null_mut(); }
        unsafe {
            let last_level = Self::task_from_node(self.head.prev);
            let same_prio = plist_same_prio(last_level);
            if (*same_prio).is_empty() {
                last_level
            } else {
                Self::task_from_same_prio((*same_prio).prev)
            }
        }
    }

    #[inline(always)]
    unsafe fn task_from_node(node: *mut ListHead) -> *mut TaskStruct {
        (node as *mut u8).sub(TaskStruct::PLIST_NODE_OFFSET) as *mut TaskStruct
    }

    #[inline(always)]
    unsafe fn task_from_same_prio(node: *mut ListHead) -> *mut TaskStruct {
        (node as *mut u8).sub(TaskStruct::PLIST_SAME_PRIO_OFFSET) as *mut TaskStruct
    }
}
