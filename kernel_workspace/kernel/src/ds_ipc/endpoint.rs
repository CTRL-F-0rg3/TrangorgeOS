use crate::cpu::scheduler::entities::task::Task;
use crate::sync::spinlock::SpinLock;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};

static NEXT_EP_ID: AtomicU64 = AtomicU64::new(1);

pub struct Endpoint {
    pub id: u64,
    pub badge: u64,
    state: SpinLock<EndpointState>,
}

struct EndpointState {
    send_queue: VecDeque<*mut Task>,
    recv_queue: VecDeque<*mut Task>,
}

unsafe impl Send for Endpoint {}
unsafe impl Sync for Endpoint {}

impl Endpoint {
    pub fn new(badge: u64) -> Self {
        Self {
            id: NEXT_EP_ID.fetch_add(1, Ordering::Relaxed),
            badge,
            state: SpinLock::new(EndpointState {
                send_queue: VecDeque::new(),
                recv_queue: VecDeque::new(),
            }),
        }
    }

    pub fn enqueue_sender(&self, task: *mut Task) {
        let mut state = self.state.lock();
        state.send_queue.push_back(task);
    }

    pub fn enqueue_receiver(&self, task: *mut Task) {
        let mut state = self.state.lock();
        state.recv_queue.push_back(task);
    }

    pub fn dequeue_sender(&self) -> Option<*mut Task> {
        let mut state = self.state.lock();
        state.send_queue.pop_front()
    }

    pub fn dequeue_receiver(&self) -> Option<*mut Task> {
        let mut state = self.state.lock();
        state.recv_queue.pop_front()
    }
}