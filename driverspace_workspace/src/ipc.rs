// driverspace_workspace/src/ipc.rs

use kstd_base::Status;
use core::sync::atomic::{AtomicU64, Ordering};

#[repr(C)]
pub struct Message {
    pub cmd: u32,
    pub arg0: u64,
    pub arg1: u64,
}

static RING_BUFFER: [AtomicU64; 1024] = {
    const INIT: AtomicU64 = AtomicU64::new(0);
    [INIT; 1024]
};

static HEAD: AtomicU64 = AtomicU64::new(0);
static TAIL: AtomicU64 = AtomicU64::new(0);

pub fn send(msg: Message) -> Result<(), Status> {
    let head = HEAD.load(Ordering::Acquire);
    let tail = TAIL.load(Ordering::Acquire);
    
    if (head + 1) % 1024 == tail {
        return Err(Status::Busy);
    }
    
    let idx = (head * 2) as usize;
    RING_BUFFER[idx].store(msg.cmd as u64, Ordering::Release);
    RING_BUFFER[idx + 1].store(msg.arg0, Ordering::Release);
    
    HEAD.store((head + 1) % 1024, Ordering::Release);
    
    Ok(())
}

pub fn receive() -> Option<Message> {
    let head = HEAD.load(Ordering::Acquire);
    let tail = TAIL.load(Ordering::Acquire);
    
    if head == tail {
        return None;
    }
    
    let idx = (tail * 2) as usize;
    let cmd = RING_BUFFER[idx].load(Ordering::Acquire) as u32;
    let arg0 = RING_BUFFER[idx + 1].load(Ordering::Acquire);
    
    TAIL.store((tail + 1) % 1024, Ordering::Release);
    
    Some(Message { cmd, arg0, arg1: 0 })
}

pub fn reply(status: Status, data: u64) {
    let msg = Message {
        cmd: status as u32,
        arg0: data,
        arg1: 0,
    };
    send(msg).ok();
}