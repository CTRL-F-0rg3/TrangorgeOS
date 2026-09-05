#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::cpu::scheduler::entities::task::{CpuContext, TaskStruct, CPU_NONE, MAX_CPUS};
use crate::cpu::scheduler::runqueue::{smp, RunQueue, BALANCE_INTERVAL_NS};

pub const LOCAL_APIC_BASE: usize = 0xFEE0_0000;
pub const APIC_REG_ID: usize = 0x020;
pub const APIC_REG_EOI: usize = 0x0B0;
pub const APIC_REG_SVR: usize = 0x0F0;
pub const APIC_REG_ICR_LOW: usize = 0x300;
pub const APIC_REG_ICR_HIGH: usize = 0x310;
pub const APIC_REG_LVT_TIMER: usize = 0x320;
pub const APIC_REG_TIMER_ICR: usize = 0x380;
pub const APIC_REG_TIMER_DIV: usize = 0x3E0;

pub const RESCHED_VECTOR: u8 = 0xFD;
pub const TIMER_VECTOR: u8 = 0xFC;

pub const TICK_HZ: u64 = 1000;
pub const TICK_NS: u64 = 1_000_000_000 / TICK_HZ;

const NULL_RQ: AtomicPtr<RunQueue> = AtomicPtr::new(ptr::null_mut());
static RQ_REGISTRY: [AtomicPtr<RunQueue>; MAX_CPUS] = [NULL_RQ; MAX_CPUS];

const NO_APIC_ID: AtomicU32 = AtomicU32::new(u32::MAX);
static APIC_ID_TABLE: [AtomicU32; MAX_CPUS] = [NO_APIC_ID; MAX_CPUS];

pub fn register_runqueue(cpu: u32, rq: *mut RunQueue) {
    if (cpu as usize) < MAX_CPUS {
        RQ_REGISTRY[cpu as usize].store(rq, Ordering::Release);
    }
}

pub fn register_apic_id(cpu: u32, apic_id: u32) {
    if (cpu as usize) < MAX_CPUS {
        APIC_ID_TABLE[cpu as usize].store(apic_id, Ordering::Release);
    }
}

pub fn snapshot_registry() -> [*mut RunQueue; MAX_CPUS] {
    let mut out = [ptr::null_mut(); MAX_CPUS];
    for i in 0..MAX_CPUS {
        out[i] = RQ_REGISTRY[i].load(Ordering::Acquire);
    }
    out
}

fn cpu_to_apic_id(cpu: u32) -> Option<u32> {
    if (cpu as usize) >= MAX_CPUS {
        return None;
    }
    let id = APIC_ID_TABLE[cpu as usize].load(Ordering::Acquire);
    if id == u32::MAX {
        None
    } else {
        Some(id)
    }
}

unsafe fn apic_write(reg: usize, value: u32) {
    ptr::write_volatile((LOCAL_APIC_BASE + reg) as *mut u32, value);
}

unsafe fn apic_read(reg: usize) -> u32 {
    ptr::read_volatile((LOCAL_APIC_BASE + reg) as *const u32)
}

unsafe fn apic_eoi() {
    apic_write(APIC_REG_EOI, 0);
}

pub fn current_cpu_id() -> u32 {
    unsafe {
        let apic_id = apic_read(APIC_REG_ID) >> 24;
        for cpu in 0..MAX_CPUS as u32 {
            if cpu_to_apic_id(cpu) == Some(apic_id) {
                return cpu;
            }
        }
    }
    CPU_NONE
}

pub unsafe fn send_ipi(target_apic_id: u32, vector: u8) {
    apic_write(APIC_REG_ICR_HIGH, target_apic_id << 24);
    apic_write(APIC_REG_ICR_LOW, vector as u32);
    while apic_read(APIC_REG_ICR_LOW) & (1 << 12) != 0 {
        core::hint::spin_loop();
    }
}

pub unsafe fn send_resched_ipi(cpu: u32) {
    if let Some(apic_id) = cpu_to_apic_id(cpu) {
        send_ipi(apic_id, RESCHED_VECTOR);
    }
}

