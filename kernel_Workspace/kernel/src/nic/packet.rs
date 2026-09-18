use crate::nic::{
    stack::{NetworkConfig, NetworkStack, PingRequest},
    types::{Ipv4Address, MacAddress},
};


#[derive(Debug, Clone, Copy)]
pub struct IcmpEchoRequest<'a> {
    pub local_mac: MacAddress,
    pub local_ip: Ipv4Address,
    pub next_hop_mac: MacAddress,
    pub destination: Ipv4Address,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: &'a [u8],
}


pub fn build_icmp_echo(
    out: &mut [u8],
    request: IcmpEchoRequest<'_>,
) -> Result<usize, crate::nic::error::PacketError> {
    let mut stack = NetworkStack::<1>::new(NetworkConfig {
        ipv4: request.local_ip,
        netmask: Ipv4Address::new(255, 255, 255, 0),
        gateway: request.local_ip,
        ttl: 64,
        arp_ttl_ms: 30_000,
    });
    stack.build_ping(
        out,
        request.local_mac,
        PingRequest {
            next_hop_mac: request.next_hop_mac,
            destination: request.destination,
            identifier: request.identifier,
            sequence: request.sequence,
            payload: request.payload,
        },
    )
}

pub fn self_test() -> Result<&'static str, &'static str> {
    let mut frame = [0u8; 128];
    let len = build_icmp_echo(
        &mut frame,
        IcmpEchoRequest {
            local_mac: MacAddress([0x02, 0, 0, 0, 0, 1]),
            local_ip: Ipv4Address::new(10, 0, 0, 2),
            next_hop_mac: MacAddress([0x02, 0, 0, 0, 0, 254]),
            destination: Ipv4Address::new(10, 0, 0, 1),
            identifier: 0x1234,
            sequence: 1,
            payload: b"ping",
        },
    )
    .map_err(|_| "icmp echo build failed")?;
    if len < 60 || frame[12..14] != [0x08, 0x00] || frame[23] != 1 || frame[34] != 8 {
        return Err("unexpected ICMP Ethernet frame");
    }
    Ok("ICMP Echo frame build verified")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_passes() {
        assert!(self_test().is_ok());
    }
}
