#![allow(dead_code)]
use core::sync::atomic::{AtomicU32, AtomicBool, AtomicPtr, Ordering};
use core::ptr;
use crate::cpu::scheduler::entities::task::{TaskStruct, TaskState, SpinLock, TaskFlags, MAX_CPUS, CPU_NONE, SchedPolicy};
use crate::cpu::scheduler::runqueue::RunQueue;
use crate::cpu::scheduler::cpumask;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopperState {
    Idle = 0,
    Preparing = 1,
    Running = 2,
    Done = 3,
    Aborted = 4,
}

#[repr(C)]
pub struct StopperWork {
    pub state: AtomicU32,
    pub fn_ptr: AtomicPtr<core::ffi::c_void>, // ZMIENIONE
    pub arg: AtomicPtr<core::ffi::c_void>,    // ZMIENIONE
    pub ret: AtomicU32,
    pub started: AtomicBool,
    pub completed: AtomicBool,
} pub completed: AtomicBool,


impl StopperWork {
    pub const fn empty() -> Self {
        Self {
            state: AtomicU32::new(StopperState::Idle as u32),
            fn_ptr: AtomicPtr::new(ptr::null_mut()),
            arg: AtomicPtr::new(ptr::null_mut()),
            ret: AtomicU32::new(0),
            started: AtomicBool::new(false),
            completed: AtomicBool::new(false),
        }
    }

    pub fn get_state(&self) -> StopperState {
        match self.state.load(Ordering::Acquire) {
            0 => StopperState::Idle,
            1 => StopperState::Preparing,
            2 => StopperState::Running,
            3 => StopperState::Done,
            4 => StopperState::Aborted,
            _ => StopperState::Idle,
        }
    }

    pub fn set_state(&self, new_state: StopperState) {
        self.state.store(new_state as u32, Ordering::Release);
    }
}

pub type StopperFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> u32;

#[repr(C)]
pub struct CpuStopper {
    pub cpu: u32,
    pub lock: SpinLock,
    pub work: StopperWork,
    pub stopper_task: AtomicPtr<TaskStruct>,
    pub enabled: AtomicBool,
    pub park_task: AtomicBool,
    pub nr_stops: AtomicU32,
}

impl CpuStopper {
    pub const fn empty() -> Self {
        Self {
            cpu: CPU_NONE,
            lock: SpinLock::new(),
            work: StopperWork::empty(),
            stopper_task: AtomicPtr::new(ptr::null_mut()),
            enabled: AtomicBool::new(false),
            park_task: AtomicBool::new(false),
            nr_stops: AtomicU32::new(0),
        }
    }

    pub fn init(&mut self, cpu: u32) {
        self.cpu = cpu;
        self.enabled.store(true, Ordering::Release);
    }

    pub fn queue_work(&self, func: StopperFn, arg: *mut core::ffi::c_void) -> bool {
        let _guard = self.lock.lock_irqsave_guard();
        
        if self.work.get_state() != StopperState::Idle {
            return false;
        }

        self.work.fn_ptr.store(func as *mut (), Ordering::Relaxed);
        self.work.arg.store(arg, Ordering::Relaxed);
        self.work.ret.store(0, Ordering::Relaxed);
        self.work.started.store(false, Ordering::Relaxed);
        self.work.completed.store(false, Ordering::Relaxed);
        self.work.set_state(StopperState::Preparing);
        
        self.nr_stops.fetch_add(1, Ordering::Relaxed);
        
        self.wake_stopper_task();
        true
    }

    fn wake_stopper_task(&self) {
        let task_ptr = self.stopper_task.load(Ordering::Acquire);
        if task_ptr.is_null() { return; }
        
        unsafe {
            let task = &*task_ptr;
            if task.state() == TaskState::Interruptible || task.state() == TaskState::Uninterruptible {
                let _ = task.wake_up();
            }
        }
    }

    pub fn execute_work(&self) {
        let _guard = self.lock.lock_irqsave_guard();
        
        if self.work.get_state() != StopperState::Preparing {
            return;
        }

        self.work.set_state(StopperState::Running);
        self.work.started.store(true, Ordering::Release);

        let func_ptr = self.work.fn_ptr.load(Ordering::Relaxed);
        let arg_ptr = self.work.arg.load(Ordering::Relaxed);

        if func_ptr.is_null() {
            self.work.set_state(StopperState::Aborted);
            return;
        }

        let func: StopperFn = unsafe { core::mem::transmute(func_ptr) };
        let ret = unsafe { func(arg_ptr) };

        self.work.ret.store(ret, Ordering::Relaxed);
        self.work.completed.store(true, Ordering::Release);
        self.work.set_state(StopperState::Done);
    }

    pub fn wait_for_completion(&self) -> u32 {
        loop {
            let state = self.work.get_state();
            if state == StopperState::Done || state == StopperState::Aborted {
                break;
            }
            core::hint::spin_loop();
        }
        
        let ret = self.work.ret.load(Ordering::Acquire);
        
        let _guard = self.lock.lock_irqsave_guard();
        self.work.set_state(StopperState::Idle);
        self.work.fn_ptr.store(ptr::null_mut(), Ordering::Relaxed);
        self.work.arg.store(ptr::null_mut(), Ordering::Relaxed);
        
        ret
    }
}

pub struct StopperRegistry {
    pub stoppers: [CpuStopper; MAX_CPUS],
}

