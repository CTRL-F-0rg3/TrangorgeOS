//! Global protocol constants — the single source of truth for the wire layout.

/// Magic value placed at the start of every message (`"TGCM"` in little endian).
pub const COMM_MAGIC: u32 = 0x5447_434D;

/// Current protocol version. Bumped on any incompatible wire change.
pub const COMM_VERSION: u8 = 1;

/// Fixed size of a single [`crate::wire::CommMsg`] slot on the wire, in bytes.
pub const COMM_MSG_SIZE: usize = 64;

/// Alignment of every [`crate::wire::CommMsg`] slot, in bytes.
pub const COMM_MSG_ALIGN: usize = 8;

/// Size of the shared-memory ring control header, in bytes.
pub const RING_CONTROL_SIZE: usize = 16;

/// Default number of message slots in a ring.
pub const RING_SLOTS_DEFAULT: usize = 256;

/// Maximum number of capability entries in a [`crate::caps::CapTable`].
pub const CAP_TABLE_SIZE: usize = 512;

/// Exact in-memory size of a [`crate::caps::CapTable`], in bytes.
pub const CAP_TABLE_BYTES: usize = core::mem::size_of::<crate::caps::CapTable>();
