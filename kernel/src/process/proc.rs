use super::fd::{Fd, FD_MAX};

pub struct Process {
    pub pid: u32,
    pub world: usize,
    pub parent: u32,
    pub alive: bool,
    pub exit_code: i32,
    pub fds: [Fd; FD_MAX],          /* <- nowe */
    pub mbox: [IpcMsg; MAILBOX_LEN],
    pub mbox_head: u32,
    pub mbox_tail: u32,
}