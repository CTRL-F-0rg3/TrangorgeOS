//! The TrangorgeOS ring-3 syscall numbers.
//!
//! # Numbering
//!
//! The table is split into four blocks, and the split is not arbitrary — it
//! keeps the numbers that already exist in the tree untouched:
//!
//! | Block | Range | Owner |
//! |-------|-------|-------|
//! | *hyper* | `0x0000`–`0x00FF` | `kernel/src/trampoline_rings/arch/x86_64/trampoline_rings.rs` |
//! | *tgs* | `0x1000`–`0x1FFF` | `kernel/src/userspace/process/syscall.rs`, `kernel/src/caps/syscalls.rs` |
//! | *posix* | `0x2000`–`0x2FFF` | **this file** — the surface `std` needs |
//! | *net* | `0x3000`–`0x3FFF` | **this file** — sockets |
//!
//! Nothing in the existing tree collides with the `posix` block: the highest
//! number the kernel currently defines is `0x1073`
//! (`kernel/src/caps/syscalls.rs`), and the `0x2001`/`0x3001`/`0x4001`
//! constants in `kernel_Workspace/core/linix_abi/src/sysnums.rs` belong to a
//! *translation table* for a Linux-compat layer that is not wired to a
//! dispatcher. Nothing is lost by giving the range to the real thing.
//!
//! The numbers themselves are the Linux ones wherever Linux has one. That is
//! not cosmetic: `std` does not care, but it means mapping a failing call onto
//! an `io::ErrorKind` has to be written once, and running unmodified Linux
//! binaries later becomes a lookup table rather than a rewrite.
//!
//! The full kernel-side contract — arguments, results, error cases — is in
//! `../../SYSCALLS.md`.

// ── block boundaries, re-used by the tests below ───────────────────────────

/// First number of the block this crate owns.
pub const POSIX_FIRST: u64 = 0x2000;
/// Last number of the block this crate owns.
pub const POSIX_LAST: u64 = 0x2FFF;
/// First number of the process/clock block.
pub const PROC_FIRST: u64 = 0x2030;
/// Last number of the process/clock block.
pub const PROC_LAST: u64 = 0x20FF;
/// First number of the socket block.
pub const NET_FIRST: u64 = 0x3000;
/// Last number of the socket block.
pub const NET_LAST: u64 = 0x3FFF;

// ── hypervisor block ───────────────────────────────────────────────────────

/// Give up the rest of the time slice; the kernel polls its rings and switches
/// to the next world. Also the "spin" primitive for a busy wait.
pub const YIELD: u64 = 1;
/// Kernel `printf` of a NUL-terminated string passed in `rdi`.
pub const LOG: u64 = 2;
/// Monotonic world-switch counter, in units of one scheduler tick.
pub const TICK: u64 = 3;

// ── tgs block (pre-existing kernel ABI) ─────────────────────────────────────

/// Terminate the calling world. `rdi` = exit status.
pub const TGS_EXIT: u64 = 0x1000;
/// Load an ELF from the VFS and add a new world. `rdi` = NUL-terminated path.
pub const TGS_SPAWN: u64 = 0x1001;
/// PID of the calling world.
pub const TGS_GETPID: u64 = 0x1002;
/// Send a 4-word message to a PID. `rdi` = pid, `rsi` = a0, `rdx` = a1.
pub const TGS_IPC_SEND: u64 = 0x1010;
/// Receive a message. Returns the sender in `rax`, payload in `rdi`/`rsi`.
pub const TGS_IPC_RECV: u64 = 0x1011;
/// Does the calling world hold a capability? `rdi` = capability id.
pub const TGS_CAP_QUERY: u64 = 0x1070;
/// Grant a capability to the calling world. `rdi` = capability id.
pub const TGS_CAP_REQUEST: u64 = 0x1071;
/// Revoke a capability from the calling world. `rdi` = capability id.
pub const TGS_CAP_RELEASE: u64 = 0x1072;
/// Number of capability denials so far.
pub const TGS_CAP_AUDIT: u64 = 0x1073;
/// Poll one key from the USB HID keyboard driver. `0` means "nothing".
pub const TGS_KEY: u64 = 0x1040;
/// Map the framebuffer and the 8x8 font into the caller. See [`UI_FB_VA`].
pub const TGS_UI_OPEN: u64 = 0x1060;

