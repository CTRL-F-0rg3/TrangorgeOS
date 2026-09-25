//! Opcodes and the per-opcode capability requirements that gate the kernel.

use crate::caps::ObjectType;
use crate::rights::Rights;

/// Opcode class (service domain), stored in the high byte of the opcode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpClass {
    Sys = 0,
    Mem = 1,
    Cap = 2,
    Ipc = 3,
    Shmem = 4,
    Video = 5,
    Audio = 6,
    Input = 7,
    Block = 8,
    Net = 9,
    Pci = 10,
    Vgpu = 11,
    Fs = 12,
}

impl OpClass {
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Sys),
            1 => Some(Self::Mem),
            2 => Some(Self::Cap),
            3 => Some(Self::Ipc),
            4 => Some(Self::Shmem),
            5 => Some(Self::Video),
            6 => Some(Self::Audio),
            7 => Some(Self::Input),
            8 => Some(Self::Block),
            9 => Some(Self::Net),
            10 => Some(Self::Pci),
            11 => Some(Self::Vgpu),
            12 => Some(Self::Fs),
            _ => None,
        }
    }
}

/// Combine a class and an operation number into a full opcode
/// (`class` in bits 8..15, `op` in bits 0..7).
pub const fn opcode(class: OpClass, op: u16) -> u32 {
    ((class as u32) << 8) | (op as u32 & 0xFF)
}

/// A decoded opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode(pub u32);

impl Opcode {
    /// Decode a raw wire value, if it names a known class.
    pub const fn from_u32(raw: u32) -> Option<Self> {
        let cls = (raw >> 8) as u8;
        if OpClass::from_u8(cls).is_some() {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn class(self) -> Option<OpClass> {
        OpClass::from_u8((self.0 >> 8) as u8)
    }

    pub const fn op(self) -> u16 {
        (self.0 & 0xFF) as u16
    }

    /// The capability requirement needed to perform this opcode.
    ///
    /// Returns `None` for opcodes that must *never* be forwarded (unknown or
    /// reserved operations); the authorization gate treats that as a hard deny.
    pub const fn required(self) -> Option<Requirement> {
        let Some(class) = self.class() else {
            return None;
        };
        let op = self.op();
        let (rights, ty) = match class {
            // System-level calls (log, yield, alloc, free, ...) only need CALL.
            OpClass::Sys => (Rights::CALL.0, None),
            OpClass::Mem => match op {
                1 => (Rights::MANAGE.0, Some(ObjectType::Memory)),          // mem_create
                3 => (Rights::MAP.0 | Rights::WRITE.0, Some(ObjectType::Memory)), // mem_map
                4 => (Rights::MAP.0, Some(ObjectType::Memory)),             // mem_unmap
                5 => (Rights::GRANT.0 | Rights::MAP.0, Some(ObjectType::Memory)), // mem_grant
                _ => (Rights::CALL.0, Some(ObjectType::Memory)),
            },
            // Capability management always requires MANAGE.
            OpClass::Cap => (Rights::MANAGE.0, None),
            OpClass::Ipc => match op {
                1 | 3 => (Rights::SEND.0, Some(ObjectType::Endpoint)), // send / call
                2 | 4 => (Rights::RECV.0, Some(ObjectType::Endpoint)), // recv / reply
                _ => (Rights::SEND.0 | Rights::RECV.0, Some(ObjectType::Endpoint)),
            },
            OpClass::Shmem => match op {
                1 => (Rights::MANAGE.0, Some(ObjectType::ShmemRegion)), // shmem_create
                _ => (Rights::MAP.0, Some(ObjectType::ShmemRegion)),    // attach / detach
            },
            // Every device-class opcode requires a CALL capability over a device.
            OpClass::Video
            | OpClass::Audio
            | OpClass::Input
            | OpClass::Block
            | OpClass::Net
            | OpClass::Pci
            | OpClass::Vgpu
            | OpClass::Fs => (Rights::CALL.0, Some(ObjectType::Device)),
        };
        Some(Requirement {
            rights: Rights(rights),
            obj_type: ty,
        })
    }
}

/// The capability a request must hold to be forwarded to the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirement {
    pub rights: Rights,
    /// If `Some`, the capability's object type must match exactly.
    pub obj_type: Option<ObjectType>,
}
