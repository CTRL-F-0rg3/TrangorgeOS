// User CPU context (equivalent to Linux pt_regs on x86_64)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UserContext {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub rbp: u64, pub rbx: u64,
    pub r11: u64, pub r10: u64, pub r9: u64, pub r8: u64,
    pub rax: u64, pub rcx: u64, pub rdx: u64, pub rsi: u64,
    pub rdi: u64,
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub eflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl UserContext {
    // Extract syscall arguments according to x86_64 Linux ABI
    #[inline]
    pub fn syscall_args(&self) -> (u64, u64, u64, u64, u64, u64) {
        (self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9)
    }

    // Set syscall return value
    #[inline]
    pub fn set_return(&mut self, val: u64) {
        self.rax = val;
    }
}