use core::ffi::c_void;
use core::ptr::{read_volatile, write_bytes, write_volatile};
use core::sync::atomic::{compiler_fence, Ordering};

use x86_64::instructions::port::Port;

use crate::mm::ffi::{contig_alloc, kvirt_to_phys};
use crate::nic::device::{NetworkDevice, PollResult, RxFrame, TxFrame};
use crate::nic::error::NetworkError;
use crate::nic::types::MacAddress;
use crate::nic::virtio::queue::{Descriptor, VIRTQ_DESC_F_WRITE};

const VIRTIO_NET_F_MAC: u32 = 1 << 5;
const STATUS_ACKNOWLEDGE: u8 = 1;
const STATUS_DRIVER: u8 = 2;
const STATUS_DRIVER_OK: u8 = 4;
const STATUS_FAILED: u8 = 128;
const RX_QUEUE_INDEX: u16 = 0;
const TX_QUEUE_INDEX: u16 = 1;
const QUEUE_SIZE: u16 = 4;
const FRAME_BYTES: usize = 1536;
const PAGE_SIZE: usize = 4096;
const HEADER_BYTES: usize = core::mem::size_of::<VirtioNetHeader>();
const DMA_FRAME_BYTES: usize = HEADER_BYTES + FRAME_BYTES;

const REG_DEVICE_FEATURES: u16 = 0;
const REG_GUEST_FEATURES: u16 = 4;
const REG_QUEUE_ADDRESS: u16 = 8;
const REG_QUEUE_SIZE: u16 = 12;
const REG_QUEUE_SELECT: u16 = 14;
const REG_QUEUE_NOTIFY: u16 = 16;
const REG_DEVICE_STATUS: u16 = 18;
const REG_ISR_STATUS: u16 = 19;
const REG_DEVICE_CONFIG: u16 = 20;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtioNetHeader {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_start: u16,
    csum_offset: u16,
}

#[derive(Clone, Copy)]
struct LegacyQueue {
    virt: *mut u8,
    phys: u64,
    size: u16,
    last_used: u16,
}

impl LegacyQueue {
    const fn empty() -> Self {
        Self {
            virt: core::ptr::null_mut(),
            phys: 0,
            size: 0,
            last_used: 0,
        }
    }

    fn allocate(size: u16) -> Result<Self, NetworkError> {
        let bytes = queue_bytes(size);
        let mut phys = 0u64;
        let mut virt = core::ptr::null_mut::<c_void>();
        let ok = unsafe {
            contig_alloc(
                bytes,
                PAGE_SIZE,
                &mut phys as *mut u64,
                &mut virt as *mut *mut c_void,
            )
        };
        if !ok || virt.is_null() || phys == 0 || phys as usize % PAGE_SIZE != 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        unsafe {
            write_bytes(virt as *mut u8, 0, bytes);
        }
        Ok(Self {
            virt: virt as *mut u8,
            phys,
            size,
            last_used: 0,
        })
    }

    fn descriptor_offset(&self, index: u16) -> usize {
        index as usize * core::mem::size_of::<Descriptor>()
    }

    fn avail_offset(&self) -> usize {
        self.size as usize * core::mem::size_of::<Descriptor>()
    }

    fn used_offset(&self) -> usize {
        let descriptor_bytes = self.size as usize * core::mem::size_of::<Descriptor>();
        let avail_bytes = 4 + self.size as usize * 2;
        align_up(descriptor_bytes + avail_bytes, PAGE_SIZE)
    }

    unsafe fn write_descriptor(&mut self, index: u16, addr: u64, len: u32, flags: u16) {
        let descriptor = self.virt.add(self.descriptor_offset(index)) as *mut Descriptor;
        write_volatile(
            descriptor,
            Descriptor {
                addr,
                len,
                flags,
                next: 0,
            },
        );
    }

    unsafe fn submit(&mut self, descriptor: u16) {
        let avail = self.virt.add(self.avail_offset());
        let idx_ptr = avail.add(2) as *mut u16;
        let idx = read_volatile(idx_ptr);
        let ring = avail.add(4) as *mut u16;
        write_volatile(ring.add(idx as usize % self.size as usize), descriptor);
        compiler_fence(Ordering::Release);
        write_volatile(idx_ptr, idx.wrapping_add(1));
    }

    unsafe fn used_index(&self) -> u16 {
        compiler_fence(Ordering::Acquire);
        read_volatile(self.virt.add(self.used_offset() + 2) as *const u16)
    }

