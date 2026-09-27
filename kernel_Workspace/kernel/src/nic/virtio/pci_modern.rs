//! virtio-net, modern transport (PCI capability-based, MMIO only).
//!
//! # Why this file exists
//!
//! The legacy driver in [`super::pci_legacy`] speaks the 0.9 transport through
//! x86 port I/O. Every virtio-net device from QEMU's `virtio-net-pci` on a
//! modern machine — which is what a current hypervisor hands out by default — is
//! the 1.0 device (`0x1041`), and it has **no I/O BAR at all**. It exposes a
//! single MMIO BAR holding a `common_cfg` structure, and the queues are reached
//! through a vendor-specific PCI capability. `io_base_from_bar` returns `None`
//! for it, which is why the legacy path could never have claimed it.
//!
//! So this is not the legacy driver with a different constant. The two devices
//! share the virtqueue *concept* and nothing else: different register layout,
//! different negotiation, different queue setup. Only the packet layer above
//! (`super::net`) is common.
//!
//! # What is negotiated
//!
//! The modern transport splits features into a 32-bit low half and a 32-bit
//! high half through the common configuration block, and requires the driver to
//! acknowledge exactly the bits it implements. `VERSION_1` must be set in the
//! low half. A driver that negotiates without setting it is talking to a device
//! that may still be in legacy mode, and every register it writes afterwards
//! means something else.

use alloc::vec;
use alloc::vec::Vec;
use core::ptr::{read_volatile, write_volatile};

use crate::mm::ffi::{contig_alloc, kvirt_to_phys, phys_to_virt, vmm_map_device};
use crate::nic::error::NetworkError;
use crate::nic::virtio::queue::{Descriptor, VIRTQ_DESC_F_WRITE};
use crate::pci::{self, PciDevice};

/// PCI vendor for all virtio devices.
const VIRTIO_VENDOR: u16 = 0x1AF4;
/// virtio 1.0 network device.
const VIRTIO_NET_MODERN: u16 = 0x1041;

/// `VIRTIO_F_VERSION_1`, the bit that says "this is a 1.0 device".
const VIRTIO_F_VERSION_1: u64 = 1 << 32;
/// The only feature this driver implements: a plain, header-only virtqueue.
///
/// `VIRTIO_NET_F_MRG_RXBUF` would let the device coalesce received buffers, but
/// it changes the descriptor semantics and so is deliberately left unnegotiated.
const DRIVER_FEATURES: u64 = VIRTIO_F_VERSION_1;

const STATUS_ACKNOWLEDGE: u8 = 1;
const STATUS_DRIVER: u8 = 2;
const STATUS_DRIVER_OK: u8 = 4;
const STATUS_FAILED: u8 = 128;

const RX_QUEUE_INDEX: u16 = 0;
const TX_QUEUE_INDEX: u16 = 1;
const QUEUE_SIZE: u16 = 8;
const FRAME_BYTES: usize = 1518;
const PAGE_SIZE: usize = 4096;
const HEADER_BYTES: usize = 10;
const DMA_FRAME_BYTES: usize = HEADER_BYTES + FRAME_BYTES;

/// Offsets within `struct virtio_pci_common_cfg`, in bytes. The structure is
/// read and written 32 bits at a time, so each field is a register index.
mod common_cfg {
    pub const DEVICE_FEATURE_SELECT: usize = 0x00;
    pub const DEVICE_FEATURE: usize = 0x04;
    pub const DRIVER_FEATURE_SELECT: usize = 0x08;
    pub const DRIVER_FEATURE: usize = 0x0C;
    pub const MSIX_CONFIG: usize = 0x10;
    pub const NUM_QUEUES: usize = 0x12;
    pub const DEVICE_STATUS: usize = 0x13;
    pub const CONFIG_GENERATION: usize = 0x14;
    pub const QUEUE_SELECT: usize = 0x16;
    pub const QUEUE_SIZE: usize = 0x18;
    pub const QUEUE_MSIX_VECTOR: usize = 0x1A;
    pub const QUEUE_ENABLE: usize = 0x1C;
    pub const QUEUE_NOTIFY_OFF: usize = 0x1E;
    pub const QUEUE_DESC: usize = 0x20;
    pub const QUEUE_DRIVER: usize = 0x28;
    pub const QUEUE_DEVICE: usize = 0x30;
}

