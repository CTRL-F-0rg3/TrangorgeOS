use super::context::Context;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

const STACK_SIZE: usize = 64 * 1024;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,

    Finished,
}

pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub ctx: Context,
    pub state: TaskState,
    pub is_idle: bool,

    _stack: Vec<u8>,
}

#[unsafe(naked)]
unsafe extern "C" fn task_entry_trampoline() -> ! {
    core::arch::naked_asm!(
        "pop rdi",
        "call {run}",
        "ud2",
        run = sym run_boxed_closure,
    );
}

extern "C" fn run_boxed_closure(closure_ptr: u64) -> ! {
    let boxed: Box<Box<dyn FnOnce() + Send + 'static>> =
        unsafe { Box::from_raw(closure_ptr as *mut Box<dyn FnOnce() + Send + 'static>) };
    (*boxed)();
    super::exit_current_task();
}

impl Task {

    pub fn new<F>(name: &'static str, f: F) -> Box<Task>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut stack = Vec::<u8>::with_capacity(STACK_SIZE);

        unsafe { stack.set_len(STACK_SIZE) };

        let stack_base = stack.as_ptr() as u64;

        let stack_top = stack_base + STACK_SIZE as u64 - 8;

        let boxed_fn: Box<dyn FnOnce() + Send + 'static> = Box::new(f);
        let double_boxed: Box<Box<dyn FnOnce() + Send + 'static>> = Box::new(boxed_fn);
        let closure_ptr = Box::into_raw(double_boxed) as u64;

        unsafe {
            let write_u64 = |offset_from_top: u64, value: u64| {
                let addr = (stack_top - offset_from_top) as *mut u64;
                addr.write(value);
            };
            write_u64(8, task_entry_trampoline as usize as u64);
            write_u64(16, 0);
            write_u64(24, 0);
            write_u64(32, 0);
            write_u64(40, 0);
            write_u64(48, 0);
            write_u64(56, 0);
            write_u64(64, closure_ptr);
        }

        Box::new(Task {
            id: TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed)),
            name,
            ctx: Context { rsp: stack_top - 56 },
            state: TaskState::Ready,
            is_idle: false,
            _stack: stack,
        })
    }

    pub fn new_idle(name: &'static str) -> Box<Task> {
        let mut t = Task::new(name, || loop {
            x86_64::instructions::hlt();
            super::yield_now();
        });
        t.is_idle = true;
        t
    }
}