/// Virtual address the kernel maps the framebuffer at for [`TGS_UI_OPEN`].
pub const UI_FB_VA: u64 = 0x5000_0000;
/// Virtual address the kernel maps the 8x8 console font at.
pub const UI_FONT_VA: u64 = 0x6000_0000;

// ── posix block ─────────────────────────────────────────────────────────────

/// Map anonymous memory. `rdi` = addr hint, `rsi` = length, `rdx` = prot,
/// `r10` = flags, `r8` = fd, `r9` = offset. Returns the mapped address.
pub const MMAP: u64 = 0x2000;
/// Unmap memory. `rdi` = addr, `rsi` = length.
pub const MUNMAP: u64 = 0x2001;
/// Change page protection. `rdi` = addr, `rsi` = length, `rdx` = prot.
pub const MPROTECT: u64 = 0x2002;
/// Advise the kernel about a range. `rdi` = addr, `rsi` = length, `rdx` = advice.
pub const MADVISE: u64 = 0x2003;
/// Resize the break. `rdi` = new break (0 queries the current one).
pub const BRK: u64 = 0x2004;
/// Read from a file descriptor. `rdi` = fd, `rsi` = buf, `rdx` = count.
pub const READ: u64 = 0x2006;
/// Write to a file descriptor. `rdi` = fd, `rsi` = buf, `rdx` = count.
pub const WRITE: u64 = 0x2007;
/// Read at an absolute offset. `rdi` = fd, `rsi` = buf, `rdx` = count,
/// `r10` = offset.
pub const PREAD: u64 = 0x2008;
/// Write at an absolute offset. Same register layout as [`PREAD`].
pub const PWRITE: u64 = 0x2009;
/// Scatter read. `rdi` = fd, `rsi` = `iovec*`, `rdx` = iovec count.
pub const READV: u64 = 0x200A;
/// Gather write. Same register layout as [`READV`].
pub const WRITEV: u64 = 0x200B;
/// Open a path. `rdi` = dirfd, `rsi` = NUL-terminated path, `rdx` = flags,
/// `r10` = mode.
pub const OPENAT: u64 = 0x200C;
/// Close a file descriptor. `rdi` = fd.
pub const CLOSE: u64 = 0x200D;
/// Reposition a descriptor. `rdi` = fd, `rsi` = offset, `rdx` = whence.
pub const LSEEK: u64 = 0x200E;
/// Stat a descriptor. `rdi` = fd, `rsi` = `Stat*` out.
pub const FSTAT: u64 = 0x200F;
/// Stat a path. `rdi` = dirfd, `rsi` = path, `rdx` = `Stat*` out, `r10` = flags.
pub const NEWFSTATAT: u64 = 0x2010;
/// Modern stat. `rdi` = dirfd, `rsi` = path, `rdx` = `Statx*` out,
/// `r10` = flags, `r8` = mask.
pub const STATX: u64 = 0x2011;
/// Read directory entries. `rdi` = fd, `rsi` = `Dirent64*` buffer,
/// `rdx` = buffer size. Returns the number of bytes written.
pub const GETDENTS64: u64 = 0x2012;
/// Device control. `rdi` = fd, `rsi` = request, `rdx` = argument.
pub const IOCTL: u64 = 0x2013;
/// Descriptor control. `rdi` = fd, `rsi` = command, `rdx` = argument.
pub const FCNTL: u64 = 0x2014;
/// Duplicate a descriptor. `rdi` = fd.
pub const DUP: u64 = 0x2015;
/// Duplicate a descriptor onto a specific number. `rdi` = oldfd, `rsi` = newfd.
pub const DUP3: u64 = 0x2016;
/// Create a pipe. `rdi` = 2-element `i32` out, `rsi` = flags.
pub const PIPE2: u64 = 0x2017;
/// Wait for readiness. `rdi` = `PollFd*`, `rsi` = count, `rdx` = timeout ms.
pub const PPOLL: u64 = 0x2018;
/// Create a directory. `rdi` = dirfd, `rsi` = path, `rdx` = mode.
pub const MKDIRAT: u64 = 0x2019;
/// Remove a path. `rdi` = dirfd, `rsi` = path, `rdx` = flags.
pub const UNLINKAT: u64 = 0x201A;
/// Rename a path. `rdi` = olddirfd, `rsi` = oldpath, `rdx` = newdirfd,
/// `r10` = newpath.
pub const RENAMEAT: u64 = 0x201B;
/// Remove a directory. `rdi` = path.
pub const RMDIR: u64 = 0x201C;
/// Check permissions. `rdi` = dirfd, `rsi` = path, `rdx` = mode.
pub const ACCESS: u64 = 0x201D;
/// Read a symbolic link. `rdi` = dirfd, `rsi` = path, `rdx` = buffer,
/// `r10` = buffer size.
pub const READLINKAT: u64 = 0x201E;
/// Create a symbolic link. `rdi` = dirfd, `rsi` = target, `rdx` = linkpath.
pub const SYMLINKAT: u64 = 0x201F;
/// Truncate a descriptor. `rdi` = fd, `rsi` = length.
pub const FTRUNCATE: u64 = 0x2020;
/// Flush a descriptor to storage. `rdi` = fd.
pub const FSYNC: u64 = 0x2021;
/// Advisory lock. `rdi` = fd, `rsi` = operation.
pub const FLOCK: u64 = 0x2022;
/// Current working directory. `rdi` = buffer, `rsi` = size.
pub const GETCWD: u64 = 0x2023;
/// Change working directory. `rdi` = path.
pub const CHDIR: u64 = 0x2024;
/// Truncate a path. `rdi` = path, `rsi` = length.
pub const TRUNCATE: u64 = 0x2025;
/// Filesystem statistics. `rdi` = path, `rsi` = `StatVfs*` out.
pub const STATVFS: u64 = 0x2026;
/// Set the umask. `rdi` = mask. Returns the previous one.
pub const UMASK: u64 = 0x2027;
/// Update access/modification times. `rdi` = dirfd, `rsi` = path,
/// `rdx` = `Timespec[2]`.
pub const UTIMENSAT: u64 = 0x2028;
/// Query a capability for a pid: `rdi` = cap id, `rsi` = pid (0 = self).
pub const TGS_CAP_QUERY_PID: u64 = 0x2029;

