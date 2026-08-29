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

use alloc::vec::Vec;

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

/// Pełny self-test złączonego systemu uprawnień (caps + policy).
/// Konwencja jak pozostałe moduły (testing::Test) — komunikat zwracany,
/// nie drukowany; run_test wypisze "caps ... [OK]" albo "[FAILED] msg".
pub fn self_test() -> crate::testing::TestResult {
    let mut ok = true;

    // 1) CapabilitySet: operacje na bitmapach
    let a = CapabilitySet::empty().add(Capability::User);
    let b = CapabilitySet::single(Capability::Driver);
    ok &= a.union(b).count() == 2;
    ok &= a.intersect(b).is_empty();
    ok &= CapabilitySet::all().contains(a);

    // 2) hierarchia: parent implikuje child
    ok &= hierarchy::implies(Capability::Root, Capability::DevPci);
    ok &= !hierarchy::implies(Capability::DevPci, Capability::Driver);
    ok &= hierarchy::depth(Capability::DevPci) == 3;

    // 3) store: rejestracja worlda user + ograniczenia dziedziczenia
    let wid = match store::register_world(None, sets::presets::standard_user()) {
        Ok(id) => id,
        Err(_) => return Err("store: register_world failed"),
    };
    ok &= store::world_has_cap(wid, Capability::FsRead);
    ok &= !store::world_has_cap(wid, Capability::DevPci);
    // child nie może dostać capability, której nie ma parent
    ok &= store::register_world(Some(wid), CapabilitySet::single(Capability::DevPci))
        .is_err();

    // 4) grant/revoke (kernel przyznaje, potem odbieramy DevMmio)
    let kw = check::kernel_world_id();
    ok &= grant::grant_cap(kw, wid, Capability::DevMmio).is_ok();
    ok &= store::world_has_cap(wid, Capability::DevMmio);
    ok &= revoke::revoke_from_world(wid, Capability::DevMmio).is_ok();
    ok &= !store::world_has_cap(wid, Capability::DevMmio);

    // 5) silnik polityki (port SPARK Policy.Evaluate)
    ok &= crate::policy::evaluate(
        crate::policy::RING_KERNEL, crate::policy::CLS_NET, 1, 0,
    ) == crate::policy::ALLOW;
    ok &= crate::policy::evaluate(
        crate::policy::RING_USER, crate::policy::CLS_NET, 1, 0,
    ) == crate::policy::DENY;
    ok &= crate::policy::evaluate(
        crate::policy::RING_USER, crate::policy::CLS_BLOCK,
        crate::policy::BLK_WRITE, 0,
    ) == crate::policy::DENY;
    ok &= crate::policy::evaluate(
        crate::policy::RING_USER, crate::policy::CLS_BLOCK, 2, 0,
    ) == crate::policy::ALLOW;
    ok &= crate::policy::evaluate(
        crate::policy::RING_DRIVER, crate::policy::CLS_NET, 1, 0,
    ) == crate::policy::ALLOW;

    // 6) złączona decyzja: polityka AND capability
    ok &= crate::policy::decide(kw, crate::policy::cmd(crate::policy::CLS_NET, 1), 0).is_ok();
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_SYS, 0), 0).is_ok();
    // user bez DevMmio -> odmowa warstwy caps
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_VIDEO, 1), 0).is_err();
    // user + NET -> odmowa reguły polityki
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_NET, 1), 0).is_err();
    // user + BLOCK/BLK_WRITE -> odmowa reguły polityki
    ok &= crate::policy::decide(
        wid,
        crate::policy::cmd(crate::policy::CLS_BLOCK, crate::policy::BLK_WRITE),
        0,
    )
    .is_err();

    // 7) audit caps + dziennik polityki żyją
    ok &= audit::count() > 0;
    ok &= crate::policy::total() > 0;
    ok &= crate::policy::denies() > 0;

    let _ = store::unregister_world(wid);

    if ok {
        Ok("sets+hierarchy+store+grant/revoke+policy+audit")
    } else {
        Err("unified permission self-check failed")
    }
}