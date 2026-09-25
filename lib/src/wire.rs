//! The fixed-size message that travels over shared memory.

use crate::consts::{COMM_MAGIC, COMM_MSG_SIZE, COMM_VERSION};
use crate::kind::MsgKind;
use crate::layer::Layer;

/// A single strictly-typed, fixed-size (64 byte) message.
///
/// The layout is `#[repr(C)]` and byte-for-byte stable; C, Odin and Ada each
/// mirror this exact structure (see `lib-C/include/tgcomm.h`,
/// `lib-odin/tgcomm.odin`, `lib-ada/tg_comm.ads`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommMsg {
    pub magic: u32,
    pub version: u8,
    pub kind: u8,
    pub layer: u8,
    pub target: u8,
    pub opcode: u32,
    pub cap: u32,
    pub seq: u32,
    pub status: i32,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
    pub a3: u64,
    pub _reserved: [u8; 8],
}

// Compile-time guarantee that the wire layout is exactly 64 bytes and 8-aligned.
const _: () = {
    assert!(core::mem::size_of::<CommMsg>() == COMM_MSG_SIZE);
    assert!(core::mem::align_of::<CommMsg>() == 8);
};

impl CommMsg {
    /// Build a zeroed message with the magic and version set.
    pub const fn new(kind: MsgKind, layer: Layer, target: Layer, opcode: u32) -> Self {
        Self {
            magic: COMM_MAGIC,
            version: COMM_VERSION,
            kind: kind as u8,
            layer: layer as u8,
            target: target as u8,
            opcode,
            cap: u32::MAX,
            seq: 0,
            status: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            _reserved: [0; 8],
        }
    }

    /// Basic structural validation (magic + version + decodable layers).
    pub fn well_formed(&self) -> bool {
        self.magic == COMM_MAGIC
            && self.version == COMM_VERSION
            && Layer::from_u8(self.layer).is_some()
            && Layer::from_u8(self.target).is_some()
    }
}

impl Default for CommMsg {
    fn default() -> Self {
        Self::new(MsgKind::Request, Layer::Kernel, Layer::Kernel, 0)
    }
}
