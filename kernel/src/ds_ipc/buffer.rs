// kernel/src/ds_ipc/buffer.rs

use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use core::mem::{size_of, align_of};
use core::ptr;
use core::slice;

use crate::caps::{CapId, Capability, CapRights, CNode};
use crate::cpu::scheduler::entities::task::{Task, TaskState};
use crate::mm::phys::{PhysFrame, FrameAllocator};
use crate::mm::virt::{PageTable, VirtualAddress, PhysicalAddress, PageSize, PageFlags};
use crate::mm::vmm::{AddressSpace, VmmError};
use crate::sync::spinlock::SpinLock;
use crate::types::{DsError, PhysAddr, VirtAddr};
use super::msg::{MessageInfo, MessageRegisters};

pub const IPC_BUFFER_MAX_SIZE: usize = 0x100000; 
pub const IPC_BUFFER_ALIGNMENT: usize = 0x1000;    
pub const MAX_MR_BYTES: usize = size_of::<MessageRegisters>();
pub const IPC_BUFFER_SLOT_INVALID: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BufferTransferMode {
    FastPathRegisters = 0,
    SlowPathZeroCopy = 1,
    SlowPathBounce   = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BufferError {
    Success = 0,
    InvalidAlignment = 1,
    SizeExceeded = 2,
    CapabilityMissing = 3,
    AddressNotMapped = 4,
    PermissionDenied = 5,
    PoolExhausted = 6,
    TlbFlushFailed = 7,
    CacheCoherencyFault = 8,
}

impl From<BufferError> for DsError {
    fn from(e: BufferError) -> Self {
        match e {
            BufferError::Success => DsError::Success,
            BufferError::CapabilityMissing | BufferError::PermissionDenied => DsError::PermissionDenied,
            BufferError::SizeExceeded | BufferError::InvalidAlignment => DsError::InvalidMessage,
            _ => DsError::Unknown,
        }
    }
}

bitflags::bitflags! {
    pub struct BufferFlags: u32 {
        const READABLE      = 1 << 0;
        const WRITABLE      = 1 << 1;
        const ZERO_COPY_OK  = 1 << 2;
        const CACHE_WB      = 1 << 3;
        const PINNED        = 1 << 4;
    }
}

#[repr(C)]
pub struct IpcBufferDescriptor {
    pub base_vaddr: VirtAddr,
    pub size_bytes: u64,
    pub flags: BufferFlags,
    pub page_count: u32,
    pub _pad: u32,
}

pub struct IpcBufferSlot {
    pub id: u64,
    pub owner_task_id: u64,
    pub descriptor: IpcBufferDescriptor,
    pub physical_frames: [Option<PhysFrame>; 256], 
    pub state: AtomicU8,
    pub lock: SpinLock<()>,
}

impl IpcBufferSlot {
    pub const STATE_FREE: u8 = 0;
    pub const STATE_REGISTERED: u8 = 1;
    pub const STATE_IN_TRANSFER: u8 = 2;
    pub const STATE_MAPPED_REMOTE: u8 = 3;

    pub fn new(id: u64, owner: u64) -> Self {
        Self {
            id,
            owner_task_id: owner,
            descriptor: IpcBufferDescriptor {
                base_vaddr: VirtAddr::new(0),
                size_bytes: 0,
                flags: BufferFlags::empty(),
                page_count: 0,
                _pad: 0,
            },
            physical_frames: [None; 256],
            state: AtomicU8::new(Self::STATE_FREE),
            lock: SpinLock::new(()),
        }
    }

    pub fn is_free(&self) -> bool {
        self.state.load(Ordering::Acquire) == Self::STATE_FREE
    }

    pub fn mark_in_transfer(&self) -> bool {
        self.state.compare_exchange(
            Self::STATE_REGISTERED, 
            Self::STATE_IN_TRANSFER, 
            Ordering::AcqRel, 
            Ordering::Relaxed
        ).is_ok()
    }

    pub fn release(&self) {
        self.state.store(Self::STATE_REGISTERED, Ordering::Release);
    }
}

pub struct IpcBufferPool {
    slots: [SpinLock<Option<IpcBufferSlot>>; 1024],
    next_id: AtomicU64,
}

impl IpcBufferPool {
    pub const fn new() -> Self {
        Self {
            slots: [const { SpinLock::new(None) }; 1024],
            next_id: AtomicU64::new(1),
        }
    }

    pub fn allocate_slot(&self, owner_task_id: u64) -> Result<u64, BufferError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let idx = (id % 1024) as usize;
        
        let mut lock = self.slots[idx].lock();
        if lock.is_some() {
            return Err(BufferError::PoolExhausted);
        }
        
        *lock = Some(IpcBufferSlot::new(id, owner_task_id));
        Ok(id)
    }

    pub fn register_buffer(
        &self, 
        slot_id: u64, 
        task: &mut Task, 
        desc: &IpcBufferDescriptor
    ) -> Result<(), BufferError> {
        if desc.size_bytes > IPC_BUFFER_MAX_SIZE as u64 {
            return Err(BufferError::SizeExceeded);
        }
        if desc.base_vaddr.as_u64() % IPC_BUFFER_ALIGNMENT as u64 != 0 {
            return Err(BufferError::InvalidAlignment);
        }

        let idx = (slot_id % 1024) as usize;
        let mut lock = self.slots[idx].lock();
        let slot = lock.as_mut().ok_or(BufferError::CapabilityMissing)?;

        if slot.owner_task_id != task.id {
            return Err(BufferError::PermissionDenied);
        }

        let page_count = ((desc.size_bytes + 0xFFF) / 0x1000) as usize;
        if page_count > 256 {
            return Err(BufferError::SizeExceeded);
        }

        let aspace = task.address_space_mut();
        for i in 0..page_count {
            let vaddr = VirtAddr::new(desc.base_vaddr.as_u64() + (i as u64 * 0x1000));
            let paddr = aspace.translate(vaddr).ok_or(BufferError::AddressNotMapped)?;
            
            let frame = PhysFrame::from_start_address(paddr);
            slot.physical_frames[i] = Some(frame);
        }

        slot.descriptor = *desc;
        slot.descriptor.page_count = page_count as u32;
        slot.state.store(IpcBufferSlot::STATE_REGISTERED, Ordering::Release);

        Ok(())
    }

    pub fn get_slot(&self, slot_id: u64) -> Option<core::sync::atomic::AtomicPtr<()>> {
        None 
    }
    
    pub fn with_slot<F, R>(&self, slot_id: u64, f: F) -> Option<R> 
    where F: FnOnce(&mut IpcBufferSlot) -> R 
    {
        let idx = (slot_id % 1024) as usize;
        let mut lock = self.slots[idx].lock();
        let slot = lock.as_mut()?;
        Some(f(slot))
    }
}

