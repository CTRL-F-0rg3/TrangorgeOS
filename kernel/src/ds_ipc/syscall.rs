use super::endpoint::Endpoint;
use super::msg::{IpcMessage, MessageInfo, MessageRegisters};
use crate::cpu::scheduler::core::current_task;
use crate::caps::check::verify_ipc_cap;
use crate::types::DsError;

#[repr(u64)]
pub enum IpcSyscall {
    Call = 0,
    Send = 1,
    Recv = 2,
    Reply = 3,
}

pub fn sys_ipc_call(
    ep_cap_idx: u64,
    msg_info: MessageInfo,
    msg_regs: &MessageRegisters,
) -> Result<IpcMessage, DsError> {
    let ep = verify_ipc_cap(current_task(), ep_cap_idx)?;
    
    let mut receiver = ep.dequeue_receiver();
    
    if let Some(recv_task) = receiver {
        transfer_message(recv_task, msg_info, msg_regs);
        wake_task(recv_task);
        
        let reply_info = MessageInfo::new(msg_info.label, 0);
        Ok(IpcMessage {
            info: reply_info,
            regs: MessageRegisters::empty(),
        })
    } else {
        let current = current_task();
        block_current_task();
        ep.enqueue_sender(current);
        switch_context();
        
        unreachable!()
    }
}

pub fn sys_ipc_recv(ep_cap_idx: u64) -> Result<IpcMessage, DsError> {
    let ep = verify_ipc_cap(current_task(), ep_cap_idx)?;
    
    if let Some(sender_task) = ep.dequeue_sender() {
        let msg = extract_message(sender_task);
        wake_task(sender_task);
        Ok(msg)
    } else {
        let current = current_task();
        block_current_task();
        ep.enqueue_receiver(current);
        switch_context();
        
        unreachable!()
    }
}

fn transfer_message(_target: *mut Task, _info: MessageInfo, _regs: &MessageRegisters) {
    // Arch-specific: copy MRs to target thread's saved context (registers)
}

fn extract_message(_sender: *mut Task) -> IpcMessage {
    // Arch-specific: read MRs from sender's saved context
    IpcMessage {
        info: MessageInfo::new(0, 0),
        regs: MessageRegisters::empty(),
    }
}

fn block_current_task() {}
fn wake_task(_task: *mut Task) {}
fn switch_context() {}