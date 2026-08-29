//! RISC-V (riscv64gc) bootstrap backend.
//!
//! Boot path: OpenSBI (`-bios default` on the QEMU `virt` machine) jumps to
//! the address in `riscv64-link.ld` (0x8020_0000) and this backend takes it
//! from there: it initializes the small bump heap (so `alloc` works kernel
//! wide), brings up the NS16550A serial console (MMIO at 0x1000_0000), prints
//! the banner through the shared `println!` pipeline and idles with `wfi`.
//!
//! Still missing (tracked for the full port): CLINT timer interrupts
//! (stimecmp/Sstc), Sv39 paging, device-tree parsing, PLIC interrupt
//! handling and the driverspace bootstrap.

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use spin::Mutex;

// ---- heap: small bump allocator (enables `alloc` kernel-wide) --------------

const HEAP_SIZE: usize = 2 * 1024 * 1024;

#[repr(C, align(4096))]
struct HeapSpace([u8; HEAP_SIZE]);

static HEAP_MEM: HeapSpace = HeapSpace([0; HEAP_SIZE]);

struct Bump {
    next: usize,
    end: usize,
}

static BUMP: Mutex<Bump> = Mutex::new(Bump { next: 0, end: 0 });

/// Initialize the bump heap. Must be called before any allocation happens
/// (done in [`init`], before the serial console and kernel main).
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
        // Bump allocator: memory is not reclaimed (kernel scaffold stage).
        // The x86_64 MM subsystem provides the full-featured allocator.
    }
}

#[cfg(target_arch = "riscv64")]
#[global_allocator]
static KERNEL_ALLOC: KernelHeap = KernelHeap;

// ---- time -------------------------------------------------------------------

/// Monotonic kernel time via the `time` CSR (`rdtime`).
///
/// NOTE: do NOT read the CLINT `mtime` MMIO register here — on the QEMU
/// `virt` machine CLINT is M-mode only (OpenSBI PMP: S/U have no access),
/// so an S-mode read faults with no trap handler installed. `rdtime`
/// (CSR 0xC01, Zicntr) is the S-mode-visible mirror of `mtime` and is what
/// the platform gives S-mode payloads.
pub fn now() -> u64 {
    let t: u64;
    unsafe {
        core::arch::asm!("rdtime {0}", out(reg) t, options(nomem, nostack));
    }
    t
}

/// Index of the CPU executing this code (single-hart: 0; SBI HSM brings
/// per-hart state with the SMP milestone).
pub fn current_cpu() -> usize {
    0
}

// ---- bootstrap ---------------------------------------------------------------

/// RISC-V panic handler: dump the message over the serial console and idle.
#[panic_handler]
fn riscv_panic(info: &PanicInfo) -> ! {
    crate::serial::write_str("\n[PANIC] ");
    crate::serial::print_args(format_args!("{}\n", info));
    crate::hlt_loop()
}

/// Early RISC-V initialization: heap first (any allocation must be possible
/// afterwards), then the serial console.
pub fn init() {
    early_uart("[C]");
    heap_init();
    early_uart("[D]");
    crate::serial::init();
    early_uart("[E]");
}

/// Current CPU ID: on UP RISC-V the boot hart is always 0.
pub fn current_cpu() -> usize {
    0
}

/// Park the CPU in the machine's idle state (hlt on x86_64, wfi on RISC-V).
pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}
pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Poll-driven UART write used BEFORE any kernel state (no statics, no
/// locks). QEMU virt maps the NS16550A UART at 0x1000_0000 (THR at +0,
/// LSR at +5, LSR bit 5 = THR empty). This is the boot diagnostic channel.
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

/// Bare-metal CPU entry point. OpenSBI transfers control here after loading
/// the kernel at the address in `riscv64-link.ld` (0x8020_0000 on `virt`).
///
/// Placed in `.text.entry`, which the linker script keeps as the very first
/// thing in `.text` — so the ELF entry address, the image base and the
/// address OpenSBI jumps to are all the same instruction.
///
/// `#[naked]` + `naked_asm!`: no compiler-generated prologue may run before
/// `gp` (global pointer, required for gp-relative relaxation done by rust-lld)
/// and `sp` (boot stack from the linker script) are initialized.
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
        "1: j 1b", // call never returns (-> !), but keep a well-defined tail
        start_rust = sym start_rust,
    );
}

/// First Rust code to run: called from `_start` with `gp`/`sp` ready.
extern "C" fn start_rust() -> ! {
    early_uart("[B]");
    crate::kernel_main_riscv()
}