use crate::Rights;
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ObjectId(pub u64);

impl ObjectId {
    pub const INVALID: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectType {
    Memory = 0,
    Endpoint = 1,
    Notification = 2,
    Channel = 3,
    ShmemRegion = 4,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ObjectMeta {
    pub id: ObjectId,
    pub kind: ObjectType,
    pub rights: Rights,
    pub ref_count: u32,
}

pub struct MemoryObject {
    pub id: ObjectId,
    pub phys_base: u64,
    pub page_count: usize,
    pub ref_count: AtomicU32,
}

impl MemoryObject {
    pub fn new(id: ObjectId, phys_base: u64, page_count: usize) -> Self {
        Self {
            id,
            phys_base,
            page_count,
            ref_count: AtomicU32::new(1),
        }
    }

    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn release(&self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::Release) == 1
    }
}

pub struct EndpointObject {
    pub id: ObjectId,
    pub paired_endpoint: ObjectId,
    pub ring_obj_id: ObjectId,
    pub ref_count: AtomicU32,
}

impl EndpointObject {
    pub fn new(id: ObjectId, paired: ObjectId, ring_obj: ObjectId) -> Self {
        Self {
            id,
            paired_endpoint: paired,
            ring_obj_id: ring_obj,
            ref_count: AtomicU32::new(1),
        }
    }

    pub fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn release(&self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::Release) == 1
    }
}

pub struct NotificationObject {
    pub id: ObjectId,
    pub word: AtomicU32,
}

impl NotificationObject {
    pub fn new(id: ObjectId) -> Self {
        Self {
            id,
            word: AtomicU32::new(0),
        }
    }

    pub fn signal(&self, bits: u32) {
        self.word.fetch_or(bits, Ordering::Release);
    }

    pub fn wait_and_clear(&self, mask: u32) -> u32 {
        loop {
            let val = self.word.load(Ordering::Acquire);
            if val & mask != 0 {
                let prev = self.word.fetch_and(!mask, Ordering::AcqRel);
                if prev & mask != 0 {
                    return prev & mask;
                }
            }
            core::hint::spin_loop();
        }
    }
}

pub enum KernelObject {
    Memory(MemoryObject),
    Endpoint(EndpointObject),
    Notification(NotificationObject),
}

impl KernelObject {
    pub fn id(&self) -> ObjectId {
        match self {
            Self::Memory(o) => o.id,
            Self::Endpoint(o) => o.id,
            Self::Notification(o) => o.id,
        }
    }

    pub fn kind(&self) -> ObjectType {
        match self {
            Self::Memory(_) => ObjectType::Memory,
            Self::Endpoint(_) => ObjectType::Endpoint,
            Self::Notification(_) => ObjectType::Notification,
        }
    }
}