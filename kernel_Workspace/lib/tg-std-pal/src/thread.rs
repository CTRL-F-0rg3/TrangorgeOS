use tg_ipc_abi::opcodes::SysOpcode;
use super::raw;

pub unsafe fn spawn(stack_ptr: u64, stack_size: u64, entry: u64, arg: u64) -> u64 {
    raw::syscall4(SysOpcode::ThreadCreate, stack_ptr, stack_size, entry, arg)
}

pub unsafe fn exit(code: u64) -> ! {
    raw::syscall1(SysOpcode::ThreadExit, code);
    core::hint::unreachable_unchecked()
}

pub unsafe fn yield_now() {
    raw::syscall0(SysOpcode::ThreadYield);
}

pub unsafe fn set_tls(ptr: u64) {
    core::arch::asm!(
        "wrfsbase {}",
        in(reg) ptr,
        options(nostack, preserves_flags)
    );
}

pub unsafe fn get_tls() -> u64 {
    let ptr: u64;
    core::arch::asm!(
        "rdfsbase {}",
        out(reg) ptr,
        options(nostack, preserves_flags)
    );
    ptr
}