// ── process / clock block ───────────────────────────────────────────────────

/// Create a task. The full `clone` argument set; unsupported until the kernel
/// programs `fs_base`, so `tgs-rt` never calls it.
pub const CLONE: u64 = 0x2030;
/// Terminate every thread in the thread group. `rdi` = status.
pub const EXIT_GROUP: u64 = 0x2031;
/// Publish the clear-thread pointer. `rdi` = address.
pub const SET_TID_ADDRESS: u64 = 0x2032;
/// TID of the calling task.
pub const GETTID: u64 = 0x2033;
/// Futex. `rdi` = address, `rsi` = op, `rdx` = value, `r10` = timeout.
pub const FUTEX: u64 = 0x2034;
/// Install a signal handler. `rdi` = signum, `rsi` = `SigAction*`,
/// `rdx` = old action out, `r10` = size.
pub const RT_SIGACTION: u64 = 0x2035;
/// Block/unblock a signal set. `rdi` = how, `rsi` = set, `rdx` = old set,
/// `r10` = size.
pub const RT_SIGPROCMASK: u64 = 0x2036;
/// Install the alternate signal stack. `rdi` = `Stack*`.
pub const SIGALTSTACK: u64 = 0x2037;
/// Sleep. `rdi` = `Timespec*` request, `rsi` = `Timespec*` remaining.
pub const NANOSLEEP: u64 = 0x2038;
/// Read a clock. `rdi` = clock id, `rsi` = `Timespec*` out.
pub const CLOCK_GETTIME: u64 = 0x2039;
/// Kernel entropy. `rdi` = buffer, `rsi` = length, `rdx` = flags.
pub const GETRANDOM: u64 = 0x203A;
/// Kernel identity. `rdi` = `Utsname*` out.
pub const UNAME: u64 = 0x203B;
/// Read a system parameter. `rdi` = name, `rsi` = out, `rdx` = size.
pub const SYSCONF: u64 = 0x203C;
/// Per-process kernel knob. `rdi` = option, `rsi` = arg2, `rdx` = arg3,
/// `r10` = arg4, `r8` = arg5.
pub const PRCTL: u64 = 0x203D;
/// Which CPUs a task may run on. `rdi` = pid, `rsi` = mask out, `rdx` = size.
pub const SCHED_GETAFFINITY: u64 = 0x203E;
/// Nudge the scheduler. Always returns 0.
pub const SCHED_YIELD: u64 = 0x203F;
/// Register a restartable-sequences area. Optional; `ENOSYS` is fine.
pub const RSEQ: u64 = 0x2040;
/// Wait for a child. `rdi` = pid, `rsi` = status out, `rdx` = options.
pub const WAIT4: u64 = 0x2041;
/// Read the real, effective, saved and filesystem UIDs. `rdi` = `Uids*` out.
pub const GETUID: u64 = 0x2042;

