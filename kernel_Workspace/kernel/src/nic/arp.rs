use crate::nic::{
    error::PacketError,
    types::{Ipv4Address, MacAddress},
};

pub const PACKET_LEN: usize = 28;
pub const HARDWARE_ETHERNET: u16 = 1;
pub const PROTOCOL_IPV4: u16 = 0x0800;
pub const OPERATION_REQUEST: u16 = 1;
pub const OPERATION_REPLY: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArpPacket {
    pub operation: u16,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
        if bytes.len() < PACKET_LEN {
            return Err(PacketError::Truncated);
        }
        if u16::from_be_bytes([bytes[0], bytes[1]]) != HARDWARE_ETHERNET
            || u16::from_be_bytes([bytes[2], bytes[3]]) != PROTOCOL_IPV4
            || bytes[4] != 6
            || bytes[5] != 4
        {
            return Err(PacketError::InvalidArp);
        }
        let operation = u16::from_be_bytes([bytes[6], bytes[7]]);
        if operation != OPERATION_REQUEST && operation != OPERATION_REPLY {
            return Err(PacketError::InvalidArp);
        }
        Ok(Self {
            operation,
            sender_mac: MacAddress(
                bytes[8..14]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            sender_ip: Ipv4Address(
                bytes[14..18]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            target_mac: MacAddress(
                bytes[18..24]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            target_ip: Ipv4Address(
                bytes[24..28]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
        })
    }

    pub fn write_to(&self, out: &mut [u8]) -> Result<usize, PacketError> {
        if out.len() < PACKET_LEN {
            return Err(PacketError::Truncated);
        }
        out[0..2].copy_from_slice(&HARDWARE_ETHERNET.to_be_bytes());
        out[2..4].copy_from_slice(&PROTOCOL_IPV4.to_be_bytes());
        out[4] = 6;
        out[5] = 4;
        out[6..8].copy_from_slice(&self.operation.to_be_bytes());
        out[8..14].copy_from_slice(&self.sender_mac.0);
        out[14..18].copy_from_slice(&self.sender_ip.0);
        out[18..24].copy_from_slice(&self.target_mac.0);
        out[24..28].copy_from_slice(&self.target_ip.0);
        Ok(PACKET_LEN)
    }

    #[inline]
    pub const fn request(
        local_mac: MacAddress,
        local_ip: Ipv4Address,
        target_ip: Ipv4Address,
    ) -> Self {
        Self {
            operation: OPERATION_REQUEST,
            sender_mac: local_mac,
            sender_ip: local_ip,
            target_mac: MacAddress::ZERO,
            target_ip,
        }
    }

    #[inline]
    pub const fn reply(local_mac: MacAddress, local_ip: Ipv4Address, request: Self) -> Self {
        Self {
            operation: OPERATION_REPLY,
            sender_mac: local_mac,
            sender_ip: local_ip,
            target_mac: request.sender_mac,
            target_ip: request.sender_ip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CacheEntry {
    ip: Ipv4Address,
    mac: MacAddress,
    expires_at_ms: u64,
}

pub struct ArpCache<const N: usize> {
    entries: [Option<CacheEntry>; N],
}

impl<const N: usize> ArpCache<N> {
    pub const fn new() -> Self {
        Self { entries: [None; N] }
    }

    pub fn lookup(&mut self, ip: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
        for slot in &mut self.entries {
            if let Some(entry) = *slot {
                if entry.expires_at_ms <= now_ms {
                    *slot = None;
                } else if entry.ip == ip {
                    return Some(entry.mac);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress, expires_at_ms: u64) {
        let mut replace = 0usize;
        let mut oldest = u64::MAX;
        for (index, slot) in self.entries.iter_mut().enumerate() {
            match slot {
                Some(entry) if entry.ip == ip => {
                    *entry = CacheEntry {
                        ip,
                        mac,
                        expires_at_ms,
                    };
                    return;
                }
                None => {
                    *slot = Some(CacheEntry {
                        ip,
                        mac,
                        expires_at_ms,
                    });
                    return;
                }
                Some(entry) if entry.expires_at_ms < oldest => {
                    oldest = entry.expires_at_ms;
                    replace = index;
                }
                _ => {}
            }
        }
        if N != 0 {
            self.entries[replace] = Some(CacheEntry {
                ip,
                mac,
                expires_at_ms,
            });
        }
    }
}

impl<const N: usize> Default for ArpCache<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arp_round_trip_and_cache_expiry() {
        let local_mac = MacAddress([2, 0, 0, 0, 0, 1]);
        let local_ip = Ipv4Address::new(10, 0, 0, 2);
        let mut bytes = [0u8; PACKET_LEN];
        let request = ArpPacket::request(local_mac, local_ip, Ipv4Address::new(10, 0, 0, 1));
        request.write_to(&mut bytes).unwrap();
        assert_eq!(ArpPacket::parse(&bytes).unwrap(), request);

        let mut cache = ArpCache::<2>::new();
        cache.insert(request.target_ip, MacAddress([0, 1, 2, 3, 4, 5]), 100);
        assert!(cache.lookup(request.target_ip, 99).is_some());
        assert!(cache.lookup(request.target_ip, 100).is_none());
    }
}
