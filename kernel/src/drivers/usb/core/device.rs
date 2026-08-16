use super::descriptor::{DeviceDesc, EndpointDesc, InterfaceDesc};
use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::host::xhci::context::Contexts;
use crate::drivers::usb::host::xhci::ring::TransferRing;
use crate::drivers::usb::UsbError;

pub struct UsbDevice {
    pub slot: u8,
    pub speed: u32,
    pub ep0_mps: u16,
    pub desc: DeviceDesc,
    pub config_value: u8,
    pub ifaces: [InterfaceDesc; 4],
    pub iface_count: usize,
    pub eps: [EndpointDesc; 8],
    pub ep_count: usize,
    pub ctx: Contexts,
    pub ep0: TransferRing,
    pub data: DmaBuf,
}

impl UsbDevice {
    pub fn new(slot: u8, speed: u32, ctx_size: usize) -> Result<Self, UsbError> {
        Ok(Self {
            slot,
            speed,
            ep0_mps: 8,
            desc: DeviceDesc::default(),
            config_value: 0,
            ifaces: [InterfaceDesc::default(); 4],
            iface_count: 0,
            eps: [EndpointDesc::default(); 8],
            ep_count: 0,
            ctx: Contexts::new(ctx_size)?,
            ep0: TransferRing::new(16)?,
            data: DmaBuf::new(512)?,
        })
    }
}