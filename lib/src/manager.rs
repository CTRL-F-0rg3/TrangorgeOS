//! The driver manager (zarządca) facade built on the communication core.

use crate::caps::{CapEntry, CapId, CapTable, ObjectType};
use crate::filter::{authorize, AuthorizeResult};
use crate::rights::Rights;
use crate::wire::CommMsg;

/// The privileged manager owns a capability table and brokers access for the
/// layers it governs. It never touches hardware directly; it only grants,
/// revokes and routes capability-checked requests.
pub struct Manager {
    table: CapTable,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            table: CapTable::new(),
        }
    }

    pub fn table(&self) -> &CapTable {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut CapTable {
        &mut self.table
    }

    /// Grant a capability and return its fresh handle.
    pub fn grant(&mut self, obj_id: u64, obj_type: ObjectType, rights: Rights) -> Option<CapId> {
        self.table.insert(CapEntry::new(obj_id, obj_type, rights))
    }

    /// Grant into a specific handle slot (e.g. [`CapId::KERNEL`]).
    pub fn grant_at(
        &mut self,
        id: CapId,
        obj_id: u64,
        obj_type: ObjectType,
        rights: Rights,
    ) -> bool {
        self.table.insert_at(id, CapEntry::new(obj_id, obj_type, rights))
    }

    /// Revoke a capability. Subsequent requests using it are denied.
    pub fn revoke(&mut self, id: CapId) -> Option<CapEntry> {
        self.table.remove(id)
    }

    /// Run the same authorization gate the kernel uses, so the manager can
    /// pre-validate requests before forwarding them.
    pub fn authorize(&self, msg: &CommMsg) -> AuthorizeResult {
        authorize(&self.table, msg)
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
