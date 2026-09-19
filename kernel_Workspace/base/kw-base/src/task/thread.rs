use kstd_base::Tid;
use super::context::TaskContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Thread {
    pub tid: Tid,
    pub state: ThreadState,
    pub context: TaskContext,
    pub kernel_stack_top: u64,
}

impl Thread {
    pub fn new(tid: Tid, entry_point: u64, stack_top: u64) -> Self {
        let mut ctx = TaskContext::default();
        ctx.rip = entry_point;
        ctx.rsp = stack_top;
        ctx.rflags = 0x202; // Interrupts enabled

        Self {
            tid,
            state: ThreadState::Ready,
            context: ctx,
            kernel_stack_top: stack_top,
        }
    }
}