    unsafe fn take_used(&mut self) -> Option<(u16, u32)> {
        let used_index = self.used_index();
        if used_index == self.last_used {
            return None;
        }
        let ring = self.virt.add(self.used_offset() + 4) as *const UsedElement;
        let element = read_volatile(ring.add(self.last_used as usize % self.size as usize));
        self.last_used = self.last_used.wrapping_add(1);
        if element.id >= self.size as u32 {
            return None;
        }
        Some((element.id as u16, element.len))
    }

    unsafe fn pending_used(&self) -> u16 {
        self.used_index().wrapping_sub(self.last_used)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct UsedElement {
    id: u32,
    len: u32,
}

pub struct VirtioPciLegacyNetDevice {
    io_base: u16,
    mac: MacAddress,
    ready: bool,
    rx_queue: LegacyQueue,
    tx_queue: LegacyQueue,
    rx_buffers: [[u8; DMA_FRAME_BYTES]; QUEUE_SIZE as usize],
    tx_buffers: [[u8; DMA_FRAME_BYTES]; QUEUE_SIZE as usize],
    tx_in_flight: [bool; QUEUE_SIZE as usize],
    rx_current: Option<u16>,
}

unsafe impl Send for VirtioPciLegacyNetDevice {}

impl VirtioPciLegacyNetDevice {
    pub const fn new(io_base: u16) -> Self {
        Self {
            io_base,
            mac: MacAddress::ZERO,
            ready: false,
            rx_queue: LegacyQueue::empty(),
            tx_queue: LegacyQueue::empty(),
            rx_buffers: [[0; DMA_FRAME_BYTES]; QUEUE_SIZE as usize],
            tx_buffers: [[0; DMA_FRAME_BYTES]; QUEUE_SIZE as usize],
            tx_in_flight: [false; QUEUE_SIZE as usize],
            rx_current: None,
        }
    }

    pub const fn io_base(&self) -> u16 {
        self.io_base
    }

    fn initialize(&mut self) -> Result<(), NetworkError> {
        if self.ready {
            return Ok(());
        }
        self.write_u8(REG_DEVICE_STATUS, 0);
        self.write_u8(REG_DEVICE_STATUS, STATUS_ACKNOWLEDGE);
        self.write_u8(REG_DEVICE_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);
        let offered = self.read_u32(REG_DEVICE_FEATURES);
        if offered & VIRTIO_NET_F_MAC == 0 {
            self.fail();
            return Err(NetworkError::UnsupportedFeatures {
                offered: offered as u64,
                requested: VIRTIO_NET_F_MAC as u64,
            });
        }
        self.write_u32(REG_GUEST_FEATURES, VIRTIO_NET_F_MAC);
        let mut mac = [0u8; 6];
        for (index, byte) in mac.iter_mut().enumerate() {
            *byte = self.read_u8(REG_DEVICE_CONFIG + index as u16);
        }
        self.mac = MacAddress(mac);
        if self.mac.is_zero() || self.mac.is_multicast() {
            self.fail();
            return Err(NetworkError::DeviceNeedsReset);
        }
        self.rx_queue = LegacyQueue::allocate(QUEUE_SIZE)?;
        self.tx_queue = LegacyQueue::allocate(QUEUE_SIZE)?;
        self.configure_queue(RX_QUEUE_INDEX, self.rx_queue)?;
        self.configure_queue(TX_QUEUE_INDEX, self.tx_queue)?;
        for index in 0..QUEUE_SIZE {
            self.post_rx(index)?;
        }
        self.notify(RX_QUEUE_INDEX);
        self.write_u8(
            REG_DEVICE_STATUS,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK,
        );
        self.ready = true;
        Ok(())
    }

    fn configure_queue(&mut self, index: u16, queue: LegacyQueue) -> Result<(), NetworkError> {
        self.write_u16(REG_QUEUE_SELECT, index);
        let max = self.read_u16(REG_QUEUE_SIZE);
        if queue.size == 0 || queue.size > max || !queue.size.is_power_of_two() {
            self.fail();
            return Err(NetworkError::InvalidQueueSize);
        }
        self.write_u16(REG_QUEUE_SIZE, queue.size);
        self.write_u32(REG_QUEUE_ADDRESS, (queue.phys >> 12) as u32);
        Ok(())
    }

    fn post_rx(&mut self, index: u16) -> Result<(), NetworkError> {
        if index >= QUEUE_SIZE {
            return Err(NetworkError::BadDescriptor);
        }
        let address = unsafe {
            kvirt_to_phys(self.rx_buffers[index as usize].as_mut_ptr() as *mut c_void)
        };
        if address == 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        unsafe {
            self.rx_queue.write_descriptor(
                index,
                address,
                DMA_FRAME_BYTES as u32,
                VIRTQ_DESC_F_WRITE,
            );
            self.rx_queue.submit(index);
        }
        Ok(())
    }

    fn reclaim_tx(&mut self) -> u16 {
        let mut reclaimed = 0u16;
        while let Some((index, _)) = unsafe { self.tx_queue.take_used() } {
            self.tx_in_flight[index as usize] = false;
            reclaimed = reclaimed.wrapping_add(1);
        }
        reclaimed
    }

    fn notify(&self, queue: u16) {
        self.write_u16(REG_QUEUE_NOTIFY, queue);
    }

    fn fail(&mut self) {
        self.write_u8(REG_DEVICE_STATUS, STATUS_FAILED);
        self.ready = false;
    }

    fn read_u8(&self, offset: u16) -> u8 {
        let mut port = Port::<u8>::new(self.io_base + offset);
        unsafe { port.read() }
    }

    fn read_u16(&self, offset: u16) -> u16 {
        let mut port = Port::<u16>::new(self.io_base + offset);
        unsafe { port.read() }
    }

    fn read_u32(&self, offset: u16) -> u32 {
        let mut port = Port::<u32>::new(self.io_base + offset);
        unsafe { port.read() }
    }

    fn write_u8(&self, offset: u16, value: u8) {
        let mut port = Port::<u8>::new(self.io_base + offset);
        unsafe {
            port.write(value);
        }
    }

    fn write_u16(&self, offset: u16, value: u16) {
        let mut port = Port::<u16>::new(self.io_base + offset);
        unsafe {
            port.write(value);
        }
    }

    fn write_u32(&self, offset: u16, value: u32) {
        let mut port = Port::<u32>::new(self.io_base + offset);
        unsafe {
            port.write(value);
        }
    }
}

impl NetworkDevice for VirtioPciLegacyNetDevice {
    fn init(&mut self) -> Result<(), NetworkError> {
        self.initialize()
    }

    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn mtu(&self) -> usize {
        1500
    }

    fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
        if !self.ready {
            return Err(NetworkError::DeviceNotReady);
        }
        if frame.bytes.is_empty() || frame.bytes.len() > FRAME_BYTES {
            return Err(NetworkError::BufferTooSmall);
        }
        self.reclaim_tx();
        let index = self
            .tx_in_flight
            .iter()
            .position(|in_flight| !*in_flight)
            .ok_or(NetworkError::QueueFull)?;
        let buffer = &mut self.tx_buffers[index];
        buffer[..HEADER_BYTES].fill(0);
        buffer[HEADER_BYTES..HEADER_BYTES + frame.bytes.len()].copy_from_slice(frame.bytes);
        let address = unsafe { kvirt_to_phys(buffer.as_mut_ptr() as *mut c_void) };
        if address == 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        unsafe {
            self.tx_queue.write_descriptor(
                index as u16,
                address,
                (HEADER_BYTES + frame.bytes.len()) as u32,
                0,
            );
            self.tx_queue.submit(index as u16);
        }
        self.tx_in_flight[index] = true;
        self.notify(TX_QUEUE_INDEX);
        Ok(())
    }

    fn poll(&mut self) -> Result<PollResult, NetworkError> {
        if !self.ready {
            return Err(NetworkError::DeviceNotReady);
        }
        let _ = self.read_u8(REG_ISR_STATUS);
        let tx_completed = self.reclaim_tx();
        let rx_available = unsafe { self.rx_queue.pending_used() };
        Ok(PollResult {
            tx_completed,
            rx_available,
            device_needs_reset: self.read_u8(REG_DEVICE_STATUS) & STATUS_FAILED != 0,
        })
    }

    fn take_rx(&mut self) -> Option<RxFrame<'_>> {
        if !self.ready || self.rx_current.is_some() {
            return None;
        }
        let (index, length) = unsafe { self.rx_queue.take_used() }?;
        let length = length as usize;
        if !(HEADER_BYTES..=DMA_FRAME_BYTES).contains(&length) {
            let _ = self.post_rx(index);
            self.notify(RX_QUEUE_INDEX);
            return None;
        }
        self.rx_current = Some(index);
        Some(RxFrame {
            buffer_id: index,
            bytes: &self.rx_buffers[index as usize][HEADER_BYTES..length],
        })
    }

    fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
        if self.rx_current != Some(buffer_id) {
            return Err(NetworkError::BadDescriptor);
        }
        self.rx_current = None;
        self.post_rx(buffer_id)?;
        self.notify(RX_QUEUE_INDEX);
        Ok(())
    }
}

const fn queue_bytes(queue_size: u16) -> usize {
    let queue_size = queue_size as usize;
    let descriptors = core::mem::size_of::<Descriptor>() * queue_size;
    let avail = 4 + 2 * queue_size;
    let used = 4 + 8 * queue_size;
    align_up(align_up(descriptors + avail, PAGE_SIZE) + used, PAGE_SIZE)
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
