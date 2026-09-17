//! Logowanie z driver space do jądra.
//!
//! Jądro przyjmuje komunikat jako C-string (wskaźnik w `arg0`, długość w `arg1`),
//! więc literały tekstowe z Rusta są tu w porządku — mają bajt NUL na końcu.

use crate::abi::DsCmd;
use crate::runtime::request;

/// Wysyła tekst do logu jądra (komenda `DsCmd::Log`).
pub fn ds_log(msg: &str) {
    let _ = request(DsCmd::Log, msg.as_ptr() as u64, msg.len() as u64, 0);
}

/// To samo, ale dla surowego wskaźnika i długości (np. bufora bajtów).
pub fn ds_log_raw(ptr: *const u8, len: usize) {
    let _ = request(DsCmd::Log, ptr as u64, len as u64, 0);
}
