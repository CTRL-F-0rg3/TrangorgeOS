

#![no_std]

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MacAdress(pub [u8; 6]);

impl MacAdress{
    pub const BROADCAST: Self = MacAdress([0xFF; 6]);

}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EthernetHeader {
    pub dst_mac: MacAdress,
    pub src_mac: MacAdress,
    pub ethertype: u16, 
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ipv4Header {
    pub version_ihl: u8,       
    pub tos: u8,               
    pub total_length: u16,      
    pub identification: u16,  
    pub flags_fragment: u16,   
    pub ttl: u8,               
    pub protocol: u8,          
    pub checksum: u16,          
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

