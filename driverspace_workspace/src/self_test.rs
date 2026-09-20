// driverspace_workspace/src/self_test.rs

use crate::{ipc, memory, drivers};
use kstd_base::Status;

pub fn run_all() -> Result<usize, Status> {
    let mut passed = 0;
    
    test_ipc_ring()?;
    passed += 1;
    
    test_memory_alloc()?;
    passed += 1;
    
    test_capability_check()?;
    passed += 1;
    
    Ok(passed)
}

fn test_ipc_ring() -> Result<(), Status> {
    let msg = ipc::Message {
        cmd: 0xFF,
        arg0: 0x12345678,
        arg1: 0xABCDEF00,
    };
    
    ipc::send(msg)?;
    
    let received = ipc::receive().ok_or(Status::IoError)?;
    
    if received.cmd != msg.cmd || received.arg0 != msg.arg0 {
        return Err(Status::InvalidArgument);
    }
    
    Ok(())
}

fn test_memory_alloc() -> Result<(), Status> {
    let ptr = memory::alloc(4096)?;
    
    if ptr.is_null() {
        return Err(Status::OutOfMemory);
    }
    
    unsafe {
        core::ptr::write_bytes(ptr, 0xAA, 4096);
        
        for i in 0..4096 {
            if core::ptr::read(ptr.add(i)) != 0xAA {
                memory::free(ptr, 4096);
                return Err(Status::IoError);
            }
        }
    }
    
    memory::free(ptr, 4096);
    
    Ok(())
}

fn test_capability_check() -> Result<(), Status> {
    let caps = crate::capabilities::current();
    
    if !caps.has(crate::capabilities::Capability::MMIO) {
        return Err(Status::PermissionDenied);
    }
    
    Ok(())
}