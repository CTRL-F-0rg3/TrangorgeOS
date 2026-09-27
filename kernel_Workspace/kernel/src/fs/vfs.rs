//! `vfs` — a dispatcher over the mounted filesystems.
//!
//! Not compiled. The file predates the `tangfs` move and was never wired into
//! `fs/mod.rs`, so nothing has ever type-checked it. Two things are known to be
//! wrong even before compilation is attempted:
//!
//! * It calls `tangfs::tfs::read_file(dev, dir, path, buf)` and
//!   `tangfs::tfs::list_dir(...)`, but [`tangfs::tfs`] exports `read_file`
//!   returning a `Vec<u8>` and calls the listing `entries`. The signatures do not
//!   match what this file passes.
//! * `Mounted::Tfs` holds only a `&'static dyn BlockDevice`, so it has to
//!   re-resolve every path from the root on each call. That is the cost of not
//!   keeping mounted state, and it is why `self_test` drives `tfs` directly
//!   rather than going through here.
//!
//! It is kept because the shape is the right one — one enum over the mounted
//! filesystems, one place that dispatches a path — and because FAT32 and ext4
//! both have to be reachable through the same door. It needs rewriting against
//! the current `tfs` API, not deleting.

use crate::fs::ext4::Ext4;
use crate::fs::fat32::Fat32;
use crate::fs::driver::registry;
use alloc::vec::Vec;

pub struct FsEntry {
    pub name: Vec<u8>,
    pub is_dir: bool,
    pub size: u64,
}

pub enum Mounted {
    Ext4(Ext4),
    Fat32(Fat32),
    Tfs(&'static dyn BlockDevice),
}

impl Mounted {
    pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Option<usize> {
        match self {
            Mounted::Ext4(f) => f.read_path(path, buf).ok(),
            Mounted::Fat32(f) => f.read_path(path, buf).ok(),
            Mounted::Tfs(d) => tangfs::tfs::read_file(*d, tangfs::tfs::ROOT_DIR, path, buf).ok(),
        }
    }

    pub fn list_path(&self, path: &str) -> Option<Vec<FsEntry>> {
        match self {
            Mounted::Ext4(f) => f.list_path(path).ok().map(|v| {
                v.into_iter().map(|e| FsEntry {
                    name: e.name,
                    is_dir: e.ftype == 2,
                    size: 0,
                }).collect()
            }),
            Mounted::Fat32(f) => f.list_path(path).ok().map(|v| {
                v.into_iter().map(|e| FsEntry {
                    name: e.name,
                    is_dir: e.is_dir,
                    size: e.size,
                }).collect()
            }),
            Mounted::Tfs(d) => tangfs::tfs::list_dir(*d, tangfs::tfs::ROOT_DIR, path).ok().map(|v| {
                v.into_iter().map(|e| FsEntry {
                    name: e.name,
                    is_dir: e.is_dir,
                    size: e.size,
                }).collect()
            }),
        }
    }
}

static mut ROOT_FS: Option<Mounted> = None;

pub fn mount_all() {
    let n = registry::count();

    for i in 0..n {
        if let Some(d) = registry::get(i) {
            if let Ok(f) = Ext4::mount(d) {
                unsafe { ROOT_FS = Some(Mounted::Ext4(f)) };
                return;
            }

            if let Ok(f) = Fat32::mount(d) {
                unsafe { ROOT_FS = Some(Mounted::Fat32(f)) };
                return;
            }
        }
    }
}

pub fn root() -> Option<&'static Mounted> {
    unsafe { ROOT_FS.as_ref() }
}
