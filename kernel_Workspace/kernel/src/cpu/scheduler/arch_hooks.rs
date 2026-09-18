#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, Ordering};

use crate::cpu::scheduler::entities::task::{CpuContext, SchedPolicy, TaskStruct, CPU_NONE, MAX_CPUS};
use crate::cpu::scheduler::runqueue::{smp, RunQueue, BALANCE_INTERVAL_NS};

pub const LOCAL_APIC_BASE: usize = 0xFEE0_0000;
pub const APIC_REG_ID: usize = 0x020;
pub const APIC_REG_EOI: usize = 0x0B0;
pub const APIC_REG_SVR: usize = 0x0F0;
pub const APIC_REG_ICR_LOW: usize = 0x300;
pub const APIC_REG_ICR_HIGH: usize = 0x310;
pub const APIC_REG_LVT_TIMER: usize = 0x320;
pub const APIC_REG_TIMER_ICR: usize = 0x380;
pub const APIC_REG_TIMER_CCR: usize = 0x390;
pub const APIC_REG_TIMER_DIV: usize = 0x3E0;

pub const RESCHED_VECTOR: u8 = 0xFD;
pub const TLB_SHOOTDOWN_VECTOR: u8 = 0xFC;
pub const TIMER_VECTOR: u8 = 0xEF;
pub const SPURIOUS_VECTOR: u8 = 0xFF;
pub const NMI_WATCHDOG_VECTOR: u8 = 0x02;

pub const TICK_HZ: u64 = 1000;
pub const TICK_NS: u64 = 1_000_000_000 / TICK_HZ;
pub const RT_WATCHDOG_LIMIT_NS: u64 = 4_000_000_000;

pub const AP_TRAMPOLINE_PAGE: u32 = 0x8000;
pub const PIT_FREQUENCY_HZ: u32 = 1_193_182;

// ----------------------------------------------------------------------
// Rejestry per-CPU
// ----------------------------------------------------------------------

const NULL_RQ: AtomicPtr<RunQueue> = AtomicPtr::new(ptr::null_mut());
static RQ_REGISTRY: [AtomicPtr<RunQueue>; MAX_CPUS] = [NULL_RQ; MAX_CPUS];

const NO_APIC_ID: AtomicU32 = AtomicU32::new(u32::MAX);
static APIC_ID_TABLE: [AtomicU32; MAX_CPUS] = [NO_APIC_ID; MAX_CPUS];

const NO_PENDING: AtomicBool = AtomicBool::new(false);
static TLB_SHOOTDOWN_PENDING: [AtomicBool; MAX_CPUS] = [NO_PENDING; MAX_CPUS];

static TSC_HZ: AtomicU64 = AtomicU64::new(0);
static BOOTED_CPUS: AtomicU32 = AtomicU32::new(0);

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

pub fn set_tsc_frequency(hz: u64) {
    TSC_HZ.store(hz, Ordering::Release);
}

pub fn tsc_frequency() -> u64 {
    TSC_HZ.load(Ordering::Acquire)
}

pub fn booted_cpu_count() -> u32 {
    BOOTED_CPUS.load(Ordering::Acquire)
}

// ----------------------------------------------------------------------
// Port I/O — uprzywilejowane, więc wyłącznie prawdziwe na jądrowym x86_64
// ----------------------------------------------------------------------

#[cfg(all(target_arch = "x86_64", not(test)))]
unsafe fn out_u8(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[cfg(all(target_arch = "x86_64", not(test)))]
unsafe fn in_u8(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nomem, nostack, preserves_flags));
    val
}

#[cfg(any(not(target_arch = "x86_64"), test))]
unsafe fn out_u8(_port: u16, _val: u8) {}

#[cfg(any(not(target_arch = "x86_64"), test))]
unsafe fn in_u8(_port: u16) -> u8 {
    0
}

// ----------------------------------------------------------------------
// CPUID / RDTSC — nieuprzywilejowane, bezpieczne również w testach
// ----------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let (eax, ebx, ecx, edx);
    core::arch::asm!(
        "cpuid",
        inout("eax") leaf => eax,
        out("ebx") ebx,
        out("ecx") ecx,
        out("edx") edx,
        options(nostack, preserves_flags)
    );
    (eax, ebx, ecx, edx)
}

#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn cpuid(_leaf: u32) -> (u32, u32, u32, u32) {
    (0, 0, 0, 0)
}

pub unsafe fn has_apic() -> bool {
    let (_, _, _, edx) = cpuid(1);
    edx & (1 << 9) != 0
}

