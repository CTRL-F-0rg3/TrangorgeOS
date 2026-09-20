use core::ptr;

#[repr(C)]
pub struct ListHead {
    next: *mut ListHead,
    prev: *mut ListHead,
}

impl ListHead {
    pub const fn new() -> Self {
        Self { next: ptr::null_mut(), prev: ptr::null_mut() }
    }

    pub fn init(&mut self) {
        self.next = self as *mut _;
        self.prev = self as *mut _;
    }

    pub fn add(&mut self, new_node: *mut ListHead) {
        unsafe {
            let next = self.next;
            (*new_node).next = next;
            (*new_node).prev = self as *mut _;
            (*next).prev = new_node;
            self.next = new_node;
        }
    }

    pub fn add_tail(&mut self, new_node: *mut ListHead) {
        unsafe {
            let prev = self.prev;
            (*new_node).next = self as *mut _;
            (*new_node).prev = prev;
            (*prev).next = new_node;
            self.prev = new_node;
        }
    }

    pub fn del(&mut self) {
        unsafe {
            let prev = self.prev;
            let next = self.next;
            (*prev).next = next;
            (*next).prev = prev;
            self.init();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.next as *const _ == self as *const _
    }
}

#[macro_export]
macro_rules! list_entry {
    ($ptr:expr, $ty:ty, $field:ident) => {
        ($ptr as usize - $crate::offset_of!($ty, $field)) as *mut $ty
    };
}