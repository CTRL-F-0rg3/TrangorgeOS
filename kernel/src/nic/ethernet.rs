use crate::nic::protocols::{EthernetHeader, MacAddress};

pub const MIN_FRAME_LEN: usize = 60;
pub const MAX_FRAME_LEN: usize = 1518;
pub const HEADER_LEN: usize = core::mem::size_of::<EthernetHeader>();

crate::test_module!({
    let mut buffer = [0u8; HEADER_LEN + 4];
    let dst = MacAddress::BROADCAST;
    let src = MacAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let written = match build_header(&mut buffer, dst, src, 0x0800) {
        Some(len) => len,
        None => return Err("build_header failed on a buffer large enough for the header"),
    };
    if written != HEADER_LEN {
        return Err("build_header returned an unexpected header length");
    }

    let (parsed, rest) = match parse_header(&buffer) {
        Some(result) => result,
        None => return Err("parse_header failed on a buffer it should be able to parse"),
    };
    if parsed.dst_mac != dst || parsed.src_mac != src {
        return Err("parsed header MAC addresses do not match what was written");
    }
    if u16::from_be(parsed.ethertype) != 0x0800 {
        return Err("parsed ethertype does not match what was written");
    }
    if rest.len() != buffer.len() - HEADER_LEN {
        return Err("parse_header returned the wrong remaining slice length");
    }

    Ok("ethernet header build/parse round trip verified")
});

pub fn parse_header(frame: &[u8]) -> Option<(EthernetHeader, &[u8])> {
    if frame.len() < HEADER_LEN {
        return None;
    }
    let header = unsafe { core::ptr::read_unaligned(frame.as_ptr() as *const EthernetHeader) };
    Some((header, &frame[HEADER_LEN..]))
}

pub fn build_header(
    buffer: &mut [u8],
    dst: MacAddress,
    src: MacAddress,
    ethertype: u16,
) -> Option<usize> {
    if buffer.len() < HEADER_LEN {
        return None;
    }
    let header = EthernetHeader {
        dst_mac: dst,
        src_mac: src,
        ethertype: ethertype.to_be(),
    };
    unsafe {
        core::ptr::write_unaligned(buffer.as_mut_ptr() as *mut EthernetHeader, header);
    }
    Some(HEADER_LEN)
}