pub unsafe fn has_x2apic() -> bool {
    let (_, _, ecx, _) = cpuid(1);
    ecx & (1 << 21) != 0
}

pub unsafe fn has_invariant_tsc() -> bool {
    let (eax, _, _, edx) = cpuid(0x8000_0000);
    if eax < 0x8000_0007 {
        return false;
    }
    let (_, _, _, edx2) = cpuid(0x8000_0007);
    let _ = edx;
    edx2 & (1 << 8) != 0
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack, preserves_flags));
    ((hi as u64) << 32) | lo as u64
}

#[cfg(not(target_arch = "x86_64"))]
pub unsafe fn rdtsc() -> u64 {
    0
}

fn ns_from_tsc(tsc_delta: u64, hz: u64) -> u64 {
    if hz == 0 {
        0
    } else {
        ((tsc_delta as u128) * 1_000_000_000u128 / hz as u128) as u64
    }
}

pub fn now_ns() -> u64 {
    ns_from_tsc(unsafe { rdtsc() }, tsc_frequency())
}

/// Kalibruje TSC względem kanału 2 PIT (jak w prawdziwym jądrze — PIT
/// jest wolniejszy, ale zawsze obecny). Wymaga wyłączonych przerwań.
pub unsafe fn calibrate_tsc_via_pit(calib_ms: u32) -> u64 {
    const PIT_GATE_PORT: u16 = 0x61;
    const PIT_CMD_PORT: u16 = 0x43;
    const PIT_CH2_DATA_PORT: u16 = 0x42;

    let count = ((PIT_FREQUENCY_HZ as u64 * calib_ms as u64) / 1000) as u16;

    let gate = in_u8(PIT_GATE_PORT);
    out_u8(PIT_GATE_PORT, (gate & 0xFD) | 0x01);
    out_u8(PIT_CMD_PORT, 0b1011_0010);
    out_u8(PIT_CH2_DATA_PORT, (count & 0xFF) as u8);
    out_u8(PIT_CH2_DATA_PORT, ((count >> 8) & 0xFF) as u8);

    let start = rdtsc();
    loop {
        if in_u8(PIT_GATE_PORT) & 0x20 != 0 {
            break;
        }
        core::hint::spin_loop();
    }
    let end = rdtsc();

    let delta = end.saturating_sub(start);
    let measured_hz = (delta * 1000) / calib_ms as u64;
    set_tsc_frequency(measured_hz);
    measured_hz
}

fn timer_count_for_hz(hz: u64) -> u32 {
    if hz == 0 {
        1_000_000
    } else {
        cmp_max_1((hz / TICK_HZ) as u32)
    }
}

fn cmp_max_1(v: u32) -> u32 {
    if v == 0 {
        1
    } else {
        v
    }
}

fn default_timer_count() -> u32 {
    timer_count_for_hz(tsc_frequency())
}

// ----------------------------------------------------------------------
// Local APIC MMIO — nieodwzorowane w przestrzeni testowej, więc stub
// dla `test`/architektur innych niż x86_64
// ----------------------------------------------------------------------

#[cfg(all(target_arch = "x86_64", not(test)))]
unsafe fn apic_write(reg: usize, value: u32) {
    ptr::write_volatile((LOCAL_APIC_BASE + reg) as *mut u32, value);
}

#[cfg(all(target_arch = "x86_64", not(test)))]
unsafe fn apic_read(reg: usize) -> u32 {
    ptr::read_volatile((LOCAL_APIC_BASE + reg) as *const u32)
}

#[cfg(any(not(target_arch = "x86_64"), test))]
unsafe fn apic_write(_reg: usize, _value: u32) {}

#[cfg(any(not(target_arch = "x86_64"), test))]
unsafe fn apic_read(_reg: usize) -> u32 {
    0
}

unsafe fn apic_eoi() {
    apic_write(APIC_REG_EOI, 0);
}

unsafe fn wait_icr_idle() {
    while apic_read(APIC_REG_ICR_LOW) & (1 << 12) != 0 {
        core::hint::spin_loop();
    }
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
    wait_icr_idle();
}

pub unsafe fn send_resched_ipi(cpu: u32) {
    if let Some(apic_id) = cpu_to_apic_id(cpu) {
        send_ipi(apic_id, RESCHED_VECTOR);
    }
}

pub unsafe fn send_tlb_shootdown(cpu: u32) {
    if (cpu as usize) >= MAX_CPUS {
        return;
    }
    TLB_SHOOTDOWN_PENDING[cpu as usize].store(true, Ordering::Release);
    if let Some(apic_id) = cpu_to_apic_id(cpu) {
        send_ipi(apic_id, TLB_SHOOTDOWN_VECTOR);
    }
}

