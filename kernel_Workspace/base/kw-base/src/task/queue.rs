use kstd_core::sync::SpinLock;
use kstd_data::list::ListHead;
use super::thread::Thread;

pub struct WaitQueue {
    lock: SpinLock<()>,
    head: ListHead,
}

impl WaitQueue {
    pub const fn new() -> Self {
        Self {
            lock: SpinLock::new(()),
            head: ListHead::new(),
        }
    }

    pub fn init(&mut self) {
        self.head.init();
    }

    pub fn sleep(&self, thread: &mut Thread) {
        let _guard = self.lock.lock();
        thread.state = super::thread::ThreadState::Blocked;
        // In a real implementation, we would link thread's list_node into self.head
        // and then call context::switch() to the scheduler.
    }

    pub fn wake_one(&self) {
        let _guard = self.lock.lock();
        if self.head.is_empty() { return; }
        // In a real implementation, we would pop the first thread from self.head,
        // set its state to Ready, and add it to the scheduler runqueue.
    }

    pub fn wake_all(&self) {
        let _guard = self.lock.lock();
        // Wake all threads in the list
    }
}