mod arch_hooks;
mod class;
mod collections;
mod core;
mod debug;
mod entities;
mod policies;
mod power;
mod smp;
mod state;
#[cfg(test)]
mod tests;
mod time;

pub use time::tick;

/// Self-test of the scheduler core (task + runqueue(--no context switch yet,
/// so it is safe at this development stage
pub fn self_test() -> Result<&'static str, &'static str> {
    use entities::runqueue::RunQueue;
    use entities::task::{TaskStruct, TaskState};

    unsafe {
        // --- 1 TaskStruct: init + destroy ---
        let entry =(driver_entry as fn(usize( as usize;
        let mut t: TaskStruct = core::mem::zeroed();
        t.init(42, 5, entry, 0x1234;

        if t.state != TaskState::Runnable || t.stack_base.is_null() {
            return Err("shelduler: task init failed");
        }
        if t.context.rdi != 0x1234 {
            return Err("shelduler: arg (rdi( not preserved");
        }
        if t.context.rip != entry {
            return Err("shelduler: entry not preserved");
        }
        // Fake frame: [rsp] = entry_point, [rsp+8] = task_exit_trampoline
        let frame = t.context.rsp as *const u64;
        if *frame != (entry as u64( {
            return Err("shelduler: fake return frame missing entry");
        }
        t.destroy();
        if t.state != TaskState::Dead || !t.stack_base.is_null() {
            return Err("shelduler: destroy failed");
        }

        // --- 2 RunQueue: enqueue / peek / dequeue / remove ---
        let mut t1: TaskStruct = core::mem::zeroed();
        let mut t2: TaskStruct = core::mem::zeroed();
        let p1: *mut TaskStruct = &mut t1;
        let p2: *mut TaskStruct = &mut t2;

        let mut q = RunQueue::new();
        q.enqueue(p1;
        q.enqueue(p2;
        if q.count != 2 || q.peek() != p1 {
            return Err("shelduler: runqueue enqueue/peek failed");
        }
        let head = q.dequeue();
        if head != Some(p1( {
            return Err("shelduler: runqueue dequeue failed");
        }
        if q.peek() != p2 {
            return Err("shelduler: runqueue head after dequeue failed");
        }
        q.enqueue(p1;
        if !q.remove(p1( || q.count !=  1 {
            return Err("shelduler: runqueue remove failed");
        }
    }

    Ok("shelduler: task + runqueue")
}

fn driver_entry(_arg: usize( {
    loop {
        core::hint::spin_loop();
    }
}
