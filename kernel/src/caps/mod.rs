//! System uprawnień (capabilities) dla TrangorgeOS.
//!
//! Każda operacja w kernelu wymaga odpowiedniej capability.
//! System jest niezależny od `policy/` - polityka decyduje co zrobić,
//! capabilities mówią co WOLNO.
//!
//! Hierarchia:
//!   CAP_ROOT > CAP_RING0 > CAP_DRIVER > CAP_USER
//!
//! Przykład użycia:
//! ```rust
//! caps::require_cap(Capability::PhysAlloc)?;
//! let frame = pmm::alloc_frame()?;
//! ```

pub mod types;
pub mod hierarchy;
pub mod sets;
pub mod store;
pub mod check;
pub mod grant;
pub mod revoke;
pub mod audit;
pub mod export;
pub mod syscalls;
pub mod policy;
pub mod defaults;

#[cfg(test)]
mod tests;

pub use types::*;
pub use hierarchy::*;
pub use sets::*;
pub use store::*;
pub use check::*;
pub use grant::*;
pub use revoke::*;
pub use audit::*;
pub use export::*;
pub use defaults::*;

/// Inicjalizacja systemu capabilities (wywołaj raz przy starcie kernela)
pub fn init() -> Result<(), &'static str> {
    store::init_store()?;
    audit::init_audit_log()?;
    defaults::install_defaults()?;
    Ok(())
}

/// Snapshot całego systemu (do debug/testów)
pub fn snapshot() -> SystemSnapshot {
    SystemSnapshot {
        global_caps: store::global_caps(),
        revoked: revoke::revoked_list(),
        audit_count: audit::count(),
        world_count: store::world_count(),
    }
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub global_caps: CapabilitySet,
    pub revoked: Vec<Capability>,
    pub audit_count: usize,
    pub world_count: usize,
}