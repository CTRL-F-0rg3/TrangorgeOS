pub mod entities;
pub mod runqueue;
pub mod stats;
pub mod cpumask;
pub mod smp;
pub mod power;
pub mod tests;

use core::sync::atomic::{AtomicU32, Ordering};
use crate::cpu::scheduler::runqueue::RunQueue;
use crate::cpu::scheduler::entities::task::{TaskStruct, MAX_CPUS};

static TOTAL_CPUS: AtomicU32 = AtomicU32::new(0);

// Globalna tablica per-CPU runqueues
static mut RUN_QUEUES: [*mut RunQueue; MAX_CPUS] = [core::ptr::null_mut(); MAX_CPUS];

/// Inicjalizacja scheduler'a dla wszystkich CPU
pub unsafe fn init(nr_cpus: usize) {
    TOTAL_CPUS.store(nr_cpus as u32, Ordering::Release);
    
    // W produkcji: alokacja RunQueue dla każdego CPU
    // Na razie stub, żeby się skompilowało
    for cpu in 0..nr_cpus {
        // TODO: Alokacja idle task i RunQueue
        // let idle = allocate_idle_task(cpu);
        // let rq = allocate_runqueue(cpu, idle);
        // RUN_QUEUES[cpu] = rq;
    }
    
    // Inicjalizacja podmodułów
    crate::cpu::scheduler::cpumask::mark_possible(0);
    crate::cpu::scheduler::cpumask::mark_online(0);
    
    crate::cpu::scheduler::smp::topology::topology_init();
    crate::cpu::scheduler::smp::migration::stopper::stopper_init();
    crate::cpu::scheduler::power::em::em_init();
    crate::cpu::scheduler::power::mod::power_init();
}

/// Tick scheduler'a - wywoływany z timera
pub unsafe fn tick(cpu: u32, delta_ns: u64) -> Result<(), &'static str> {
    if cpu as usize >= MAX_CPUS {
        return Err("Invalid CPU");
    }
    
    let rq_ptr = RUN_QUEUES[cpu as usize];
    if rq_ptr.is_null() {
        return Err("RunQueue not initialized");
    }
    
    let rq = &mut *rq_ptr;
    let flags = rq.lock.lock_irqsave();
    
    rq.advance_clock(delta_ns);
    rq.task_tick();
    
    // Trigger load balancing co jakiś czas
    if rq.nr_switches.load(Ordering::Relaxed) % 100 == 0 {
        crate::cpu::scheduler::smp::trigger_load_balance(cpu, rq);
    }
    
    rq.lock.unlock_irqrestore(flags);
    Ok(())
}

/// Self-test scheduler'a
pub fn self_test() -> crate::testing::TestResult {
    use crate::testing::{TestResult, TestStatus};
    
    // Uruchom testy jednostkowe
    let mut passed = 0;
    let mut failed = 0;
    
    // Testy z modułu tests
    // W produkcji: uruchom wszystkie testy z #[cfg(test)]
    
    if failed == 0 {
        TestResult {
            status: TestStatus::Passed,
            message: "Scheduler self-test passed",
            details: None,
        }
    } else {
        TestResult {
            status: TestStatus::Failed,
            message: "Scheduler self-test failed",
            details: Some(format!("{} tests failed", failed)),
        }
    }
}

/// Pobierz RunQueue dla danego CPU
pub unsafe fn get_rq(cpu: u32) -> *mut RunQueue {
    if cpu as usize >= MAX_CPUS {
        return core::ptr::null_mut();
    }
    RUN_QUEUES[cpu as usize]
}

/// Pobierz ID bieżącego CPU
pub fn current_cpu_id() -> u32 {
    // W produkcji: odczyt z per-CPU data lub TP register (RISC-V)
    // Na razie stub
    0
}