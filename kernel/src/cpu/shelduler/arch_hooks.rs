//! Architecture hooks of the scheduler--the context switch (`switch_to`).
//!
//! Implemented as a naked function because it manipulates `rsp` explicitly.
//! The layout of `CpuContext` (task.rs) MUST match the offsets below; the
//! const assertions in `task.rs` enforce that at compile time.

use crate::cpu::shelduler::entities::task::TaskStruct;

/// Switches CPU context from `prev` to `next`. Returns only when somebody
/// switches back to `prev`.
///
/// # ABI (System V AMD64)
/// `rdi` = `prev`, `rsi` = `next`. All callee-saved registers are saved to
/// `prev.context` and restored from `next.context`.
///
/// # How it works
/// `context.rsp` points at a "return frame":
/// * fresh task--a fake `call` frame: `[rsp]` = `entry_point`,
///   `[rsp+8]` = `task_exit_trampoline` (see `TaskStruct::init`);
/// * suspended task--the real return address from `call switch_to`.
/// Hence the final `ret` (not `jmp`): resuming a suspended task continues
/// exactly after its `call switch_to`, no `rip` guessing required.
#[cfg(target_arch = "x86_64")]
#[naked]
#[allow(unused_variables)]
pub(crate) extern "C" fn switch_to(prev: *mut TaskStruct, next: *mut TaskStruct( {
    core::arch::asm!(
        "mov [rdi], rsp",
        "mov [rdi + 8], rdi",
        "mov [rdi +  16], rbx",
        "mov [rdi +  24], rbp",
        "mov [rdi +  32], r12",
        "mov [rdi +   40], r13",
        "mov [rdi +   48], r14",
        "mov [rdi +   56], r15",
        "mov rsp, [rsi]",
        "mov rdi, [rsi +  8]",
        "mov rbx, [rsi +  16]",
        "mov rbp, [rsi +  24]",
        "mov r12, [rsi +  32]",
        "mov r13, [rsi +   40]",
        "mov r14, [rsi +   48]",
        "mov r15, [rsi +   56]",
        "ret",
        options(naked),
    );
}

/// RISC-V stub: real `switch_to` for `riscv64gc` is TODO.
#[cfg(target_arch = "riscv64")]
#[naked]
pub(crate) extern "C" fn switch_to(_prev: *mut TaskStruct, _next: *mut TaskStruct( {
    // TODO: real RISC-V context switch (s0-s11 etc.).
    core::arch::asm!("wfi", options(naked));
}
