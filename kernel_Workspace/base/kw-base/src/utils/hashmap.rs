use crate::core::{KResult, KernelError};
use kstd_alloc::{Allocator, Layout, KernelHeap};
use core::hash::{Hash, Hasher};
use core::mem;

struct FnvHasher(u64);
impl Hasher for FnvHasher {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

fn hash_key<K: Hash>(key: &K) -> usize {
    let mut h = FnvHasher(0xcbf29ce484222325);
    key.hash(&mut h);
    h.finish() as usize
}

enum Slot<K, V> { Empty, Occupied(K, V), Tombstone }

pub struct HashMap<K, V> {
    slots: *mut Slot<K, V>,
    cap: usize,
    len: usize,
}

unsafe impl<K: Send, V: Send> Send for HashMap<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for HashMap<K, V> {}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self { slots: core::ptr::null_mut(), cap: 0, len: 0 }
    }

    fn grow(&mut self, new_cap: usize) {
        let layout = Layout::new(new_cap * mem::size_of::<Slot<K, V>>(), mem::align_of::<Slot<K, V>>());
        let new_slots = KernelHeap.alloc(layout).unwrap() as *mut Slot<K, V>;
        for i in 0..new_cap {
            unsafe { new_slots.add(i).write(Slot::Empty); }
        }

        let old_slots = self.slots;
        let old_cap = self.cap;
        self.slots = new_slots;
        self.cap = new_cap;
        self.len = 0;

        if !old_slots.is_null() {
            for i in 0..old_cap {
                let slot = unsafe { old_slots.add(i).read() };
                if let Slot::Occupied(k, v) = slot {
                    self.insert(k, v);
                }
            }
            let old_layout = Layout::new(old_cap * mem::size_of::<Slot<K, V>>(), mem::align_of::<Slot<K, V>>());
            KernelHeap.dealloc(old_slots as *mut u8, old_layout);
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        if self.cap == 0 || self.len * 4 >= self.cap * 3 {
            self.grow(if self.cap == 0 { 8 } else { self.cap * 2 });
        }
        let h = hash_key(&key);
        let mut idx = h % self.cap;
        loop {
            let slot = unsafe { &mut *self.slots.add(idx) };
            match slot {
                Slot::Empty | Slot::Tombstone => {
                    *slot = Slot::Occupied(key, val);
                    self.len += 1;
                    return;
                }
                Slot::Occupied(k, v) if *k == key => {
                    *slot = Slot::Occupied(key, val);
                    return;
                }
                _ => { idx = (idx + 1) % self.cap; }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if self.cap == 0 { return None; }
        let h = hash_key(key);
        let mut idx = h % self.cap;
        loop {
            let slot = unsafe { &*self.slots.add(idx) };
            match slot {
                Slot::Empty => return None,
                Slot::Occupied(k, v) if *k == *key => return Some(v),
                _ => { idx = (idx + 1) % self.cap; }
            }
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.cap == 0 { return None; }
        let h = hash_key(key);
        let mut idx = h % self.cap;
        loop {
            let slot = unsafe { &mut *self.slots.add(idx) };
            match slot {
                Slot::Empty => return None,
                Slot::Occupied(k, v) if *k == *key => return Some(v),
                _ => { idx = (idx + 1) % self.cap; }
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.cap == 0 { return None; }
        let h = hash_key(key);
        let mut idx = h % self.cap;
        loop {
            let slot = unsafe { &mut *self.slots.add(idx) };
            match slot {
                Slot::Empty => return None,
                Slot::Occupied(k, _) if *k == *key => {
                    let old = mem::replace(slot, Slot::Tombstone);
                    self.len -= 1;
                    if let Slot::Occupied(_, v) = old { return Some(v); }
                }
                _ => { idx = (idx + 1) % self.cap; }
            }
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

impl<K, V> Drop for HashMap<K, V> {
    fn drop(&mut self) {
        if !self.slots.is_null() {
            for i in 0..self.cap {
                unsafe { self.slots.add(i).drop_in_place(); }
            }
            let layout = Layout::new(self.cap * mem::size_of::<Slot<K, V>>(), mem::align_of::<Slot<K, V>>());
            KernelHeap.dealloc(self.slots as *mut u8, layout);
        }
    }
}