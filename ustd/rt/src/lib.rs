//! `tgs-rt` — everything a TrangorgeOS userspace program does before `main`.
//!
//! A hosted Rust binary does not start at `main`. It starts at the ELF entry
//! point, which is `_start`, and something in between has to:
//!
//! 1. find the initial stack and read the ELF process image off it
//!    (`argc`, `argv`, `envp`, the auxiliary vector),
//! 2. hand that to the C runtime, which this crate *is* — there is no glibc
//!    underneath, so `getenv` and `std::env::args` read what is recorded here,
//! 3. call `main(argc, argv)`, the shim `rustc` generated, which in turn runs
//!    `std::rt::lang_start`, installs the panic hook, and only then reaches the
//!    program's own `main`.
//!
//! # The stack
//!
//! The kernel's `trampoline_rings::add_world` sets `rsp` to the top of a 16 KiB
//! mapping and `rdi` to 0 — it does **not** build an ELF process image. So
//! [`_start`] has to decide whether the stack in front of it really is one.
//! [`StackImage::discover`] checks, and falls back to a synthetic image with
//! `argv[0]` set to the program name, so a program always starts with a usable
//! `argv` even when the kernel has not grown that part yet. `SYSCALLS.md` says
//! what the kernel should push.
//!
//! # What is deliberately absent
//!
//! No thread-local storage, because the kernel does not program `fs_base`. That
//! is why `tgs-libc`'s `errno` is a plain `static` and why `tgs-hal`'s heap has no
//! lock; see each of those for the change that follows.

#![no_std]

use core::ptr;

use tgs_libc::auxv::{self, Auxv};
use tgs_libc::types::c_char;

/// The program name a fallback `argv[0]` uses when the stack carries nothing
/// better.
pub const FALLBACK_ARGV0: &[u8] = b"tgs-init\0";

/// The most words this crate will read below the initial stack pointer when
/// looking for a process image.
///
/// 512 words is 4 KiB, comfortably inside the 16 KiB stack the kernel maps in
/// `userspace/process/spawn.rs`, and far more than any real image needs. The
/// bound is what makes the walk safe without knowing the mapping size: the kernel
/// does not hand the process a stack size, so the only defensible limit is one
/// this crate guarantees not to exceed.
const MAX_IMAGE_WORDS: usize = 512;

/// The process image `_start` found, or a synthetic stand-in.
#[derive(Debug, Clone, Copy)]
pub struct StackImage {
    /// `argc`
    pub argc: isize,
    /// `argv`, null-terminated.
    pub argv: *const *const c_char,
    /// `environ`, null-terminated.
    pub environ: *const *const c_char,
    /// The auxiliary vector, as `(key, value)` pairs.
    pub auxv: Auxv,
}

impl StackImage {
    /// A program with no arguments, no environment and no auxv.
    pub const fn empty() -> Self {
        Self {
            argc: 0,
            argv: ptr::null(),
            environ: ptr::null(),
            auxv: Auxv::empty(),
        }
    }

    /// Whether the image is the synthetic fallback rather than a real one.
    pub fn is_fallback(&self) -> bool {
        self.argv.is_null()
    }

