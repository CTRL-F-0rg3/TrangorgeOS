use crate::vec::Vec;

struct Node<K, V> {
    key: K, val: V,
    left: usize, right: usize,
    height: i32,
}

pub struct TreeMap<K, V> {
    nodes: Vec<Node<K, V>>,
    free_list: Vec<usize>,
    root: usize,
}

impl<K: Ord, V> TreeMap<K, V> {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), free_list: Vec::new(), root: 0 }
    }

    fn alloc_node(&mut self, key: K, val: V) -> usize {
        if let Some(idx) = self.free_list.pop() {
            self.nodes[idx - 1] = Node { key, val, left: 0, right: 0, height: 1 };
            idx
        } else {
            self.nodes.push(Node { key, val, left: 0, right: 0, height: 1 });
            self.nodes.len()
        }
    }

    fn free_node(&mut self, idx: usize) {
        self.free_list.push(idx);
    }

    fn height(&self, idx: usize) -> i32 {
        if idx == 0 { 0 } else { self.nodes[idx - 1].height }
    }

    fn update_height(&mut self, idx: usize) {
        let l = self.height(self.nodes[idx - 1].left);
        let r = self.height(self.nodes[idx - 1].right);
        self.nodes[idx - 1].height = 1 + if l > r { l } else { r };
    }

    fn balance_factor(&self, idx: usize) -> i32 {
        if idx == 0 { 0 } 
        else { self.height(self.nodes[idx - 1].left) - self.height(self.nodes[idx - 1].right) }
    }

    fn rotate_right(&mut self, y: usize) -> usize {
        let x = self.nodes[y - 1].left;
        let t2 = self.nodes[x - 1].right;
        self.nodes[x - 1].right = y;
        self.nodes[y - 1].left = t2;
        self.update_height(y);
        self.update_height(x);
        x
    }

    fn rotate_left(&mut self, x: usize) -> usize {
        let y = self.nodes[x - 1].right;
        let t2 = self.nodes[y - 1].left;
        self.nodes[y - 1].left = x;
        self.nodes[x - 1].right = t2;
        self.update_height(x);
        self.update_height(y);
        y
    }

    fn balance(&mut self, idx: usize) -> usize {
        self.update_height(idx);
        let bf = self.balance_factor(idx);
        if bf > 1 {
            if self.balance_factor(self.nodes[idx - 1].left) < 0 {
                let l = self.nodes[idx - 1].left;
                self.nodes[idx - 1].left = self.rotate_left(l);
            }
            return self.rotate_right(idx);
        }
        if bf < -1 {
            if self.balance_factor(self.nodes[idx - 1].right) > 0 {
                let r = self.nodes[idx - 1].right;
                self.nodes[idx - 1].right = self.rotate_right(r);
            }
            return self.rotate_left(idx);
        }
        idx
    }

    fn insert_rec(&mut self, idx: usize, key: K, val: V) -> usize {
        if idx == 0 { return self.alloc_node(key, val); }
        
        let i = idx - 1;
        if key < self.nodes[i].key {
            self.nodes[i].left = self.insert_rec(self.nodes[i].left, key, val);
        } else if key > self.nodes[i].key {
            self.nodes[i].right = self.insert_rec(self.nodes[i].right, key, val);
        } else {
            self.nodes[i].val = val;
            return idx;
        }
        self.balance(idx)
    }

    pub fn insert(&mut self, key: K, val: V) {
        self.root = self.insert_rec(self.root, key, val);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut idx = self.root;
        while idx != 0 {
            let i = idx - 1;
            if *key < self.nodes[i].key { idx = self.nodes[i].left; }
            else if *key > self.nodes[i].key { idx = self.nodes[i].right; }
            else { return Some(&self.nodes[i].val); }
        }
        None
    }

    pub fn len(&self) -> usize { self.nodes.len() - self.free_list.len() }
    pub fn is_empty(&self) -> bool { self.root == 0 }
}