pub unsafe fn init_local_apic(spurious_vector: u8) {
    apic_write(APIC_REG_SVR, spurious_vector as u32 | (1 << 8));
}

pub unsafe fn init_local_timer(initial_count: u32, divide: u32) {
    apic_write(APIC_REG_TIMER_DIV, divide);
    apic_write(APIC_REG_LVT_TIMER, TIMER_VECTOR as u32 | (1 << 17));
    apic_write(APIC_REG_TIMER_ICR, initial_count);
}

extern "C" {
    fn arch_context_switch(prev_ctx: *mut CpuContext, next_ctx: *const CpuContext);
}

pub unsafe fn context_switch(rq: &mut RunQueue) {
    let flags = rq.lock.lock_irqsave();
    let (prev, next) = rq.schedule();
    rq.lock.unlock_irqrestore(flags);

    if core::ptr::eq(prev, next) {
        return;
    }
    (*prev).context.save_fpu();
    (*next).context.restore_fpu();
    arch_context_switch(
        &mut (*prev).context as *mut CpuContext,
        &(*next).context as *const CpuContext,
    );
    rq.schedule_tail(prev);
}

fn maybe_balance(cpu: u32, registry: &[*mut RunQueue; MAX_CPUS]) {
    let rq_ptr = registry[cpu as usize];
    if rq_ptr.is_null() {
        return;
    }
    unsafe {
        let rq = &*rq_ptr;
        let now = rq.clock.load(Ordering::Relaxed);
        let next = rq.next_balance.load(Ordering::Relaxed);
        if now >= next {
            rq.next_balance.store(now + BALANCE_INTERVAL_NS, Ordering::Relaxed);
            smp::load_balance(registry);
        }
    }
}

pub unsafe fn timer_interrupt_handler(cpu: u32) {
    let registry = snapshot_registry();
    if let Some(&rq_ptr) = registry.get(cpu as usize) {
        if !rq_ptr.is_null() {
            let rq = &mut *rq_ptr;

            let flags = rq.lock.lock_irqsave();
            rq.advance_clock(TICK_NS);
            rq.task_tick();
            let need_resched = (*rq.current()).needs_resched();
            rq.lock.unlock_irqrestore(flags);

            if need_resched {
                context_switch(rq);
            }

            maybe_balance(cpu, &registry);
        }
    }
    apic_eoi();
}

pub unsafe fn resched_ipi_handler(cpu: u32) {
    let registry = snapshot_registry();
    if let Some(&rq_ptr) = registry.get(cpu as usize) {
        if !rq_ptr.is_null() {
            let rq = &mut *rq_ptr;
            if (*rq.current()).needs_resched() {
                context_switch(rq);
            }
        }
    }
    apic_eoi();
}

pub unsafe fn wake_up(task: *mut TaskStruct) {
    let registry = snapshot_registry();
    if smp::wake_up_process(task, &registry).is_err() {
        return;
    }
    let target = (*task).cpu.load(Ordering::Acquire);
    if target == CPU_NONE {
        return;
    }
    let this_cpu = current_cpu_id();
    if target != this_cpu {
        send_resched_ipi(target);
    }
}

pub unsafe fn idle_loop(cpu: u32) -> ! {
    loop {
        let registry = snapshot_registry();
        if smp::idle_balance(cpu, &registry) {
            let rq_ptr = registry[cpu as usize];
            if !rq_ptr.is_null() {
                context_switch(&mut *rq_ptr);
            }
        }
        core::arch::asm!("sti", "hlt", options(nomem, nostack));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_defaults_to_null_for_untouched_cpu() {
        let snap = snapshot_registry();
        assert!(snap[MAX_CPUS - 1].is_null());
    }

    #[test]
    fn apic_id_table_defaults_to_none() {
        assert_eq!(cpu_to_apic_id(1), None);
        register_apic_id(1, 7);
        assert_eq!(cpu_to_apic_id(1), Some(7));
    }

    #[test]
    fn register_runqueue_ignores_out_of_range_cpu() {
        register_runqueue(MAX_CPUS as u32 + 10, ptr::null_mut());
    }
}