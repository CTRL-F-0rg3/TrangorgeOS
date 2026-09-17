//! Kernel API syscall gateway (`kapi-syscall`).
//!
//! Cienka warstwa pomiędzy driver space a jądrem. Podobnie jak inne sterowniki
//! w tym repozytorium, funkcje tutaj wołają symbole `extern "C"` eksportowane
//! przez jądro (bridge ABI), zamiast wbudowywać konkretne ABI w kod sterownika.
//! Dzięki temu zmiana ABI jądra wymaga zmian tylko po stronie jądra.

#![no_std]

use kapi_abi::errors::DsError;

// (edycja 2024 wymaga `unsafe extern` dla bloków z deklaracjami FFI)
unsafe extern "C" {
    /// Wysyła jednokierunkową wiadomość IPC do `target`.
    /// Zwraca 0 przy sukcesie albo kod `DsError`.
    fn kapi_ipc_send(target: u32, opcode: u32, payload: *const u8, payload_len: u32) -> i32;

    /// Wysyła żądanie IPC i czeka na odpowiedź do bufora `reply`.
    /// Zwraca 0 przy sukcesie albo kod `DsError`.
    fn kapi_ipc_call(
        target: u32,
        opcode: u32,
        payload: *const u8,
        payload_len: u32,
        reply: *mut u8,
        reply_len: u32,
    ) -> i32;
}

#[inline]
fn status(rc: i32) -> Result<(), DsError> {
    if rc == 0 {
        Ok(())
    } else {
        Err(DsError::from_u32(rc as u32))
    }
}

/// Wysyła wiadomość IPC (bez oczekiwania na odpowiedź).
pub fn sys_ipc_send(
    target: u32,
    opcode: u32,
    payload: *const u8,
    payload_len: u32,
) -> Result<(), DsError> {
    status(unsafe { kapi_ipc_send(target, opcode, payload, payload_len) })
}

/// Wysyła żądanie IPC i odbiera odpowiedź.
pub fn sys_ipc_call(
    target: u32,
    opcode: u32,
    payload: *const u8,
    payload_len: u32,
    reply: *mut u8,
    reply_len: u32,
) -> Result<(), DsError> {
    status(unsafe {
        kapi_ipc_call(target, opcode, payload, payload_len, reply, reply_len)
    })
}
