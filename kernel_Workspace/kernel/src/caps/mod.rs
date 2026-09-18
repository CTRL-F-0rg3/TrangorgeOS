use alloc::vec::Vec;
use crate::println;

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

pub fn init() -> Result<(), &'static str> {
    store::init_store()?;
    audit::init_audit_log()?;
    defaults::install_defaults()?;
    Ok(())
}

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

pub fn self_test() -> crate::testing::TestResult {
    let mut ok = true;

    let mut chk = |step: u32, cond: bool| -> bool {
        if !cond {
            println!("[caps] self_test step {} FAILED", step);
        }
        cond
    };

    let a = CapabilitySet::empty().add(Capability::User);
    let b = CapabilitySet::single(Capability::Driver);
    ok &= chk(101, a.union(b).count() == 2);
    ok &= chk(102, a.intersect(b).is_empty());
    ok &= chk(103, CapabilitySet::all().contains(a));

    ok &= chk(201, hierarchy::implies(Capability::Root, Capability::DevPci));
    ok &= chk(202, !hierarchy::implies(Capability::DevPci, Capability::Driver));
    ok &= chk(203, hierarchy::depth(Capability::DevPci) == 3);

    let wid = match store::register_world(None, sets::presets::standard_user()) {
        Ok(id) => id,
        Err(_) => return Err("store: register_world failed"),
    };
    ok &= chk(301, store::world_has_cap(wid, Capability::FsRead));
    ok &= chk(302, !store::world_has_cap(wid, Capability::DevPci));

    ok &= chk(
        303,
        store::register_world(Some(wid), CapabilitySet::single(Capability::DevPci)).is_err(),
    );

    let kw = check::kernel_world_id();
    ok &= chk(401, grant::grant_cap(kw, wid, Capability::DevMmio).is_ok());
    ok &= chk(402, store::world_has_cap(wid, Capability::DevMmio));
    ok &= chk(403, revoke::revoke_from_world(wid, Capability::DevMmio).is_ok());
    ok &= chk(404, !store::world_has_cap(wid, Capability::DevMmio));

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

    ok &= crate::policy::decide(kw, crate::policy::cmd(crate::policy::CLS_NET, 1), 0).is_ok();
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_SYS, 0), 0).is_ok();
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_VIDEO, 1), 0).is_err();
    ok &= crate::policy::decide(wid, crate::policy::cmd(crate::policy::CLS_NET, 1), 0).is_err();
    ok &= crate::policy::decide(
        wid,
        crate::policy::cmd(crate::policy::CLS_BLOCK, crate::policy::BLK_WRITE),
        0,
    )
    .is_err();

    ok &= chk(701, audit::count() > 0);
    ok &= chk(702, crate::policy::total() > 0);
    ok &= chk(703, crate::policy::denies() > 0);

    let _ = store::unregister_world(wid);

    if ok {
        Ok("sets+hierarchy+store+grant/revoke+policy+audit")
    } else {
        Err("unified permission self-check failed")
    }
}