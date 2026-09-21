use super::sysnums;
use super::errno;
use tg_std_pal::raw;

pub unsafe fn handle_linux_trap(
    linux_num: u32, 
    a1: u64, 
    a2: u64, 
    a3: u64, 
    a4: u64, 
    a5: u64, 
    a6: u64
) -> u64 {
    let native_op = match sysnums::translate(linux_num) {
        Some(op) => op,
        None => return errno::to_linux_errno(6) as u64, 
    };

    let tg_ret = raw::syscall6(native_op, a1, a2, a3, a4, a5, a6);

    if tg_ret > 0xFFFFFFFFFFFFF000 {
        errno::to_linux_errno(tg_ret as u32) as u64 | 0xFFFFFFFFFFFFF000
    } else {
        tg_ret
    }
}