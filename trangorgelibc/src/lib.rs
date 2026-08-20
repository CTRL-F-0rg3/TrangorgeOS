
#![no_std]

const SYS_YIELD: u64 = 1;
const SYS_LOG: u64 = 2;
const SYS_EXIT: u64 = 4;

fn syscall2(num: u64, a0: u64) -> u64 {
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") a0,
            lateout("rax") ret,
        );
    }
    ret
}

pub fn log(s: &str) {
    syscall2(SYS_LOG, s.as_ptr() as u64);
}

pub fn yield_cpu() {
    syscall2(SYS_YIELD, 0);
}

pub fn exit(code: i32) -> ! {
    syscall2(SYS_EXIT, code as u64);
    loop {}
}