#![no_std]

use crate::formatter::LogBuffer;
use crate::levels::LogLevel;
use kapi_abi::{DsCmd, DsMsg};
use kapi_syscall::sys_ipc_call;

fn uart_putchar(c: u8) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3F8u16, in("al") c, options(nomem, nostack, preserves_flags));
    }
}

pub fn emit_log(level: LogLevel, module: &str, buf: &LogBuffer) {
    let bytes = buf.as_bytes();
    
    for &b in bytes {
        uart_putchar(b);
    }
    uart_putchar(b'\n');
    
    let _ = sys_ipc_call(
        DsCmd::Log,
        bytes.as_ptr() as u64,
        bytes.len() as u64,
        level as u64,
    );
}