// ── net block ───────────────────────────────────────────────────────────────

/// Create a socket. `rdi` = domain, `rsi` = type, `rdx` = protocol.
pub const SOCKET: u64 = 0x3000;
/// Bind a socket. `rdi` = fd, `rsi` = `Sockaddr*`, `rdx` = length.
pub const BIND: u64 = 0x3001;
/// Listen on a bound socket. `rdi` = fd, `rsi` = backlog.
pub const LISTEN: u64 = 0x3002;
/// Accept a connection. `rdi` = fd, `rsi` = `Sockaddr*`, `rdx` = length in.
pub const ACCEPT: u64 = 0x3003;
/// Connect a socket. `rdi` = fd, `rsi` = `Sockaddr*`, `rdx` = length.
pub const CONNECT: u64 = 0x3004;
/// Send a datagram. `rdi` = fd, `rsi` = buf, `rdx` = len, `r10` = flags,
/// `r8` = `Sockaddr*`, `r9` = addr length.
pub const SENDTO: u64 = 0x3005;
/// Receive a datagram. `rdi` = fd, `rsi` = buf, `rdx` = len, `r10` = flags,
/// `r8` = `Sockaddr*`, `r9` = length in.
pub const RECVFROM: u64 = 0x3006;
/// Socket options. `rdi` = fd, `rsi` = level, `rdx` = name, `r10` = value,
/// `r8` = length.
pub const SETSOCKOPT: u64 = 0x3007;
/// Read socket options. `rdi` = fd, `rsi` = level, `rdx` = name, `r10` = value,
/// `r8` = length in.
pub const GETSOCKOPT: u64 = 0x3008;
/// Shut a socket down. `rdi` = fd, `rsi` = how.
pub const SHUTDOWN: u64 = 0x3009;
/// Local address of a socket. `rdi` = fd, `rsi` = `Sockaddr*`, `rdx` = length.
pub const GETSOCKNAME: u64 = 0x300A;
/// Peer address of a socket. Same register layout as [`GETSOCKNAME`].
pub const GETPEERNAME: u64 = 0x300B;

// ── constants mirrored from libc ────────────────────────────────────────────
//
// `tgs-hal` deliberately carries its own copy so the crate depends on nothing
// and so the numbers cannot drift with whichever `libc` release `std` happens
// to pin. The `tgs-libc` shims assert these against the real `libc` at
// compile time, so a drift is a build error rather than a runtime mystery.

/// Readable pages.
pub const PROT_READ: u32 = 0x1;
/// Writable pages.
pub const PROT_WRITE: u32 = 0x2;
/// Executable pages.
pub const PROT_EXEC: u32 = 0x4;
/// No access at all.
pub const PROT_NONE: u32 = 0x0;

