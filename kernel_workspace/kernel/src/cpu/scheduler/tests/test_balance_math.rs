use crate::cpu::scheduler::entities::task::{TaskStruct, SchedPolicy, CpuMask, CPU_NONE, TaskFlags, TaskState};
use crate::cpu::scheduler::runqueue::{RunQueue, EnqueueFlags};
use crate::cpu::scheduler::runqueue::smp;

fn make_idle(pid: u64, cpu: u32) -> TaskStruct {
    let mut t = TaskStruct::blank();
    t.init_test_stub(pid, SchedPolicy::Idle, 0);
    let _ = cpu;
    t
}

fn make_task(pid: u64, policy: SchedPolicy, nice: i8) -> TaskStruct {
    let mut t = TaskStruct::blank();
    t.init_test_stub(pid, policy, nice);
    t
}

fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
    t as *mut TaskStruct
}

#[test]
fn select_task_rq_returns_preferred_cpu_when_idle() {
    let mut idle0 = make_idle(100, 0);
    let mut idle1 = make_idle(101, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t = make_task(1, SchedPolicy::Normal, 0);
    t.se.last_cpu = 1;

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    unsafe {
        let chosen = smp::select_task_rq(ptr_of(&mut t), &registry);
        assert_eq!(chosen, 1);
    }
}

#[test]
fn select_task_rq_falls_back_to_least_loaded() {
    let mut idle0 = make_idle(100, 0);
    let mut idle1 = make_idle(101, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut filler = make_task(2, SchedPolicy::Normal, 0);
    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.enqueue_task(ptr_of(&mut filler), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let mut t = make_task(1, SchedPolicy::Normal, 0);
    t.se.last_cpu = CPU_NONE;

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    unsafe {
        let chosen = smp::select_task_rq(ptr_of(&mut t), &registry);
        assert_eq!(chosen, 1);
    }
}

#[test]
fn select_task_rq_respects_cpus_allowed_mask() {
    let mut idle0 = make_idle(100, 0);
    let mut idle1 = make_idle(101, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t = make_task(1, SchedPolicy::Normal, 0);
    t.se.cpus_allowed = CpuMask::single(0);
    t.se.last_cpu = 1;

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    unsafe {
        let chosen = smp::select_task_rq(ptr_of(&mut t), &registry);
        assert_eq!(chosen, 0);
    }
}

#[test]
fn wake_up_process_enqueues_to_target_cpu() {
    let mut idle0 = make_idle(100, 0);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    rq0.bind_idle_task();

    let mut t = make_task(1, SchedPolicy::Normal, 0);
    t.set_state(TaskState::Uninterruptible).unwrap();
    t.se.last_cpu = CPU_NONE;

    let registry: [*mut RunQueue; 1] = [&mut rq0];
    unsafe {
        smp::wake_up_process(ptr_of(&mut t), &registry).unwrap();
        assert_eq!(t.state(), TaskState::Runnable);
        assert!(t.se.on_rq);
        assert_eq!(rq0.nr_running(), 1);
    }
}

#[test]
fn idle_balance_pulls_task_from_overloaded_cpu() {
    let mut idle0 = make_idle(200, 0);
    let mut idle1 = make_idle(201, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t0 = make_task(1, SchedPolicy::Normal, 0);
    let mut t1 = make_task(2, SchedPolicy::Normal, 0);
    let mut t2 = make_task(3, SchedPolicy::Normal, 0);

    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
        rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
        rq0.activate_task(ptr_of(&mut t2), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let pulled = unsafe { smp::idle_balance(1, &registry) };
    assert!(pulled);
    assert_eq!(rq0.nr_running(), 2);
    assert_eq!(rq1.nr_running(), 1);
}

#[test]
fn load_balance_moves_task_across_uneven_queues() {
    let mut idle0 = make_idle(0, 0);
    let mut idle1 = make_idle(0, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut tasks: [TaskStruct; 4] = core::array::from_fn(|_| TaskStruct::blank());
    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        for (i, t) in tasks.iter_mut().enumerate() {
            t.init_test_stub(i as u64 + 1, SchedPolicy::Normal, 0);
            rq0.activate_task(t as *mut TaskStruct, EnqueueFlags::ENQUEUE_NEW);
        }
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let moved = unsafe { smp::load_balance(&registry) };
    assert_eq!(moved, 1);
    assert_eq!(rq0.nr_running(), 3);
    assert_eq!(rq1.nr_running(), 1);
}

#[test]
fn no_migration_when_cpus_allowed_forbids_it() {
    let mut idle0 = make_idle(200, 0);
    let mut idle1 = make_idle(201, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t0 = make_task(1, SchedPolicy::Normal, 0);
    let mut t1 = make_task(2, SchedPolicy::Normal, 0);
    t0.se.cpus_allowed = CpuMask::single(0);
    t1.se.cpus_allowed = CpuMask::single(0);

    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
        rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let pulled = unsafe { smp::idle_balance(1, &registry) };
    assert!(!pulled);
    assert_eq!(rq0.nr_running(), 2);
    assert_eq!(rq1.nr_running(), 0);
}

#[test]
fn migrated_task_updates_cpu_and_rq_pointer() {
    let mut idle0 = make_idle(200, 0);
    let mut idle1 = make_idle(201, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t0 = make_task(1, SchedPolicy::Normal, 0);
    let mut t1 = make_task(2, SchedPolicy::Normal, 0);

    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
        rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let pulled = unsafe { smp::idle_balance(1, &registry) };
    assert!(pulled);

    unsafe {
        let moved = if (*ptr_of(&mut t0)).cpu.load(core::sync::atomic::Ordering::Relaxed) == 1 {
            ptr_of(&mut t0)
        } else {
            ptr_of(&mut t1)
        };
        assert_eq!((*moved).cpu.load(core::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!((*moved).rq_ptr(), &rq1 as *const RunQueue as *mut core::ffi::c_void);
        assert!((*moved).stats.nr_migrations >= 1);
    }
}

#[test]
fn pf_no_migrate_blocks_migration() {
    let mut idle0 = make_idle(200, 0);
    let mut idle1 = make_idle(201, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t0 = make_task(1, SchedPolicy::Normal, 0);
    let mut t1 = make_task(2, SchedPolicy::Normal, 0);
    t0.flags.fetch_insert(TaskFlags::PF_NO_MIGRATE);
    t1.flags.fetch_insert(TaskFlags::PF_NO_MIGRATE);

    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
        rq0.activate_task(ptr_of(&mut t1), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let pulled = unsafe { smp::idle_balance(1, &registry) };
    assert!(!pulled);
}

#[test]
fn load_balance_is_noop_below_threshold() {
    let mut idle0 = make_idle(0, 0);
    let mut idle1 = make_idle(0, 1);
    let mut rq0 = RunQueue::new(0, ptr_of(&mut idle0));
    let mut rq1 = RunQueue::new(1, ptr_of(&mut idle1));
    rq0.bind_idle_task();
    rq1.bind_idle_task();

    let mut t0 = make_task(1, SchedPolicy::Normal, 0);
    unsafe {
        let f0 = rq0.lock.lock_irqsave();
        rq0.activate_task(ptr_of(&mut t0), EnqueueFlags::ENQUEUE_NEW);
        rq0.lock.unlock_irqrestore(f0);
    }

    let registry: [*mut RunQueue; 2] = [&mut rq0, &mut rq1];
    let moved = unsafe { smp::load_balance(&registry) };
    assert_eq!(moved, 0);
}