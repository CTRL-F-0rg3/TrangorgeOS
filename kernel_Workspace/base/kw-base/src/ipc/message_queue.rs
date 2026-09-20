use crate::core::{KResult, KernelError};

extern "C" {
    fn k_ipc_send(dst: u32, a0: u64, a1: u64) -> i32;
    fn k_ipc_recv(a0: *mut u64, a1: *mut u64) -> i32;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IpcMessage {
    pub from: u32,
    pub arg0: u64,
    pub arg1: u64,
}

pub fn send(target_pid: u32, arg0: u64, arg1: u64) -> KResult<()> {
    let rc = unsafe { k_ipc_send(target_pid, arg0, arg1) };
    if rc == 0 { Ok(()) } else { Err(KernelError::from_code(rc)) }
}

pub fn recv() -> KResult<IpcMessage> {
    let mut a0: u64 = 0;
    let mut a1: u64 = 0;
    let from = unsafe { k_ipc_recv(&mut a0, &mut a1) };
    if from < 0 {
        Err(KernelError::from_code(from))
    } else {
        Ok(IpcMessage { from: from as u32, arg0: a0, arg1: a1 })
    }
}