/// The mapping is not visible to other tasks.
pub const MAP_PRIVATE: u32 = 0x02;
/// The mapping is writable and may be shared with a child.
pub const MAP_SHARED: u32 = 0x01;
/// Use the address the caller asked for, or fail.
pub const MAP_FIXED: u32 = 0x10;
/// The mapping is not backed by a file.
pub const MAP_ANONYMOUS: u32 = 0x20;

/// Seek from the start of the file.
pub const SEEK_SET: i32 = 0;
/// Seek from the current position.
pub const SEEK_CUR: i32 = 1;
/// Seek from the end of the file.
pub const SEEK_END: i32 = 2;

/// Open for reading.
pub const O_RDONLY: u32 = 0o0;
/// Open for writing.
pub const O_WRONLY: u32 = 0o1;
/// Open for reading and writing.
pub const O_RDWR: u32 = 0o2;
/// Append on every write.
pub const O_APPEND: u32 = 0o2000;
/// Create the file if it is missing.
pub const O_CREAT: u32 = 0o100;
/// Truncate an existing regular file.
pub const O_TRUNC: u32 = 0o1000;
/// Fail instead of following a final symlink.
pub const O_NOFOLLOW: u32 = 0o400000;
/// Open in non-blocking mode.
pub const O_NONBLOCK: u32 = 0o4000;
/// Fail unless the path is a directory.
pub const O_DIRECTORY: u32 = 0o200000;
/// Close the descriptor on `exec`.
pub const O_CLOEXEC: u32 = 0o2000000;

/// The clock a process measures durations with.
pub const CLOCK_MONOTONIC: i32 = 1;
/// The wall clock.
pub const CLOCK_REALTIME: i32 = 0;

/// `getrandom` may return early.
pub const GRND_NONBLOCK: u32 = 0x0001;
/// `getrandom` must not use hardware entropy.
pub const GRND_RANDOM: u32 = 0x0002;

/// Block until at least one descriptor is ready.
pub const POLLIN: i16 = 0x001;
/// Block until a descriptor can be written without blocking.
pub const POLLOUT: i16 = 0x004;
/// Something went wrong; check `revents`.
pub const POLLERR: i16 = 0x008;
/// The peer hung up.
pub const POLLHUP: i16 = 0x010;
/// Invalid descriptor; check `revents`.
pub const POLLNVAL: i16 = 0x020;

// ── `*at` path flags ────────────────────────────────────────────────────────
//
// `AT_FDCWD` is the only one this crate ever passes: there is no per-process
// working-directory fallback to get wrong, and every caller supplies an absolute
// path, which makes the dirfd argument inert.

/// Interpret a relative path against the calling process's working directory.
pub const AT_FDCWD: i32 = -100;
/// `newfstatat` and `statx` should not follow a final symbolic link.
pub const AT_SYMLINK_NOFOLLOW: u32 = 0x100;
/// `unlinkat` should remove a directory rather than a file.
pub const AT_REMOVEDIR: u32 = 0x200;



#[cfg(test)]
mod tests {
    use super::*;

    /// Every syscall number this crate declares, for the uniqueness check.
    const ALL: &[u64] = &[
        YIELD, LOG, TICK, TGS_EXIT, TGS_SPAWN, TGS_GETPID, TGS_IPC_SEND,
        TGS_IPC_RECV, TGS_KEY, TGS_UI_OPEN, TGS_CAP_QUERY, TGS_CAP_REQUEST,
        TGS_CAP_RELEASE, TGS_CAP_AUDIT, MMAP, MUNMAP, MPROTECT, MADVISE, BRK,
        READ, WRITE, PREAD, PWRITE, READV, WRITEV, OPENAT, CLOSE, LSEEK,
        FSTAT, NEWFSTATAT, STATX, GETDENTS64, IOCTL, FCNTL, DUP, DUP3, PIPE2,
        PPOLL, MKDIRAT, UNLINKAT, RENAMEAT, RMDIR, ACCESS, READLINKAT,
        SYMLINKAT, FTRUNCATE, FSYNC, FLOCK, GETCWD, CHDIR, TRUNCATE, STATVFS,
        UMASK, UTIMENSAT, TGS_CAP_QUERY_PID, CLONE, EXIT_GROUP,
        SET_TID_ADDRESS, GETTID, FUTEX, RT_SIGACTION, RT_SIGPROCMASK,
        SIGALTSTACK, NANOSLEEP, CLOCK_GETTIME, GETRANDOM, UNAME, SYSCONF,
        PRCTL, SCHED_GETAFFINITY, SCHED_YIELD, RSEQ, WAIT4, GETUID, SOCKET,
        BIND, LISTEN, ACCEPT, CONNECT, SENDTO, RECVFROM, SETSOCKOPT,
        GETSOCKOPT, SHUTDOWN, GETSOCKNAME, GETPEERNAME,
    ];

