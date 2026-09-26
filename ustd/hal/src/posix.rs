//! The typed syscall surface `std` needs, on top of the raw gate.
//!
//! Everything here is a thin, arity-checked wrapper: it puts the arguments in
//! the right registers, decodes the result, and turns pointers back into slices.
//! There is no buffering, no retry loop and no policy — that belongs in
//! `tgs-libc`, which is the layer that knows what `std` expects.
//!
//! # Argument order
//!
//! Linux order, so `mmap` is `(addr, len, prot, flags, fd, offset)` and not the
//! argument order TrangorgeOS's own `SYS_IPC_SEND` uses. Keeping the Linux order
//! means the argument lists in `SYSCALLS.md` read the same as the ones in the
//! kernel, and a future Linux binary only needs a number translation.

use crate::errno::Result;
use crate::gate::{self, Syscall};
use crate::sysnum::{self, *};

/// A `timespec`, as both the clock and the sleep syscalls pass it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Timespec {
    /// Whole seconds.
    pub sec: i64,
    /// Nanoseconds, always in `0..1_000_000_000`.
    pub nsec: i64,
}

impl Timespec {
    /// A timespec from a total number of nanoseconds.
    pub const fn from_nanos(total: i128) -> Self {
        Self {
            sec: (total / 1_000_000_000) as i64,
            nsec: (total % 1_000_000_000) as i64,
        }
    }

    /// The timespec as a total number of nanoseconds.
    pub const fn to_nanos(self) -> i128 {
        self.sec as i128 * 1_000_000_000 + self.nsec as i128
    }
}

/// An `iovec`, for `readv`/`writev`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Iovec {
    /// The buffer.
    pub base: *mut u8,
    /// Its length.
    pub len: usize,
}

/// One `pollfd`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PollFd {
    /// The descriptor to watch.
    pub fd: i32,
    /// What to wait for.
    pub events: i16,
    /// What happened.
    pub revents: i16,
}

/// A `stat` result, restricted to the fields `std` reads.
///
/// `std` asks for `STATX_BASIC_STATS` and reads `st_mode`, `st_nlink`, `st_uid`,
/// `st_gid`, `st_size`, `st_blksize`, `st_blocks` and the four timestamps. The
/// kernel fills what it has and zeroes the rest; a filesystem with no inode
/// numbers reports `st_ino == 0` and `st_nlink == 1`, which is what makes
/// `metadata.permissions().readonly()` answer "no" rather than lying.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stat {
    /// Device the inode lives on.
    pub st_dev: u64,
    /// Inode number, or 0.
    pub st_ino: u64,
    /// File mode, including the type bits in the high nibble.
    pub st_mode: u32,
    /// Hard-link count.
    pub st_nlink: u32,
    /// Owning user.
    pub st_uid: u32,
    /// Owning group.
    pub st_gid: u32,
    /// Reserved; kept so the offsets after it match `struct stat`.
    pub __pad0: u32,
    /// Device number for a special file.
    pub st_rdev: u64,
    /// Size in bytes.
    pub st_size: i64,
    /// Preferred I/O block size.
    pub st_blksize: i64,
    /// Size in 512-byte blocks.
    pub st_blocks: i64,
    /// Last access.
    pub st_atime: Timespec,
    /// Last data modification.
    pub st_mtime: Timespec,
    /// Last status change.
    pub st_ctime: Timespec,
    /// Unused, present so the trailing padding matches `struct stat`.
    pub __unused: [i64; 3],
}

impl Stat {
    /// Whether the entry is a regular file.
    pub fn is_file(&self) -> bool {
        self.st_mode & S_IFMT == S_IFREG
    }

    /// Whether the entry is a directory.
    pub fn is_dir(&self) -> bool {
        self.st_mode & S_IFMT == S_IFDIR
    }

    /// Whether any execute bit is set.
    pub fn is_executable(&self) -> bool {
        self.st_mode & 0o111 != 0
    }
}

/// `S_IFMT`: the mask that selects the file-type bits of `st_mode`.
pub const S_IFMT: u32 = 0o170000;
/// `S_IFREG`: a regular file.
pub const S_IFREG: u32 = 0o100000;
/// `S_IFDIR`: a directory.
pub const S_IFDIR: u32 = 0o040000;
/// `S_IFLNK`: a symbolic link.
pub const S_IFLNK: u32 = 0o120000;

/// The typed syscall surface, bound to one gate.
#[derive(Clone, Copy)]
pub struct Posix<'a> {
    gate: &'a dyn Syscall,
}

// `#[derive(Debug)]` would need `dyn Syscall: Debug`, and a gate has no
// business being printable anyway.
impl core::fmt::Debug for Posix<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Posix")
    }
}