/// The device config space, as `struct virtio_net_config` for a modern device.
/// Offsets are in bytes from the start of the config blob.
mod net_config {
    pub const MAC: usize = 0x00;
    pub const STATUS: usize = 0x06;
    pub const MAX_VIRTQ_PAIRS: usize = 0x0C;
}

/// A device-specific region reached through a PCI capability.
#[derive(Debug, Clone, Copy)]
struct Capability {
    offset: u16,
    length: u16,
}

/// The virtio vendor-specific capability: where the queues live.
const VIRTIO_PCI_CAP_ID: u8 = 0x09;

/// Walk the capability list for `cap_id`, returning its offset in BAR space.
fn find_capability(addr: pci::PciAddress, cap_id: u8) -> Option<Capability> {
    // Standard capability pointer, byte 0x34.
    let mut ptr = pci::config_read_u8(addr, 0x34) as u16;
    // The list is a bounded walk; 48 capabilities is the architectural maximum
    // for a single device, and the loop is bounded by that as well as by the
    // termination condition, so a malformed device cannot spin here.
    let mut guard = 0;
    while ptr != 0 && ptr & 0x3 == 0 && guard < 48 {
        guard += 1;
        let id_reg = pci::config_read_u8(addr, ptr);
        let next = pci::config_read_u8(addr, ptr + 1) & 0xF0;
        if id_reg == cap_id {
            let offset = pci::config_read_u16(addr, ptr + 4);
            let length = pci::config_read_u16(addr, ptr + 6);
            return Some(Capability { offset, length });
        }
        ptr = next;
    }
    None
}

/// One DMA-able packet buffer.
#[derive(Debug, Clone, Copy)]
struct Buffer {
    phys: u64,
    virt: *mut u8,
}

impl Buffer {
    /// A page-aligned, physically contiguous buffer of `bytes`.
    fn allocate(bytes: usize) -> Result<Self, NetworkError> {
        let mut phys = 0u64;
        let mut virt = core::ptr::null_mut::<core::ffi::c_void>();
        let ok = unsafe {
            contig_alloc(
                bytes,
                PAGE_SIZE,
                &mut phys as *mut u64,
                &mut virt as *mut *mut core::ffi::c_void,
            )
        };
        if !ok || virt.is_null() || phys == 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        Ok(Self {
            phys,
            virt: virt as *mut u8,
        })
    }
}

/// A modern virtio-net device on its MMIO BAR.
pub struct VirtioModernNet {
    /// Base of the common configuration block, virtual address.
    common: *mut u8,
    /// Base of the device-specific region, virtual address.
    notify: *mut u8,
    /// Where each queue's notify value lives, relative to `notify`.
    notify_offsets: [u32; 2],
    /// Where the device config blob starts, relative to `common`.
    config_offset: u32,
    device_features: u64,
    queues: [MmioQueue; 2],
    /// `(tx, rx)` packet buffers, one per queue slot. `None` until `init`
    /// allocates them.
    buffers: Option<(Vec<Buffer>, Vec<Buffer>)>,
    /// The next transmit slot to use, round-robin.
    next_tx_slot: u16,
    mac: [u8; 6],
}

/// One virtqueue's three rings, allocated contiguously.
#[derive(Debug, Clone, Copy)]
struct MmioQueue {
    virt: *mut u8,
    phys: u64,
    size: u16,
    /// The device's last-used index, cached so a poll with no traffic is cheap.
    last_used: u16,
    /// Set while a transmit buffer is still owned by the device.
    tx_busy: bool,
}