    /// The numbers the kernel already dispatches today. If any of these ever
    /// equals one of ours, a caller would silently reach a different syscall,
    /// so this test is the guard against that.
    const KERNEL_OWNED: &[u64] = &[
        YIELD, LOG, TICK, TGS_EXIT, TGS_SPAWN, TGS_GETPID, TGS_IPC_SEND,
        TGS_IPC_RECV, TGS_KEY, TGS_UI_OPEN, TGS_CAP_QUERY, TGS_CAP_REQUEST,
        TGS_CAP_RELEASE, TGS_CAP_AUDIT,
    ];

    #[test]
    fn no_two_syscalls_share_a_number() {
        let mut sorted = ALL.to_vec();
        sorted.sort_unstable();
        for pair in sorted.windows(2) {
            assert_ne!(pair[0], pair[1], "0x{:04x} is declared twice", pair[0]);
        }
    }

    #[test]
    fn posix_block_does_not_shadow_a_kernel_number() {
        for &n in ALL {
            if (POSIX_FIRST..=POSIX_LAST).contains(&n) {
                assert!(
                    !KERNEL_OWNED.contains(&n),
                    "0x{n:04x} is both ours and the kernel's",
                );
            }
        }
    }

    #[test]
    fn each_number_sits_in_the_block_it_is_documented_in() {
        for &n in &[MMAP, BRK, READ, WRITE, OPENAT, CLOSE, GETDENTS64, STATX] {
            assert!((POSIX_FIRST..=POSIX_LAST).contains(&n), "0x{n:04x}");
        }
        for &n in &[CLONE, EXIT_GROUP, CLOCK_GETTIME, GETRANDOM, UNAME, WAIT4] {
            assert!((PROC_FIRST..=PROC_LAST).contains(&n), "0x{n:04x}");
        }
        for &n in &[SOCKET, BIND, CONNECT, RECVFROM, GETPEERNAME] {
            assert!((NET_FIRST..=NET_LAST).contains(&n), "0x{n:04x}");
        }
    }

    #[test]
    fn the_blocks_do_not_overlap_each_other() {
        assert!(POSIX_LAST < NET_FIRST);
        assert!(PROC_LAST < POSIX_LAST);
    }

    /// The octal `O_*` values are the single easiest thing in this file to get
    /// wrong, and a wrong `O_CREAT` shows up only as "my file was not created".
    #[test]
    fn open_flags_keep_their_octal_values() {
        assert_eq!(O_RDONLY, 0);
        assert_eq!(O_WRONLY, 1);
        assert_eq!(O_RDWR, 2);
        assert_eq!(O_CREAT, 0o100);
        assert_eq!(O_TRUNC, 0o1000);
        assert_eq!(O_APPEND, 0o2000);
        assert_eq!(O_NONBLOCK, 0o4000);
    }

    #[test]
    fn access_mode_bits_are_disjoint_from_the_creation_bits() {
        let mode = O_RDONLY | O_WRONLY | O_RDWR;
        for extra in [O_CREAT, O_TRUNC, O_APPEND, O_NONBLOCK, O_DIRECTORY] {
            assert_eq!(mode & extra, 0, "{extra:#o} collides with the mode");
        }
    }
}
