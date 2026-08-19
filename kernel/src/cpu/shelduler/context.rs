

use core::arch::naked_asm;

#[repr(C)]
pub struct Context {
    pub rsp: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Context { rsp: 0 }
    }
}


#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(from: *mut Context, to: *const Context) {
    // rdi = from, rsi = to (System V AMD64 ABI)
    naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",   // (*from).rsp = rsp bieżącego zadania
        "mov rsp, [rsi]",   // rsp = (*to).rsp
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",              // skacze pod adres zdjęty ze szczytu stosu `to`
    );
}