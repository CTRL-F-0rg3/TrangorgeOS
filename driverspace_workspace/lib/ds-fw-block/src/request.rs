//! I/O request structures and states.

use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RequestState {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
}

/// High-level I/O request with metadata.
pub struct IoRequest {
    pub id: u64,
    pub lba: u64,
    pub block_count: u32,
    pub buffer_ptr: *mut u8,
    pub buffer_len: usize,
    pub is_write: bool,
    pub priority: u8,
    state: AtomicU32,
    pub callback: Option<fn(&IoRequest)>,
}

impl IoRequest {
    pub fn new(id: u64, lba: u64, block_count: u32, buffer: *mut u8, len: usize, is_write: bool) -> Self {
        Self {
            id,
            lba,
            block_count,
            buffer_ptr: buffer,
            buffer_len: len,
            is_write,
            priority: 0,
            state: AtomicU32::new(RequestState::Pending as u32),
            callback: None,
        }
    }

    pub fn state(&self) -> RequestState {
        match self.state.load(Ordering::Acquire) {
            0 => RequestState::Pending,
            1 => RequestState::InProgress,
            2 => RequestState::Completed,
            3 => RequestState::Failed,
            _ => RequestState::Failed,
        }
    }

    pub fn mark_in_progress(&self) {
        self.state.store(RequestState::InProgress as u32, Ordering::Release);
    }

    pub fn mark_completed(&self) {
        self.state.store(RequestState::Completed as u32, Ordering::Release);
        if let Some(cb) = self.callback {
            cb(self);
        }
    }

    pub fn mark_failed(&self) {
        self.state.store(RequestState::Failed as u32, Ordering::Release);
    }
}