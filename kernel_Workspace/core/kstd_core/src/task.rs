use kstd_base::{TaskId, WorldId, Status};

extern "C" {
    fn sched_create_task(world_id: u32, flags: u32, out_task_id: *mut u64) -> bool;
    fn sched_destroy_task(task_id: u64) -> bool;
    fn sched_yield() -> bool;
    fn sched_get_current_task() -> u64;
}

pub struct Task {
    pub id: TaskId,
    pub world: WorldId,
}

impl Task {
    pub fn create(world: WorldId, flags: u32) -> Result<Self, Status> {
        let mut id: u64 = 0;
        if unsafe { sched_create_task(world.0, flags, &mut id) } {
            Ok(Self { id: TaskId(id), world })
        } else {
            Err(Status::OutOfMemory)
        }
    }

    pub fn destroy(&self) -> Result<(), Status> {
        if unsafe { sched_destroy_task(self.id.0) } {
            Ok(())
        } else {
            Err(Status::InvalidArgument)
        }
    }

    pub fn current() -> TaskId {
        TaskId(unsafe { sched_get_current_task() })
    }

    pub fn yield_now() {
        unsafe { sched_yield(); }
    }
}