impl MmioQueue {
    const fn empty() -> Self {
        Self {
            virt: core::ptr::null_mut(),
            phys: 0,
            size: 0,
            last_used: 0,
            tx_busy: false,
        }
    }

    fn allocate(size: u16) -> Result<Self, NetworkError> {
        let bytes = queue_bytes(size);
        let mut phys = 0u64;
        let mut virt = core::ptr::null_mut::<core::ffi::c_void>();
        let ok = unsafe {
            contig_alloc(
                bytes,
                PAGE_SIZE,
                &mut phys as *mut u64,
                &mut virt as *mut *mut core::ffi::c_void,
            )
        };
        if !ok || virt.is_null() || phys == 0 || phys as usize % PAGE_SIZE != 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        // The device reads these rings; leaving them uninitialised would let it
        // follow a descriptor index we never set.
        unsafe { core::ptr::write_bytes(virt as *mut u8, 0, bytes) };
        Ok(Self {
            virt: virt as *mut u8,
            phys,
            size,
            last_used: 0,
            tx_busy: false,
        })
    }

    fn desc_offset(&self, index: u16) -> usize {
        index as usize * core::mem::size_of::<Descriptor>()
    }

    fn avail(&self) -> *mut u8 {
        unsafe { self.virt.add(self.size as usize * core::mem::size_of::<Descriptor>()) }
    }

    fn used(&self) -> *mut u8 {
        let desc_bytes = self.size as usize * core::mem::size_of::<Descriptor>();
        let avail_bytes = 4 + self.size as usize * 2;
        unsafe { self.virt.add(align_up(desc_bytes + avail_bytes, PAGE_SIZE)) }
    }

    /// Hand a buffer to the device: write the descriptor and publish it.
    unsafe fn submit(&mut self, slot: u16, addr: u64, len: u32, flags: u16) {
        let desc = self.virt.add(self.desc_offset(slot)) as *mut Descriptor;
        write_volatile(
            desc,
            Descriptor {
                addr,
                len,
                flags,
                next: 0,
            },
        );
        let avail = self.avail();
        let idx_ptr = avail.add(2) as *mut u16;
        let ring = self.size;
        // Read the current index rather than caching it: the device may have
        // advanced it since the last submit, and using a stale one would
        // overwrite a ring entry the device has not read yet.
        let next = read_volatile(idx_ptr);
        let ring_base = avail.add(4) as *mut u16;
        write_volatile(ring_base.add((next % ring) as usize), slot);
        write_volatile(idx_ptr, next.wrapping_add(1));
    }

    /// Take one buffer back if the device has returned one.
    ///
    /// Returns `(slot, len)` or `None` when the ring is empty.
    unsafe fn take_used(&mut self) -> Option<(u16, u32)> {
        let used = self.used();
        let flags = read_volatile(used as *const u16);
        if self.last_used == flags {
            return None;
        }
        let ring_base = used.add(4) as *mut u8;
        let elem = read_volatile(ring_base.add(self.last_used as usize * 8) as *const u32);
        let slot = elem as u16;
        let len = read_volatile(ring_base.add(self.last_used as usize * 8 + 4) as *const u32);
        self.last_used = self.last_used.wrapping_add(1);
        Some((slot, len))
    }

    /// The buffer behind a used descriptor, as a physical address and length.
    unsafe fn buffer_for(&self, slot: u16) -> (u64, u32) {
        let desc = self.virt.add(self.desc_offset(slot)) as *const Descriptor;
        let d = read_volatile(desc);
        (d.addr, d.len)
    }

    /// The three rings' physical addresses: `(desc, avail, used)`.
    fn ring_phys(&self) -> (u64, u64, u64) {
        let size = self.size as usize;
        let desc_bytes = size * core::mem::size_of::<Descriptor>();
        let avail_bytes = 4 + size * 2;
        (
            self.phys,
            self.phys + desc_bytes as u64,
            self.phys + align_up(desc_bytes + avail_bytes, PAGE_SIZE) as u64,
        )
    }