pub struct TransferContext<'a> {
    pub sender: &'a mut Task,
    pub receiver: &'a mut Task,
    pub msg_info: MessageInfo,
    pub sender_mrs: &'a MessageRegisters,
    pub receiver_mrs: &'a mut MessageRegisters,
    pub buffer_pool: &'a IpcBufferPool,
}

impl<'a> TransferContext<'a> {
    pub fn determine_mode(&self) -> BufferTransferMode {
        let payload_len = self.msg_info.length as usize;
        if payload_len <= MAX_MR_BYTES {
            return BufferTransferMode::FastPathRegisters;
        }

        if self.msg_info.extra_caps > 0 {
            if self.can_do_zero_copy() {
                return BufferTransferMode::SlowPathZeroCopy;
            }
        }

        BufferTransferMode::SlowPathBounce
    }

    fn can_do_zero_copy(&self) -> bool {
        true 
    }

    pub fn execute_transfer(&mut self) -> Result<(), BufferError> {
        match self.determine_mode() {
            BufferTransferMode::FastPathRegisters => self.transfer_via_registers(),
            BufferTransferMode::SlowPathZeroCopy => self.transfer_zero_copy(),
            BufferTransferMode::SlowPathBounce => self.transfer_via_bounce(),
        }
    }

    fn transfer_via_registers(&mut self) -> Result<(), BufferError> {
        let len = self.msg_info.length as usize;
        let src_bytes = unsafe { 
            slice::from_raw_parts(self.sender_mrs as *const _ as *const u8, len) 
        };
        let dst_bytes = unsafe { 
            slice::from_raw_parts_mut(self.receiver_mrs as *mut _ as *mut u8, len) 
        };
        
        dst_bytes[..len].copy_from_slice(&src_bytes[..len]);
        Ok(())
    }

