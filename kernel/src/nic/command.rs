use crate::nic::device::NetworkDevice;
use crate::nic::error::NetworkError;
use crate::nic::ping::{PingClient, PingResult};
use crate::nic::stack::NetworkConfig;
use crate::nic::types::{Ipv4Address, MacAddress};

pub struct NetworkCommandRunner<const ARP_ENTRIES: usize> {
    ping: PingClient<ARP_ENTRIES>,
    identifier: u16,
    next_sequence: u16,
    active: bool,
}

impl<const ARP_ENTRIES: usize> NetworkCommandRunner<ARP_ENTRIES> {
    pub const fn new(config: NetworkConfig, local_mac: MacAddress, identifier: u16) -> Self {
        Self {
            ping: PingClient::new(config, local_mac),
            identifier,
            next_sequence: 1,
            active: false,
        }
    }

    pub fn start_ping(
        &mut self,
        device: &mut dyn NetworkDevice,
        now_ms: u64,
        destination: Ipv4Address,
    ) -> Result<PingResult, NetworkError> {
        if self.active {
            return Err(NetworkError::QueueFull);
        }
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        if self.next_sequence == 0 {
            self.next_sequence = 1;
        }
        let result = self.ping.start(
            device,
            destination,
            self.identifier,
            sequence,
            now_ms,
        )?;
        self.active = true;
        Ok(result)
    }

    pub fn poll(
        &mut self,
        device: &mut dyn NetworkDevice,
        now_ms: u64,
    ) -> Result<Option<PingResult>, NetworkError> {
        if !self.active {
            return Ok(None);
        }
        let result = self.ping.poll(device, now_ms)?;
        if matches!(result, PingResult::EchoReply { .. }) {
            self.active = false;
            return Ok(Some(result));
        }
        if matches!(result, PingResult::Waiting) {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

pub fn parse_ipv4(input: &str) -> Option<Ipv4Address> {
    let mut octets = [0u8; 4];
    let mut count = 0usize;
    for part in input.split('.') {
        if count == octets.len() {
            return None;
        }
        octets[count] = parse_octet(part)?;
        count += 1;
    }
    if count != octets.len() {
        return None;
    }
    Some(Ipv4Address(octets))
}

fn parse_octet(input: &str) -> Option<u8> {
    if input.is_empty() || input.len() > 3 {
        return None;
    }
    let mut value = 0u16;
    for byte in input.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add((byte - b'0') as u16)?;
        if value > 255 {
            return None;
        }
    }
    Some(value as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ipv4() {
        assert_eq!(parse_ipv4("8.8.8.8"), Some(Ipv4Address::new(8, 8, 8, 8)));
    }

    #[test]
    fn rejects_invalid_ipv4() {
        assert_eq!(parse_ipv4("8.8.8.256"), None);
        assert_eq!(parse_ipv4("8.8.8"), None);
    }
}