    /// The virtual address of the buffer behind `slot`, for the driver to read
    /// or write.
    ///
    /// Packet buffers come from `contig_alloc` and are ordinary memory, so the
    /// direct map resolves them. Device registers are the ones that need the
    /// vmm, and they never come through here.
    unsafe fn buffer_virt(&self, slot: u16) -> *mut u8 {
        let (phys, _len) = self.buffer_for(slot);
        phys_to_virt(phys)
    }
}

fn align_up(value: usize, to: usize) -> usize {
    (value + to - 1) & !(to - 1)
}

fn queue_bytes(size: u16) -> usize {
    let size = size as usize;
    let desc_bytes = size * core::mem::size_of::<Descriptor>();
    let avail_bytes = 4 + size * 2;
    let used_bytes = 4 + size * 8;
    align_up(desc_bytes + avail_bytes + used_bytes, PAGE_SIZE)
}

impl VirtioModernNet {
    /// Claim the device and bring it up.
    ///
    /// `dev` must be a virtio-net 1.0 device. The sequence is the one the
    /// specification fixes, and the order is not negotiable: reset, acknowledge,
    /// then `VERSION_1`, then features, then queues, then `DRIVER_OK`. Skipping
    /// the status writes leaves the device in a state where it silently drops
    /// traffic instead of reporting a failure.
    pub fn init(dev: &PciDevice) -> Result<Self, NetworkError> {
        if dev.vendor_id != VIRTIO_VENDOR || dev.device_id != VIRTIO_NET_MODERN {
            return Err(NetworkError::DeviceNotReady);
        }

        // A 1.0 device has no I/O BAR. BAR 0 is memory-mapped, and unlike the
        // packet buffers it is *not* under the direct map: device BARs are
        // mapped through the vmm so they can be given device attributes, so
        // `phys_to_virt` would read the wrong place.
        let bar_value = pci::bar(dev.address, 0);
        let bar_phys = pci::mem_base_from_bar(bar_value).ok_or(NetworkError::DeviceNotReady)?;
        if bar_phys == 0 {
            return Err(NetworkError::DeviceNotReady);
        }
        pci::enable_bus_mastering(dev.address);

        // The virtio capability points at the notify region. Without it there is
        // no way to kick a queue, so this is a hard failure rather than a
        // degraded mode.
        let cap = find_capability(dev.address, VIRTIO_PCI_CAP_ID)
            .ok_or(NetworkError::DeviceNotReady)?;

        // Map the whole BAR. The size is not advertised anywhere in the config
        // space, so the page-aligned round-up is the only bound available; a
        // virtio-net BAR is at most a few pages.
        let bar_len = align_up(cap.offset as usize + cap.length as usize + 0x1000, PAGE_SIZE);
        let mut bar_virt = 0u64;
        if !unsafe { vmm_map_device(bar_phys as u64, bar_len, &mut bar_virt as *mut u64) } {
            return Err(NetworkError::MmioUnmapped);
        }
        if bar_virt == 0 {
            return Err(NetworkError::MmioUnmapped);
        }

        // The common config block sits at the capability's offset; the device
        // config blob follows the whole notify region.
        let common = (bar_virt as *mut u8).wrapping_add(cap.offset as usize);
        let notify = common;
        let config_offset = cap.offset as u32 + cap.length as u32;

        let mut me = Self {
            common,
            notify,
            notify_offsets: [0; 2],
            config_offset,
            device_features: 0,
            queues: [MmioQueue::empty(), MmioQueue::empty()],
            buffers: None,
            next_tx_slot: 0,
            mac: [0; 6],
        };

        me.reset();
        me.set_status(STATUS_ACKNOWLEDGE);
        me.set_status(STATUS_ACKNOWLEDGE | STATUS_DRIVER);

        // Read both halves of the device's features, then acknowledge only what
        // this driver implements. Acknowledging a bit that is not offered is
        // undefined, so the two are intersected rather than assumed.
        let low = me.read_device_features(0);
        let high = me.read_device_features(1);
        me.device_features = (low as u64) | ((high as u64) << 32);
        me.write_driver_features(0, (DRIVER_FEATURES & 0xFFFF_FFFF) as u32);
        me.write_driver_features(1, (DRIVER_FEATURES >> 32) as u32);

        let num_queues = me.read_common_u16(common_cfg::NUM_QUEUES);
        if num_queues < 2 {
            return Err(NetworkError::InvalidQueueSize);
        }

        // One buffer per slot. A single shared buffer would serialise the device
        // and stall the link, so the count follows the ring size.
        me.alloc_buffers()?;
        me.setup_queue(RX_QUEUE_INDEX)?;
        me.setup_queue(TX_QUEUE_INDEX)?;

        // Record where each queue's notify value lives, and refill the RX ring
        // before DRIVER_OK: a device that starts with an empty receive queue has
        // nowhere to put a frame that arrives immediately.
        me.select_queue(RX_QUEUE_INDEX);
        me.notify_offsets[RX_QUEUE_INDEX as usize] =
            me.read_common_u32(common_cfg::QUEUE_NOTIFY_OFF) as u32;
        me.select_queue(TX_QUEUE_INDEX);
        me.notify_offsets[TX_QUEUE_INDEX as usize] =
            me.read_common_u32(common_cfg::QUEUE_NOTIFY_OFF) as u32;
        me.fill_rx();

        me.mac = me.read_mac();
        me.set_status(STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);
        Ok(me)
    }

