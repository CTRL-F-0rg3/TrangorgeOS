use crate::nic::device::{NetworkDevice, TxFrame};
use crate::nic::error::{NetworkError, PacketError};
use crate::nic::stack::{NetworkConfig, NetworkStack, PingRequest, StackEvent};
use crate::nic::types::{Ipv4Address, MacAddress};

const FRAME_BYTES: usize = 1536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingResult {
    ArpRequestSent,
    EchoRequestSent,
    EchoReply { source: Ipv4Address, sequence: u16 },
    Waiting,
}

pub struct PingClient<const ARP_ENTRIES: usize> {
    stack: NetworkStack<ARP_ENTRIES>,
    local_mac: MacAddress,
    frame: [u8; FRAME_BYTES],
    destination: Option<Ipv4Address>,
    identifier: u16,
    sequence: u16,
}

impl<const ARP_ENTRIES: usize> PingClient<ARP_ENTRIES> {
    pub const fn new(config: NetworkConfig, local_mac: MacAddress) -> Self {
        Self {
            stack: NetworkStack::new(config),
            local_mac,
            frame: [0; FRAME_BYTES],
            destination: None,
            identifier: 0,
            sequence: 0,
        }
    }

    pub fn start(
        &mut self,
        device: &mut dyn NetworkDevice,
        destination: Ipv4Address,
        identifier: u16,
        sequence: u16,
        now_ms: u64,
    ) -> Result<PingResult, NetworkError> {
        self.destination = Some(destination);
        self.identifier = identifier;
        self.sequence = sequence;
        self.send_pending(device, now_ms)
    }

    pub fn poll(
        &mut self,
        device: &mut dyn NetworkDevice,
        now_ms: u64,
    ) -> Result<PingResult, NetworkError> {
        device.poll()?;
        let (buffer_id, event) = match device.take_rx() {
            Some(frame) => {
                let buffer_id = frame.buffer_id;
                let event = self
                    .stack
                    .process_rx(self.local_mac, frame.bytes, now_ms)
                    .map_err(map_packet_error)?;
                (buffer_id, event)
            }
            None => return Ok(PingResult::Waiting),
        };
        device.recycle_rx(buffer_id)?;
        match event {
            StackEvent::ArpResolved { .. } => self.send_pending(device, now_ms),
            StackEvent::EchoReply {
                source,
                identifier,
                sequence,
            } if identifier == self.identifier && sequence == self.sequence => {
                Ok(PingResult::EchoReply { source, sequence })
            }
            _ => Ok(PingResult::Waiting),
        }
    }

    fn send_pending(
        &mut self,
        device: &mut dyn NetworkDevice,
        now_ms: u64,
    ) -> Result<PingResult, NetworkError> {
        let destination = self.destination.ok_or(NetworkError::DeviceNotReady)?;
        if let Some(next_hop_mac) = self.stack.next_hop_mac(destination, now_ms) {
            let length = self
                .stack
                .build_ping(
                    &mut self.frame,
                    self.local_mac,
                    PingRequest {
                        next_hop_mac,
                        destination,
                        identifier: self.identifier,
                        sequence: self.sequence,
                        payload: b"kernel-ping",
                    },
                )
                .map_err(map_packet_error)?;
            device.submit_tx(TxFrame::new(&self.frame[..length]))?;
            return Ok(PingResult::EchoRequestSent);
        }
        let length = self
            .stack
            .build_arp_request(&mut self.frame, self.local_mac, destination)
            .map_err(map_packet_error)?;
        device.submit_tx(TxFrame::new(&self.frame[..length]))?;
        Ok(PingResult::ArpRequestSent)
    }
}

fn map_packet_error(error: PacketError) -> NetworkError {
    match error {
        PacketError::Truncated => NetworkError::BufferTooSmall,
        _ => NetworkError::ReceiveFailed,
    }
}