pub unsafe fn broadcast_tlb_shootdown(except_cpu: u32) {
    for cpu in 0..MAX_CPUS as u32 {
        if cpu != except_cpu && cpu_to_apic_id(cpu).is_some() {
            send_tlb_shootdown(cpu);
        }
    }
}

unsafe fn flush_tlb() {
    let cr3: u64;
    core::arch::asm!("mov {0}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
    core::arch::asm!("mov cr3, {0}", in(reg) cr3, options(nomem, nostack));
}

pub unsafe fn init_local_apic() {
    apic_write(APIC_REG_SVR, SPURIOUS_VECTOR as u32 | (1 << 8));
}

pub unsafe fn init_local_timer() {
    apic_write(APIC_REG_TIMER_DIV, 0b1011);
    apic_write(APIC_REG_LVT_TIMER, TIMER_VECTOR as u32 | (1 << 17));
    apic_write(APIC_REG_TIMER_ICR, default_timer_count());
}

// ----------------------------------------------------------------------
// IDT — instalacja bramek przerwań (x86_64, 64-bitowe deskryptory bramek)
// ----------------------------------------------------------------------

pub const IDT_SIZE: usize = 256;
pub const GATE_INTERRUPT: u8 = 0x8E;
pub const GATE_TRAP: u8 = 0x8F;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self { offset_low: 0, selector: 0, ist: 0, type_attr: 0, offset_mid: 0, offset_high: 0, zero: 0 }
    }

    pub fn set(&mut self, handler: usize, selector: u16, ist: u8, type_attr: u8) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = ((handler >> 32) & 0xFFFF_FFFF) as u32;
        self.selector = selector;
        self.ist = ist & 0x7;
        self.type_attr = type_attr;
        self.zero = 0;
    }

    pub fn is_present(&self) -> bool {
        self.type_attr & 0x80 != 0
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::missing(); IDT_SIZE];

pub unsafe fn install_gate(vector: u8, handler: usize, selector: u16, ist: u8, type_attr: u8) {
    let entry_ptr = ptr::addr_of_mut!(IDT[vector as usize]);
    (*entry_ptr).set(handler, selector, ist, type_attr);
}

#[cfg(all(target_arch = "x86_64", not(test)))]
pub unsafe fn load_idt() {
    let desc = IdtDescriptor {
        limit: (core::mem::size_of::<[IdtEntry; IDT_SIZE]>() - 1) as u16,
        base: ptr::addr_of!(IDT) as u64,
    };
    core::arch::asm!("lidt [{0}]", in(reg) &desc, options(readonly, nostack, preserves_flags));
}

#[cfg(any(not(target_arch = "x86_64"), test))]
pub unsafe fn load_idt() {}

// ----------------------------------------------------------------------
// Uruchamianie procesorów aplikacyjnych (INIT-SIPI-SIPI)
// ----------------------------------------------------------------------

pub unsafe fn start_application_processor(apic_id: u32) {
    apic_write(APIC_REG_ICR_HIGH, apic_id << 24);
    apic_write(APIC_REG_ICR_LOW, 0x4500);
    wait_icr_idle();
    busy_wait_ns(10_000_000);

    for _ in 0..2 {
        apic_write(APIC_REG_ICR_HIGH, apic_id << 24);
        apic_write(APIC_REG_ICR_LOW, 0x4600 | (AP_TRAMPOLINE_PAGE >> 12));
        wait_icr_idle();
        busy_wait_ns(200_000);
    }
}

unsafe fn busy_wait_ns(ns: u64) {
    let hz = tsc_frequency();
    if hz == 0 {
        return;
    }
    let cycles = (hz as u128 * ns as u128 / 1_000_000_000u128) as u64;
    let start = rdtsc();
    while rdtsc().wrapping_sub(start) < cycles {
        core::hint::spin_loop();
    }
}

pub unsafe fn cpu_bringup(cpu: u32, apic_id: u32, rq: *mut RunQueue) -> ! {
    register_apic_id(cpu, apic_id);
    register_runqueue(cpu, rq);
    init_local_apic();
    init_local_timer();
    BOOTED_CPUS.fetch_add(1, Ordering::AcqRel);
    idle_loop(cpu)
}

