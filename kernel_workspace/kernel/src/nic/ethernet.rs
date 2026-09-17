use crate::nic::{error::PacketError, types::MacAddress};

pub const HEADER_LEN: usize = 14;
pub const MIN_FRAME_NO_FCS: usize = 60;
pub const DEFAULT_MTU: usize = 1500;

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthernetHeader {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ether_type: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct EthernetFrame<'a> {
    pub header: EthernetHeader,
    pub payload: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
        if bytes.len() < HEADER_LEN {
            return Err(PacketError::Truncated);
        }

        let header = EthernetHeader {
            destination: MacAddress(bytes[0..6].try_into().map_err(|_| PacketError::Truncated)?),
            source: MacAddress(
                bytes[6..12]
                    .try_into()
                    .map_err(|_| PacketError::Truncated)?,
            ),
            ether_type: u16::from_be_bytes([bytes[12], bytes[13]]),
        };

        Ok(Self {
            header,
            payload: &bytes[HEADER_LEN..],
        })
    }

    #[inline]
    pub fn is_for(&self, local: MacAddress) -> bool {
        self.header.destination == local || self.header.destination.is_broadcast()
    }
}

pub fn build(
    out: &mut [u8],
    destination: MacAddress,
    source: MacAddress,
    ether_type: u16,
) -> Result<&mut [u8], PacketError> {
    if out.len() < HEADER_LEN {
        return Err(PacketError::Truncated);
    }
    out[0..6].copy_from_slice(&destination.0);
    out[6..12].copy_from_slice(&source.0);
    out[12..14].copy_from_slice(&ether_type.to_be_bytes());
    Ok(&mut out[HEADER_LEN..])
}

pub fn pad_to_minimum(frame: &mut [u8], logical_len: usize) -> Result<usize, PacketError> {
    if logical_len < HEADER_LEN || logical_len > frame.len() {
        return Err(PacketError::Truncated);
    }
    let wire_len = logical_len.max(MIN_FRAME_NO_FCS);
    if wire_len > frame.len() {
        return Err(PacketError::Truncated);
    }
    frame[logical_len..wire_len].fill(0);
    Ok(wire_len)
}

pub fn self_test() -> Result<&'static str, &'static str> {
    let source = MacAddress([0x02, 0, 0, 0, 0, 1]);
    let mut bytes = [0u8; MIN_FRAME_NO_FCS];
    let payload = build(&mut bytes, MacAddress::BROADCAST, source, ETHERTYPE_ARP)
        .map_err(|_| "ethernet build failed")?;
    payload[..3].copy_from_slice(&[1, 2, 3]);
    let len = pad_to_minimum(&mut bytes, HEADER_LEN + 3).map_err(|_| "ethernet padding failed")?;
    let frame = EthernetFrame::parse(&bytes[..len]).map_err(|_| "ethernet parse failed")?;
    if frame.header.source != source || frame.header.ether_type != ETHERTYPE_ARP {
        return Err("ethernet header mismatch");
    }
    Ok("ethernet build/parse/padding verified")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethernet_round_trip_and_padding() {
        let src = MacAddress([2, 0, 0, 0, 0, 1]);
        let mut bytes = [0u8; MIN_FRAME_NO_FCS];
        let payload = build(&mut bytes, MacAddress::BROADCAST, src, ETHERTYPE_ARP).unwrap();
        payload[..3].copy_from_slice(&[1, 2, 3]);
        let len = pad_to_minimum(&mut bytes, HEADER_LEN + 3).unwrap();
        assert_eq!(len, MIN_FRAME_NO_FCS);
        let parsed = EthernetFrame::parse(&bytes[..len]).unwrap();
        assert_eq!(parsed.header.source, src);
        assert_eq!(parsed.header.ether_type, ETHERTYPE_ARP);
    }
}
