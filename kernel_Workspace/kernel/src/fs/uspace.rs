//! The kernel's half of the handoff into userspace.
//!
//! # What the kernel owns
//!
//! One directory: `/kernel/`. Everything below it belongs to userspace, which
//! grows its own tree the first time it runs. The split is not cosmetic — it is
//! what makes `/kernel/` meaningful. A kernel that created `userspace/user/…`
//! would be deciding the layout of a layer it does not implement, and a change
//! to that layout would have to go through the kernel.
//!
//! # Why the tree is built here anyway
//!
//! This is the one place where the kernel walks the shared layout, and it does
//! so *provisionally*: it creates the whole tree so a first boot has something
//! to show, and prints which paths it created. Userspace still owns the
//! contract; the kernel is only obeying it early. When userspace runs for real
//! it re-runs the same [`tg_comm::layout::required_dirs`] list and finds
//! everything already present, which is exactly the idempotent case.
//!
//! # Path handling
//!
//! `tfs::mkdir` takes a *directory inode*, not a path, so each level is walked
//! one component at a time from the root. That is the only way to reach
//! `/kernel/userspace/user/root/shell` with the current TFS.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;

use crate::fs::driver::block::BlockDevice;
use crate::fs::tangfs::tfs;
use tg_comm::layout;

/// What [`enter`] managed to create.
pub struct Handoff {
    /// Paths created by this run.
    pub created: Vec<String>,
    /// Paths that were already there.
    pub existed: Vec<String>,
    /// Paths that could not be created, with the reason.
    pub failed: Vec<(String, &'static str)>,
}

impl Handoff {
    /// Whether the whole tree is present.
    pub fn is_complete(&self) -> bool {
        self.failed.is_empty()
    }

    /// One line per outcome, for the boot log.
    pub fn report(&self, out: &mut impl Write) {
        let _ = writeln!(out, "[uspace] tree: {} created, {} present",
            self.created.len(), self.existed.len());
        for p in &self.created {
            let _ = writeln!(out, "[uspace]   + {p}");
        }
        for (p, why) in &self.failed {
            let _ = writeln!(out, "[uspace]   ! {p}: {why}");
        }
    }
}

/// Create the system tree on `dev` and report what happened.
///
/// Idempotent: a second run finds everything present and creates nothing. That
/// is the normal case, since the tree survives reboots and this runs on every
/// boot.
///
/// `find_dir` returning `NotFound` is how "this level does not exist yet" is
/// spelled, so that is the branch that creates.
pub fn enter(dev: &dyn BlockDevice) -> Handoff {
    let mut h = Handoff {
        created: Vec::new(),
        existed: Vec::new(),
        failed: Vec::new(),
    };

    // `required_dirs` is ordered parents-first, so a single pass suffices.
    for target in layout::required_dirs() {
        match walk_and_create(dev, &target) {
            Ok(true) => h.created.push(target),
            Ok(false) => h.existed.push(target),
            Err(why) => h.failed.push((target, why)),
        }
    }

    h
}

/// Resolve `path` from the root, creating each missing level.
///
/// Returns `true` if anything was created, `false` if the whole path was
/// already there.
fn walk_and_create(dev: &dyn BlockDevice, path: &str) -> Result<bool, &'static str> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut cur = tfs::ROOT_DIR;
    let mut made = false;

    for part in parts {
        match tfs::find_dir(dev, cur, part) {
            Ok(next) => cur = next,
            Err(_) => {
                // The name may already exist as a *file*, in which case mkdir
                // reports NotDir. That is a real conflict and must not be
                // swallowed as "created".
                match tfs::mkdir(dev, cur, part) {
                    Ok(()) => {
                        made = true;
                        cur = tfs::find_dir(dev, cur, part)
                            .map_err(|_| "created but not found")?;
                    }
                    Err(tfs::FsError::NotDir) => return Err("a file already has this name"),
                    Err(tfs::FsError::DiskFull) => return Err("no space left"),
                    Err(_) => return Err("mkdir failed"),
                }
            }
        }
    }

    Ok(made)
}

/// The userspace greeting, shown on entry.
///
/// Kept here as a `&str` rather than only inside the userspace shell so that
/// the kernel can print it at the moment of handoff. Two copies of this string
/// would be two things to keep in sync, and they are both saying the same thing:
/// this is the point where control passes.
pub const GREETING: &str = "hello in userspace";

/// Run the userspace login and shell on `dev`.
///
/// This is the handoff, and unlike [`enter`] it *runs* something: it asks for a
/// user name, authenticates it, and on success hands the session to the
/// userspace shell until that session ends — at which point the login prompt
/// comes back rather than the machine dropping to a dead prompt.
///
/// Returns only if there is no usable device, because a userspace with no disk
/// underneath it would be a shell whose every `ls` fails.
pub fn run(dev: &'static dyn BlockDevice) {
    crate::fs::session::run(dev);
}
