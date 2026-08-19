//! Multi-core management (SMP): CPU detection via ACPI/MADT, Local APIC
//! initialization, starting APs via the trampoline (INIT-SIPI-SIPI), and a
//! self-test confirming the cores run and respond to IPIs.

pub mod acpi;
pub mod lapic;
pub mod shelduler;
pub mod trampoline;

use crate::println;
use crate::testing::TestResult;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// Maksymalna liczba obsługiwanych CPU (indeks 0 = BSP).
pub const MAX_CPUS: usize = 32;
/// Rozmiar stosu per-CPU (64 KiB).
const AP_STACK_SIZE: usize = 64 * 1024;

static TOTAL_CPUS: AtomicU32 = AtomicU32::new(1);
// Physical memory offset from the bootloader, used to read ACPI tables.
static PHYS_OFFSET: AtomicU64 = AtomicU64::new(0);

static AP_STARTED: [AtomicBool; MAX_CPUS] = [const { AtomicBool::new(false) }; MAX_CPUS];
static AP_DONE: [AtomicBool; MAX_CPUS] = [const { AtomicBool::new(false) }; MAX_CPUS];
static AP_APIC_ID_SEEN: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];
static EXPECTED_APIC_IDS: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];

/// Licznik współdzielony przez wszystkie rdzenie w teście współbieżności.
static SHARED_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Ile razy każdy AP ma inkrementować SHARED_COUNTER.
const INCREMENTS_PER_AP: u64 = 1000;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct ApStack([u8; AP_STACK_SIZE]);

static AP_STACKS: [ApStack; MAX_CPUS] = [ApStack([0; AP_STACK_SIZE]); MAX_CPUS];

fn stack_top_of(i: usize) -> u64 {
    let base = &AP_STACKS[i] as *const ApStack as usize;
    // -8, żeby przy wejściu (jmp, bez push adresu powrotnego) spełnić
    // konwencję SysV: rsp % 16 == 8 na początku funkcji.
    (base + AP_STACK_SIZE - 8) as u64
}

/// Aktywna liczba iteracji opóźnienia (przybliżone ms na QEMU).
fn delay(iterations: u64) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

fn delay_ms(ms: u64) {
    delay(ms * 100_000);
}

/// Buduje i ładuje na bieżącym rdzeniu własny GDT + TSS (z własnym stosem IST
/// dla double fault). Ustawia też DS/ES/SS na ważny segment danych — AP wchodzi
/// z trampoliny z SS=0x20 (selektor trampoliny), który nie istnieje w nowym GDT,
/// przez co `iretq` w obsłudze przerwania robi #GP -> double fault.
fn load_cpu_gdt(ist_stack_top: VirtAddr) {
    let tss: &'static mut TaskStateSegment = Box::leak(Box::new(TaskStateSegment::new()));
    tss.interrupt_stack_table[crate::gdt::DOUBLE_FAULT_IST_INDEX as usize] = ist_stack_top;

    let gdt: &'static mut GlobalDescriptorTable = Box::leak(Box::new(GlobalDescriptorTable::new()));
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(tss));

    gdt.load();
    unsafe {
        use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
        CS::set_reg(code_selector);
        DS::set_reg(data_selector);
        ES::set_reg(data_selector);
        SS::set_reg(data_selector);
        x86_64::instructions::tables::load_tss(tss_selector);
    }
}

