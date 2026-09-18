pub struct RingBuffer<const N: usize> {
    data: [u8; N],
    head: usize,
    tail: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self { data: [0; N], head: 0, tail: 0 }
    }

    pub fn push(&mut self, b: u8) -> bool {
        let next_head = (self.head + 1) % N;
        if next_head == self.tail { return false; }
        self.data[self.head] = b;
        self.head = next_head;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail { return None; }
        let b = self.data[self.tail];
        self.tail = (self.tail + 1) % N;
        Some(b)
    }

    pub fn is_empty(&self) -> bool { self.head == self.tail }
    pub fn is_full(&self) -> bool { (self.head + 1) % N == self.tail }
    pub fn clear(&mut self) { self.head = 0; self.tail = 0; }
}

impl<const N: usize> crate::traits::Write for RingBuffer<N> {
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        for b in s.as_bytes() {
            if !self.push(*b) { return Err(()); }
        }
        Ok(())
    }
}