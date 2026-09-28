use crate::gdt;
use crate::println;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;
use x86_64::registers::control::Cr2;
use x86_64::instructions::port::Port;
use crate::cpu::lapic;

// Interrupts module
pub static BREAKPOINT_HITS: AtomicU64 = AtomicU64::new(0);
pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
pub static KEYBOARD_HITS: AtomicU64 = AtomicU64::new(0);
pub static IPI_HITS: AtomicU64 = AtomicU64::new(0);

pub const IPI_VECTOR: u8 = 0x30;
// This is a test to ensure that the IDT is loaded correctly and that the timer IRQ is working.
crate::test_module!({
    let hits_before = BREAKPOINT_HITS.load(Ordering::SeqCst);
    x86_64::instructions::interrupts::int3();
    let hits_after = BREAKPOINT_HITS.load(Ordering::SeqCst);
    if hits_after != hits_before + 1 {
        return Err("breakpoint handler did not increment counter - IDT not loaded correctly?");
    }

    let ticks_before = TIMER_TICKS.load(Ordering::SeqCst);
    let mut waited = 0;
    while TIMER_TICKS.load(Ordering::SeqCst) == ticks_before && waited < 1_000_000 {
        x86_64::instructions::hlt();
        waited += 1;
    }
    if TIMER_TICKS.load(Ordering::SeqCst) == ticks_before {
        return Err("timer IRQ never arrived - PIC/IDT wiring broken");
    }

    Ok("breakpoint counted + timer IRQ confirmed live")
});
// Initialize the Programmable Interrupt Controller (PIC)
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}
extern "x86-interrupt" 
fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    BREAKPOINT_HITS.fetch_add(1, Ordering::SeqCst); // dlatego systemy operaCYcyjne są takie trudne do napisania, bo trzeba pamiętać o wszystkim, a nie tylko o tym co się robi w danej chwili 
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
// Initialize the Interrupt Descriptor Table (IDT)
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt[IPI_VECTOR].set_handler_fn(ipi_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

/// Assert that the interrupt lines the kernel uses are actually enabled.
///
/// # Why this is not optional
///
/// `ChainedPics::initialize` — called from `arch::x86_64::init` — programs the
/// vector offsets and then *restores the mask registers it found*, deliberately:
/// it has no opinion about which inputs the guest wants. So after
/// `initialize` the PIC mask is whatever the firmware left behind, and nothing
/// in this kernel has ever asserted that the lines it installs handlers for are
/// among the unmasked ones.
///
/// That is a real gap rather than a hypothetical one. A line whose mask bit is
/// set produces no interrupt no matter that its handler is registered in the IDT
/// and is correct — the interrupt simply never reaches the CPU. The keyboard
/// line is the one that matters here: the driver reports a keyboard present and
/// `terminal::push_scancode` is wired to the handler, so every layer above the
/// PIC looks healthy while the keyboard stays permanently dead.
///
/// Note that `pic8259` 0.10 exposes no `unmask`: the mask registers are read and
/// written whole via `read_masks`/`write_masks`, so the bits to clear are
/// computed here. The indices are *vector* numbers, which is what `InterruptIndex`
/// holds, so each one is biased back down to a line number before being shifted.
pub fn unmask_irqs() {
    /// The mask bit for a vector: bias the vector down to a line, then shift.
    const fn bit(vector: u8) -> u8 {
        1u8 << (vector - PIC_1_OFFSET)
    }

    unsafe {
        let mut pics = PICS.lock();
        let [master, slave] = pics.read_masks();
        // Both lines live on the master PIC. The slave mask is passed through
        // untouched rather than assumed, since this kernel drives it indirectly
        // through the local APIC.
        let keep = bit(InterruptIndex::Timer.as_u8()) | bit(InterruptIndex::Keyboard.as_u8());
        pics.write_masks(master & !keep, slave);
    }
}
// Interrupt handlers
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    // Route through panic! (and therefore the panic screen) instead of
    // println!() + hlt_loop() directly — previously a page fault only ever
    // wrote into the invisible VGA text buffer, then halted, so it looked
    // exactly like a silent freeze on whatever was already on screen.
    panic!(
        "EXCEPTION: PAGE FAULT\naccessed address: {:#x}\nerror code: {:?}\n{:#?}",
        Cr2::read_raw(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


// Interrupt handlers
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);

    // Drain the shared-memory (tg_comm) communication rings for both the
    // driver space and the user space (ring 3).
    crate::tgcomm::driverspace::poll();
    crate::tgcomm::userspace::poll();

    unsafe {
        let _ = crate::cpu::scheduler::tick(0, 1_000_000);
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
        
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    KEYBOARD_HITS.fetch_add(1, Ordering::Relaxed);

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    crate::terminal::push_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn ipi_handler(_stack_frame: InterruptStackFrame) {
    IPI_HITS.fetch_add(1, Ordering::SeqCst);
    crate::cpu::lapic::eoi();
}
