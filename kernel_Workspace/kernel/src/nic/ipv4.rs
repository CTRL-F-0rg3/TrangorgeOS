use crate::nic::{checksum, error::PacketError, types::Ipv4Address};

pub const MIN_HEADER_LEN: usize = 20;
pub const PROTOCOL_ICMP: u8 = 1;
pub const PROTOCOL_TCP: u8 = 6;
pub const PROTOCOL_UDP: u8 = 17;

#[derive(Debug, Clone, Copy)]
pub struct Ipv4Packet<'a> {
    pub protocol: u8,
    pub source: Ipv4Address,
    pub destination: Ipv4Address,
    pub ttl: u8,
    pub identification: u16,
    pub payload: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
        if bytes.len() < MIN_HEADER_LEN {
            return Err(PacketError::Truncated);
        }
        if bytes[0] >> 4 != 4 {
            return Err(PacketError::InvalidIpv4Version);
        }
        let header_len = ((bytes[0] & 0x0f) as usize) * 4;
        if header_len < MIN_HEADER_LEN || header_len > bytes.len() {
            return Err(PacketError::InvalidIpv4HeaderLength);
        }
        let total_len = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
        if total_len < header_len || total_len > bytes.len() {
            return Err(PacketError::InvalidIpv4Length);
        }
        if !checksum::is_valid(&bytes[..header_len]) {
            return Err(PacketError::InvalidIpv4Checksum);
        }
        let flags_and_offset = u16::from_be_bytes([bytes[6], bytes[7]]);
        if flags_and_offset & 0x3fff != 0 {
            return Err(PacketError::FragmentedIpv4);
        }
        if bytes[8] == 0 {
            return Err(PacketError::InvalidIpv4Length);
        }
        Ok(Self {
            protocol: bytes[9],
            source: Ipv4Address(
                bytes[12..16]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            destination: Ipv4Address(
                bytes[16..20]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            ttl: bytes[8],
            identification: u16::from_be_bytes([bytes[4], bytes[5]]),
            payload: &bytes[header_len..total_len],
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ipv4Header {
    pub protocol: u8,
    pub source: Ipv4Address,
    pub destination: Ipv4Address,
    pub identification: u16,
    pub ttl: u8,
}

impl Ipv4Header {
    pub fn write<'a>(
        &self,
        out: &'a mut [u8],
        payload_len: usize,
    ) -> Result<&'a mut [u8], PacketError> {
        let total_len = MIN_HEADER_LEN
            .checked_add(payload_len)
            .filter(|len| *len <= u16::MAX as usize)
            .ok_or(PacketError::InvalidIpv4Length)?;
        if out.len() < total_len {
            return Err(PacketError::Truncated);
        }
        out[..MIN_HEADER_LEN].fill(0);
        out[0] = 0x45;
        out[2..4].copy_from_slice(&(total_len as u16).to_be_bytes());
        out[4..6].copy_from_slice(&self.identification.to_be_bytes());
        out[6..8].copy_from_slice(&0x4000u16.to_be_bytes()); // Don't Fragment.
        out[8] = self.ttl;
        out[9] = self.protocol;
        out[12..16].copy_from_slice(&self.source.0);
        out[16..20].copy_from_slice(&self.destination.0);
        let header_checksum = checksum::checksum(&out[..MIN_HEADER_LEN]);
        out[10..12].copy_from_slice(&header_checksum.to_be_bytes());
        Ok(&mut out[MIN_HEADER_LEN..total_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_build_then_parse() {
        let header = Ipv4Header {
            protocol: PROTOCOL_ICMP,
            source: Ipv4Address::new(10, 0, 0, 2),
            destination: Ipv4Address::new(10, 0, 0, 1),
            identification: 7,
            ttl: 64,
        };
        let mut bytes = [0u8; 64];
        let payload = header.write(&mut bytes, 4).unwrap();
        payload.copy_from_slice(&[1, 2, 3, 4]);
        let parsed = Ipv4Packet::parse(&bytes[..24]).unwrap();
        assert_eq!(parsed.protocol, PROTOCOL_ICMP);
        assert_eq!(parsed.payload, &[1, 2, 3, 4]);
    }
}
