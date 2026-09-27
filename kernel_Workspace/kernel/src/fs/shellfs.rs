//! `tfs`-backed filesystem access for the userspace shell.
//!
//! # Why this lives in the kernel, not in the userspace crate
//!
//! The `tfs` implementation is inside the kernel and the userspace crate is
//! `std`. Linking one to the other is not possible today, so the adapter that
//! speaks `tfs` on one side and the shell's `Fs` trait on the other has to be
//! built against both — which means in the kernel. When the ring-3 boundary
//! becomes real this moves to userspace unchanged: the trait and the paths are
//! already the right shape for that, and only the transport changes.
//!
//! # The shape problem, and what it costs
//!
//! `tfs` addresses directories by *inode*, not by path. Every path call
//! therefore resolves the components from the root on the way in, and the
//! resolved inode is used for the operation. That is O(depth) per call, which
//! is why a production implementation would cache; here the depth is six, so
//! the cost is not worth a cache and its invalidation bug would be.
//!
//! Nothing is cached and nothing is held between calls, so a shell and the
//! kernel's own `self_test` can both write the same volume without one seeing
//! the other's stale view.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;

use super::driver::block::BlockDevice;
use super::tangfs::tfs;
use tg_comm::layout;

/// One entry in a directory listing.
///
/// Mirrors `userspace::apps::shell::DirEntry`. Duplicated rather than imported
/// for the reason in the module docs: the two crates cannot see each other yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u32,
}

/// What the shell needs from a filesystem.
///
/// The same five operations the userspace shell declares, so once the boundary
/// exists this is satisfied by implementing that trait instead.
pub trait Fs {
    fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, String>;
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String>;
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String>;
    fn make_dir(&mut self, path: &str) -> Result<(), String>;
    fn remove(&mut self, path: &str) -> Result<(), String>;
}

/// A block device, resolved once.
pub struct TfsFs<'a> {
    dev: &'a dyn BlockDevice,
    root: u32,
}

impl<'a> TfsFs<'a> {
    /// `dev`, starting at the volume root.
    pub fn new(dev: &'a dyn BlockDevice) -> Self {
        Self {
            dev,
            root: tfs::ROOT_DIR,
        }
    }

    /// Resolve a path to a directory inode.
    ///
    /// An empty path and `/` both mean the root. `..` needs no handling here:
    /// `layout::normalize` has already collapsed it, so this only walks forward.
    fn resolve_dir(&self, path: &str) -> Result<u32, String> {
        let mut cur = self.root;
        for part in layout::normalize(path).split('/') {
            if part.is_empty() {
                continue;
            }
            cur = tfs::find_dir(self.dev, cur, part)
                .map_err(|_| format!("no such directory: {part}"))?;
        }
        Ok(cur)
    }

    /// Split a path into its parent inode and its leaf name.
    fn split(&self, path: &str) -> Result<(u32, String), String> {
        let norm = layout::normalize(path);
        let name = norm
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("no file name in {path}"))?
            .to_string();
        let dir_path = match norm.rfind('/') {
            Some(0) | None => "/".to_string(),
            Some(i) => norm[..i].to_string(),
        };
        let dir = self.resolve_dir(&dir_path)?;
        Ok((dir, name))
    }
}

impl Fs for TfsFs<'_> {
    fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, String> {
        let dir = self.resolve_dir(path)?;
        let entries = tfs::entries(self.dev, dir).map_err(|e| format!("ls: {e:?}"))?;
        Ok(entries
            .into_iter()
            .map(|(name, size, kind)| DirEntry {
                name,
                // tfs kind 2 is a directory; see tfs::KIND_DIR.
                is_dir: kind == 2,
                size,
            })
            .collect())
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let (dir, name) = self.split(path)?;
        tfs::read_file(self.dev, dir, &name).map_err(|e| format!("{name}: {e:?}"))
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        let (dir, name) = self.split(path)?;
        tfs::write_file(self.dev, dir, &name, data)
            .map_err(|e| format!("{name}: {e:?}"))
    }

    fn make_dir(&mut self, path: &str) -> Result<(), String> {
        let (dir, name) = self.split(path)?;
        tfs::mkdir(self.dev, dir, &name)
            .map_err(|e| format!("mkdir {name}: {e:?}"))
    }

    fn remove(&mut self, path: &str) -> Result<(), String> {
        let (dir, name) = self.split(path)?;
        tfs::remove(self.dev, dir, &name)
            .map_err(|e| format!("rm {name}: {e:?}"))
    }
}