    fn transfer_zero_copy(&mut self) -> Result<(), BufferError> {
        let sender_buf_cap_idx = self.sender_mrs.mr[0];
        let receiver_buf_cap_idx = self.receiver_mrs.mr[0];
        let offset = self.sender_mrs.mr[1] as usize;
        let length = self.sender_mrs.mr[2] as usize;

        let sender_cap = self.sender.cnode.resolve_cap(sender_buf_cap_idx)
            .ok_or(BufferError::CapabilityMissing)?;
        
        if !sender_cap.rights.contains(CapRights::READ) {
            return Err(BufferError::PermissionDenied);
        }

        let receiver_cap = self.receiver.cnode.resolve_cap(receiver_buf_cap_idx)
            .ok_or(BufferError::CapabilityMissing)?;

        if !receiver_cap.rights.contains(CapRights::WRITE) {
            return Err(BufferError::PermissionDenied);
        }

        let sender_slot_id = sender_cap.object_id;
        let receiver_slot_id = receiver_cap.object_id;

        let mut sender_frames = [None; 256];
        let mut sender_page_count = 0;

        self.buffer_pool.with_slot(sender_slot_id, |slot| {
            if slot.owner_task_id != self.sender.id {
                return Err(BufferError::PermissionDenied);
            }
            if !slot.mark_in_transfer() {
                return Err(BufferError::CapabilityMissing); 
            }
            sender_page_count = slot.descriptor.page_count as usize;
            for i in 0..sender_page_count {
                sender_frames[i] = slot.physical_frames[i];
            }
            Ok(())
        }).unwrap()?;

        let mut pages_to_map = (length + 0xFFF) / 0x1000;
        let start_page = offset / 0x1000;
        let page_offset_in_first = offset % 0x1000;

        let mut receiver_aspace = self.receiver.address_space_mut();
        let receiver_base_vaddr = self.buffer_pool.with_slot(receiver_slot_id, |slot| {
            slot.descriptor.base_vaddr
        }).unwrap();

        for i in 0..pages_to_map {
            let src_frame_idx = start_page + i;
            if src_frame_idx >= sender_page_count {
                return Err(BufferError::SizeExceeded);
            }

            let phys_frame = sender_frames[src_frame_idx].ok_or(BufferError::AddressNotMapped)?;
            
            let dst_vaddr = VirtAddr::new(
                receiver_base_vaddr.as_u64() + ((start_page + i) as u64 * 0x1000)
            );

            let old_frame = receiver_aspace.unmap_page(dst_vaddr)?;
            
            let flags = PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER;
            receiver_aspace.map_page(dst_vaddr, phys_frame, flags)?;

            if let Some(old) = old_frame {
                crate::mm::phys::global_frame_allocator().dealloc_frame(old);
            }
        }

        self.buffer_pool.with_slot(receiver_slot_id, |slot| {
            slot.state.store(IpcBufferSlot::STATE_MAPPED_REMOTE, Ordering::Release);
        });

        arch_flush_tlb_single(self.receiver.id, receiver_base_vaddr);
        arch_clean_invalidate_dcache_region(
            receiver_base_vaddr, 
            (pages_to_map * 0x1000) as u64
        );

        self.receiver_mrs.mr[0] = receiver_slot_id;
        self.receiver_mrs.mr[1] = page_offset_in_first as u64;
        self.receiver_mrs.mr[2] = length as u64;

        Ok(())
    }

