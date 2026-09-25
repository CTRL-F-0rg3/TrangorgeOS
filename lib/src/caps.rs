//! Capability identifiers, object types, entries and the capability table.

use crate::consts::CAP_TABLE_SIZE;
use crate::rights::Rights;

/// An opaque capability handle (an index into a [`CapTable`]).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapId(pub u32);

impl CapId {
    pub const INVALID: Self = Self(u32::MAX);
    pub const KERNEL: Self = Self(0);
    pub const MANAGER: Self = Self(1);
}

/// The kind of kernel object a capability refers to.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Memory = 0,
    Endpoint = 1,
    Notification = 2,
    Channel = 3,
    ShmemRegion = 4,
    Device = 5,
    Irq = 6,
}

impl ObjectType {
    /// Convert a raw wire byte into an `ObjectType`, if it is valid.
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Memory),
            1 => Some(Self::Endpoint),
            2 => Some(Self::Notification),
            3 => Some(Self::Channel),
            4 => Some(Self::ShmemRegion),
            5 => Some(Self::Device),
            6 => Some(Self::Irq),
            _ => None,
        }
    }
}

/// A single capability entry: the object plus the rights its holder possesses.
///
/// `#[repr(C)]` with a fixed 16-byte layout so it can be mirrored byte-for-byte
/// in C, Odin and Ada. `valid == 0` marks an empty slot.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapEntry {
    pub obj_id: u64,
    pub obj_type: ObjectType,
    pub valid: u8,
    pub _pad: [u8; 2],
    pub rights: Rights,
}

impl CapEntry {
    /// An empty (invalid) slot.
    pub const EMPTY: Self = Self {
        obj_id: 0,
        obj_type: ObjectType::Memory,
        valid: 0,
        _pad: [0; 2],
        rights: Rights::NONE,
    };

    /// A live capability entry.
    pub const fn new(obj_id: u64, obj_type: ObjectType, rights: Rights) -> Self {
        Self {
            obj_id,
            obj_type,
            valid: 1,
            _pad: [0; 2],
            rights,
        }
    }
}

/// A fixed-size capability table (one per communication context).
///
/// It is a strict *allow-list*: the absence of an entry, or insufficient
/// rights on the entry, is always interpreted as "deny". The layout is
/// `#[repr(C)]` so the C / Odin / Ada clients can reserve its exact storage.
#[repr(C)]
pub struct CapTable {
    entries: [CapEntry; CAP_TABLE_SIZE],
    next: u32,
    _pad: u32,
}

impl CapTable {
    pub const fn new() -> Self {
        Self {
            entries: [CapEntry::EMPTY; CAP_TABLE_SIZE],
            next: 2,
            _pad: 0,
        }
    }

    /// Re-initialize the table in place (used by the C ABI bootstrap).
    pub fn init(&mut self) {
        *self = Self::new();
    }

    /// Insert a live entry and return its fresh handle.
    pub fn insert(&mut self, entry: CapEntry) -> Option<CapId> {
        let idx = self.next as usize;
        if idx >= CAP_TABLE_SIZE {
            return None;
        }
        self.next += 1;
        self.entries[idx] = entry;
        Some(CapId(idx as u32))
    }

    /// Insert (or overwrite) an entry at a specific handle slot.
    pub fn insert_at(&mut self, id: CapId, entry: CapEntry) -> bool {
        let idx = id.0 as usize;
        if idx >= CAP_TABLE_SIZE {
            return false;
        }
        self.entries[idx] = entry;
        true
    }

    /// Look up a live entry by handle.
    pub fn get(&self, id: CapId) -> Option<&CapEntry> {
        let idx = id.0 as usize;
        if idx >= CAP_TABLE_SIZE {
            return None;
        }
        let e = &self.entries[idx];
        if e.valid == 0 {
            None
        } else {
            Some(e)
        }
    }

    /// Remove an entry, returning it if it was live.
    pub fn remove(&mut self, id: CapId) -> Option<CapEntry> {
        let idx = id.0 as usize;
        if idx >= CAP_TABLE_SIZE {
            return None;
        }
        let e = self.entries[idx];
        if e.valid == 0 {
            return None;
        }
        self.entries[idx].valid = 0;
        Some(e)
    }

    /// Whether `id` holds at least `required` rights.
    pub fn check_rights(&self, id: CapId, required: Rights) -> bool {
        match self.get(id) {
            Some(e) => e.rights.contains(required),
            None => false,
        }
    }

    /// Whether `id` holds at least `required` rights over an object of `ty`
    /// (when `ty` is `Some`), and the entry is live.
    pub fn check(&self, id: CapId, required: Rights, ty: Option<ObjectType>) -> bool {
        match self.get(id) {
            Some(e) => e.rights.contains(required) && ty.map_or(true, |t| e.obj_type == t),
            None => false,
        }
    }
}
