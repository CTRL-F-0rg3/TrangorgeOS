use crate::nic::protocols::MacAddress;

pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;

pub const VIRTIO_NET_HDR_F_NEEDS_CSUM: u8 = 1;
pub const VIRTIO_NET_HDR_GSO_NONE: u8 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioNetHeader {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
}

pub struct VirtioNetDevice {
    pub(crate) mmio_base: usize,
    pub(crate) mac: MacAddress,
    pub(crate) ready: bool,
}

impl VirtioNetDevice {
    pub const fn new(mmio_base: usize) -> Self {
        VirtioNetDevice {
            mmio_base,
            mac: MacAddress::ZERO,
            ready: false,
        }
    }

    pub fn mmio_base(&self) -> usize {
        self.mmio_base
    }
}