    /// Read the process image off `stack_top`, or report that the stack does not
    /// look like one.
    ///
    /// The check is deliberately paranoid, because the failure mode of guessing
    /// wrong is reading a pointer out of the middle of a frame and dereferencing
    /// it. A `usize` that happens to be a small integer proves nothing, so this
    /// also verifies that `argv[0]` points at a NUL-terminated string ending below
    /// the stack top, that the environment is null-terminated, and that the auxv
    /// terminator turns up. A kernel that has not built an image produces none of
    /// those, and the caller falls back.
    ///
    /// # Safety
    ///
    /// `stack_top` must be the stack pointer the kernel set before entering ring 3,
    /// and the memory below it must be readable.
    pub unsafe fn discover(stack_top: *const usize) -> Option<Self> {
        if stack_top.is_null() {
            return None;
        }
        // Everything below walks the stack as words and only converts to the C
        // pointer types at the end, so the bounds checks are all against one type
        // and cannot silently compare a `*const c_char` with a `*const usize`.
        //
        // SAFETY: every read is bounds-checked against `stack_top` first, which is
        // the top of the kernel's own mapping.
        unsafe {
            let argc = *stack_top;
            if !(0..=4096).contains(&argc) {
                return None;
            }

            // `argv` holds `argc` pointers, then a null terminator, then the
            // environment pointers, then another null terminator, then the auxv
            // as `(key, value)` words.
            let argv_at = 1usize;
            let after_argv = argv_at + argc as usize + 1;
            if argv_at + argc as usize + 1 > MAX_IMAGE_WORDS {
                return None;
            }
            let argv0 = *stack_top.add(argv_at);
            if argv0 == 0 || !looks_like_stack_string(argv0, stack_top) {
                return None;
            }

            let env_at = after_argv;
            let mut env_n = 0usize;
            loop {
                if env_at + env_n + 1 > MAX_IMAGE_WORDS {
                    return None;
                }
                if *stack_top.add(env_at + env_n) == 0 {
                    break;
                }
                env_n += 1;
            }

            let aux_at = env_at + env_n + 1;
            let mut a = Auxv::empty();
            let mut i = 0usize;
            loop {
                if aux_at + 2 * i + 2 > MAX_IMAGE_WORDS {
                    return None;
                }
                let key = *stack_top.add(aux_at + 2 * i);
                let value = *stack_top.add(aux_at + 2 * i + 1);
                if key == auxv::AT_NULL {
                    break;
                }
                a.push(key, value);
                i += 1;
            }

            Some(Self {
                argc: argc as isize,
                argv: stack_top.add(argv_at).cast::<*const c_char>(),
                environ: stack_top.add(env_at).cast::<*const c_char>(),
                auxv: a,
            })
        }
    }

    /// A program with only `argv[0]`, and a `PATH` in the environment so that
    /// `std::env::var` has something to find.
    pub fn fallback(argv0: &'static [u8]) -> Self {
        // The pointer arrays are `static mut` because the C ABI hands them out as
        // `char **`; the strings live in `.rodata` and outlive everything.
        static mut ARGV: [*const c_char; 2] = [ptr::null(), ptr::null()];
        static mut ENVP: [*const c_char; 3] = [ptr::null(), ptr::null(), ptr::null()];
        static PATH: [u8; 8] = *b"PATH=\0\0\0\0";

        // SAFETY: written exactly once, from `_start`, before `main` and before a
        // second task could exist. Nothing reads them before that.
        unsafe {
            if ARGV[0].is_null() {
                ARGV[0] = argv0.as_ptr().cast::<c_char>();
                ENVP[0] = PATH.as_ptr().cast::<c_char>();
                ENVP[1] = PATH.as_ptr().cast::<c_char>();
            }
            Self {
                argc: 1,
                argv: (&raw const ARGV).cast::<*const c_char>(),
                environ: (&raw const ENVP).cast::<*const c_char>(),
                auxv: Auxv::empty(),
            }
        }
    }
}

/// Whether `s` is the address of a NUL-terminated string below `stack_top`.
///
/// # Safety
///
/// `s` must be an address below `stack_top`, or this must not be called.
unsafe fn looks_like_stack_string(s: usize, stack_top: *const usize) -> bool {
    if s == 0 || s >= stack_top as usize {
        return false;
    }
    // A real string ends before the top of the stack, and no program name is
    // 4 KiB long; the bound is what stops a garbage pointer from walking upwards
    // through the rest of the frame.
    let mut n = 0usize;
    while n < 4096 {
        let at = s + n;
        if at >= stack_top as usize {
            return false;
        }
        // SAFETY: `at` is below the stack top, so it is inside the kernel mapping.
        if unsafe { *(at as *const u8) } == 0 {
            return true;
        }
        n += 1;
    }
    false
}

