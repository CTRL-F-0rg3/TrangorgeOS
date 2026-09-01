use crate::nic::{
    arp::{ArpCache, ArpPacket, OPERATION_REPLY, OPERATION_REQUEST},
    error::PacketError,
    ethernet::{self, EthernetFrame, ETHERTYPE_ARP, ETHERTYPE_IPV4},
    icmp::{self, IcmpPacket},
    ipv4::{Ipv4Header, Ipv4Packet, PROTOCOL_ICMP},
    types::{Ipv4Address, MacAddress},
};

#[derive(Debug, Clone, Copy)]
pub struct NetworkConfig {
    pub ipv4: Ipv4Address,
    pub netmask: Ipv4Address,
    pub gateway: Ipv4Address,
    pub ttl: u8,
    pub arp_ttl_ms: u64,
}

impl NetworkConfig {
    #[inline]
    pub const fn next_hop(&self, destination: Ipv4Address) -> Ipv4Address {
        if self.ipv4.is_in_subnet(destination, self.netmask) {
            destination
        } else {
            self.gateway
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackEvent {
    Ignored,
    ArpResolved {
        ip: Ipv4Address,
        mac: MacAddress,
    },
    ArpRequestForUs {
        sender_ip: Ipv4Address,
        sender_mac: MacAddress,
    },
    EchoReply {
        source: Ipv4Address,
        identifier: u16,
        sequence: u16,
    },
    EchoRequestForUs {
        source: Ipv4Address,
        identifier: u16,
        sequence: u16,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct PingRequest<'a> {
    pub next_hop_mac: MacAddress,
    pub destination: Ipv4Address,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: &'a [u8],
}

pub struct NetworkStack<const ARP_ENTRIES: usize> {
    config: NetworkConfig,
    arp: ArpCache<ARP_ENTRIES>,
    next_ip_identification: u16,
}

impl<const ARP_ENTRIES: usize> NetworkStack<ARP_ENTRIES> {
    pub const fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            arp: ArpCache::new(),
            next_ip_identification: 1,
        }
    }

    #[inline]
    pub const fn config(&self) -> NetworkConfig {
        self.config
    }

    #[inline]
    pub fn next_hop_mac(&mut self, destination: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
        self.arp.lookup(self.config.next_hop(destination), now_ms)
    }

    pub fn build_arp_request(
        &self,
        out: &mut [u8],
        local_mac: MacAddress,
        destination: Ipv4Address,
    ) -> Result<usize, PacketError> {
        let payload = ethernet::build(out, MacAddress::BROADCAST, local_mac, ETHERTYPE_ARP)?;
        let arp = ArpPacket::request(
            local_mac,
            self.config.ipv4,
            self.config.next_hop(destination),
        );
        let arp_len = arp.write_to(payload)?;
        ethernet::pad_to_minimum(out, ethernet::HEADER_LEN + arp_len)
    }

    pub fn build_ping(
        &mut self,
        out: &mut [u8],
        local_mac: MacAddress,
        request: PingRequest<'_>,
    ) -> Result<usize, PacketError> {
        let eth_payload = ethernet::build(out, request.next_hop_mac, local_mac, ETHERTYPE_IPV4)?;
        let icmp_len = icmp::ECHO_HEADER_LEN
            .checked_add(request.payload.len())
            .ok_or(PacketError::InvalidIcmp)?;
        let ip = Ipv4Header {
            protocol: PROTOCOL_ICMP,
            source: self.config.ipv4,
            destination: request.destination,
            identification: self.next_ip_identification,
            ttl: self.config.ttl,
        };
        self.next_ip_identification = self.next_ip_identification.wrapping_add(1);
        let icmp_bytes = ip.write(eth_payload, icmp_len)?;
        icmp::write_echo_request(
            icmp_bytes,
            request.identifier,
            request.sequence,
            request.payload,
        )?;
        let logical_len = ethernet::HEADER_LEN + 20 + icmp_len;
        ethernet::pad_to_minimum(out, logical_len)
    }

    pub fn process_rx(
        &mut self,
        local_mac: MacAddress,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<StackEvent, PacketError> {
        let frame = EthernetFrame::parse(bytes)?;
        if !frame.is_for(local_mac) {
            return Ok(StackEvent::Ignored);
        }
        match frame.header.ether_type {
            ETHERTYPE_ARP => self.process_arp(frame, local_mac, now_ms),
            ETHERTYPE_IPV4 => self.process_ipv4(frame),
            _ => Ok(StackEvent::Ignored),
        }
    }

    fn process_arp(
        &mut self,
        frame: EthernetFrame<'_>,
        local_mac: MacAddress,
        now_ms: u64,
    ) -> Result<StackEvent, PacketError> {
        let arp = ArpPacket::parse(frame.payload)?;
        if arp.sender_mac != frame.header.source || arp.sender_ip.is_unspecified() {
            return Err(PacketError::InvalidArp);
        }
        self.arp.insert(
            arp.sender_ip,
            arp.sender_mac,
            now_ms.saturating_add(self.config.arp_ttl_ms),
        );

        match arp.operation {
            OPERATION_REPLY if arp.target_ip == self.config.ipv4 && arp.target_mac == local_mac => {
                Ok(StackEvent::ArpResolved {
                    ip: arp.sender_ip,
                    mac: arp.sender_mac,
                })
            }
            OPERATION_REQUEST if arp.target_ip == self.config.ipv4 => {
                Ok(StackEvent::ArpRequestForUs {
                    sender_ip: arp.sender_ip,
                    sender_mac: arp.sender_mac,
                })
            }
            _ => Ok(StackEvent::Ignored),
        }
    }

    fn process_ipv4(&mut self, frame: EthernetFrame<'_>) -> Result<StackEvent, PacketError> {
        let ip = Ipv4Packet::parse(frame.payload)?;
        if ip.destination != self.config.ipv4 {
            return Ok(StackEvent::Ignored);
        }
        if ip.protocol != PROTOCOL_ICMP {
            return Ok(StackEvent::Ignored);
        }
        let icmp = IcmpPacket::parse(ip.payload)?;
        match icmp.icmp_type {
            icmp::TYPE_ECHO_REPLY => Ok(StackEvent::EchoReply {
                source: ip.source,
                identifier: icmp.identifier,
                sequence: icmp.sequence,
            }),
            icmp::TYPE_ECHO_REQUEST => Ok(StackEvent::EchoRequestForUs {
                source: ip.source,
                identifier: icmp.identifier,
                sequence: icmp.sequence,
            }),
            _ => Ok(StackEvent::Ignored),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_MAC: MacAddress = MacAddress([2, 0, 0, 0, 0, 1]);
    const LOCAL_IP: Ipv4Address = Ipv4Address::new(10, 0, 0, 2);
    const GATEWAY: Ipv4Address = Ipv4Address::new(10, 0, 0, 1);

    fn stack() -> NetworkStack<4> {
        NetworkStack::new(NetworkConfig {
            ipv4: LOCAL_IP,
            netmask: Ipv4Address::new(255, 255, 255, 0),
            gateway: GATEWAY,
            ttl: 64,
            arp_ttl_ms: 30_000,
        })
    }

    #[test]
    fn outside_subnet_uses_gateway() {
        let stack = stack();
        assert_eq!(stack.config.next_hop(Ipv4Address::new(8, 8, 8, 8)), GATEWAY);
    }

    #[test]
    fn arp_request_and_ping_are_ethernet_padded() {
        let mut stack = stack();
        let mut frame = [0u8; 128];
        let arp_len = stack
            .build_arp_request(&mut frame, LOCAL_MAC, Ipv4Address::new(8, 8, 8, 8))
            .unwrap();
        assert_eq!(arp_len, ethernet::MIN_FRAME_NO_FCS);
        let ping_len = stack
            .build_ping(
                &mut frame,
                LOCAL_MAC,
                PingRequest {
                    next_hop_mac: MacAddress([0, 1, 2, 3, 4, 5]),
                    destination: Ipv4Address::new(8, 8, 8, 8),
                    identifier: 1,
                    sequence: 1,
                    payload: b"ok",
                },
            )
            .unwrap();
        assert_eq!(ping_len, ethernet::MIN_FRAME_NO_FCS);
    }
}