impl<'a> Posix<'a> {
    /// A surface reaching the kernel through `gate`.
    pub const fn new(gate: &'a dyn Syscall) -> Self {
        Self { gate }
    }

    /// The surface for the real kernel.
    pub fn global() -> Posix<'static> {
        Posix::new(gate::global())
    }

    /// One raw call, already decoded.
    fn call(&self, num: u64, args: [u64; 6]) -> Result<usize> {
        // SAFETY: forwarding the caller's arguments verbatim; the contract of
        // `Syscall::call` is that the implementation does not dereference them.
        let raw = unsafe { self.gate.call(num, args) };
        gate::check(raw)
    }

    // ── memory ──────────────────────────────────────────────────────────────

    /// Map memory. Returns the address the kernel chose.
    pub fn mmap(
        &self,
        addr: u64,
        len: usize,
        prot: u32,
        flags: u32,
        fd: i32,
        offset: u64,
    ) -> Result<usize> {
        self.call(
            sysnum::MMAP,
            [
                addr,
                len as u64,
                prot as u64,
                flags as u64,
                fd as u32 as u64,
                offset,
            ],
        )
    }

    /// Unmap memory.
    pub fn munmap(&self, addr: usize, len: usize) -> Result<usize> {
        self.call(sysnum::MUNMAP, [addr as u64, len as u64, 0, 0, 0, 0])
    }

    /// Change page protection.
    pub fn mprotect(&self, addr: usize, len: usize, prot: u32) -> Result<usize> {
        self.call(sysnum::MPROTECT, [addr as u64, len as u64, prot as u64, 0, 0, 0])
    }

    /// Resize the break, or query it when `new` is 0.
    pub fn brk(&self, new: u64) -> Result<usize> {
        self.call(sysnum::BRK, [new, 0, 0, 0, 0, 0])
    }

    // ── descriptors ─────────────────────────────────────────────────────────

    /// Open `path` relative to `dirfd` (`-100` for "absolute").
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory, and `stat`
    /// must point at writable storage for a [`Stat`] when it is not null.
    pub unsafe fn openat(
        &self,
        dirfd: i32,
        path: *const u8,
        flags: u32,
        mode: u32,
        stat: *mut Stat,
    ) -> Result<usize> {
        // SAFETY: the caller's contract says both pointers are valid for the
        // duration of the call; nothing here retains them.
        self.call(
            sysnum::OPENAT,
            [
                dirfd as u32 as u64,
                path as u64,
                flags as u64,
                mode as u64,
                gate::addr_of(stat),
                0,
            ],
        )
    }

    /// Read up to `buf.len()` bytes from `fd`.
    pub fn read(&self, fd: i32, buf: &mut [u8]) -> Result<usize> {
        // SAFETY: `buf` is a live, exclusively borrowed slice, so the address
        // and length the kernel is told about describe writable memory for the
        // duration of the call.
        self.call(
            sysnum::READ,
            [
                fd as u32 as u64,
                gate::addr_of(buf.as_mut_ptr()),
                buf.len() as u64,
                0,
                0,
                0,
            ],
        )
    }

    /// Write `buf` to `fd`.
    pub fn write(&self, fd: i32, buf: &[u8]) -> Result<usize> {
        // SAFETY: `buf` is a live shared slice, readable for the duration of the
        // call, and the kernel is told not to write through it.
        self.call(
            sysnum::WRITE,
            [fd as u32 as u64, gate::addr_of(buf.as_ptr()), buf.len() as u64, 0, 0, 0],
        )
    }

    /// Read at an absolute offset, leaving the descriptor's position alone.
    pub fn pread(&self, fd: i32, buf: &mut [u8], offset: u64) -> Result<usize> {
        // SAFETY: as `read`, plus the offset is a plain integer.
        self.call(
            sysnum::PREAD,
            [
                fd as u32 as u64,
                gate::addr_of(buf.as_mut_ptr()),
                buf.len() as u64,
                offset,
                0,
                0,
            ],
        )
    }

    /// Write at an absolute offset, leaving the descriptor's position alone.
    pub fn pwrite(&self, fd: i32, buf: &[u8], offset: u64) -> Result<usize> {
        // SAFETY: as `write`, plus the offset is a plain integer.
        self.call(
            sysnum::PWRITE,
            [
                fd as u32 as u64,
                gate::addr_of(buf.as_ptr()),
                buf.len() as u64,
                offset,
                0,
                0,
            ],
        )
    }

    /// Close `fd`.
    pub fn close(&self, fd: i32) -> Result<usize> {
        self.call(sysnum::CLOSE, [fd as u32 as u64, 0, 0, 0, 0, 0])
    }

    /// Reposition `fd`.
    pub fn lseek(&self, fd: i32, offset: i64, whence: i32) -> Result<usize> {
        self.call(
            sysnum::LSEEK,
            [fd as u32 as u64, offset as u64, whence as u32 as u64, 0, 0, 0],
        )
    }

    /// Stat a descriptor.
    pub fn fstat(&self, fd: i32, stat: &mut Stat) -> Result<usize> {
        // SAFETY: `stat` is a live, exclusively borrowed `Stat`, so the kernel
        // has writable storage of exactly the right size for the call.
        self.call(
            sysnum::FSTAT,
            [fd as u32 as u64, gate::addr_of_mut(stat), 0, 0, 0, 0],
        )
    }

    /// Stat a path, without following a final symbolic link when `nofollow` is
    /// set.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory and `stat`
    /// must be writable.
    pub unsafe fn stat_at(
        &self,
        dirfd: i32,
        path: *const u8,
        stat: &mut Stat,
        nofollow: bool,
    ) -> Result<usize> {
        // SAFETY: the caller's contract covers both pointers.
        self.call(
            sysnum::NEWFSTATAT,
            [
                dirfd as u32 as u64,
                path as u64,
                gate::addr_of_mut(stat),
                if nofollow {
                    AT_SYMLINK_NOFOLLOW as u64
                } else {
                    0
                },
                0,
                0,
            ],
        )
    }

    /// Read directory records into `buf`, returning the bytes written.
    ///
    /// # Safety
    ///
    /// `buf` must be writable for `buf.len()` bytes; the kernel fills it with
    /// whole [`crate::dirent`] records.
    pub unsafe fn getdents64(&self, fd: i32, buf: &mut [u8]) -> Result<usize> {
        // SAFETY: the caller promised `buf` is writable for the call.
        self.call(
            sysnum::GETDENTS64,
            [
                fd as u32 as u64,
                gate::addr_of_mut(buf.as_mut_ptr()),
                buf.len() as u64,
                0,
                0,
                0,
            ],
        )
    }

    /// Device control.
    pub fn ioctl(&self, fd: i32, request: u64, arg: u64) -> Result<usize> {
        self.call(
            sysnum::IOCTL,
            [fd as u32 as u64, request, arg, 0, 0, 0],
        )
    }

    /// Descriptor control.
    pub fn fcntl(&self, fd: i32, cmd: i32, arg: u64) -> Result<usize> {
        self.call(
            sysnum::FCNTL,
            [fd as u32 as u64, cmd as u32 as u64, arg, 0, 0, 0],
        )
    }

    /// Duplicate a descriptor at the lowest free number.
    pub fn dup(&self, fd: i32) -> Result<usize> {
        self.call(sysnum::DUP, [fd as u32 as u64, 0, 0, 0, 0, 0])
    }

    /// Duplicate a descriptor onto a specific number.
    pub fn dup3(&self, oldfd: i32, newfd: i32) -> Result<usize> {
        self.call(
            sysnum::DUP3,
            [oldfd as u32 as u64, newfd as u32 as u64, 0, 0, 0, 0],
        )
    }

    /// Create a pipe, writing the two ends into `fds`.
    pub fn pipe2(&self, fds: &mut [i32; 2], flags: u32) -> Result<usize> {
        // SAFETY: `fds` is a live, exclusively borrowed pair of `i32`, which is
        // exactly what the kernel writes.
        self.call(
            sysnum::PIPE2,
            [gate::addr_of_mut(fds), flags as u64, 0, 0, 0, 0],
        )
    }

    /// Wait for readiness. A `timeout_ms` of `-1` blocks indefinitely.
    ///
    /// The timeout goes into the register sign-extended, so `-1` arrives as
    /// `u64::MAX`. Passing `0xFFFF_FFFF` instead would be indistinguishable from
    /// a 49-day wait, which is exactly the kind of bug that only shows up on a
    /// machine nobody is watching.
    pub fn ppoll(&self, fds: &mut [PollFd], timeout_ms: i32) -> Result<usize> {
        // SAFETY: `fds` is a live, exclusively borrowed slice of `PollFd`, and
        // the kernel writes only within it.
        self.call(
            sysnum::PPOLL,
            [
                gate::addr_of_mut(fds.as_mut_ptr()),
                fds.len() as u64,
                timeout_ms as i64 as u64,
                0,
                0,
                0,
            ],
        )
    }

    // ── directories ─────────────────────────────────────────────────────────

    /// Create a directory.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory.
    pub unsafe fn mkdirat(&self, dirfd: i32, path: *const u8, mode: u32) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer.
        self.call(
            sysnum::MKDIRAT,
            [dirfd as u32 as u64, path as u64, mode as u64, 0, 0, 0],
        )
    }

    /// Remove a name. [`AT_REMOVEDIR`] selects directory removal.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory.
    pub unsafe fn unlinkat(&self, dirfd: i32, path: *const u8, flags: u32) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer.
        self.call(
            sysnum::UNLINKAT,
            [dirfd as u32 as u64, path as u64, flags as u64, 0, 0, 0],
        )
    }

    /// Rename a path.
    ///
    /// # Safety
    ///
    /// Both paths must be NUL-terminated strings in readable memory.
    pub unsafe fn renameat(
        &self,
        olddirfd: i32,
        oldpath: *const u8,
        newdirfd: i32,
        newpath: *const u8,
    ) -> Result<usize> {
        // SAFETY: the caller's contract covers both pointers.
        self.call(
            sysnum::RENAMEAT,
            [
                olddirfd as u32 as u64,
                oldpath as u64,
                newdirfd as u32 as u64,
                newpath as u64,
                0,
                0,
            ],
        )
    }

    /// Check whether the calling process may access `path`.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory.
    pub unsafe fn access(&self, dirfd: i32, path: *const u8, mode: u32) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer.
        self.call(
            sysnum::ACCESS,
            [dirfd as u32 as u64, path as u64, mode as u64, 0, 0, 0],
        )
    }

    /// Truncate a path.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory.
    pub unsafe fn truncate(&self, path: *const u8, len: u64) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer.
        self.call(sysnum::TRUNCATE, [path as u64, len, 0, 0, 0, 0])
    }

    /// Truncate a descriptor.
    pub fn ftruncate(&self, fd: i32, len: u64) -> Result<usize> {
        self.call(sysnum::FTRUNCATE, [fd as u32 as u64, len, 0, 0, 0, 0])
    }

    /// Flush a descriptor to storage.
    pub fn fsync(&self, fd: i32) -> Result<usize> {
        self.call(sysnum::FSYNC, [fd as u32 as u64, 0, 0, 0, 0, 0])
    }

    /// The current working directory, NUL-terminated, into `buf`.
    pub fn getcwd(&self, buf: &mut [u8]) -> Result<usize> {
        // SAFETY: `buf` is a live, exclusively borrowed slice the kernel may
        // write up to `buf.len()` bytes into; it is also expected to leave a NUL
        // in there, which `tgs-libc` checks.
        self.call(
            sysnum::GETCWD,
            [gate::addr_of_mut(buf.as_mut_ptr()), buf.len() as u64, 0, 0, 0, 0],
        )
    }

    /// Change the working directory.
    ///
    /// # Safety
    ///
    /// `path` must be a NUL-terminated string in readable memory.
    pub unsafe fn chdir(&self, path: *const u8) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer.
        self.call(sysnum::CHDIR, [path as u64, 0, 0, 0, 0, 0])
    }

    /// Set the umask, returning the previous one.
    pub fn umask(&self, mask: u32) -> Result<usize> {
        self.call(sysnum::UMASK, [mask as u64, 0, 0, 0, 0, 0])
    }

    // ── process and clock ───────────────────────────────────────────────────

    /// The calling process's id.
    pub fn getpid(&self) -> Result<u32> {
        Ok(self.call(sysnum::TGS_GETPID, [0; 6])? as u32)
    }

    /// The calling thread's id.
    pub fn gettid(&self) -> Result<u32> {
        Ok(self.call(sysnum::GETTID, [0; 6])? as u32)
    }

    /// Read a clock.
    pub fn clock_gettime(&self, clock: i32) -> Result<Timespec> {
        let mut ts = Timespec::default();
        // SAFETY: `ts` is a live, exclusively borrowed `Timespec` of exactly the
        // size the kernel writes, and it outlives the call.
        self.call(
            sysnum::CLOCK_GETTIME,
            [clock as u32 as u64, gate::addr_of_mut(&mut ts), 0, 0, 0, 0],
        )?;
        Ok(ts)
    }

    /// Sleep for `request`, returning the unslept remainder.
    pub fn nanosleep(&self, request: &Timespec) -> Result<Timespec> {
        let mut left = Timespec::default();
        // SAFETY: both timespecs are live, exclusively borrowed values of the
        // size the kernel reads and writes, and both outlive the call.
        self.call(
            sysnum::NANOSLEEP,
            [
                gate::addr_of(request),
                gate::addr_of_mut(&mut left),
                0,
                0,
                0,
                0,
            ],
        )?;
        Ok(left)
    }

    /// Fill `buf` with entropy.
    pub fn getrandom(&self, buf: &mut [u8]) -> Result<usize> {
        // SAFETY: `buf` is a live, exclusively borrowed slice the kernel writes
        // up to `buf.len()` bytes into.
        self.call(
            sysnum::GETRANDOM,
            [
                gate::addr_of_mut(buf.as_mut_ptr()),
                buf.len() as u64,
                0,
                0,
                0,
                0,
            ],
        )
    }

    /// Publish the address the kernel should clear when this thread exits.
    pub fn set_tid_address(&self, addr: u64) -> Result<usize> {
        self.call(sysnum::SET_TID_ADDRESS, [addr, 0, 0, 0, 0, 0])
    }

    /// Yield the rest of the time slice.
    pub fn sched_yield(&self) -> Result<usize> {
        self.call(sysnum::SCHED_YIELD, [0; 6])
    }

    /// The CPUs this process may run on, as a bitmask of `usize` words.
    pub fn sched_getaffinity(&self, out: &mut [usize]) -> Result<usize> {
        // SAFETY: `out` is a live, exclusively borrowed slice of `usize`, and
        // `out.len() * size_of::<usize>()` is exactly the byte count the kernel
        // is told it may write.
        self.call(
            sysnum::SCHED_GETAFFINITY,
            [
                0,
                gate::addr_of_mut(out.as_mut_ptr()),
                (out.len() * core::mem::size_of::<usize>()) as u64,
                0,
                0,
                0,
            ],
        )
    }

    // ── net ─────────────────────────────────────────────────────────────────

    /// Create a socket.
    pub fn socket(&self, domain: i32, kind: i32, protocol: i32) -> Result<usize> {
        self.call(
            sysnum::SOCKET,
            [
                domain as u32 as u64,
                kind as u32 as u64,
                protocol as u32 as u64,
                0,
                0,
                0,
            ],
        )
    }

    /// Connect a socket.
    ///
    /// # Safety
    ///
    /// `addr` must point at `len` readable bytes of a socket address.
    pub unsafe fn connect(&self, fd: i32, addr: *const u8, len: u32) -> Result<usize> {
        // SAFETY: the caller's contract covers the pointer and the length.
        self.call(
            sysnum::CONNECT,
            [fd as u32 as u64, addr as u64, len as u64, 0, 0, 0],
        )
    }

    /// Send a datagram on `fd`.
    pub fn send(&self, fd: i32, buf: &[u8], flags: i32) -> Result<usize> {
        // SAFETY: `buf` is a live shared slice, readable for the duration of the
        // call, and the kernel is not told it may write through it.
        self.call(
            sysnum::SENDTO,
            [
                fd as u32 as u64,
                gate::addr_of(buf.as_ptr()),
                buf.len() as u64,
                flags as u32 as u64,
                0,
                0,
            ],
        )
    }

    /// Receive a datagram on `fd`.
    pub fn recv(&self, fd: i32, buf: &mut [u8], flags: i32) -> Result<usize> {
        // SAFETY: `buf` is a live, exclusively borrowed slice the kernel writes
        // up to `buf.len()` bytes into, and the source address is passed as null
        // because this wrapper does not report the peer.
        self.call(
            sysnum::RECVFROM,
            [
                fd as u32 as u64,
                gate::addr_of_mut(buf.as_mut_ptr()),
                buf.len() as u64,
                flags as u32 as u64,
                0,
                0,
            ],
        )
    }

    // ── termination ─────────────────────────────────────────────────────────

    /// Terminate every thread of this process.
    ///
    /// `#[cold]` so the spin a broken kernel would leave behind stays out of the
    /// callers' hot paths.
    #[cold]
    pub fn exit_group(&self, status: i32) -> ! {
        // SAFETY: an exit syscall carries no pointers. The loop is only reached
        // if the kernel failed to terminate the process, which would otherwise
        // let `main` return into a half-torn-down world.
        let _ = self.call(sysnum::EXIT_GROUP, [status as u32 as u64, 0, 0, 0, 0, 0]);
        loop {
            core::hint::spin_loop();
        }
    }

    /// Terminate the calling process, through TrangorgeOS's original
    /// `SYS_EXIT`.
    ///
    /// Equivalent to [`Posix::exit_group`] until the kernel grows real threads.
    #[cold]
    pub fn exit(&self, status: i32) -> ! {
        // SAFETY: an exit syscall carries no pointers.
        let _ = self.call(sysnum::TGS_EXIT, [status as u32 as u64, 0, 0, 0, 0, 0]);
        loop {
            core::hint::spin_loop();
        }
    }
}