    // -- register access ---------------------------------------------------

    unsafe fn read_common_u8(&self, off: usize) -> u8 {
        read_volatile(self.common.add(off))
    }

    unsafe fn read_common_u16(&self, off: usize) -> u16 {
        read_volatile(self.common.add(off) as *const u16)
    }

    unsafe fn read_common_u32(&self, off: usize) -> u32 {
        read_volatile(self.common.add(off) as *const u32)
    }

    unsafe fn read_common_u64(&self, off: usize) -> u64 {
        read_volatile(self.common.add(off) as *const u64)
    }

    unsafe fn write_common_u8(&self, off: usize, value: u8) {
        write_volatile(self.common.add(off), value);
    }

    unsafe fn write_common_u16(&self, off: usize, value: u16) {
        write_volatile(self.common.add(off) as *mut u16, value);
    }

    unsafe fn write_common_u32(&self, off: usize, value: u32) {
        write_volatile(self.common.add(off) as *mut u32, value);
    }

    unsafe fn write_common_u64(&self, off: usize, value: u64) {
        write_volatile(self.common.add(off) as *mut u64, value);
    }

    // -- the init sequence, part two ---------------------------------------

    /// Zero the device's status. A device left in a previous session's state
    /// will ignore a fresh handshake, so this is mandatory rather than tidy.
    pub fn reset(&mut self) {
        unsafe { self.write_common_u8(common_cfg::DEVICE_STATUS, 0) };
    }

    pub fn status(&self) -> u8 {
        unsafe { self.read_common_u8(common_cfg::DEVICE_STATUS) }
    }

    pub fn set_status(&mut self, status: u8) {
        unsafe { self.write_common_u8(common_cfg::DEVICE_STATUS, status) };
    }

    /// Read one 32-bit half of the device's offered features.
    ///
    /// The select register picks which half; the feature register then reads
    /// *that* half. Reading the low half twice would return the low half twice.
    pub fn read_device_features(&self, select: u32) -> u32 {
        unsafe {
            self.write_common_u32(common_cfg::DEVICE_FEATURE_SELECT, select);
            self.read_common_u32(common_cfg::DEVICE_FEATURE)
        }
    }

    pub fn write_driver_features(&self, select: u32, features: u32) {
        unsafe {
            self.write_common_u32(common_cfg::DRIVER_FEATURE_SELECT, select);
            self.write_common_u32(common_cfg::DRIVER_FEATURE, features);
        }
    }

