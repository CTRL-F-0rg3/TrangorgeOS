use core::ptr;

pub type TaskId = u64;
pub const KERNEL_STACK_SIZE: usize = 16 * 1024;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Runnable = 0,
    Blocked = 1,
    Stopped = 2,
    Zombie = 3,
    Dead = 4,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CpuContext {
    pub rsp: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
}

#[repr(C)]
pub struct SchedEntity {
    pub rq_next: *mut TaskStruct,
    pub rq_prev: *mut TaskStruct,
    pub vruntime: u64,
    pub weight: u64,
}

#[repr(C)]
pub struct TaskStruct {
    pub id: TaskId,
    pub state: TaskState,
    pub priority: u8,
    pub stack_base: *mut u8,
    pub stack_size: usize,
    pub context: CpuContext,
    pub sched: SchedEntity,
}

extern "C" {
    fn alloc_stack(size: usize) -> *mut u8;
    fn free_stack(ptr: *mut u8, size: usize);
}

impl TaskStruct {
    pub unsafe fn init(&mut self, id: TaskId, priority: u8, entry_point: usize, arg: usize) {
        self.id = id;
        self.priority = priority;
        self.state = TaskState::Runnable;
        
        self.stack_size = KERNEL_STACK_SIZE;
        self.stack_base = alloc_stack(self.stack_size);
        if self.stack_base.is_null() {
            self.state = TaskState::Dead;
            return;
        }
        
        let mut stack_top = self.stack_base.add(self.stack_size) as usize;
        stack_top &= !0xF; // 16-byte alignment (x86_64 ABI)
        
        // Push the argument onto the stack
        // Decrement stack pointer to make space for the argument
        // Store the argument at the new stack pointer location
        // This is necessary because when the task starts executing, it will expect its argument to be on the stack
        // Decrement stack pointer to make space for the argument
        // Store the argument at the new stack pointer location
        stack_top -= 8;
        *(stack_top as *mut usize) = arg;

        self.context = CpuContext {
            rsp: stack_top as u64,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: entry_point as u64,
        };
        
        self.sched = SchedEntity {
            rq_next: ptr::null_mut(),
            rq_prev: ptr::null_mut(),
            vruntime: 0,
            weight: 1024,
        };
    }
    
    pub unsafe fn destroy(&mut self) {
        if !self.stack_base.is_null() {
            free_stack(self.stack_base, self.stack_size);
            self.stack_base = ptr::null_mut();
        }
        self.state = TaskState::Dead;
    }
}