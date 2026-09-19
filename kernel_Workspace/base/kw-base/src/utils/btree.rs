use crate::core::KResult;
use kstd_alloc::{Allocator, Layout, KernelHeap};
use core::mem;

const B: usize = 4;
const MAX_KEYS: usize = 2 * B - 1;
const MIN_KEYS: usize = B - 1;

struct Node<K, V> {
    keys: [mem::MaybeUninit<K>; MAX_KEYS],
    vals: [mem::MaybeUninit<V>; MAX_KEYS],
    children: [*mut Node<K, V>; MAX_KEYS + 1],
    len: usize,
    leaf: bool,
}

impl<K: Ord, V> Node<K, V> {
    fn new_leaf() -> *mut Self {
        let layout = Layout::new(mem::size_of::<Self>(), mem::align_of::<Self>());
        let ptr = KernelHeap.alloc(layout).unwrap() as *mut Self;
        unsafe {
            (*ptr).len = 0;
            (*ptr).leaf = true;
            (*ptr).children = [core::ptr::null_mut(); MAX_KEYS + 1];
        }
        ptr
    }

    fn new_internal() -> *mut Self {
        let ptr = Self::new_leaf();
        unsafe { (*ptr).leaf = false; }
        ptr
    }
}

pub struct BTreeMap<K, V> {
    root: *mut Node<K, V>,
    len: usize,
}

unsafe impl<K: Send, V: Send> Send for BTreeMap<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for BTreeMap<K, V> {}

impl<K: Ord, V> BTreeMap<K, V> {
    pub fn new() -> Self {
        Self { root: core::ptr::null_mut(), len: 0 }
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.root.is_null() {
            self.root = Node::new_leaf();
        }

        let r = self.root;
        if unsafe { (*r).len == MAX_KEYS } {
            let s = Node::new_internal();
            unsafe {
                (*s).children[0] = r;
                self.split_child(s, 0);
            }
            self.root = s;
            self.insert_non_full(s, key, val);
        } else {
            self.insert_non_full(r, key, val);
        }
        self.len += 1;
    }

    fn split_child(&mut self, parent: *mut Node<K, V>, i: usize) {
        let child = unsafe { (*parent).children[i] };
        let new_node = if unsafe { (*child).leaf } { Node::new_leaf() } else { Node::new_internal() };
        let mid = MIN_KEYS;

        unsafe {
            for j in 0..MIN_KEYS {
                (*new_node).keys[j] = mem::MaybeUninit::new((*child).keys[mid + 1 + j].assume_init_read());
                (*new_node).vals[j] = mem::MaybeUninit::new((*child).vals[mid + 1 + j].assume_init_read());
            }
            if !(*child).leaf {
                for j in 0..B {
                    (*new_node).children[j] = (*child).children[mid + 1 + j];
                }
            }
            (*new_node).len = MIN_KEYS;
            (*child).len = MIN_KEYS;

            for j in ((*parent).len + 1..i + 1).rev() {
                (*parent).children[j] = (*parent).children[j - 1];
            }
            (*parent).children[i + 1] = new_node;

            for j in ((*parent).len..i).rev() {
                (*parent).keys[j] = mem::MaybeUninit::new((*parent).keys[j - 1].assume_init_read());
                (*parent).vals[j] = mem::MaybeUninit::new((*parent).vals[j - 1].assume_init_read());
            }
            (*parent).keys[i] = mem::MaybeUninit::new((*child).keys[mid].assume_init_read());
            (*parent).vals[i] = mem::MaybeUninit::new((*child).vals[mid].assume_init_read());
            (*parent).len += 1;
        }
    }

    fn insert_non_full(&mut self, node: *mut Node<K, V>, key: K, val: V) {
        let mut i = unsafe { (*node).len };
        unsafe {
            if (*node).leaf {
                while i > 0 && key < *(*node).keys[i - 1].assume_init_ref() {
                    (*node).keys[i] = mem::MaybeUninit::new((*node).keys[i - 1].assume_init_read());
                    (*node).vals[i] = mem::MaybeUninit::new((*node).vals[i - 1].assume_init_read());
                    i -= 1;
                }
                (*node).keys[i] = mem::MaybeUninit::new(key);
                (*node).vals[i] = mem::MaybeUninit::new(val);
                (*node).len += 1;
            } else {
                while i > 0 && key < *(*node).keys[i - 1].assume_init_ref() {
                    i -= 1;
                }
                if (*(*node).children[i]).len == MAX_KEYS {
                    self.split_child(node, i);
                    if key > *(*node).keys[i].assume_init_ref() {
                        i += 1;
                    }
                }
                self.insert_non_full((*node).children[i], key, val);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if self.root.is_null() { return None; }
        let mut node = self.root;
        loop {
            let n = unsafe { &*node };
            let mut i = 0;
            while i < n.len && *key > *unsafe { n.keys[i].assume_init_ref() } {
                i += 1;
            }
            if i < n.len && *key == *unsafe { n.keys[i].assume_init_ref() } {
                return Some(unsafe { n.vals[i].assume_init_ref() });
            }
            if n.leaf { return None; }
            node = n.children[i];
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}