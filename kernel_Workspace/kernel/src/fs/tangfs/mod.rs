//! `tangfs` — the native filesystem, in two generations.
//!
//! # Two implementations, one name
//!
//! This directory holds two versions of the same filesystem:
//!
//! * [`tfs`] — the working one. Flat directories, a bump allocator, no
//!   directories-within-directories beyond a fixed extent. 367 lines, and it is
//!   what `fs::self_test` exercises on every boot: format, write, read back,
//!   `mkdir`, `rm`, `rmdir`. **This is the one that runs.**
//!
//! * `superblock`, `btree`, `inode`, `journal`, `dir`, `file` — the next
//!   generation. A b-tree, extents, a journal for crash recovery, and an inode
//!   layer. Roughly 690 lines, and it has never compiled.
//!
//! They are the same filesystem at two stages, not two filesystems. The second
//! generation is what the bit-set directory work replaces, and the first
//! generation is what stays in the tree until it does.
//!
//! # Why the second generation is not declared here
//!
//! Because it does not build, and declaring it would take the kernel's build
//! down with it. Its known gaps, in the order they have to be closed:
//!
//! | Missing | Referenced by | Consequence |
//! |---|---|---|
//! | `extent.rs` | `mod.rs` | `E0583`, the module does not exist at all |
//! | `crate::fs::vfs` | six files | `E0432`; `vfs.rs` is not declared in `fs/mod.rs` |
//! | `crate::fs::driver::BlockDevice` | `mod.rs` | `E0432`; the path is `driver::block::BlockDevice` |
//! | `crate::time` | `mod.rs` | `E0433`; no such module in the kernel |
//!
//! 58 errors in total, none of them in [`tfs`]. Restoring these declarations is a
//! separate piece of work from moving `tfs.rs` here, and conflating the two
//! would mean a tree that does not compile at the end of either.

pub mod tfs;

// The next generation stays undeclared until it builds. See the module docs for
// what has to land first. The files are kept rather than deleted: they are the
// design, and deleting them would throw away the part worth keeping.
