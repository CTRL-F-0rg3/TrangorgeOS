//! The process's open-file table.
//!
//! `std` numbers its descriptors and expects the usual invariants: 0/1/2 are
//! the standard streams, a descriptor is a small dense integer, and using one
//! after `close` gives `EBADF` rather than reading someone else's file. Linux
//! gets the second half for free from the kernel's per-process table; here the
//! kernel hands back whatever descriptor number it likes, so userspace has to
//! keep the bookkeeping itself — which is what this module is.
//!
//! # Why not just forward to the kernel?
//!
//! Because a TrangorgeOS kernel that implements `openat` for one process still
//! has to answer "what is descriptor 7?" on every `read`, and the natural place
//! for that answer is the side that already has to know which paths the
//! capability model let through. Keeping the table here also means the
//! descriptor-number invariants `std` relies on are testable without a kernel.

use alloc::vec::Vec;

use crate::errno::{self, Result};

/// Standard input.
pub const STDIN: i32 = 0;
/// Standard output.
pub const STDOUT: i32 = 1;
/// Standard error.
pub const STDERR: i32 = 2;

/// How many descriptors a process may hold open.
///
/// Deliberately small and fixed. A table of this size lives in `.bss`, so it is
/// available before the heap is, and a program that wants more than this is a
/// program the kernel's own limit has to agree with anyway.
pub const MAX_FDS: usize = 1024;

/// What a descriptor refers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// The three standard streams, and anything else userspace only passes on.
    Stream,
    /// A regular file open at `pos` bytes.
    File {
        /// The path it was opened with, kept for diagnostics and for `dup`.
        path: Vec<u8>,
        /// The next byte `read` will consume.
        pos: u64,
        /// The flags it was opened with, so `O_APPEND` survives a `dup`.
        flags: u32,
    },
    /// A directory being iterated.
    Dir {
        /// The path it was opened with.
        path: Vec<u8>,
        /// The `d_off` to resume from.
        cursor: i64,
    },
    /// A socket, passed through to the network syscalls untouched.
    Socket,
    /// An event-poll set.
    Poll,
    /// A pipe end.
    Pipe,
    /// A descriptor whose kind userspace deliberately does not model.
    Other,
}

/// One entry of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// What the descriptor refers to.
    pub kind: Kind,
    /// How many names refer to the same underlying object, as far as userspace
    /// can tell. Informational: the *real* sharing is the kernel's open file
    /// description, and [`Table::close`] unlinks a name whether this is 1 or 10.
    pub shared: u32,
}

/// The process's open-file table.
#[derive(Debug)]
pub struct Table {
    slots: Vec<Option<Slot>>,
    /// The lowest descriptor number not currently in use. Reused rather than
    /// bumped, so a program that opens and closes one file forever does not walk
    /// `next_free` up to `MAX_FDS`.
    next_free: i32,
    /// How many names are in the table, counting the standard streams.
    open: usize,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// An empty table with 0, 1 and 2 reserved as the standard streams.
    pub fn new() -> Self {
        let mut slots: Vec<Option<Slot>> = Vec::new();
        slots.resize_with(MAX_FDS, || None);
        for fd in 0..3 {
            slots[fd as usize] = Some(Slot {
                kind: Kind::Stream,
                shared: 1,
            });
        }
        Self {
            slots,
            next_free: 3,
            open: 3,
        }
    }

    /// How many descriptor names are in use, counting the standard streams.
    pub fn open_count(&self) -> usize {
        self.open
    }

    /// The lowest free descriptor number.
    pub fn next_free(&self) -> i32 {
        self.next_free
    }

    /// Whether `fd` names something.
    pub fn is_open(&self, fd: i32) -> bool {
        self.get(fd).is_ok()
    }

    /// The slot for `fd`, or [`errno::EBADF`].
    pub fn get(&self, fd: i32) -> Result<&Slot> {
        self.slots
            .get(fd as usize)
            .and_then(Option::as_ref)
            .ok_or(errno::EBADF)
    }

    /// The slot for `fd`, mutably, or [`errno::EBADF`].
    pub fn get_mut(&mut self, fd: i32) -> Result<&mut Slot> {
        self.slots
            .get_mut(fd as usize)
            .and_then(Option::as_mut)
            .ok_or(errno::EBADF)
    }

    /// What `fd` refers to, or [`errno::EBADF`].
    pub fn kind(&self, fd: i32) -> Result<Kind> {
        Ok(self.get(fd)?.kind.clone())
    }