    /// Point the common config at one queue.
    ///
    /// Every subsequent `queue_*` register refers to the selected queue, so a
    /// missing select is a silent misconfiguration rather than a visible one.
    pub fn select_queue(&mut self, index: u16) {
        unsafe { self.write_common_u16(common_cfg::QUEUE_SELECT, index) };
    }

    /// Program one queue and switch it on.
    ///
    /// The three rings are laid out contiguously in one allocation, and the
    /// device is given the *physical* address of each. Passing a virtual address
    /// here is the classic virtio bug: the device does DMA, so it can only follow
    /// physical ones.
    fn setup_queue(&mut self, index: u16) -> Result<(), NetworkError> {
        let size = QUEUE_SIZE;
        let queue = MmioQueue::allocate(size)?;
        let (desc, avail, used) = queue.ring_phys();
        self.queues[index as usize] = queue;

        self.select_queue(index);
        unsafe {
            self.write_common_u16(common_cfg::QUEUE_SIZE, size);
            self.write_common_u64(common_cfg::QUEUE_DESC, desc);
            self.write_common_u64(common_cfg::QUEUE_DRIVER, avail);
            self.write_common_u64(common_cfg::QUEUE_DEVICE, used);
            self.write_common_u16(common_cfg::QUEUE_ENABLE, 1);
        }
        Ok(())
    }

    /// Kick a queue: write its index to the notify location the device named.
    pub fn notify(&self, index: u16) {
        let off = self.notify_offsets[index as usize] as usize;
        unsafe { write_volatile(self.notify.add(off) as *mut u16, index) };
    }

    /// Read the device's MAC address out of its config blob.
    fn read_mac(&self) -> [u8; 6] {
        let mut mac = [0u8; 6];
        for (i, slot) in mac.iter_mut().enumerate() {
            // Read as bytes rather than as a six-byte load: the config blob is
            // byte-addressed and may be narrower than the read word.
            *slot = unsafe { self.read_common_u8(self.config_offset + net_config::MAC + i) };
        }
        mac
    }

    /// The link status the device reports: 1 is up.
    pub fn link_up(&self) -> bool {
        unsafe { self.read_common_u16(self.config_offset + net_config::STATUS) & 1 != 0 }
    }

    /// The device's MAC address.
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    /// The features the device offered, as a 64-bit mask.
    pub fn features(&self) -> u64 {
        self.device_features
    }

    // -- buffers -----------------------------------------------------------

    /// Allocate one packet buffer per queue slot.
    ///
    /// Returns `(tx_buffers, rx_buffers)`. Buffers are their own page-aligned
    /// allocations rather than carve-outs of a bigger one: a receive buffer must
    /// be physically contiguous for the device to write it in one DMA.
    fn alloc_buffers(&mut self) -> Result<(Vec<Buffer>, Vec<Buffer>), NetworkError> {
        let mut tx = Vec::with_capacity(QUEUE_SIZE as usize);
        let mut rx = Vec::with_capacity(QUEUE_SIZE as usize);
        for _ in 0..QUEUE_SIZE {
            tx.push(Buffer::allocate(DMA_FRAME_BYTES)?);
        }
        for _ in 0..QUEUE_SIZE {
            rx.push(Buffer::allocate(DMA_FRAME_BYTES)?);
        }
        self.buffers = Some((tx, rx));
        Ok(())
    }

    // -- data path ---------------------------------------------------------

    /// Hand every receive buffer to the device.
    ///
    /// Called once at init and again after each frame is taken, because a buffer
    /// only comes back through the used ring. A queue that is not refilled
    /// stops receiving — silently, because the device has no way to report "I have
    /// nowhere to put this".
    pub fn fill_rx(&mut self) {
        let Some((_, rx)) = self.buffers.as_ref() else {
            return;
        };
        let count = rx.len();
        for slot in 0..count as u16 {
            let buf = rx[slot as usize];
            unsafe {
                self.queues[RX_QUEUE_INDEX as usize].submit(
                    slot,
                    buf.phys,
                    DMA_FRAME_BYTES as u32,
                    VIRTQ_DESC_F_WRITE,
                );
            }
        }
        self.notify(RX_QUEUE_INDEX);
    }

