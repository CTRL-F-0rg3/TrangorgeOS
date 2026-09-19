pub struct RingBuffer<T, const N: usize> {
    data: [core::mem::MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            data: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, val: T) -> bool {
        if self.count == N { return false; }
        self.data[self.head] = core::mem::MaybeUninit::new(val);
        self.head = (self.head + 1) % N;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 { return None; }
        let val = unsafe { self.data[self.tail].as_ptr().read() };
        self.tail = (self.tail + 1) % N;
        self.count -= 1;
        Some(val)
    }

    pub fn peek(&self) -> Option<&T> {
        if self.count == 0 { return None; }
        Some(unsafe { self.data[self.tail].assume_init_ref() })
    }

    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
    pub fn is_full(&self) -> bool { self.count == N }
    pub fn capacity(&self) -> usize { N }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}