use crate::nic::{checksum, error::PacketError};

pub const TYPE_ECHO_REPLY: u8 = 0;
pub const TYPE_ECHO_REQUEST: u8 = 8;
pub const ECHO_HEADER_LEN: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct IcmpPacket<'a> {
    pub icmp_type: u8,
    pub code: u8,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: &'a [u8],
}

impl<'a> IcmpPacket<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
        if bytes.len() < ECHO_HEADER_LEN {
            return Err(PacketError::Truncated);
        }
        if bytes[1] != 0 || (bytes[0] != TYPE_ECHO_REQUEST && bytes[0] != TYPE_ECHO_REPLY) {
            return Err(PacketError::InvalidIcmp);
        }
        if !checksum::is_valid(bytes) {
            return Err(PacketError::InvalidIcmpChecksum);
        }
        Ok(Self {
            icmp_type: bytes[0],
            code: bytes[1],
            identifier: u16::from_be_bytes([bytes[4], bytes[5]]),
            sequence: u16::from_be_bytes([bytes[6], bytes[7]]),
            payload: &bytes[ECHO_HEADER_LEN..],
        })
    }

    #[inline]
    pub fn is_reply_for(&self, identifier: u16, sequence: u16) -> bool {
        self.icmp_type == TYPE_ECHO_REPLY
            && self.identifier == identifier
            && self.sequence == sequence
    }
}

pub fn write_echo_request(
    out: &mut [u8],
    identifier: u16,
    sequence: u16,
    payload: &[u8],
) -> Result<usize, PacketError> {
    let total_len = ECHO_HEADER_LEN
        .checked_add(payload.len())
        .ok_or(PacketError::InvalidIcmp)?;
    if out.len() < total_len {
        return Err(PacketError::Truncated);
    }
    out[..total_len].fill(0);
    out[0] = TYPE_ECHO_REQUEST;
    out[4..6].copy_from_slice(&identifier.to_be_bytes());
    out[6..8].copy_from_slice(&sequence.to_be_bytes());
    out[ECHO_HEADER_LEN..total_len].copy_from_slice(payload);
    let csum = checksum::checksum(&out[..total_len]);
    out[2..4].copy_from_slice(&csum.to_be_bytes());
    Ok(total_len)
}

pub fn write_echo_reply(out: &mut [u8], request: IcmpPacket<'_>) -> Result<usize, PacketError> {
    if request.icmp_type != TYPE_ECHO_REQUEST {
        return Err(PacketError::InvalidIcmp);
    }
    let len = write_echo_request(out, request.identifier, request.sequence, request.payload)?;
    out[0] = TYPE_ECHO_REPLY;
    out[2..4].fill(0);
    let csum = checksum::checksum(&out[..len]);
    out[2..4].copy_from_slice(&csum.to_be_bytes());
    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_request_and_reply_keep_identity_and_payload() {
        let mut request_bytes = [0u8; 32];
        let len = write_echo_request(&mut request_bytes, 0x1234, 9, b"ping").unwrap();
        let request = IcmpPacket::parse(&request_bytes[..len]).unwrap();
        assert_eq!(request.identifier, 0x1234);
        assert_eq!(request.payload, b"ping");

        let mut reply_bytes = [0u8; 32];
        let reply_len = write_echo_reply(&mut reply_bytes, request).unwrap();
        let reply = IcmpPacket::parse(&reply_bytes[..reply_len]).unwrap();
        assert!(reply.is_reply_for(0x1234, 9));
        assert_eq!(reply.payload, b"ping");
    }
}
