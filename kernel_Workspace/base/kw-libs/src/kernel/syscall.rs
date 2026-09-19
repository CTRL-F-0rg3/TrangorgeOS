// Low-level syscall trigger. 
// Used by userspace libraries (trangorgelibc) or driver-space daemons to talk to the kernel.
// In kernel-space, you would call kw_base::api::dispatch directly.

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn syscall0(num: u64) -> i64 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") num,
            out("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn syscall3(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            out("rax") ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}