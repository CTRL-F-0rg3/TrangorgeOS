use tg_cap_core::{ObjectId, Rights};
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Handle(pub u32);

impl Handle {
    pub const INVALID: Self = Self(u32::MAX);
    pub const KERNEL: Self = Self(0);
    pub const MANAGER: Self = Self(1);
}

#[derive(Debug, Clone, Copy)]
pub struct CapSlot {
    pub obj_id: ObjectId,
    pub rights: Rights,
    pub offset_pages: u32,
    pub length_pages: u32,
}

impl CapSlot {
    pub fn new(obj_id: ObjectId, rights: Rights, offset: u32, length: u32) -> Self {
        Self {
            obj_id,
            rights,
            offset_pages: offset,
            length_pages: length,
        }
    }

    pub fn derive(&self, new_rights: Rights, offset: u32, length: u32) -> Option<Self> {
        if !self.rights.contains(new_rights) {
            return None;
        }
        if offset < self.offset_pages
            || (offset + length) > (self.offset_pages + self.length_pages)
        {
            return None;
        }
        Some(Self {
            obj_id: self.obj_id,
            rights: new_rights,
            offset_pages: offset,
            length_pages: length,
        })
    }
}

const TABLE_SIZE: usize = 512;

pub struct HandleTable {
    slots: [Option<CapSlot>; TABLE_SIZE],
    next: AtomicU32,
}

impl HandleTable {
    pub const fn new() -> Self {
        Self {
            slots: [None; TABLE_SIZE],
            next: AtomicU32::new(2),
        }
    }

    pub fn insert(&mut self, slot: CapSlot) -> Option<Handle> {
        let h = self.next.fetch_add(1, Ordering::Relaxed) as usize;
        if h >= TABLE_SIZE {
            return None;
        }
        self.slots[h] = Some(slot);
        Some(Handle(h as u32))
    }

    pub fn get(&self, handle: Handle) -> Option<&CapSlot> {
        let idx = handle.0 as usize;
        if idx >= TABLE_SIZE {
            return None;
        }
        self.slots[idx].as_ref()
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut CapSlot> {
        let idx = handle.0 as usize;
        if idx >= TABLE_SIZE {
            return None;
        }
        self.slots[idx].as_mut()
    }

    pub fn remove(&mut self, handle: Handle) -> Option<CapSlot> {
        let idx = handle.0 as usize;
        if idx >= TABLE_SIZE {
            return None;
        }
        self.slots[idx].take()
    }

    pub fn check_rights(&self, handle: Handle, required: Rights) -> bool {
        match self.get(handle) {
            Some(slot) => slot.rights.contains(required),
            None => false,
        }
    }
}