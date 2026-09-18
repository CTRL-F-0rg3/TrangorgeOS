use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use spin::Mutex;


const HEAP_SIZE: usize = 2 * 1024 * 1024;

#[repr(C, align(4096))]
struct HeapSpace([u8; HEAP_SIZE]);

static HEAP_MEM: HeapSpace = HeapSpace([0; HEAP_SIZE]);

struct Bump {
    next: usize,
    end: usize,
}

static BUMP: Mutex<Bump> = Mutex::new(Bump { next: 0, end: 0 });

pub fn heap_init() {
    let base = &HEAP_MEM as *const HeapSpace as usize;
    let mut b = BUMP.lock();
    b.next = base;
    b.end = base + HEAP_SIZE;
}

pub struct KernelHeap;

unsafe impl GlobalAlloc for KernelHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut b = BUMP.lock();
        let align = layout.align().max(core::mem::size_of::<usize>());
        let cur = (b.next + align - 1) & !(align - 1);
        match cur.checked_add(layout.size()) {
            Some(new_next) if new_next <= b.end => {
                b.next = new_next;
                cur as *mut u8
            }
            _ => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    }
}

#[cfg(target_arch = "riscv64")]
#[global_allocator]
static KERNEL_ALLOC: KernelHeap = KernelHeap;

pub fn now() -> u64 {
    let t: u64;
    unsafe {
        core::arch::asm!("rdtime {0}", out(reg) t, options(nomem, nostack));
    }
    t
}

#[panic_handler]
fn riscv_panic(info: &PanicInfo) -> ! {
    crate::serial::write_str("\n[PANIC] ");
    crate::serial::print_args(format_args!("{}\n", info));
    crate::hlt_loop()
}

pub fn init() {
    early_uart("[C]");
    heap_init();
    early_uart("[D]");
    crate::serial::init();
    early_uart("[E]");
}

pub fn current_cpu() -> usize {
    0
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

pub fn early_uart(s: &str) {
    const THR: *mut u8 = 0x1000_0000 as *mut u8;
    const LSR: *const u8 = 0x1000_0005 as *const u8;
    for b in s.bytes() {
        unsafe {
            while LSR.read_volatile() & 0x20 == 0 {}
            THR.write_volatile(b);
        }
    }
}


#[unsafe(naked)]
#[no_mangle]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        ".option push",
        ".option norelax",
        "la gp, __global_pointer$",
        ".option pop",
        "la sp, _stack_top",
        "call {start_rust}",
        "1: j 1b",
        start_rust = sym start_rust,
    );
}

extern "C" fn start_rust() -> ! {
    early_uart("[B]");
    crate::kernel_main_riscv()
}