    /// Claim the lowest free descriptor for `kind`.
    ///
    /// Returns [`errno::EMFILE`] when the table is full, which is what `std`
    /// turns into "too many open files" rather than a panic.
    pub fn alloc(&mut self, kind: Kind) -> Result<i32> {
        let from = (self.next_free as usize).min(MAX_FDS);
        let Some(at) = (from..MAX_FDS).find(|&i| self.slots[i].is_none()) else {
            return Err(errno::EMFILE);
        };

        self.slots[at] = Some(Slot { kind, shared: 1 });
        self.next_free = at as i32;
        self.open += 1;
        Ok(at as i32)
    }

    /// Release the name `fd`.
    ///
    /// The name is *always* unlinked, even when [`Slot::shared`] says other
    /// names refer to the same object. That is what POSIX says a `close` does:
    /// the descriptor stops existing immediately, and the open file description
    /// behind it survives until its last descriptor goes away. A table where
    /// `close` merely decremented a counter would leave the closed number
    /// usable, and `std` checks that it is not.
    pub fn close(&mut self, fd: i32) -> Result<()> {
        // Validates the descriptor before anything is touched.
        self.get(fd)?;
        self.slots[fd as usize] = None;
        self.open -= 1;
        if fd < self.next_free {
            self.next_free = fd;
        }
        Ok(())
    }

    /// Give `fd` a second name, at the lowest free descriptor.
    ///
    /// Both names refer to the same underlying object, so closing one does not
    /// close the other: the share count is what enforces that.
    pub fn dup(&mut self, fd: i32) -> Result<i32> {
        let kind = self.kind(fd)?;
        let new = self.alloc(kind)?;
        if let Some(slot) = self.slots.get_mut(fd as usize).and_then(Option::as_mut) {
            slot.shared += 1;
        }
        Ok(new)
    }

    /// Move `fd` onto a specific number, closing whatever was there.
    pub fn dup_to(&mut self, fd: i32, newfd: i32) -> Result<()> {
        if !(0..MAX_FDS as i32).contains(&newfd) {
            return Err(errno::EBADF);
        }
        if newfd == fd {
            return Ok(());
        }
        let kind = self.kind(fd)?;
        // `dup2` releases the target name outright, so it drops the slot without
        // a share-count decrement.
        if self.slots[newfd as usize].is_some() {
            self.slots[newfd as usize] = None;
            self.open -= 1;
        }
        self.slots[newfd as usize] = Some(Slot { kind, shared: 1 });
        self.open += 1;
        Ok(())
    }

    /// The current file position of `fd`.
    pub fn pos(&self, fd: i32) -> Result<u64> {
        match &self.get(fd)?.kind {
            Kind::File { pos, .. } => Ok(*pos),
            _ => Err(errno::ESPIPE),
        }
    }

    /// Move the file position of `fd`.
    pub fn set_pos(&mut self, fd: i32, pos: u64) -> Result<()> {
        match &mut self.get_mut(fd)?.kind {
            Kind::File { pos: p, .. } => {
                *p = pos;
                Ok(())
            }
            _ => Err(errno::ESPIPE),
        }
    }

    /// Advance the position of `fd` by `n` bytes and return the new value.
    pub fn advance(&mut self, fd: i32, n: usize) -> Result<u64> {
        match &mut self.get_mut(fd)?.kind {
            Kind::File { pos, .. } => {
                *pos = pos.saturating_add(n as u64);
                Ok(*pos)
            }
            _ => Err(errno::ESPIPE),
        }
    }

    /// The `d_off` a directory descriptor should resume from.
    pub fn dir_cursor(&self, fd: i32) -> Result<i64> {
        match &self.get(fd)?.kind {
            Kind::Dir { cursor, .. } => Ok(*cursor),
            _ => Err(errno::ENOTDIR),
        }
    }

    /// Set the `d_off` a directory descriptor resumes from.
    pub fn set_dir_cursor(&mut self, fd: i32, cursor: i64) -> Result<()> {
        match &mut self.get_mut(fd)?.kind {
            Kind::Dir { cursor: c, .. } => {
                *c = cursor;
                Ok(())
            }
            _ => Err(errno::ENOTDIR),
        }
    }

    /// Open a regular file descriptor for `path`.
    pub fn open_file(&mut self, path: &str, flags: u32) -> Result<i32> {
        self.alloc(Kind::File {
            path: path.as_bytes().to_vec(),
            pos: 0,
            flags,
        })
    }

    /// Open a directory descriptor for `path`.
    pub fn open_dir(&mut self, path: &str) -> Result<i32> {
        self.alloc(Kind::Dir {
            path: path.as_bytes().to_vec(),
            cursor: 0,
        })
    }

    /// The path a file or directory descriptor was opened with.
    pub fn path_of(&self, fd: i32) -> Result<&[u8]> {
        match &self.get(fd)?.kind {
            Kind::File { path, .. } | Kind::Dir { path, .. } => Ok(path),
            _ => Err(errno::ENOTDIR),
        }
    }
}
