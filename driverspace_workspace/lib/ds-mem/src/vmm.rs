use kapi_abi::primitives::Handle;
use kapi_abi::errors::DsError;
use kapi_abi::opcodes::Opcode;
use kapi_syscall::sys_ipc_call;

pub fn map_memory(
    manager_ep: Handle,
    phys_addr: u64,
    virt_addr: u64,
    size: u64,
    flags: u64,
) -> Result<(), DsError> {
    let mut payload = [phys_addr, virt_addr, size, flags];
    let mut reply_buf = [0u64; 1];

    let res = unsafe {
        sys_ipc_call(
            manager_ep.0,
            Opcode::MemMapMmio as u32, 
            payload.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&payload) as u32,
            reply_buf.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&reply_buf) as u32,
        )
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn unmap_memory(manager_ep: Handle, virt_addr: u64, size: u64) -> Result<(), DsError> {
    let mut payload = [virt_addr, size];
    let mut reply_buf = [0u64; 1];

    let res = unsafe {
        sys_ipc_call(
            manager_ep.0,
            Opcode::MemMapMmio as u32, 
            payload.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&payload) as u32,
            reply_buf.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&reply_buf) as u32,
        )
    };

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}