use crate::nic::virtio::descriptor::VirtqDescriptor;

pub const VIRTQ_ALIGN: usize = 4096;

crate::test_module!({
    let size_for_256 = legacy_queue_size(256);
    if size_for_256 % VIRTQ_ALIGN != 0 {
        return Err("legacy queue size is not aligned to the required page size");
    }
    if legacy_queue_size(512) <= size_for_256 {
        return Err("legacy queue size did not grow with queue_size");
    }
    Ok("legacy virtqueue size calculation is aligned and monotonic")
});

#[repr(C)]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; 0],
}

#[repr(C)]
pub struct VirtqUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtqUsedElem; 0],
}

pub fn legacy_queue_size(queue_size: u16) -> usize {
    let queue_size = queue_size as usize;
    let desc_table = core::mem::size_of::<VirtqDescriptor>() * queue_size;
    let avail = 4 + 2 * queue_size;
    let used_offset = align_up(desc_table + avail, VIRTQ_ALIGN);
    let used = 4 + 8 * queue_size;
    align_up(used_offset + used, VIRTQ_ALIGN)
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
