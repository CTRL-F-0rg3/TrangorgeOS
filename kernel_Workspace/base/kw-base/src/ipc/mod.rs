pub mod message_queue;
pub mod pipe;
pub mod shared_mem;
pub mod signal;
pub mod socket;

pub use message_queue::{IpcMessage, send, recv};
pub use shared_mem::SharedMemory;