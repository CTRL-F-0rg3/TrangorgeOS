//! Nested capability permissions for userspace.
//!
//! Userspace is a *nested system*: the session (root user process) holds the
//! capabilities granted by the kernel, and every app / userdriver it spawns
//! receives a **derived** (subset) capability table. A child can never hold a
//! right its parent does not hold. This is the userspace mirror of the
//! kernel's capability model (`tg_comm::CapTable`) and is consistent with it.

pub use tg_comm::{CapEntry, CapId, CapTable, ObjectType, Rights};

/// A userspace process (app or userdriver). Owns a capability table and a
/// parent link for the nested permission hierarchy.
pub struct Process {
    pub pid: u32,
    pub parent: Option<u32>,
    pub table: CapTable,
}

impl Process {
    pub fn new(pid: u32, parent: Option<u32>) -> Self {
        Self {
            pid,
            parent,
            table: CapTable::new(),
        }
    }

    /// Grant a capability into this process's table.
    pub fn grant(&mut self, id: CapId, entry: CapEntry) -> bool {
        self.table.insert_at(id, entry)
    }

    /// Whether this process holds at least `required` rights over `ty`.
    pub fn check(&self, id: CapId, required: Rights, ty: Option<ObjectType>) -> bool {
        self.table.check(id, required, ty)
    }
}

/// The userspace session: the root of the nested hierarchy. It receives the
/// kernel-granted capabilities and derives subsets for children.
pub struct Session {
    next_pid: u32,
}

impl Session {
    pub fn new() -> Self {
        Self { next_pid: 1 }
    }

    /// Spawn a child process whose capabilities are a *subset* of `parent`'s.
    ///
    /// Returns `None` if the requested grant is not a subset of the parent's
    /// holdings — this is the monotonicity check that keeps the nested system
    /// safe (and mirrors the `lib-ada` capability proof).
    pub fn spawn_derived(
        &mut self,
        parent: &Process,
        grants: &[(CapId, CapEntry)],
    ) -> Option<Process> {
        let pid = self.next_pid;
        self.next_pid += 1;

        let mut child = Process::new(pid, Some(parent.pid));

        for (id, entry) in grants {
            // A child may only hold rights the parent actually holds.
            match parent.table.get(*id) {
                Some(p) if p.rights.contains(entry.rights) && p.obj_type == entry.obj_type => {
                    child.table.insert_at(*id, *entry);
                }
                _ => return None, // denied: not a subset of parent's holdings
            }
        }

        Some(child)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
