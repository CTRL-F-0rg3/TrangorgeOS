#![allow(dead_code)]
use core::sync::atomic::{AtomicU64, Ordering};
use crate::cpu::scheduler::entities::task::{MAX_CPUS, CPU_NONE};
use crate::cpu::scheduler::smp::migration::stopper;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiType {
    Resched = 0,
    CallFunction = 1,
    MigrationStop = 2,
    TlbShootdown = 3,
    NmiBacktrace = 4,
}

pub static IPI_RESCHED_COUNT: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];
pub static IPI_CALL_COUNT: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];

pub unsafe fn send_resched_ipi(cpu: u32) {
    if cpu as usize >= MAX_CPUS { return; }
    
    if cpu == crate::cpu::scheduler::current_cpu_id() {
        return;
    }
    
    IPI_RESCHED_COUNT[cpu as usize].fetch_add(1, Ordering::Relaxed);
    
    arch_send_ipi(cpu, IpiType::Resched);
}

pub unsafe fn send_migration_stop_ipi(cpu: u32) {
    if cpu as usize >= MAX_CPUS { return; }
    IPI_CALL_COUNT[cpu as usize].fetch_add(1, Ordering::Relaxed);
    arch_send_ipi(cpu, IpiType::MigrationStop);
}

unsafe fn arch_send_ipi(cpu: u32, ipi_type: IpiType) {
    // W środowisku produkcyjnym:
    // x86_64: apic::send_ipi(cpu, vector)
    // RISC-V: sbi::send_ipi(cpu_mask)
    // Na razie stub, żeby linker nie krzyczał.
    let _ = (cpu, ipi_type);
}

pub unsafe fn handle_ipi_entry(cpu: u32, ipi_type: IpiType) {
    match ipi_type {
        IpiType::Resched => {
            let rq_ptr = crate::cpu::scheduler::runqueue::get_rq(cpu);
            if !rq_ptr.is_null() {
                let rq = &mut *rq_ptr;
                rq.check_preempt_curr(rq.current());
            }
        }
        IpiType::MigrationStop => {
            stopper::cpu_stopper_irq_handler(cpu);
        }
        IpiType::CallFunction => {
            // call_function_interrupt()
        }
        IpiType::TlbShootdown => {
            // flush_tlb_others()
        }
        IpiType::NmiBacktrace => {
            // nmi_cpu_backtrace()
        }
    }
}