use kstd_base::{TaskId, Status};

extern "C" {
    fn thread_create(task_id: u64, entry: u64, stack_ptr: u64, arg: u64, out_tid: *mut u64) -> bool;
    fn thread_exit() -> !;
    fn thread_get_current_id() -> u64;
}

pub struct Thread {
    pub id: u64,
    pub task: TaskId,
}

impl Thread {
    pub fn spawn(task: TaskId, entry: u64, stack_ptr: u64, arg: u64) -> Result<Self, Status> {
        let mut tid: u64 = 0;
        if unsafe { thread_create(task.0, entry, stack_ptr, arg, &mut tid) } {
            Ok(Self { id: tid, task })
        } else {
            Err(Status::OutOfMemory)
        }
    }

    pub fn exit() -> ! {
        unsafe { thread_exit(); }
    }

    pub fn current_id() -> u64 {
        unsafe { thread_get_current_id() }
    }
}