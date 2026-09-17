//! Transport layer: sends formatted logs via IPC or UART fallback.

use crate::formatter::LogBuffer;
use crate::levels::LogLevel;
use kapi_abi::opcodes::Opcode;
use kapi_abi::payloads::sys::LogPayload;

// Fallback UART port (x86_64 COM1)
const UART_PORT: u16 = 0x3F8;

#[inline(always)]
fn uart_putchar(c: u8) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") UART_PORT, in("al") c, options(nostack, preserves_flags));
    }
}

pub fn emit_log(level: LogLevel, module: &str, buf: &LogBuffer) {
    // 1. Fast fallback: UART (crucial for early boot before IPC is up)
    for &b in b"[" { uart_putchar(b); }
    for &b in level.as_str().as_bytes() { uart_putchar(b); }
    for &b in b"] " { uart_putchar(b); }
    for &b in module.as_bytes() { uart_putchar(b); }
    for &b in b": " { uart_putchar(b); }
    for &b in buf.as_bytes() { uart_putchar(b); }
    uart_putchar(b'\n');

    // 2. IPC Transport (if Manager is ready)
    let manager_ep = &crate::MANAGER_ENDPOINT;
    if manager_ep.is_valid() {
        let payload = LogPayload {
            level: level as u8,
            module_len: module.len() as u16,
            msg_len: buf.as_bytes().len() as u16,
            _pad: 0,
        };
        
        // Note: In a real impl, we'd copy module and msg into a shared IPC buffer.
        // Here we demonstrate the syscall trigger.
        let _ = kapi_syscall::sys_ipc_send(
            manager_ep.handle(),
            Opcode::SysLog as u32,
            &payload as *const _ as *const u8,
            core::mem::size_of::<LogPayload>() as u32,
        );
    }
}