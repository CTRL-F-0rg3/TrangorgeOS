use crate::protocols::*;

fn calculate_ipv4_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for chunk in header.chunks(2) {
        let word = if chunk.len() == 2 {
            ((chunk[0] as u32) << 8) | (chunk[1] as u32)
        } else {
            (chunk[0] as u32) << 8
        };
        sum = sum.wrapping_add(word);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

pub struct UdpPacketBuilder<'a> {
    pub src_mac: MacAddress,
    pub dst_mac: MacAddress,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: &'a [u8],
}

impl<'a> UdpPacketBuilder<'a> {
    pub fn build(self, buffer: &mut [u8]) -> Result<usize, ()> {
        let udp_len = (core::mem::size_of::<UdpHeader>() + self.payload.len()) as u16;
        let ip_len = (core::mem::size_of::<Ipv4Header>() as u16) + udp_len;
        let total_len = core::mem::size_of::<EthernetHeader>() + (ip_len as usize);

        if buffer.len() < total_len {
            return Err(()); 
        }

        
        
        let eth = EthernetHeader {
            dst_mac: self.dst_mac,
            src_mac: self.src_mac,
            ethertype: 0x0800u16.to_be(), 
        };

      
        let mut ip = Ipv4Header {
            version_ihl: 0x45, 
            tos: 0,
            total_length: ip_len.to_be(),
            identification: 0x1234u16.to_be(),
            flags_fragment: 0x4000u16.to_be(), 
            ttl: 64,
            protocol: 17, 
            checksum: 0,
            src_ip: self.src_ip,
            dst_ip: self.dst_ip,
        };

        
        let ip_bytes: &[u8; 20] = unsafe { core::mem::transmute(&ip) };
        ip.checksum = calculate_ipv4_checksum(ip_bytes).to_be();

        
        let udp = UdpHeader {
            src_port: self.src_port.to_be(),
            dst_port: self.dst_port.to_be(),
            length: udp_len.to_be(),
            checksum: 0, 
        };

       
        let eth_size = core::mem::size_of::<EthernetHeader>();
        let ip_size = core::mem::size_of::<Ipv4Header>();
        let udp_size = core::mem::size_of::<UdpHeader>();

        unsafe {
            core::ptr::copy_nonoverlapping(&eth as *const _ as *const u8, buffer.as_mut_ptr(), eth_size);
            core::ptr::copy_nonoverlapping(&ip as *const _ as *const u8, buffer.as_mut_ptr().add(eth_size), ip_size);
            core::ptr::copy_nonoverlapping(&udp as *const _ as *const u8, buffer.as_mut_ptr().add(eth_size + ip_size), udp_size);
        }

        let payload_offset = eth_size + ip_size + udp_size;
        buffer[payload_offset..payload_offset + self.payload.len()].copy_from_slice(self.payload);

        Ok(total_len)
    }
}