use crate::core::{KResult, KernelError};

extern "C" {
    fn k_net_socket(domain: u32, sock_type: u32, proto: u32) -> i32;
    fn k_net_bind(fd: i32, addr: *const u8, len: u32) -> i32;
    fn k_net_send(fd: i32, buf: *const u8, len: u32, flags: u32) -> i32;
    fn k_net_recv(fd: i32, buf: *mut u8, len: u32, flags: u32) -> i32;
}

pub fn socket(domain: u32, sock_type: u32, proto: u32) -> KResult<u32> {
    let rc = unsafe { k_net_socket(domain, sock_type, proto) };
    if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as u32) }
}

pub fn send(fd: u32, buf: *const u8, len: u32, flags: u32) -> KResult<usize> {
    if buf.is_null() { return Err(KernelError::BadAddress); }
    let rc = unsafe { k_net_send(fd as i32, buf, len, flags) };
    if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as usize) }
}

pub fn recv(fd: u32, buf: *mut u8, len: u32, flags: u32) -> KResult<usize> {
    if buf.is_null() { return Err(KernelError::BadAddress); }
    let rc = unsafe { k_net_recv(fd as i32, buf, len, flags) };
    if rc < 0 { Err(KernelError::from_code(rc)) } else { Ok(rc as usize) }
}