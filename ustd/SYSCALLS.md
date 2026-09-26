# The TrangorgeOS userspace syscall contract
#
# What the kernel has to implement for `ustd` — the sysroot that gives userspace a
# real `std` — to work. Everything here is consumed by `hal/src/posix.rs` and
# `libc/src/*`, so a change on the kernel side and a change here have to happen
# together.
#
# ## Register mapping
#
# | Register | Role |
# |----------|------|
# | `rax`    | syscall number in, result out |
# | `rdi`    | argument 0 |
# | `rsi`    | argument 1 |
# | `rdx`    | argument 2 |
# | `r10`    | argument 3 (skips `rcx`, which `int` clobbers) |
# | `r8`     | argument 4 |
# | `r9`     | argument 5 |
#
# Six arguments is the Linux maximum and therefore the maximum `std` needs.
# Arguments past a call's arity must be zero: `tgs-hal` leaves them zeroed and a
# test asserts it.
#
# ## Return values
#
# * Success: a non-negative value in `rax`. For `mmap` that is an address, for
#   `read`/`write` a byte count, for `getdents64` the number of bytes written, for
#   `clock_gettime` 0.
# * Failure: `-errno` in `rax`. The numbers are the Linux ones, so
#   `std::io::Error::from_raw_os_error` maps them to the right `ErrorKind` without
#   a translation table.
#
# # Required changes to the existing kernel
#
# Two, and both are small.
#
# 1. **`syscall.rs`'s fallback must be `-ENOSYS`, not `u64::MAX`.**
#    `handle()` ends in `_ => u64::MAX`. That is `-1`, which is `EPERM` — a real
#    error the capability model returns on a denied path. Mapping it to
#    "`u64::MAX` means not implemented" would turn a policy denial into a missing
#    kernel feature, which is exactly how a real security bug hides. In kernel
#    terms, return `u64::MAX - 37`. `tgs-hal::errno::from_raw` documents the same
#    reasoning and has a test for it (`minus_one_is_eperm_not_not_implemented`).
#
# 2. **`add_world` should push an ELF process image onto the user stack.**
#    `trampoline_rings::add_world` sets `rsp` to the top of a 16 KiB mapping and
#    `rdi` to 0, and nothing else. `ustd`'s `_start` reads the standard layout off
#    the stack — `argc`, `argv[]`, a null, `envp[]`, a null, the auxv, a null — and
#    falls back to a synthetic `argv` of one element when it is not there. So
#    `std::env::args()` works either way, but `std::env::vars()` only sees the real
#    block once the kernel pushes one. The auxv should include at least `AT_PAGESZ`
#    (6) and `AT_RANDOM` (25): `std` reads the first to size its stack guard and
#    the second to seed `HashMap`.
#
#    While there, the stack is worth enlarging: 16 KiB is tight for a program that
#    has a real `std` on it, which is the whole point of this work.
#
# ## Descriptor rules
#
# `tgs_hal::fd` keeps the descriptor table in userspace, which is a choice with
# consequences the kernel should know about:
#
# * Descriptors 0, 1 and 2 are the standard streams and are always open.
# * A descriptor is a small dense integer. The lowest free one is where `openat`
#   should place the next file.
# * `close` must unlink the name immediately. A later `read` on a closed
#   descriptor must be `EBADF`, not a silent read of whatever took the slot.
# * `dup` gives a second name for the same open file description, and closing one
#   name must leave the other working. The *position* is shared; userspace's copy
#   of it is only a cache, so `lseek` on either name is what actually moves it.
# * A full table is `EMFILE`, not `ENOMEM`.
#
# ## Memory
#
# `mmap` with `MAP_ANONYMOUS` and `fd == -1` is the only mapping `ustd` asks for at
# run time, and the heap is built on nothing else. The requirements on the kernel:
#
# * The returned address is page-aligned. A heap that handed back an address 16
#   bytes into a page would put every `malloc`ed block at the wrong alignment,
#   because the allocator's own header has to sit below the payload.
# * Success is reported with a non-zero address. A kernel that reports success and
#   returns 0 has failed, and `tgs-hal` turns that into `ENOMEM` rather than
#   letting every later write hit the unmapped page at zero.
# * `fd` is an `int`; the upper 32 bits of its register are undefined by the SysV
#   ABI, so mask to 32 bits before comparing against `AT_FDCWD`.
# * `munmap` must accept the exact `(addr, len)` pair `mmap` returned. The heap
#   hands that pair straight back for a block it gave a whole mapping to.
#
# ## Files
#
# `struct stat` is `tgs_hal::posix::Stat`, which matches the Linux x86_64 layout
# for the fields `std` reads. `std` asks for `STATX_BASIC_STATS` and reads
# `st_mode`, `st_nlink`, `st_uid`, `st_gid`, `st_size`, `st_blksize`, `st_blocks`
# and the three timestamps. A filesystem with no inode numbers reports `st_ino == 0`
# and `st_nlink == 1` — which is what makes `metadata.permissions().readonly()`
# answer "no" rather than lie.
#
# `getdents64` must write `struct linux_dirent64` records: a 19-byte **packed**
# header followed by a NUL-terminated name, each record rounded up to an 8-byte
# boundary. `std::fs::read_dir` walks the buffer itself rather than calling
# `readdir`, so the layout is not negotiable; `tgs_hal::dirent` has the codec and
# the tests for it, including a test that the header is 19 bytes and not 24.
#
# A record with `d_off == 0` ends the directory, and a call that returns 0 bytes
# also ends it. `std` stops at the first of either.
#
# ## Clocks and entropy
#
# * `CLOCK_MONOTONIC` (1) must be monotonic and must never go backwards, or
#   `std::time::Instant` produces a zero or negative duration.
# * `CLOCK_REALTIME` (0) backs `SystemTime::now`.
# * `getrandom` must fill the buffer or report how much it filled. `std` needs it
#   for `HashMap`'s `RandomState`, and a `getrandom` that returns 0 without writing
#   makes every `HashMap` in the program use the same seed.
#
# `tgs-libc` falls back to a SplitMix64 stream seeded from `rdtsc` and the load
# address when the kernel answers `ENOSYS`, so a `HashMap` works either way. That
# fallback is not cryptographic and is documented as such.
#
# ## What deliberately returns `ENOSYS`
#
# Not a gap in this list, a decision:
#
# | Call | Why |
# |------|-----|
# | `clone`, `set_tid_address`, `rseq` | no `fs_base` yet, so a second task would corrupt the allocator. `pthread_create` answers `EAGAIN` for the same reason. |
# | `futex` | same; there is nothing to wait on with one task. |
# | `sigaction`, `rt_sigprocmask`, `sigaltstack` | no signal delivery. `std` tolerates `ENOSYS` from these — it installs a stack-overflow handler and carries on when it cannot. |
# | `socket`, `bind`, `listen`, `accept`, `connect` | the network stack is in driver space, not ring 3. The numbers are reserved so the shim is already right when it arrives. |
#
# ## The table
#
# Numbers are `ustd/hal/src/sysnum.rs`; the argument columns are the registers
# above, so `r10` is argument 3 and `r8` is argument 4.
#
# ### `0x2000`–`0x2029` — files, memory, descriptors
#
# | Number | Name | a0 | a1 | a2 | a3 | a4 | a5 | Returns |
# |--------|------|----|----|----|----|----|----|---------|
# | `0x2000` | `mmap` | addr | len | prot | flags | fd | offset | address |
# | `0x2001` | `munmap` | addr | len | — | — | — | — | 0 |
# | `0x2002` | `mprotect` | addr | len | prot | — | — | — | 0 |
# | `0x2003` | `madvise` | addr | len | advice | — | — | — | 0 |
# | `0x2004` | `brk` | new break (0 = query) | — | — | — | — | — | break |
# | `0x2006` | `read` | fd | buf\* | count | — | — | — | bytes |
# | `0x2007` | `write` | fd | buf\* | count | — | — | — | bytes |
# | `0x2008` | `pread` | fd | buf\* | count | offset | — | — | bytes |
# | `0x2009` | `pwrite` | fd | buf\* | count | offset | — | — | bytes |
# | `0x200A` | `readv` | fd | iovec\* | count | — | — | — | bytes |
# | `0x200B` | `writev` | fd | iovec\* | count | — | — | — | bytes |
# | `0x200C` | `openat` | dirfd | path\* | flags | mode | stat\* | — | fd |
# | `0x200D` | `close` | fd | — | — | — | — | — | 0 |
# | `0x200E` | `lseek` | fd | offset | whence | — | — | — | offset |
# | `0x200F` | `fstat` | fd | stat\* | — | — | — | — | 0 |
# | `0x2010` | `newfstatat` | dirfd | path\* | stat\* | flags | — | — | 0 |
# | `0x2011` | `statx` | dirfd | path\* | statx\* | flags | mask | — | 0 |
# | `0x2012` | `getdents64` | fd | dirent\* | count | — | — | — | bytes |
# | `0x2013` | `ioctl` | fd | request | arg | — | — | — | 0 |
# | `0x2014` | `fcntl` | fd | cmd | arg | — | — | — | 0 |
# | `0x2015` | `dup` | fd | — | — | — | — | — | fd |
# | `0x2016` | `dup3` | oldfd | newfd | — | — | — | — | newfd |
# | `0x2017` | `pipe2` | fds\* | flags | — | — | — | — | 0 |
# | `0x2018` | `ppoll` | pollfd\* | count | timeout ms | — | — | — | ready |
# | `0x2019` | `mkdirat` | dirfd | path\* | mode | — | — | — | 0 |
# | `0x201A` | `unlinkat` | dirfd | path\* | flags | — | — | — | 0 |
# | `0x201B` | `renameat` | olddirfd | oldpath\* | newdirfd | newpath\* | — | — | 0 |
# | `0x201C` | `rmdir` | path\* | — | — | — | — | — | 0 |
# | `0x201D` | `access` | dirfd | path\* | mode | — | — | — | 0 |
# | `0x201E` | `readlinkat` | dirfd | path\* | buf\* | buflen | — | — | bytes |
# | `0x201F` | `symlinkat` | dirfd | target\* | linkpath\* | — | — | — | 0 |
# | `0x2020` | `ftruncate` | fd | length | — | — | — | — | 0 |
# | `0x2021` | `fsync` | fd | — | — | — | — | — | 0 |
# | `0x2022` | `flock` | fd | operation | — | — | — | — | 0 |
# | `0x2023` | `getcwd` | buf\* | size | — | — | — | — | bytes |
# | `0x2024` | `chdir` | path\* | — | — | — | — | — | 0 |
# | `0x2025` | `truncate` | path\* | length | — | — | — | — | 0 |
# | `0x2026` | `statvfs` | path\* | statvfs\* | — | — | — | — | 0 |
# | `0x2027` | `umask` | mask | — | — | — | — | — | previous |
# | `0x2028` | `utimensat` | dirfd | path\* | timespec\[2\] | — | — | — | 0 |
# | `0x2029` | `tgs_cap_query_pid` | cap id | pid (0 = self) | — | — | — | — | 0/1 |
#
# ### `0x2030`–`0x2042` — process and clock
#
# | Number | Name | a0 | a1 | a2 | a3 | a4 | a5 | Returns |
# |--------|------|----|----|----|----|----|----|---------|
# | `0x2030` | `clone` | flags | stack | parent\* | tls | ctid\* | — | tid |
# | `0x2031` | `exit_group` | status | — | — | — | — | — | never |
# | `0x2032` | `set_tid_address` | addr | — | — | — | — | — | 0 |
# | `0x2033` | `gettid` | — | — | — | — | — | — | tid |
# | `0x2034` | `futex` | addr | op | value | timeout\* | — | — | — |
# | `0x2035` | `rt_sigaction` | signum | act\* | oldact\* | size | — | — | 0 |
# | `0x2036` | `rt_sigprocmask` | how | set\* | oldset\* | size | — | — | 0 |
# | `0x2037` | `sigaltstack` | ss\* | old\* | — | — | — | — | 0 |
# | `0x2038` | `nanosleep` | req\* | rem\* | — | — | — | — | 0 |
# | `0x2039` | `clock_gettime` | clock id | timespec\* | — | — | — | — | 0 |
# | `0x203A` | `getrandom` | buf\* | len | flags | — | — | — | bytes |
# | `0x203B` | `uname` | utsname\* | — | — | — | — | — | 0 |
# | `0x203C` | `sysconf` | name | out\* | size | — | — | — | value |
# | `0x203D` | `prctl` | option | arg2 | arg3 | arg4 | arg5 | — | 0 |
# | `0x203E` | `sched_getaffinity` | pid | mask\* | size | — | — | — | bytes |
# | `0x203F` | `sched_yield` | — | — | — | — | — | — | 0 |
# | `0x2040` | `rseq` | — | — | — | — | — | — | 0/`ENOSYS` |
# | `0x2041` | `wait4` | pid | status\* | options | — | — | — | pid |
# | `0x2042` | `getuid` | uids\* | — | — | — | — | — | 0 |
#
# ### `0x3000`–`0x300B` — sockets
#
# | Number | Name | a0 | a1 | a2 | a3 | a4 | a5 | Returns |
# |--------|------|----|----|----|----|----|----|---------|
# | `0x3000` | `socket` | domain | type | protocol | — | — | — | fd |
# | `0x3001` | `bind` | fd | sockaddr\* | len | — | — | — | 0 |
# | `0x3002` | `listen` | fd | backlog | — | — | — | — | 0 |
# | `0x3003` | `accept` | fd | sockaddr\* | len\* | — | — | — | fd |
# | `0x3004` | `connect` | fd | sockaddr\* | len | — | — | — | 0 |
# | `0x3005` | `sendto` | fd | buf\* | len | flags | sockaddr\* | addrlen | bytes |
# | `0x3006` | `recvfrom` | fd | buf\* | len | flags | sockaddr\* | addrlen\* | bytes |
# | `0x3007` | `setsockopt` | fd | level | name | value\* | len | — | 0 |
# | `0x3008` | `getsockopt` | fd | level | name | value\* | len\* | — | 0 |
# | `0x3009` | `shutdown` | fd | how | — | — | — | — | 0 |
# | `0x300A` | `getsockname` | fd | sockaddr\* | len | — | — | — | 0 |
# | `0x300B` | `getpeername` | fd | sockaddr\* | len | — | — | — | 0 |
#
# ### Pre-existing
#
# The `0x0000`–`0x1FFF` blocks are the kernel's own and are unchanged by this work:
# `1` yield, `2` log, `3` tick, `0x1000` exit, `0x1001` spawn, `0x1002` getpid,
# `0x1010`/`0x1011` IPC, `0x1040` key, `0x1060` framebuffer, `0x1070`–`0x1073`
# capabilities. `tgs_hal::sysnum` has a test asserting that nothing in the POSIX or
# net block collides with any of them, so a future addition that reuses a number is
# a test failure rather than a subtle misroute.
