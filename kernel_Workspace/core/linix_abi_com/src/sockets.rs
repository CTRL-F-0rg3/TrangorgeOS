use core::net::{Ipv4Addr, Ipv6Addr};

// Address families
pub const AF_UNIX: u16 = 1;
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 10;

// Socket types
pub const SOCK_STREAM: u16 = 1;
pub const SOCK_DGRAM: u16 = 2;
pub const SOCK_RAW: u16 = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrIn {
    pub sin_family: u16,
    pub sin_port: u16, // Network byte order
    pub sin_addr: Ipv4Addr,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockAddrIn6 {
    pub sin6_family: u16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: Ipv6Addr,
    pub sin6_scope_id: u32,
}

// Translate Linux socket type to native TrangorgeOS IPC/Net channel kind
#[inline]
pub fn map_socket_type(family: u16, sock_type: u16) -> Option<u8> {
    match (family, sock_type) {
        (AF_INET, SOCK_STREAM) => Some(0x01), // Native TCP Channel
        (AF_INET, SOCK_DGRAM) => Some(0x02),  // Native UDP Port
        (AF_UNIX, SOCK_STREAM) => Some(0x10), // Native Local Pipe/Channel
        _ => None,
    }
}