impl StopperRegistry {
    pub const fn empty() -> Self {
        const EMPTY_STOPPER: CpuStopper = CpuStopper::empty();
        Self {
            stoppers: [EMPTY_STOPPER; MAX_CPUS],
        }
    }

    pub fn init_cpu(&mut self, cpu: u32) {
        if (cpu as usize) < MAX_CPUS {
            self.stoppers[cpu as usize].init(cpu);
        }
    }

    pub fn get(&self, cpu: u32) -> Option<&CpuStopper> {
        if (cpu as usize) < MAX_CPUS {
            Some(&self.stoppers[cpu as usize])
        } else {
            None
        }
    }
}

pub static mut GLOBAL_STOPPERS: StopperRegistry = StopperRegistry::empty();

pub unsafe fn stopper_init() {
    for cpu in cpumask::online_mask().iter() {
        GLOBAL_STOPPERS.init_cpu(cpu);
    }
}

pub unsafe fn stop_one_cpu(cpu: u32, func: StopperFn, arg: *mut core::ffi::c_void) -> u32 {
    let stopper = match GLOBAL_STOPPERS.get(cpu) {
        Some(s) => s,
        None => return 1,
    };

    if !stopper.queue_work(func, arg) {
        return 2;
    }

    crate::cpu::scheduler::smp::ipi::send_resched_ipi(cpu);

    stopper.wait_for_completion()
}

pub unsafe fn stop_two_cpus(cpu1: u32, cpu2: u32, func: StopperFn, arg: *mut core::ffi::c_void) -> u32 {
    if cpu1 == cpu2 {
        return stop_one_cpu(cpu1, func, arg);
    }

    let target_cpu = if cpu1 < cpu2 { cpu1 } else { cpu2 };
    let other_cpu = if cpu1 < cpu2 { cpu2 } else { cpu1 };

    let stopper1 = match GLOBAL_STOPPERS.get(target_cpu) {
        Some(s) => s,
        None => return 1,
    };
    let stopper2 = match GLOBAL_STOPPERS.get(other_cpu) {
        Some(s) => s,
        None => return 1,
    };

    let _g1 = stopper1.lock.lock_irqsave_guard();
    let _g2 = stopper2.lock.lock_irqsave_guard();

    if stopper1.work.get_state() != StopperState::Idle || stopper2.work.get_state() != StopperState::Idle {
        return 2;
    }

    stopper1.work.fn_ptr.store(func as *mut (), Ordering::Relaxed);
    stopper1.work.arg.store(arg, Ordering::Relaxed);
    stopper1.work.set_state(StopperState::Preparing);
    
    stopper2.work.fn_ptr.store(func as *mut (), Ordering::Relaxed);
    stopper2.work.arg.store(arg, Ordering::Relaxed);
    stopper2.work.set_state(StopperState::Preparing);

    drop(_g1);
    drop(_g2);

    crate::cpu::scheduler::smp::ipi::send_resched_ipi(target_cpu);
    crate::cpu::scheduler::smp::ipi::send_resched_ipi(other_cpu);

    let ret1 = stopper1.wait_for_completion();
    let ret2 = stopper2.wait_for_completion();

    if ret1 != 0 { ret1 } else { ret2 }
}

pub unsafe fn cpu_stopper_irq_handler(cpu: u32) {
    let stopper = match GLOBAL_STOPPERS.get(cpu) {
        Some(s) => s,
        None => return,
    };

    if stopper.work.get_state() == StopperState::Preparing {
        stopper.execute_work();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn dummy_stopper_fn(_arg: *mut core::ffi::c_void) -> u32 {
        0
    }

    #[test]
    fn stopper_state_transitions() {
        let work = StopperWork::empty();
        assert_eq!(work.get_state(), StopperState::Idle);
        
        work.set_state(StopperState::Preparing);
        assert_eq!(work.get_state(), StopperState::Preparing);
        
        work.set_state(StopperState::Running);
        assert_eq!(work.get_state(), StopperState::Running);
        
        work.set_state(StopperState::Done);
        assert_eq!(work.get_state(), StopperState::Done);
    }

    #[test]
    fn stopper_registry_bounds_check() {
        let reg = StopperRegistry::empty();
        assert!(reg.get(0).is_some());
        assert!(reg.get(MAX_CPUS as u32).is_none());
    }

    #[test]
    fn queue_execute_and_wait_roundtrip_returns_value_from_function() {
        let mut stopper = CpuStopper::empty();
        stopper.init(7);
        assert!(stopper.queue_work(dummy_stopper_fn, ptr::null_mut()));
        assert_eq!(stopper.work.get_state(), StopperState::Preparing);

        stopper.execute_work();
        assert_eq!(stopper.work.get_state(), StopperState::Done);

        let ret = stopper.wait_for_completion();
        assert_eq!(ret, 0);
        assert_eq!(stopper.work.get_state(), StopperState::Idle);
    }

    #[test]
    fn queue_work_rejects_a_second_job_while_busy() {
        let mut stopper = CpuStopper::empty();
        stopper.init(7);
        assert!(stopper.queue_work(dummy_stopper_fn, ptr::null_mut()));
        assert!(!stopper.queue_work(dummy_stopper_fn, ptr::null_mut()));
    }

    #[test]
    fn execute_work_without_queued_job_is_a_noop() {
        let mut stopper = CpuStopper::empty();
        stopper.init(7);
        stopper.execute_work();
        assert_eq!(stopper.work.get_state(), StopperState::Idle);
    }
}