/// Punkt wejścia każdego AP, wywoływany z trampoliny w 64-bit long mode.
extern "C" fn ap_entry(cpu_index: u64) -> ! {
    let i = cpu_index as usize;

    // Stos jest już ustawiony przez trampolinę — sygnalizuj start, żeby BSP
    // mógł bezpiecznie nadpisać pola trampoliny dla następnego AP.
    AP_STARTED[i].store(true, Ordering::SeqCst);

    // Własny GDT + TSS (własny stos IST).
    load_cpu_gdt(VirtAddr::new(stack_top_of(i)));

    // Własny Local APIC (SVR, LINT zamaskowane — bez PIC/NMI na AP).
    lapic::enable_ap();

    // IDT jest współdzielona — załaduj ją na tym rdzeniu.
    crate::interrupts::init_idt();

    // Samotest rdzenia: zweryfikuj własny APIC ID, licz współdzielony licznik.
    let apic_id = lapic::id();
    AP_APIC_ID_SEEN[i].store(apic_id, Ordering::SeqCst);
    for _ in 0..INCREMENTS_PER_AP {
        SHARED_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    AP_DONE[i].store(true, Ordering::SeqCst);

    // Odbieraj IPI, poza tym śpij.
    x86_64::instructions::interrupts::enable();
    loop {
        x86_64::instructions::hlt();
    }
}

fn wait_for(flag: &AtomicBool, max_ms: u64) -> bool {
    for _ in 0..max_ms {
        if flag.load(Ordering::SeqCst) {
            return true;
        }
        delay_ms(1);
    }
    flag.load(Ordering::SeqCst)
}

/// Wykrywa CPU (ACPI/MADT), inicjalizuje Local APIC i startuje AP.
pub fn init(boot_info: &'static bootloader::BootInfo) {
    let phys_offset = boot_info.physical_memory_offset;
    PHYS_OFFSET.store(phys_offset, Ordering::Relaxed);

    let Some(rsdp) = acpi::find_rsdp(phys_offset) else {
        println!("[cpu] no ACPI RSDP found — single CPU (BSP only)");
        return;
    };
    let rsdp = unsafe { acpi::parse_rsdp(phys_offset, rsdp) };
    let Some(madt_addr) = (unsafe { acpi::find_madt(phys_offset, &rsdp) }) else {
        println!("[cpu] no MADT found — single CPU (BSP only)");
        return;
    };
    let madt = unsafe { acpi::parse_madt(phys_offset, madt_addr) };

    let enabled: Vec<u32> = madt
        .cpus
        .iter()
        .filter(|c| c.enabled)
        .map(|c| c.apic_id)
        .collect();

    println!(
        "[cpu] MADT: {} CPU(s), LAPIC base 0x{:x}",
        enabled.len(),
        madt.lapic_base
    );

    if !lapic::init(madt.lapic_base) {
        println!("[cpu] LAPIC init failed");
        return;
    }
    println!("[cpu] lapic init ok (x2apic={})", lapic::is_x2apic());
    lapic::enable_bsp();
    println!("[cpu] lapic enabled, id={}", lapic::id());

    let bsp_id = lapic::id();
    let aps: Vec<u32> = enabled.into_iter().filter(|&id| id != bsp_id).collect();

    TOTAL_CPUS.store(aps.len() as u32 + 1, Ordering::SeqCst);
    shelduler::init(TOTAL_CPUS.load(Ordering::Acquire) as usize);

    if aps.is_empty() {
        println!("[cpu] single CPU (BSP only), APIC id {}", bsp_id);
        return;
    }

    // Zainstaluj trampolinę raz (identity mapping + kopiowanie kodu).
    trampoline::install(
        unsafe { crate::mm::ffi::paging_read_cr3() },
        ap_entry as usize as u64,
    );
    println!("[cpu] trampoline installed, {} AP(s) to start", aps.len());

    // INIT jest zawsze broadcast (all-excl-self) — wyślij raz, resetując wszystkie
    // AP, zanim którykolwiek zacznie działać. Potem budzimy każdy AP celowanym SIPI.
    println!("[cpu] init ipi (broadcast)");
    lapic::send_init_ipi();
    delay_ms(20);

    // Startuj AP sekwencyjnie (SIPI-SIPI), czekając na każdy.
    for (idx, &apic_id) in aps.iter().enumerate() {
        let cpu_index = idx + 1;
        EXPECTED_APIC_IDS[cpu_index].store(apic_id, Ordering::SeqCst);
        trampoline::set_stack_and_arg(stack_top_of(cpu_index), cpu_index as u64);

        println!("[cpu] sipi -> apic {}", apic_id);
        lapic::send_startup_ipi(apic_id, (trampoline::TRAMPOLINE_BASE >> 12) as u8);
        delay_ms(1);
        lapic::send_startup_ipi(apic_id, (trampoline::TRAMPOLINE_BASE >> 12) as u8);

        if wait_for(&AP_STARTED[cpu_index], 1000) {
            println!("[cpu] AP #{} (apic id {}) started", cpu_index, apic_id);
        } else {
            println!(
                "[cpu] AP #{} (apic id {}) FAILED to start",
                cpu_index, apic_id
            );
        }
    }

    println!("[cpu] SMP: BSP + {} AP(s)", aps.len());
}

/// Liczba wykrytych CPU (BSP + AP).
pub fn total_cpus() -> u32 {
    TOTAL_CPUS.load(Ordering::SeqCst)
}

/// Powers off the machine via ACPI (PM1a_CNT: SLP_TYP = 5, SLP_EN). Returns
/// false if the ACPI tables cannot be found; otherwise never returns.
pub fn poweroff() -> bool {
    let phys_offset = PHYS_OFFSET.load(Ordering::Relaxed);

    let Some(rsdp_addr) = acpi::find_rsdp(phys_offset) else {
        return false;
    };
    let rsdp = unsafe { acpi::parse_rsdp(phys_offset, rsdp_addr) };
    let Some(fadt) = (unsafe { acpi::find_fadt(phys_offset, &rsdp) }) else {
        return false;
    };
    let info = unsafe { acpi::parse_fadt(phys_offset, fadt) };
    if info.pm1a_cnt_blk == 0 {
        return false;
    }

    use x86_64::instructions::port::Port;
    let mut port = Port::<u16>::new(info.pm1a_cnt_blk as u16);
    unsafe { port.write(0x3400) };

    // The write should power the machine off; if it does not, halt forever.
    loop {
        x86_64::instructions::hlt();
    }
}

/// Reboots the machine by pulsing the CPU reset line through the 8042
/// keyboard controller. Never returns.
pub fn reboot() -> ! {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut kbd = Port::<u8>::new(0x64);
        // Wait for the keyboard controller input buffer to become empty.
        while kbd.read() & 0x02 != 0 {
            core::hint::spin_loop();
        }
        kbd.write(0xFE);
    }

    // If the reset did not happen, halt forever.
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn self_test() -> TestResult {
    let total = TOTAL_CPUS.load(Ordering::SeqCst);
    if total <= 1 {
        return Ok("single CPU — SMP bring-up skipped");
    }

    let expected_aps = (total - 1) as usize;

    for i in 1..=expected_aps {
        if !AP_STARTED[i].load(Ordering::SeqCst) {
            return Err("an AP failed to start");
        }
    }

    for i in 1..=expected_aps {
        if !AP_DONE[i].load(Ordering::SeqCst) {
            return Err("an AP did not finish its self-test");
        }
        let seen = AP_APIC_ID_SEEN[i].load(Ordering::SeqCst);
        let exp = EXPECTED_APIC_IDS[i].load(Ordering::SeqCst);
        if seen != exp {
            return Err("AP reported unexpected APIC id");
        }
    }

    // Współbieżność: każdy AP inkrementuje SHARED_COUNTER INCREMENTS_PER_AP razy.
    let expected_count = (expected_aps as u64) * INCREMENTS_PER_AP;
    let count = SHARED_COUNTER.load(Ordering::SeqCst);
    if count != expected_count {
        return Err("shared counter mismatch (concurrency broken)");
    }

    // IPI: wyślij fixed IPI do każdego AP i sprawdź odbiór.
    let before = crate::interrupts::IPI_HITS.load(Ordering::SeqCst);
    for i in 1..=expected_aps {
        lapic::send_ipi(
            crate::interrupts::IPI_VECTOR as u32,
            EXPECTED_APIC_IDS[i].load(Ordering::SeqCst),
        );
    }
    delay_ms(100);
    let after = crate::interrupts::IPI_HITS.load(Ordering::SeqCst);
    if after - before < expected_aps as u64 {
        return Err("IPI delivery to APs failed");
    }

    Ok("SMP bring-up + per-AP self-test + IPI roundtrip")
}
