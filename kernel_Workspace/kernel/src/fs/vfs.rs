//! `vfs` — a dispatcher over the mounted filesystems.
//!
//! # Why this shape
//!
//! One enum over the mounted filesystems, one place that turns a path into
//! bytes. Every filesystem in the kernel is reachable through the same door, so
//! a caller never has to know which one it got.
//!
//! # Why the three arms differ
//!
//! The arms are not three copies of the same body, and pretending otherwise
//! hides real differences:
//!
//! * `Ext4` holds its own device and has path-aware `read_path`/`list_path`, so
//!   it needs no help.
//! * `Fat32` likewise holds its device (it was given a `dev` field for exactly
//!   this reason) and has the same pair.
//! * `Tangfs` is a set of free functions over a `&dyn BlockDevice` with no
//!   mounted state, so the path is walked by hand: split on `/`, `find_dir` for
//!   each component, `read_file` on the leaf. That is the cost of not having a
//!   mount object, and it is why the other two are preferred for new work.
//!
//! # The `is_dir` convention
//!
//! `tangfs::tfs::entries` returns `(name, size, kind)` with `kind` 1 for a file
//! and 2 for a directory. The literal is used rather than a named constant
//! because `tangfs` does not export one; `KIND_DIR_FOR_VFS` below is the single
//! place that knows it.

use crate::fs::driver::block::BlockDevice;
use crate::fs::driver::registry;
use crate::fs::ext4::Ext4;
use crate::fs::fat32::Fat32;
use crate::fs::tangfs::tfs;
use alloc::vec::Vec;

/// `tangfs`' directory kind byte, as returned by `entries`.
const KIND_DIR_FOR_VFS: u8 = 2;

pub struct FsEntry {
    pub name: Vec<u8>,
    pub is_dir: bool,
    pub size: u64,
}

pub enum Mounted {
    Ext4(Ext4),
    Fat32(Fat32),
    Tangfs(&'static dyn BlockDevice),
}

impl Mounted {
    /// Read a whole file into `buf`, returning the number of bytes written.
    ///
    /// A buffer smaller than the file truncates rather than failing, which is
    /// what a caller with a fixed-size buffer wants. `tangfs` returns a
    /// `Vec`, so its result is copied across the same limit.
    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Option<usize> {
        match self {
            Mounted::Ext4(f) => f.read_path(path, buf).ok(),
            Mounted::Fat32(f) => f.read_path(path, buf).ok(),
            Mounted::Tangfs(d) => {
                // `tfs` takes `&dyn BlockDevice`; the enum holds the reference
                // itself, so pass the pointee, not the reference-to-reference.
                let (dir, name) = split_parent(path)?;
                let dir = match dir {
                    None => tfs::ROOT_DIR,
                    Some(p) => tfs::find_dir(*d, tfs::ROOT_DIR, p).ok()?,
                };
                let data = tfs::read_file(*d, dir, name).ok()?;
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                Some(n)
            }
        }
    }

    /// List one directory.
    pub fn list_path(&self, path: &str) -> Option<Vec<FsEntry>> {
        match self {
            Mounted::Ext4(f) => f.list_path(path).ok().map(|v| {
                v.into_iter()
                    .map(|e| FsEntry {
                        name: e.name,
                        // ext4's ftype: 2 is EXT4_FT_DIR.
                        is_dir: e.ftype == 2,
                        size: 0,
                    })
                    .collect()
            }),
            Mounted::Fat32(f) => f.list_path(path).ok().map(|v| {
                v.into_iter()
                    .map(|(e, _loc)| FsEntry {
                        // `short_name` decodes the 8.3 field and inserts the dot;
                        // the raw `name` is space-padded on disk.
                        name: e.short_name(),
                        is_dir: e.is_dir(),
                        size: e.size as u64,
                    })
                    .collect()
            }),
            Mounted::Tangfs(d) => {
                let dir = if path.is_empty() || path == "/" {
                    tfs::ROOT_DIR
                } else {
                    tfs::find_dir(*d, tfs::ROOT_DIR, path).ok()?
                };
                tfs::entries(*d, dir).ok().map(|v| {
                    v.into_iter()
                        .map(|(name, size, kind)| FsEntry {
                            name: name.into_bytes(),
                            is_dir: kind == KIND_DIR_FOR_VFS,
                            size: size as u64,
                        })
                        .collect()
                })
            }
        }
    }
}

/// Split "a/b/c" into `("a/b", "c")`. `None` when there is no leaf to act on.
fn split_parent(path: &str) -> Option<(Option<&str>, &str)> {
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    match path.rfind('/') {
        Some(i) => {
            let parent = &path[..i];
            let name = &path[i + 1..];
            if name.is_empty() {
                None
            } else {
                Some((Some(parent), name))
            }
        }
        None => Some((None, path)),
    }
}

/// The mounted root, if any. Set once by [`mount_all`] during init and read-only
/// afterwards.
///
/// The write happens in `init`, before the terminal or the shell starts, and
/// nothing writes again. [`root`] is the only reader and it hands out a
/// `&'static` copy of the enum's *place*, which is sound only under that
/// invariant — if a second writer ever appears, this becomes unsound rather
/// than merely racy. A `Once`-guarded write would remove the footgun, but
/// `Once` over a non-`Copy` payload needs a `static mut` anyway, so the
/// invariant is stated here instead.
static mut ROOT_FS: Option<Mounted> = None;

/// Probe every registered block device and mount the first filesystem found.
///
/// Order is deliberate, most-specific first: ext4 and FAT32 carry magic numbers
/// and validate their geometry, so a probe failure is a real answer. `Tangfs`
/// has no magic beyond its own superblock, so it is the last resort — trying it
/// first would happily mount an ext4 volume and return its raw superblock bytes
/// as a directory.
pub fn mount_all() {
    let n = registry::count();

    for i in 0..n {
        let Some(d) = registry::get(i) else { continue };

        if let Ok(f) = Ext4::mount(d) {
            unsafe { ROOT_FS = Some(Mounted::Ext4(f)) };
            return;
        }

        if let Ok(f) = Fat32::mount(d) {
            unsafe { ROOT_FS = Some(Mounted::Fat32(f)) };
            return;
        }
    }

    // Fall back to the data disk as Tangfs. `root_device` prefers the second
    // registered device, which is where a standalone data.img is attached.
    if let Some(d) = crate::fs::root_device() {
        if tfs::read_superblock(d).is_ok() {
            unsafe { ROOT_FS = Some(Mounted::Tangfs(d)) };
        }
    }
}

/// The mounted root filesystem, if one was found.
pub fn root() -> Option<&'static Mounted> {
    // Safe under the `mount_all`-writes-once invariant documented on ROOT_FS.
    unsafe { (*core::ptr::addr_of!(ROOT_FS)).as_ref() }
}
