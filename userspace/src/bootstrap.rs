//! Creating the userspace tree on first boot.
//!
//! # Why an abstraction over the volume
//!
//! The real volume is TFS in the kernel, reached over a shared-memory bridge.
//! Reaching it from a host test is impossible, and reaching it from the kernel
//! during boot is the one moment where a bug is hardest to see. So
//! [`Volume`] is a trait, the real implementation forwards to TFS, and
//! [`MemoryVolume`] implements it against a map. The bootstrap logic below is
//! the same code in both cases — which is the point: the tree that gets tested
//! is the tree that ships.
//!
//! # Creation is idempotent
//!
//! First boot and every later boot run the same code. [`Volume::mkdir`] is
//! therefore asked to succeed when the directory already exists, and
//! [`build_tree`] reports how much it actually changed so a caller can tell a
//! fresh install from an existing one.

use std::collections::BTreeMap;

use crate::layout;

/// A directory the system can create and inspect.
///
/// Deliberately small: only what the bootstrap needs, so a test double is a
/// map and the real one forwards six calls to TFS.
pub trait Volume {
    /// Create `path` and every missing parent. Succeeds if it already exists.
    fn mkdir(&mut self, path: &str) -> Result<(), String>;

    /// Whether `path` exists as a directory.
    fn exists(&mut self, path: &str) -> Result<bool, String>;
}

/// What a [`build_tree`] run did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TreeReport {
    /// Directories created by this run.
    pub created: Vec<String>,
    /// Directories that were already there.
    pub existed: Vec<String>,
    /// Directories that could not be created, with the reason.
    pub failed: Vec<(String, String)>,
}

impl TreeReport {
    /// Whether every required directory is present afterwards.
    ///
    /// `failed` is the honest test: a run that reported no errors but produced
    /// nothing is a bug in the volume, not a success.
    pub fn is_complete(&self) -> bool {
        self.failed.is_empty() && self.created.len() + self.existed.len() == layout::required_dirs().len()
    }

    /// Whether this was a fresh install rather than an existing system.
    pub fn is_fresh_install(&self) -> bool {
        !self.created.is_empty() && self.existed.is_empty()
    }
}

/// Create every directory in [`layout::required_dirs`], in order.
///
/// Parent directories come first in that list, so this is a single pass. A
/// failure on one directory does not abort the rest: a system missing
/// `terminal/` is still usable, and reporting every failure at once is worth
/// more than stopping at the first.
pub fn build_tree(vol: &mut impl Volume) -> TreeReport {
    let mut report = TreeReport::default();

    for dir in layout::required_dirs() {
        match vol.exists(&dir) {
            Ok(true) => {
                report.existed.push(dir);
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                report.failed.push((dir, e));
                continue;
            }
        }

        match vol.mkdir(&dir) {
            Ok(()) => report.created.push(dir),
            Err(e) => report.failed.push((dir, e)),
        }
    }

    report
}

/// An in-memory [`Volume`], for tests and for the host-side simulation.
///
/// Parent creation is recursive because that is what a real filesystem does, and
/// the test is worthless if the double is stricter or laxer than the target.
#[derive(Debug, Default)]
pub struct MemoryVolume {
    dirs: BTreeMap<String, ()>,
    /// When set, every operation fails with this message. Used to test the
    /// reporting path, which is otherwise unreachable.
    pub fail_with: Option<String>,
}

impl MemoryVolume {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every directory currently present, sorted.
    pub fn dirs(&self) -> Vec<String> {
        self.dirs.keys().cloned().collect()
    }

    /// Whether `path` is present.
    pub fn has(&self, path: &str) -> bool {
        self.dirs.contains_key(&layout::normalize(path))
    }
}

impl Volume for MemoryVolume {
    fn mkdir(&mut self, path: &str) -> Result<(), String> {
        if let Some(why) = &self.fail_with {
            return Err(why.clone());
        }
        let path = layout::normalize(path);
        if path == "/" {
            return Ok(());
        }

        // Create the parent first: `/a/b/c` implies `/a/b` and `/a`.
        if let Some(i) = path.rfind('/') {
            let parent = &path[..i];
            if !self.dirs.contains_key(parent) {
                self.mkdir(parent)?;
            }
        }
        self.dirs.insert(path, ());
        Ok(())
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        if let Some(why) = &self.fail_with {
            return Err(why.clone());
        }
        Ok(self.dirs.contains_key(&layout::normalize(path)))
    }
}