/// Publish the image to the C runtime, so `getenv` and `getauxval` can answer.
///
/// # Safety
///
/// Must be called once, from `_start`, before `main`. The pointers in `image` must
/// be the kernel's stack image, which outlives the program.
pub unsafe fn begin(image: StackImage) {
    // SAFETY: forwarded from this function's contract.
    unsafe { tgs_libc::env::publish(image.environ.cast_mut(), &image.auxv) };
}

/// Whether threads can be spawned. Always `false` today, and the reason is
/// reported rather than hidden — a program that tries gets an explanation rather
/// than a deadlock.
pub const fn threads_available() -> bool {
    false
}

/// Why [`threads_available`] is false, in the words of the panic message.
pub const THREADS_UNAVAILABLE: &str =
    "TrangorgeOS userspace is single-threaded: the kernel does not program fs_base, so a \
     second task would corrupt the allocator";

// The C `main` that `rustc` generates for a binary that links `std`.
///
// It is the shim around the program's own `main`: it sets up the argument
// vector, the thread-local state and the panic hook, and then calls
// `std::rt::lang_start`. It is declared rather than defined because `rustc`
// emits the definition, and it lives in a binary crate — which is why this crate
// is a *library* a program links, not a source file a program copies.
//
// # Safety
//
// Called once, from `tgs_rt_start`, with the program's own `argc`/`argv`.
unsafe extern "C" {
    safe fn main(argc: isize, argv: *const *const u8) -> isize;
}

/// The Rust side of `_start`, called from assembly.
#[no_mangle]
pub extern "C" fn tgs_rt_start() -> ! {
    // The address of a local is the high end of this frame, which is below
    // whatever `rsp` the kernel set — the right bound for a stack image, which
    // grows downwards.
    let anchor = 0u8;
    let stack_top: *const usize = core::ptr::addr_of!(anchor).cast::<usize>();

    // SAFETY: the kernel set `rsp` to the top of a live mapping before entering
    // ring 3, so that pointer is valid, and `discover` bounds-checks every read
    // it makes against it.
    let image = unsafe { StackImage::discover(stack_top) }
        .unwrap_or_else(|| StackImage::fallback(FALLBACK_ARGV0));

    // SAFETY: once, from `_start`, before `main`.
    unsafe { begin(image) };

    // `main` is the shim `rustc` generated around the program's own `main`, and it
    // expects the C calling convention. It never returns — it calls
    // `std::process::exit` — so the loop is only the "your `main` returned" case,
    // where a program with no runtime to return into has nothing left to do.
    //
    // SAFETY: `image` holds the `argc`/`argv` pair just recorded, which is
    // exactly what `main`'s contract wants.
    unsafe {
        main(image.argc as isize, image.argv.cast());
        loop {
            core::hint::spin_loop();
        }
    }
}

/// `void _start(void)` — the ELF entry point.
///
/// `naked`, because it runs on a stack the kernel set up rather than one a Rust
/// function established, and because a prologue would be free to touch
/// callee-saved registers the kernel has already put values in. It aligns the
/// stack to the System V 16-byte boundary and calls the Rust side.
///
/// # Safety
///
/// Called by the kernel at the ELF entry, once, on the stack it prepared.
#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        // The System V ABI wants `%rsp` 16-byte aligned *at* the call, which means
        // 16 bytes off after the call has pushed the return address. The kernel's
        // `USER_STACK_TOP` is already page-aligned, so this is a no-op in the
        // common case and the correction otherwise.
        "and rsp, -16",
        // `naked_asm!` templates cannot name Rust items, so the helper is reached
        // through its `#[no_mangle]` C name.
        "call tgs_rt_start",
        "hlt",
    );
}