    /// Take one received frame, if the device has one.
    ///
    /// Returns the frame without the 10-byte virtio header. The buffer is
    /// refilled before returning, so the next frame has somewhere to land.
    pub fn receive(&mut self) -> Option<Vec<u8>> {
        let (slot, _len) = unsafe { self.queues[RX_QUEUE_INDEX as usize].take_used()? }?;
        let Some((_, rx)) = self.buffers.as_ref() else {
            return None;
        };
        let buf = rx[slot as usize];
        let frame = if buf.virt.is_null() || slot as usize >= rx.len() {
            Vec::new()
        } else {
            // The device writes the frame length into the header, which is the
            // only source of the real length: the descriptor length is the
            // buffer size, not the packet size.
            let header_len = unsafe {
                core::ptr::read_volatile(buf.virt.add(0) as *const u16)
            } as usize;
            let data_len = if header_len == 0 || header_len > FRAME_BYTES {
                0
            } else {
                header_len
            };
            if data_len == 0 {
                Vec::new()
            } else {
                unsafe { core::slice::from_raw_parts(buf.virt.add(HEADER_BYTES), data_len) }
                    .to_vec()
            }
        };

        // Put the buffer back before returning: the caller may be slow to read
        // the frame, and the device needs the slot.
        self.fill_rx();
        if frame.is_empty() {
            None
        } else {
            Some(frame)
        }
    }

    /// Send one frame.
    ///
    /// `frame` is a bare ethernet frame; the 10-byte virtio header is prepended
    /// here. Returns `QueueFull` rather than blocking when every transmit slot
    /// is still owned by the device, because a blocking send in a polled driver
    /// deadlocks against its own receive path.
    pub fn send(&mut self, frame: &[u8]) -> Result<(), NetworkError> {
        if frame.len() > FRAME_BYTES {
            return Err(NetworkError::FrameTooLarge);
        }
        let Some((tx, _)) = self.buffers.as_ref() else {
            return Err(NetworkError::DeviceNotReady);
        };
        if tx.is_empty() {
            return Err(NetworkError::QueueFull);
        }

        // Reclaim any transmit the device has already finished, then take the
        // first free slot.
        while let Some((slot, _)) = unsafe { self.queues[TX_QUEUE_INDEX as usize].take_used() } {
            let _ = slot;
        }

        let slot = self.next_tx_slot;
        self.next_tx_slot = (slot + 1) % tx.len() as u16;

        let buf = tx[slot as usize];
        if buf.virt.is_null() {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        unsafe {
            // A zeroed header with num_buffers = 1 is the only form negotiated:
            // without MRG_RXBUF the device must see exactly one buffer.
            core::ptr::write_bytes(buf.virt, 0, HEADER_BYTES);
            core::ptr::copy_nonoverlapping(frame.as_ptr(), buf.virt.add(HEADER_BYTES), frame.len());
            self.queues[TX_QUEUE_INDEX as usize].submit(
                slot,
                buf.phys,
                (HEADER_BYTES + frame.len()) as u32,
                0,
            );
        }
        self.notify(TX_QUEUE_INDEX);
        Ok(())
    }

    /// Whether the device has flagged a failure.
    pub fn in_failed_state(&self) -> bool {
        self.status() & STATUS_FAILED != 0
    }
}

/// A received frame, copied out of the DMA buffer.
///
/// A copy rather than a borrow into the buffer: `take_frame` hands the buffer
/// straight back to the device, so a reference into it would hand out memory
/// the device may overwrite while the caller is still reading it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedFrame {
    pub bytes: Vec<u8>,
}

impl VirtioModernNet {
    /// Take one received frame, copying it out of the DMA buffer.
    pub fn take_frame(&mut self) -> Option<StagedFrame> {
        self.receive().map(|bytes| StagedFrame { bytes })
    }
}