// ----------------------------------------------------------------------
// Przełączenie kontekstu i obsługa przerwań planisty
// ----------------------------------------------------------------------

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

    if (*next).policy == SchedPolicy::Fifo || (*next).policy == SchedPolicy::RoundRobin {
        (*next).rt.watchdog_stamp = now_ns();
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

pub unsafe fn tlb_shootdown_handler(cpu: u32) {
    if (cpu as usize) < MAX_CPUS && TLB_SHOOTDOWN_PENDING[cpu as usize].swap(false, Ordering::AcqRel) {
        flush_tlb();
    }
    apic_eoi();
}

pub unsafe fn spurious_interrupt_handler() {
    apic_eoi();
}

pub unsafe fn nmi_watchdog_handler(cpu: u32) {
    let registry = snapshot_registry();
    let rq_ptr = match registry.get(cpu as usize) {
        Some(&p) if !p.is_null() => p,
        _ => return,
    };
    let rq = &*rq_ptr;
    let curr = rq.current();
    if curr.is_null() {
        return;
    }
    if (*curr).policy != SchedPolicy::Fifo {
        return;
    }
    let now = now_ns();
    if now.saturating_sub((*curr).rt.watchdog_stamp) > RT_WATCHDOG_LIMIT_NS {
        (*curr).set_need_resched();
    }
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

#[cfg(all(target_arch = "x86_64", not(test)))]
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

#[cfg(any(not(target_arch = "x86_64"), test))]
pub unsafe fn idle_loop(cpu: u32) -> ! {
    let registry = snapshot_registry();
    let _ = smp::idle_balance(cpu, &registry);
    loop {
        core::hint::spin_loop();
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
    fn apic_id_table_defaults_to_none_then_roundtrips() {
        assert_eq!(cpu_to_apic_id(2), None);
        register_apic_id(2, 9);
        assert_eq!(cpu_to_apic_id(2), Some(9));
    }

    #[test]
    fn register_runqueue_ignores_out_of_range_cpu() {
        register_runqueue(MAX_CPUS as u32 + 10, ptr::null_mut());
    }

    #[test]
    fn ns_from_tsc_is_zero_without_calibration() {
        assert_eq!(ns_from_tsc(1_000_000, 0), 0);
    }

    #[test]
    fn ns_from_tsc_scales_correctly() {
        // 1 GHz zegar: 1_000_000 cykli = 1_000_000 ns.
        assert_eq!(ns_from_tsc(1_000_000, 1_000_000_000), 1_000_000);
        // 2 GHz zegar: te same cykle to połowa czasu.
        assert_eq!(ns_from_tsc(1_000_000, 2_000_000_000), 500_000);
    }

    #[test]
    fn timer_count_for_hz_has_sane_fallback() {
        assert_eq!(timer_count_for_hz(0), 1_000_000);
        assert_eq!(timer_count_for_hz(1_000_000_000), 1_000_000);
    }

    #[test]
    fn timer_count_for_hz_never_returns_zero() {
        assert_eq!(timer_count_for_hz(TICK_HZ / 2), 1);
    }

    #[test]
    fn tlb_pending_flag_sets_and_clears() {
        let idx = MAX_CPUS - 2;
        TLB_SHOOTDOWN_PENDING[idx].store(true, Ordering::Release);
        assert!(TLB_SHOOTDOWN_PENDING[idx].swap(false, Ordering::AcqRel));
        assert!(!TLB_SHOOTDOWN_PENDING[idx].load(Ordering::Acquire));
    }

    #[test]
    fn idt_entry_encodes_and_reports_present() {
        let mut e = IdtEntry::missing();
        assert!(!e.is_present());
        e.set(0xDEAD_BEEF_1234, 0x08, 1, GATE_INTERRUPT);
        assert!(e.is_present());
    }

    #[test]
    fn idt_entry_missing_is_not_present() {
        assert!(!IdtEntry::missing().is_present());
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn cpuid_leaf_zero_reports_a_nonzero_max_leaf() {
        unsafe {
            let (eax, _, _, _) = cpuid(0);
            assert!(eax >= 1);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn rdtsc_does_not_go_backwards_across_two_reads() {
        unsafe {
            let a = rdtsc();
            let b = rdtsc();
            assert!(b >= a);
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[test]
    fn cpuid_and_rdtsc_are_zero_off_x86_64() {
        unsafe {
            assert_eq!(cpuid(0), (0, 0, 0, 0));
            assert_eq!(rdtsc(), 0);
        }
    }

    #[test]
    fn set_and_read_tsc_frequency_is_stable_within_this_test() {
        set_tsc_frequency(3_000_000_000);
        assert_eq!(tsc_frequency(), 3_000_000_000);
    }

    #[test]
    fn booted_cpu_count_starts_at_or_above_zero() {
        assert!(booted_cpu_count() < u32::MAX);
    }
}