    fn transfer_via_bounce(&mut self) -> Result<(), BufferError> {
        let sender_vaddr = VirtAddr::new(self.sender_mrs.mr[0]);
        let length = self.sender_mrs.mr[1] as usize;
        let receiver_slot_id = self.receiver_mrs.mr[2];

        if length > IPC_BUFFER_MAX_SIZE {
            return Err(BufferError::SizeExceeded);
        }

        let pages_needed = (length + 0xFFF) / 0x1000;
        let mut bounce_frames = [None; 256];
        
        let alloc = crate::mm::phys::global_frame_allocator();
        for i in 0..pages_needed {
            let frame = alloc.alloc_frame().ok_or(BufferError::PoolExhausted)?;
            bounce_frames[i] = Some(frame);
        }

        let kernel_vaddr_base = crate::mm::vmm::kernel_direct_map_vaddr(
            bounce_frames[0].unwrap().start_address()
        );

        let res = unsafe {
            safe_copy_from_user(
                self.sender,
                sender_vaddr,
                kernel_vaddr_base.as_mut_ptr::<u8>(),
                length
            )
        };

        if let Err(e) = res {
            for frame in bounce_frames.iter().flatten() {
                alloc.dealloc_frame(*frame);
            }
            return Err(e);
        }

        let mut receiver_vaddr = VirtAddr::new(0);
        self.buffer_pool.with_slot(receiver_slot_id, |slot| {
            if slot.owner_task_id != self.receiver.id {
                return Err(BufferError::PermissionDenied);
            }
            receiver_vaddr = slot.descriptor.base_vaddr;
            Ok(())
        }).unwrap()?;

        let res = unsafe {
            safe_copy_to_user(
                self.receiver,
                receiver_vaddr,
                kernel_vaddr_base.as_ptr::<u8>(),
                length
            )
        };

        for frame in bounce_frames.iter().flatten() {
            alloc.dealloc_frame(*frame);
        }

        if let Err(e) = res {
            return Err(e);
        }

        self.receiver_mrs.mr[0] = receiver_slot_id;
        self.receiver_mrs.mr[1] = 0;
        self.receiver_mrs.mr[2] = length as u64;

        Ok(())
    }
}

unsafe fn safe_copy_from_user(
    task: &Task,
    src_vaddr: VirtAddr,
    dst_kernel_ptr: *mut u8,
    len: usize
) -> Result<(), BufferError> {
    let aspace = task.address_space();
    let mut remaining = len;
    let mut src_offset = 0;
    let mut dst_offset = 0;

    while remaining > 0 {
        let current_vaddr = VirtAddr::new(src_vaddr.as_u64() + src_offset as u64);
        let page_offset = current_vaddr.as_u64() & 0xFFF;
        let copy_len = core::cmp::min(remaining, (0x1000 - page_offset) as usize);

        let src_paddr = aspace.translate(current_vaddr).ok_or(BufferError::AddressNotMapped)?;
        
        let flags = aspace.get_page_flags(current_vaddr).ok_or(BufferError::AddressNotMapped)?;
        if !flags.contains(PageFlags::USER) || !flags.contains(PageFlags::PRESENT) {
            return Err(BufferError::PermissionDenied);
        }

        let src_kernel_ptr = crate::mm::vmm::kernel_direct_map_vaddr(src_paddr)
            .as_ptr::<u8>()
            .add(page_offset as usize);

        ptr::copy_nonoverlapping(
            src_kernel_ptr,
            dst_kernel_ptr.add(dst_offset),
            copy_len
        );

        src_offset += copy_len;
        dst_offset += copy_len;
        remaining -= copy_len;
    }

    Ok(())
}

unsafe fn safe_copy_to_user(
    task: &mut Task,
    dst_vaddr: VirtAddr,
    src_kernel_ptr: *const u8,
    len: usize
) -> Result<(), BufferError> {
    let aspace = task.address_space_mut();
    let mut remaining = len;
    let mut dst_offset = 0;
    let mut src_offset = 0;

    while remaining > 0 {
        let current_vaddr = VirtAddr::new(dst_vaddr.as_u64() + dst_offset as u64);
        let page_offset = current_vaddr.as_u64() & 0xFFF;
        let copy_len = core::cmp::min(remaining, (0x1000 - page_offset) as usize);

        let dst_paddr = aspace.translate(current_vaddr).ok_or(BufferError::AddressNotMapped)?;
        
        let flags = aspace.get_page_flags(current_vaddr).ok_or(BufferError::AddressNotMapped)?;
        if !flags.contains(PageFlags::USER) || !flags.contains(PageFlags::PRESENT) || !flags.contains(PageFlags::WRITABLE) {
            return Err(BufferError::PermissionDenied);
        }

        let dst_kernel_ptr = crate::mm::vmm::kernel_direct_map_vaddr(dst_paddr)
            .as_mut_ptr::<u8>()
            .add(page_offset as usize);

        ptr::copy_nonoverlapping(
            src_kernel_ptr.add(src_offset),
            dst_kernel_ptr,
            copy_len
        );

        dst_offset += copy_len;
        src_offset += copy_len;
        remaining -= copy_len;
    }

    Ok(())
}

