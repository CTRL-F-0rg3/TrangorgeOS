// driverspace_workspace/src/memory.rs

use kstd_base::Status;

pub fn alloc(size: usize) -> Result<*mut u8, Status> {
    let msg = crate::ipc::Message {
        cmd: 0x10,
        arg0: size as u64,
        arg1: 0,
    };
    
    crate::ipc::send(msg)?;
    
    match crate::ipc::receive() {
        Some(reply) if reply.cmd == 0 => {
            Ok(reply.arg0 as *mut u8)
        }
        Some(reply) => Err(kstd_base::Status::from(reply.cmd)),
        None => Err(Status::IoError),
    }
}

pub fn free(ptr: *mut u8, size: usize) {
    let msg = crate::ipc::Message {
        cmd: 0x11,
        arg0: ptr as u64,
        arg1: size as u64,
    };
    
    crate::ipc::send(msg).ok();
}