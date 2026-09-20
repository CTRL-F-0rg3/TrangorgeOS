use core::{mem, ops::{Deref, DerefMut}, ptr::{self, NonNull}};
use kstd_alloc::{Allocator, Layout, KernelHeap};

pub struct Vec<T> {
    ptr: NonNull<T>,
    len: usize,
    cap: usize,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub const fn new() -> Self {
        Self { ptr: NonNull::dangling(), len: 0, cap: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let mut v = Self::new();
        if cap > 0 { v.grow_to(cap); }
        v
    }

    fn grow_to(&mut self, new_cap: usize) {
        let elem_size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        let new_ptr = if self.cap == 0 {
            KernelHeap.alloc(Layout::new(new_cap * elem_size, align)).unwrap() as *mut T
        } else {
            KernelHeap.realloc(
                self.ptr.as_ptr() as *mut u8, 
                Layout::new(self.cap * elem_size, align), 
                new_cap * elem_size
            ).unwrap() as *mut T
        };
        self.ptr = NonNull::new(new_ptr).unwrap();
        self.cap = new_cap;
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap {
            let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
            self.grow_to(new_cap);
        }
        unsafe { ptr::write(self.ptr.as_ptr().add(self.len), val); }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 { None } 
        else {
            self.len -= 1;
            Some(unsafe { ptr::read(self.ptr.as_ptr().add(self.len)) })
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.cap }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T: Copy> Vec<T> {
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let new_len = self.len + slice.len();
        if new_len > self.cap {
            let mut new_cap = if self.cap == 0 { 4 } else { self.cap };
            while new_cap < new_len { new_cap *= 2; }
            self.grow_to(new_cap);
        }
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), self.ptr.as_ptr().add(self.len), slice.len());
        }
        self.len = new_len;
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] { self.as_slice() }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] { self.as_mut_slice() }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            unsafe {
                for i in 0..self.len { ptr::drop_in_place(self.ptr.as_ptr().add(i)); }
                let layout = Layout::new(self.cap * mem::size_of::<T>(), mem::align_of::<T>());
                KernelHeap.dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}