pub fn revoke_ipc_buffer_capabilities(cnode: &mut CNode, buffer_slot_id: u64) {
    for i in 0..cnode.capacity() {
        if let Some(cap) = cnode.lookup_cap(i) {
            if cap.object_type == crate::caps::types::ObjectType::IpcBuffer && cap.object_id == buffer_slot_id {
                cnode.revoke_cap(i);
            }
        }
    }
}

pub fn cleanup_task_ipc_buffers(pool: &IpcBufferPool, task_id: u64) {
    for i in 0..1024 {
        pool.with_slot(i as u64, |slot| {
            if slot.owner_task_id == task_id {
                slot.state.store(IpcBufferSlot::STATE_FREE, Ordering::Release);
                for frame in slot.physical_frames.iter_mut() {
                    *frame = None;
                }
                slot.descriptor.size_bytes = 0;
            }
        });
    }
}

fn arch_flush_tlb_single(task_id: u64, vaddr: VirtAddr) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) vaddr.as_u64(), options(nostack, preserves_flags));
    }
    
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!("sfence.vma {}, zero", in(reg) vaddr.as_u64(), options(nostack, preserves_flags));
    }
}

fn arch_clean_invalidate_dcache_region(vaddr: VirtAddr, size: u64) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut addr = vaddr.as_u64();
        let end = addr + size;
        while addr < end {
            core::arch::asm!(
                "dc civac, {}",
                in(reg) addr,
                options(nostack, preserves_flags)
            );
            addr += 64; 
        }
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
}

pub fn sys_ipc_buffer_register(
    task: &mut Task, 
    pool: &IpcBufferPool,
    desc_ptr: VirtAddr
) -> Result<u64, DsError> {
    let mut desc = IpcBufferDescriptor {
        base_vaddr: VirtAddr::new(0),
        size_bytes: 0,
        flags: BufferFlags::empty(),
        page_count: 0,
        _pad: 0,
    };

    unsafe {
        let desc_kernel_ptr = crate::mm::vmm::kernel_direct_map_vaddr(
            task.address_space().translate(desc_ptr).ok_or(DsError::InvalidMessage)?
        );
        
        ptr::copy_nonoverlapping(
            desc_kernel_ptr.as_ptr::<IpcBufferDescriptor>(),
            &mut desc,
            1
        );
    }

    let slot_id = pool.allocate_slot(task.id).map_err(|e| DsError::from(e))?;
    pool.register_buffer(slot_id, task, &desc).map_err(|e| DsError::from(e))?;

    Ok(slot_id)
}

pub fn sys_ipc_buffer_unregister(
    task: &mut Task,
    pool: &IpcBufferPool,
    slot_id: u64
) -> Result<(), DsError> {
    pool.with_slot(slot_id, |slot| {
        if slot.owner_task_id != task.id {
            return Err(BufferError::PermissionDenied);
        }
        if slot.state.load(Ordering::Acquire) == IpcBufferSlot::STATE_IN_TRANSFER {
            return Err(BufferError::CapabilityMissing); 
        }
        
        slot.state.store(IpcBufferSlot::STATE_FREE, Ordering::Release);
        for frame in slot.physical_frames.iter_mut() {
            *frame = None;
        }
        Ok(())
    }).ok_or(DsError::InvalidHandle)?
    .map_err(|e| DsError::from(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_alignment_check() {
        let desc = IpcBufferDescriptor {
            base_vaddr: VirtAddr::new(0x1000),
            size_bytes: 0x2000,
            flags: BufferFlags::READABLE | BufferFlags::WRITABLE,
            page_count: 0,
            _pad: 0,
        };
        assert_eq!(desc.base_vaddr.as_u64() % IPC_BUFFER_ALIGNMENT as u64, 0);
    }
}