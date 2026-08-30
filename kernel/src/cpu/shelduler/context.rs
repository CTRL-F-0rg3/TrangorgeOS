use core::arch::naked_asm;

/// Portable CPU context — arch-neutral w obrębie sheldenera.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Context {
    #[cfg(target_arch = "x86_64")]
    pub rsp: u64,
    #[cfg(target_arch = "riscv64")]
    pub sp: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Self {
            #[cfg(target_arch = "x86_64")]
            rsp: 0,
            #[cfg(target_arch = "riscv64")]
            sp: 0,
        }
    }

    /// Bootstrapuje nowy wątek: ustawia pudełko stosu tak, by `switch_to`
    /// „wrócił” do `entry(argument)`.
    pub fn bootstrap(stack_top: u64, entry: u64, argument: u64) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 SysV ABI: 6 volatile + 16-byte aligned; zapas 48 bajtów
            // kryje pierwsze 6 arg (rdi,rsi,rdx,rcx,r8,r9) i przechowywane.
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

        #[cfg(target_arch = "riscv64")]
        {
            // RISC-V calling convention (S-type / stack grows down, 16-byte align):
            // callee-saved `ra, s0-s11, gp, tp` = 14 reg × 8 B = 112 B; zapas 128 B.
            let top = (stack_top & !15u64).saturating_sub(16);
            unsafe {
                let p = top as *mut u64;
                p.sub(0).write(entry);      // "return address" = entry
                p.sub(1).write(argument);   // a0 (pierwszy arg)
                p.sub(2).write(0);
                // reszta zer
                for i in 3..15 {
                    p.sub(i).write(0);
                }
            }
            Self {
                sp: top.saturating_sub(128),
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "riscv64")))]
        {
            // Domyślnie: nie kompilujemy kontekstu na innych platformach.
            compile_error!(
                "Context::bootstrap: nieobsługiwana architektura — dodaj backend dla tej platformy."
            );
            Self {
                #[cfg(target_arch = "x86_64")]
                rsp: 0,
                #[cfg(target_arch = "riscv64")]
                sp: 0,
            }
        }
    }
}

#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(from: *mut Context, to: *const Context) {
    #[cfg(target_arch = "x86_64")]
    {
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

        #[cfg(target_arch = "riscv64")]
    {
        // RISC-V (S-Type / lp64d ABI): a0 = `from`, a1 = `to`.
        // Minimalny switch:
        //  1. zapisujemy aktualny sp do `from->sp`
        //  2. ładujemy nowy sp z `to->sp`
        //  (caller‑saved ra jest już zapisany w bootstrap frame)
        naked_asm!(
            "sd   sp, 0(a0)",          // from->sp = current sp
            "ld   sp, 0(a1)",          // sp = to->sp
            "ret",
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "riscv64")))]
    {
        compile_error!("switch_to: nieobsługiwana architektura");
        naked_asm!();
    }
}
