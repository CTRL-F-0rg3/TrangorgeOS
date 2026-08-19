//! Minimalny kontekst ABI x86_64 używany przez dobrowolne przełączanie tasków.
use core::arch::naked_asm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub rsp: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Self { rsp: 0 }
    }

    /// Tworzy stos początkowy zgodny z sekwencją `switch_to`.
    ///
    /// `switch_to` zdejmie sześć rejestrów, a następnie wykona `ret`. Dlatego
    /// układ zawiera sześć zerowanych rejestrów oraz adres funkcji wejściowej.
    pub fn bootstrap(stack_top: u64, entry: u64, argument: u64) -> Self {
        let top = (stack_top & !15u64).saturating_sub(8);
        unsafe {
            let p = top as *mut u64;
            p.sub(0).write(entry);
            p.sub(1).write(0);
            p.sub(2).write(0);
            p.sub(3).write(0);
            p.sub(4).write(0);
            p.sub(5).write(0);
            p.sub(6).write(0);
            p.sub(7).write(argument);
        }
        Self {
            rsp: top.saturating_sub(48),
        }
    }
}

/// Przełącza dobrowolnie z jednego stosu kernela na drugi.
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(from: *mut Context, to: *const Context) {
    naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",
        "mov rsp, [rsi]",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}
