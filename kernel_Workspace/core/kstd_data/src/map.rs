use crate::vec::Vec;
use core::hash::{Hash, Hasher};

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

enum Bucket<K, V> { Empty, Occupied(K, V), Tombstone }

pub struct HashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    len: usize,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self { buckets: Vec::new(), len: 0 }
    }

    fn hash_key(key: &K) -> usize {
        let mut hasher = FnvHasher(0xcbf29ce484222325);
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn idx(&self, hash: usize) -> usize { hash % self.buckets.len() }

    pub fn insert(&mut self, key: K, val: V) {
        if self.buckets.is_empty() || (self.len * 4) >= (self.buckets.len() * 3) {
            self.resize(if self.buckets.is_empty() { 8 } else { self.buckets.len() * 2 });
        }
        
        let hash = Self::hash_key(&key);
        let mut idx = self.idx(hash);
        
        loop {
            match &self.buckets[idx] {
                Bucket::Empty | Bucket::Tombstone => {
                    self.buckets[idx] = Bucket::Occupied(key, val);
                    self.len += 1;
                    return;
                }
                Bucket::Occupied(k, _) if *k == key => {
                    self.buckets[idx] = Bucket::Occupied(key, val);
                    return;
                }
                _ => { idx = (idx + 1) % self.buckets.len(); }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        if self.buckets.is_empty() { return None; }
        let hash = Self::hash_key(key);
        let mut idx = self.idx(hash);
        
        loop {
            match &self.buckets[idx] {
                Bucket::Empty => return None,
                Bucket::Occupied(k, v) if *k == *key => return Some(v),
                _ => { idx = (idx + 1) % self.buckets.len(); }
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.buckets.is_empty() { return None; }
        let hash = Self::hash_key(key);
        let mut idx = self.idx(hash);
        
        loop {
            match &self.buckets[idx] {
                Bucket::Empty => return None,
                Bucket::Occupied(k, _) if *k == *key => {
                    let old = core::mem::replace(&mut self.buckets[idx], Bucket::Tombstone);
                    self.len -= 1;
                    if let Bucket::Occupied(_, v) = old { return Some(v); }
                }
                _ => { idx = (idx + 1) % self.buckets.len(); }
            }
        }
    }

    fn resize(&mut self, new_cap: usize) {
        let old_buckets = core::mem::replace(&mut self.buckets, Vec::with_capacity(new_cap));
        for _ in 0..new_cap { self.buckets.push(Bucket::Empty); }

        self.len = 0;
        for b in old_buckets.as_slice() {
            if let Bucket::Occupied(k, v) = b {
                let k = unsafe { core::ptr::read(k) };
                let v = unsafe { core::ptr::read(v) };
                self.insert(k, v);
            }

        }
    }

    
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}