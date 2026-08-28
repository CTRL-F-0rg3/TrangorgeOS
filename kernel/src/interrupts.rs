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
use crate::cpu::shelduler;


pub static BREAKPOINT_HITS: AtomicU64 = AtomicU64::new(0);
pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
pub static KEYBOARD_HITS: AtomicU64 = AtomicU64::new(0);
pub static IPI_HITS: AtomicU64 = AtomicU64::new(0);

pub const IPI_VECTOR: u8 = 0x30;

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
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

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



extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);

    let _ = crate::cpu::shelduler::tick(0, 1_000_000);
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
