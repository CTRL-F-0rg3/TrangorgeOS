//! The authorization gate: the *only* path from shared memory to the kernel.
//!
//! Every request is first parsed and validated, then checked against the
//! sender's capability table. A request is forwarded to the kernel handler
//! **if and only if** [`authorize`] returns [`AuthorizeResult::Forwarded`].
//! This is the security property proven in `lib-ada` (see `Tg_Comm_Security`).

use crate::caps::CapTable;
use crate::consts::{COMM_MAGIC, COMM_VERSION};
use crate::layer::Layer;
use crate::opcodes::Opcode;
use crate::wire::CommMsg;

/// Outcome of the authorization gate.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizeResult {
    /// Authorized: the kernel handler may act on this message.
    Forwarded = 0,
    /// Magic or version or layer bytes are invalid.
    Malformed = 1,
    /// The opcode names no known, non-reserved operation.
    UnknownOp = 2,
    /// The source layer is not allowed to talk to the target layer.
    RouteDenied = 3,
    /// The named capability does not exist.
    NoCapability = 4,
    /// The capability exists but carries insufficient rights.
    RightsInsufficient = 5,
    /// The capability's object type does not match the opcode's requirement.
    TypeMismatch = 6,
}

/// Decide whether `msg` may be forwarded to the kernel on behalf of `table`.
///
/// This function is **pure** with respect to the table (it only reads it) and
/// returns the single verdict. The kernel must never act on a message unless
/// this returns [`AuthorizeResult::Forwarded`].
pub fn authorize(table: &CapTable, msg: &CommMsg) -> AuthorizeResult {
    // 1. Structural validation (magic, version, decodable layers).
    if msg.magic != COMM_MAGIC || msg.version != COMM_VERSION {
        return AuthorizeResult::Malformed;
    }
    let Some(src) = Layer::from_u8(msg.layer) else {
        return AuthorizeResult::Malformed;
    };
    let Some(tgt) = Layer::from_u8(msg.target) else {
        return AuthorizeResult::Malformed;
    };

    // 2. Routing policy (who may talk to whom).
    if !src.may_target(tgt) {
        return AuthorizeResult::RouteDenied;
    }

    // 3. The opcode must be known and non-reserved.
    let Some(op) = Opcode::from_u32(msg.opcode) else {
        return AuthorizeResult::UnknownOp;
    };
    let Some(req) = op.required() else {
        return AuthorizeResult::UnknownOp;
    };

    // 4. The capability must exist and satisfy the requirement.
    let cap = crate::caps::CapId(msg.cap);
    match table.get(cap) {
        None => AuthorizeResult::NoCapability,
        Some(entry) => {
            if !entry.rights.contains(req.rights) {
                AuthorizeResult::RightsInsufficient
            } else if let Some(t) = req.obj_type {
                if entry.obj_type != t {
                    AuthorizeResult::TypeMismatch
                } else {
                    AuthorizeResult::Forwarded
                }
            } else {
                AuthorizeResult::Forwarded
            }
        }
    }
}

/// A sink that receives the *authorized* requests.
pub trait KernelHandler {
    /// Handle an authorized request and return its status code.
    fn handle(&mut self, msg: &CommMsg) -> i32;
}

/// The gate that owns the capability table and forwards only authorized work.
pub struct Gate<H: KernelHandler> {
    table: CapTable,
    handler: H,
}

impl<H: KernelHandler> Gate<H> {
    pub fn new(table: CapTable, handler: H) -> Self {
        Self { table, handler }
    }

    pub fn table(&self) -> &CapTable {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut CapTable {
        &mut self.table
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    /// Parse, authorize and — only if authorized — dispatch a message.
    ///
    /// The kernel handler is invoked **only** when [`authorize`] yields
    /// [`AuthorizeResult::Forwarded`]. This single point of entry enforces
    /// the invariant:
    ///
    /// ```text
    /// handler called  =>  authorize(table, msg) == Forwarded
    /// ```
    pub fn dispatch(&mut self, msg: &mut CommMsg) -> AuthorizeResult {
        match authorize(&self.table, msg) {
            AuthorizeResult::Forwarded => {
                msg.status = self.handler.handle(msg);
                AuthorizeResult::Forwarded
            }
            other => other,
        }
    }
}
