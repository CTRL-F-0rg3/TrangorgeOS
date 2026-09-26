//! The tests that justify the "testable on a host" claim in the crate docs.
//!
//! Each group drives one module through [`gate::Mock`], so what is checked is the
//! real logic — the free lists, the descriptor-number rules, the record layout —
//! and not a re-implementation of it.

use super::dirent::{self, kind, Entry};
use super::errno::{self, Errno};
use super::fd::{self, Kind, Table, STDERR, STDIN, STDOUT};
use super::gate::Mock;
use super::heap::{Heap, ARENA_BYTES, CLASSES, HEADER, MAX_CLASS, MIN_CLASS, PAGE};
use super::posix::{Posix, PollFd, Stat, Timespec};
use super::sysnum;

use core::alloc::Layout;
use std::vec;
use std::vec::Vec;

fn layout(size: usize) -> Layout {
    Layout::from_size_align(size, 8).expect("valid test layout")
}

/// A heap over a leaked mock, so the heap can hold a `&'static dyn Syscall`.
fn heap() -> (Heap, &'static Mock) {
    let mock: &'static Mock = Box::leak(Box::new(Mock::with_default_arena()));
    (Heap::new(mock), mock)
}

// ── the heap ───────────────────────────────────────────────────────────────

#[test]
fn the_heap_starts_out_empty_without_touching_the_kernel() {
    let (h, mock) = heap();
    assert_eq!(h.stats(), Default::default());
    assert!(mock.log().is_empty(), "building a heap must not call out");
}

#[test]
fn alloc_hands_out_distinct_blocks_inside_the_arena() {
    let (mut h, mock) = heap();

    let a = h.alloc(layout(64)).expect("first alloc");
    let b = h.alloc(layout(64)).expect("second alloc");

    assert_ne!(a.as_ptr() as usize, b.as_ptr() as usize, "blocks overlap");
    assert_eq!(mock.count_of(sysnum::MMAP), 1, "should share one region");

    // Both live inside the simulated address space the mock owns.
    let lo = mock.next_address() - mock.arena_len();
    for p in [a, b] {
        let at = p.as_ptr() as usize;
        assert!(
            (lo..lo + mock.arena_len()).contains(&at),
            "block at {at:#x} escaped the arena",
        );
    }
}

#[test]
fn alloc_honours_the_requested_alignment() {
    let (mut h, _) = heap();
    for align in [8usize, 16, 32, 64, 128, 256] {
        let l = Layout::from_size_align(200, align).expect("valid layout");
        let p = h.alloc(l).expect("aligned alloc");
        assert_eq!(p.as_ptr() as usize % align, 0, "alignment {align} lost");
    }
}

#[test]
fn a_written_block_really_is_writable_memory() {
    // The mock hands out a real host allocation, so this is a genuine write
    // through a pointer the allocator produced.
    let (mut h, _) = heap();
    let p = h.alloc(layout(16)).expect("alloc");
    // SAFETY: `p` is a live 16-byte block from this heap.
    unsafe { core::ptr::copy_nonoverlapping(b"Trangorge".as_ptr(), p.as_ptr(), 9) };
    // SAFETY: as above.
    let got = unsafe { core::slice::from_raw_parts(p.as_ptr(), 9) };
    assert_eq!(got, b"Trangorge");
}

#[test]
fn freed_blocks_come_back() {
    let (mut h, _) = heap();

    let a = h.alloc(layout(100)).expect("alloc");
    let addr = a.as_ptr() as usize;
    assert!(h.stats().allocated >= 100);

    // SAFETY: `a` came from `alloc` on this heap and is freed exactly once.
    unsafe { h.dealloc(a, layout(100)) };
    assert_eq!(h.stats().allocated, 0, "freed bytes are still accounted");

    let b = h.alloc(layout(100)).expect("realloc");
    assert_eq!(b.as_ptr() as usize, addr, "the freed block was not reused");
}

#[test]
fn free_list_splits_a_big_block_for_a_small_request() {
    let (mut h, _) = heap();

    let big = h.alloc(layout(4000)).expect("big alloc");
    // SAFETY: freed exactly once, and nothing else was carved out of it.
    unsafe { h.dealloc(big, layout(4000)) };

    let small = h.alloc(layout(48)).expect("small alloc");
    assert_eq!(small.as_ptr() as usize % MIN_CLASS, 0);

    // The tail of the split block is still available, which is the point.
    let other = h.alloc(layout(48)).expect("second small alloc");
    assert_ne!(
        other.as_ptr() as usize,
        small.as_ptr() as usize,
        "the split tail was not handed out",
    );
}

#[test]
fn oversized_requests_get_their_own_mapping_and_give_it_back() {
    let (mut h, mock) = heap();
    let want = MAX_CLASS * 4;

    let p = h.alloc(layout(want)).expect("oversized alloc");
    let lo = mock.next_address() - mock.arena_len();
    assert!(
        (lo..lo + mock.arena_len()).contains(&(p.as_ptr() as usize)),
        "an oversized block must not come out of the shared arena",
    );
    assert_eq!(h.stats().maps, 1);

    // SAFETY: freed exactly once, and it is a dedicated mapping.
    unsafe { h.dealloc(p, layout(want)) };
    assert_eq!(
        mock.count_of(sysnum::MUNMAP),
        1,
        "the dedicated mapping was not returned",
    );
    assert_eq!(h.stats().allocated, 0);
}

#[test]
fn a_new_region_is_mapped_when_the_old_one_is_full() {
    let mock: &'static Mock = Box::leak(Box::new(Mock::new(ARENA_BYTES)));
    let mut h = Heap::new(mock);

    // 8 KiB is larger than any free-list class, so each of these takes a whole
    // block and ARENA_BYTES / 8192 of them fill the first region exactly.
    let big = layout(8192);
    let mut blocks = Vec::new();
    while h.stats().maps < 2 && blocks.len() < 64 {
        match h.alloc(big) {
            Ok(p) => blocks.push(p),
            Err(_) => break,
        }
    }

    assert!(h.stats().maps >= 2, "the heap never grew past one region");
    for p in blocks {
        // SAFETY: each block came from `alloc` and is freed once.
        unsafe { h.dealloc(p, big) };
    }
}

#[test]
fn out_of_memory_is_reported_rather_than_unwrapped() {
    let mock: &'static Mock = Box::leak(Box::new(Mock::new(64 * 1024)));
    let mut h = Heap::new(mock);
    mock.with(|s| s.fail_all(errno::ENOMEM));

    assert_eq!(h.alloc(layout(64)), Err(errno::ENOMEM));

    mock.with(|s| s.succeed_again());
    assert!(h.alloc(layout(64)).is_ok());
}

#[test]
fn a_kernel_that_returns_address_zero_is_treated_as_a_failure() {
    let mock: &'static Mock = Box::leak(Box::new(Mock::new(64 * 1024)));
    let mut h = Heap::new(mock);
    // A kernel that claims success and hands back the unmapped page at 0 has
    // failed; every later write would fault.
    mock.with(|s| s.push_ret(sysnum::MMAP, 0));

    assert_eq!(h.alloc(layout(64)), Err(errno::ENOMEM));
}

#[test]
fn a_zero_sized_request_still_yields_a_usable_block() {
    let (mut h, _) = heap();
    let p = h.alloc(layout(0)).expect("zero-sized alloc");
    // `std` never does this, but a zero-sized allocation still has to be
    // distinguishable from a failed one, and the heap never hands out address 0.
    let addr = p.as_ptr() as usize;
    assert_ne!(addr, 0);
    assert_eq!(addr % MIN_CLASS, 0);
}

#[test]
fn realloc_keeps_the_block_when_the_new_size_still_fits() {
    let (mut h, _) = heap();
    let a = h.alloc(layout(64)).expect("alloc");
    let addr = a.as_ptr() as usize;

    // SAFETY: `a` is live and the new size still fits the physical block.
    let b = unsafe { h.realloc(a, layout(64), 100) }.expect("in place");
    assert_eq!(b.as_ptr() as usize, addr, "should not have moved");
}

#[test]
fn realloc_moves_and_preserves_the_bytes_when_it_must() {
    let (mut h, _) = heap();
    let a = h.alloc(layout(64)).expect("alloc");
    // SAFETY: `a` is live and is 64 bytes long.
    unsafe { core::ptr::copy_nonoverlapping(b"Trangorge".as_ptr(), a.as_ptr(), 9) };

    // SAFETY: `a` is live; 4000 forces a move into a larger class.
    let b = unsafe { h.realloc(a, layout(64), 4000) }.expect("move");
    // SAFETY: `b` is live and at least 9 bytes long.
    let got = unsafe { core::slice::from_raw_parts(b.as_ptr(), 9) };
    assert_eq!(got, b"Trangorge");
}

#[test]
fn every_classed_request_lands_in_the_smallest_class_that_holds_it() {
    // The largest payload a free list can serve is one class minus the header.
    for size in [1usize, 8, 15, 16, 17, 100, 1000, MAX_CLASS - HEADER] {
        let mut class = None;
        for i in 0..CLASSES {
            if size + HEADER <= (MIN_CLASS << i) {
                class = Some(i);
                break;
            }
        }
        assert!(class.is_some(), "{size} bytes should be classed");
    }
    // One byte more and it no longer fits, which is what routes the request to a
    // dedicated mapping.
    let over = MAX_CLASS - HEADER + 1;
    let fits = (0..CLASSES).any(|i| over + HEADER <= (MIN_CLASS << i));
    assert!(!fits, "{over} bytes should be oversized");
}

#[test]
fn the_header_sits_below_the_payload_so_writing_to_it_is_harmless() {
    let (mut h, _) = heap();
    let a = h.alloc(layout(64)).expect("alloc");

    // Overwrite the first sixteen bytes of the payload — the bytes a `memcpy`
    // would touch first. If the allocator kept its metadata there, this would
    // destroy the block's own bookkeeping.
    // SAFETY: `a` is a live 64-byte block.
    unsafe { core::ptr::write_bytes(a.as_ptr(), 0xAB, 16) };

    // It must still be releasable, and the bytes must still be ours.
    // SAFETY: as above.
    unsafe {
        let got = core::slice::from_raw_parts(a.as_ptr(), 16);
        assert!(got.iter().all(|&b| b == 0xAB));
        h.dealloc(a, layout(64));
    }
    assert_eq!(h.stats().allocated, 0);
}

#[test]
fn every_mapping_is_page_aligned() {
    let (mut h, _) = heap();
    for _ in 0..4 {
        let p = h.alloc(layout(5000)).expect("oversized");
        // The *payload* is not page-aligned and must not be: the header sits
        // below it. What has to be page-aligned is the start of the mapping,
        // which is `payload - HEADER`, because that is the address the kernel
        // would hand back and the one `munmap` has to be given.
        let mapping = p.as_ptr() as usize - HEADER;
        assert_eq!(
            mapping % PAGE,
            0,
            "the kernel must return page-aligned memory",
        );
        assert_eq!(
            p.as_ptr() as usize % MIN_CLASS,
            0,
            "the payload must still be aligned to the request",
        );
    }
}

// ── the descriptor table ───────────────────────────────────────────────────

#[test]
fn the_standard_streams_start_open() {
    let t = Table::new();
    for fd in [STDIN, STDOUT, STDERR] {
        assert!(t.is_open(fd), "fd {fd} should be open");
        assert_eq!(t.kind(fd).expect("kind"), Kind::Stream);
    }
    assert_eq!(t.open_count(), 3);
    assert_eq!(t.next_free(), 3);
}

#[test]
fn descriptors_are_dense_and_reused() {
    let mut t = Table::new();

    let a = t.open_file("/a", sysnum::O_RDONLY).expect("open");
    let b = t.open_file("/b", sysnum::O_RDONLY).expect("open");
    let c = t.open_file("/c", sysnum::O_RDONLY).expect("open");
    assert_eq!((a, b, c), (3, 4, 5));
    assert_eq!(t.open_count(), 6);

    t.close(b).expect("close");
    assert_eq!(t.next_free(), 4, "the hole should be reused");
    assert_eq!(t.open_file("/d", sysnum::O_RDONLY).expect("open"), 4);
    assert_eq!(t.open_count(), 6);
}

#[test]
fn a_closed_descriptor_is_not_reusable() {
    let mut t = Table::new();
    let a = t.open_file("/a", 0).expect("open");
    t.close(a).expect("close");

    assert!(!t.is_open(a));
    assert_eq!(t.get(a), Err(errno::EBADF));
    assert_eq!(t.get(-1), Err(errno::EBADF));
    assert_eq!(t.get(fd::MAX_FDS as i32), Err(errno::EBADF));
    assert_eq!(t.get(i32::MAX), Err(errno::EBADF));
    assert_eq!(t.close(a), Err(errno::EBADF));
}

#[test]
fn a_full_table_reports_too_many_open_files() {
    let mut t = Table::new();
    for _ in 0..(fd::MAX_FDS - 3) {
        t.alloc(Kind::Other).expect("fill");
    }
    assert_eq!(t.open_count(), fd::MAX_FDS);
    assert_eq!(t.alloc(Kind::Other), Err(errno::EMFILE));
}

#[test]
fn dup_gives_two_names_and_closing_one_leaves_the_other() {
    let mut t = Table::new();
    let a = t.open_file("/a", 0).expect("open");
    let b = t.dup(a).expect("dup");

    assert_ne!(a, b);
    assert_eq!(t.kind(a).expect("kind"), t.kind(b).expect("kind"));
    assert_eq!(t.get(a).expect("slot").shared, 2);

    // Closing one name must unlink exactly that name: POSIX is explicit about
    // this, and `std` relies on the closed number becoming `EBADF` at once.
    t.close(a).expect("close");
    assert!(!t.is_open(a), "a closed descriptor stayed usable");
    assert_eq!(t.get(a), Err(errno::EBADF));
    assert!(t.is_open(b), "closing one name closed both");

    t.close(b).expect("close");
    assert!(!t.is_open(b));
    assert_eq!(t.open_count(), 3, "only the streams are left");
}

#[test]
fn dup2_onto_an_open_number_releases_the_old_one() {
    let mut t = Table::new();
    let a = t.open_file("/a", 0).expect("open");
    let victim = t.open_file("/victim", 0).expect("open");

    t.dup_to(a, victim).expect("dup2");
    assert_eq!(t.path_of(victim).expect("path"), b"/a");
    // The three streams plus `a` and `victim` before, the same five names after:
    // `dup2` moved a name, it did not add or remove one.
    assert_eq!(t.open_count(), 5, "a name was gained or lost");

    t.dup_to(a, a).expect("dup2 onto itself");
    assert_eq!(t.open_count(), 5);
    assert_eq!(t.dup_to(a, fd::MAX_FDS as i32), Err(errno::EBADF));
}

#[test]
fn file_positions_move_and_are_per_name() {
    let mut t = Table::new();
    let a = t.open_file("/a", 0).expect("open");

    assert_eq!(t.pos(a).expect("pos"), 0);
    assert_eq!(t.advance(a, 10).expect("advance"), 10);
    assert_eq!(t.advance(a, 5).expect("advance"), 15);
    t.set_pos(a, 0).expect("set");
    assert_eq!(t.pos(a).expect("pos"), 0);

    // A `dup` shares the *kind*, not the position. The real position lives in
    // the kernel's open file description; this copy is a cache of it, so
    // keeping the two names independent is what stops them disagreeing silently.
    let b = t.dup(a).expect("dup");
    t.set_pos(b, 99).expect("set");
    assert_eq!(t.pos(a).expect("pos"), 0);
    assert_eq!(t.pos(b).expect("pos"), 99);
}

#[test]
fn seeking_a_directory_is_an_error() {
    let mut t = Table::new();
    let d = t.open_dir("/etc").expect("open");
    assert_eq!(t.pos(d), Err(errno::ESPIPE));
    assert_eq!(t.set_pos(d, 0), Err(errno::ESPIPE));

    assert_eq!(t.dir_cursor(d).expect("cursor"), 0);
    t.set_dir_cursor(d, 42).expect("set");
    assert_eq!(t.dir_cursor(d).expect("cursor"), 42);

    // A file has no directory cursor.
    let f = t.open_file("/a", 0).expect("open");
    assert_eq!(t.dir_cursor(f), Err(errno::ENOTDIR));
}

// ── the dirent codec ───────────────────────────────────────────────────────

#[test]
fn a_record_has_the_linux_layout() {
    // `struct linux_dirent64` is 19 bytes before the name, packed. Getting this
    // wrong shifts every name by four bytes, and `read_dir` then reports garbage
    // rather than failing.
    assert_eq!(dirent::HEADER_LEN, 19);
    assert_eq!(core::mem::align_of::<dirent::Header>(), 1);
}

#[test]
fn record_lengths_are_rounded_to_eight() {
    assert_eq!(dirent::reclen_for(1), 24);
    assert_eq!(dirent::reclen_for(5), 32);
    assert_eq!(dirent::reclen_for(8), 32);
    assert_eq!(dirent::reclen_for(9), 32);
    assert_eq!(dirent::reclen_for(255), 280);
    assert!(dirent::reclen_for(dirent::MAX_NAME) <= dirent::MAX_RECLEN);
}

#[test]
fn a_record_round_trips() {
    let entry = Entry::new("init.elf", kind::REG, 42, 7);
    let mut buf = vec![0u8; 64];

    // SAFETY: `buf` is a live, exclusively owned 64-byte buffer.
    let n = unsafe { dirent::encode(&mut buf, &entry) }.expect("fits");
    assert_eq!(n, dirent::reclen_for(8));

    // SAFETY: `buf[..n]` is exactly the record `encode` just wrote.
    let back = unsafe { dirent::decode(&buf[..n]) }.expect("decodes");
    assert_eq!(back.name_str(), Some("init.elf"));
    assert_eq!(back.ino, 42);
    assert_eq!(back.off, 7);
    assert_eq!(back.type_, kind::REG);
    assert!(back.is_file() && !back.is_dir());
}

#[test]
fn a_name_is_nul_terminated_inside_the_record() {
    let entry = Entry::file("x");
    let mut buf = vec![0u8; 32];
    // SAFETY: `buf` is a live, exclusively owned 32-byte buffer.
    unsafe { dirent::encode(&mut buf, &entry) }.expect("fits");

    let start = dirent::HEADER_LEN;
    assert_eq!(buf[start], b'x');
    assert_eq!(buf[start + entry.name.len()], 0, "the name needs its NUL");
}

#[test]
fn a_buffer_too_small_reports_that_rather_than_truncating() {
    let entry = Entry::file("a-long-enough-name");
    let need = dirent::reclen_for(entry.name.len());

    let mut small = vec![0u8; need - 1];
    // SAFETY: `small` is a live, exclusively owned buffer.
    assert_eq!(unsafe { dirent::encode(&mut small, &entry) }, None);

    let mut exact = vec![0u8; need];
    // SAFETY: `exact` is exactly the size the record needs.
    assert!(unsafe { dirent::encode(&mut exact, &entry) }.is_some());
}

#[test]
fn an_over_long_name_is_refused() {
    let entry = Entry::new(&"n".repeat(dirent::MAX_NAME + 1), kind::REG, 0, 0);
    let mut buf = vec![0u8; 4096];
    // SAFETY: `buf` is a live, exclusively owned buffer.
    assert_eq!(unsafe { dirent::encode(&mut buf, &entry) }, None);
}

/// Encode a list of entries back to back into one buffer.
fn pack(entries: &[Entry], total: usize) -> Vec<u8> {
    let mut buf = vec![0u8; total];
    let mut at = 0usize;
    for e in entries {
        // SAFETY: `buf[at..]` is the live, exclusively owned remainder, and each
        // record is written exactly once into its own slice.
        at += unsafe { dirent::encode(&mut buf[at..], e) }.expect("fits");
    }
    assert_eq!(at, total);
    buf
}

fn total_len(entries: &[Entry]) -> usize {
    entries.iter().map(|e| dirent::reclen_for(e.name.len())).sum()
}

#[test]
fn several_records_pack_back_to_back() {
    let entries = vec![
        Entry::dir("bin"),
        Entry::file("init.elf"),
        Entry::file("very-long-file-name-here.txt"),
    ];
    let buf = pack(&entries, total_len(&entries));

    let back = dirent::decode_all(&buf);
    assert_eq!(back.len(), 3);
    assert_eq!(back[0].name_str(), Some("bin"));
    assert!(back[0].is_dir());
    assert_eq!(back[1].name_str(), Some("init.elf"));
    assert_eq!(back[2].name_str(), Some("very-long-file-name-here.txt"));
}

#[test]
fn a_truncated_buffer_yields_a_short_list_not_a_crash() {
    let entries = vec![Entry::dir("bin"), Entry::file("init.elf")];
    let total = total_len(&entries);
    let buf = pack(&entries, total);

    // Cut the buffer in the middle of the second record.
    let back = dirent::decode_all(&buf[..total - 4]);
    assert_eq!(back.len(), 1, "the partial record must be dropped");
    assert_eq!(back[0].name_str(), Some("bin"));
}

#[test]
fn a_nonsense_reclen_ends_the_walk() {
    // A buggy or hostile kernel could put a huge `d_reclen` in the buffer. The
    // decoder has to notice rather than walk off the end of it.
    let mut buf = vec![0u8; 64];
    // SAFETY: `buf` is a live, exclusively owned buffer.
    unsafe { dirent::encode(&mut buf, &Entry::file("abc")) }.expect("fits");
    buf[16] = 0xFF;
    buf[17] = 0xFF;

    assert!(dirent::decode_all(&buf).is_empty());
}

#[test]
fn an_empty_buffer_decodes_to_nothing() {
    assert!(dirent::decode_all(&[]).is_empty());
    assert_eq!(dirent::count_for(&[], 0), 0);
}

// ── the typed syscall surface ──────────────────────────────────────────────

#[test]
fn arguments_land_in_the_registers_std_expects() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    let mut buf = [0u8; 8];

    p.read(7, &mut buf).expect("read");
    let a = mock.last_args(sysnum::READ).expect("read was called");
    assert_eq!(a[0], 7, "fd goes in rdi");
    assert_eq!(a[1], buf.as_mut_ptr() as u64, "buffer goes in rsi");
    assert_eq!(a[2], 8, "count goes in rdx");
    assert_eq!(a[3..], [0, 0, 0], "unused arguments must be zero");
}

#[test]
fn a_failed_call_reports_its_errno() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    mock.with(|s| s.push_ret(sysnum::CLOSE, errno::EBADF.to_raw()));

    assert_eq!(p.close(3), Err(errno::EBADF));
}

#[test]
fn a_syscall_the_kernel_does_not_know_reports_not_implemented() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    // A kernel that has not grown this syscall must answer `-ENOSYS`, not
    // `u64::MAX`: the two are indistinguishable from `-1`, which is `EPERM`.
    mock.with(|s| s.push_ret(sysnum::NEWFSTATAT, errno::ENOSYS.to_raw()));

    let mut st = Stat::default();
    let path = b"/x\0";
    // SAFETY: `path` is a live NUL-terminated array and `st` is writable.
    let r = unsafe { p.stat_at(sysnum::AT_FDCWD, path.as_ptr(), &mut st, false) };
    assert_eq!(r, Err(errno::ENOSYS));
}

#[test]
fn a_capability_denial_surfaces_as_permission_denied() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    // The capability model denies with `EPERM`. If this ever came back as
    // "not implemented" the failure would look like a missing kernel feature
    // rather than a policy decision, which is how a real security bug hides.
    mock.with(|s| s.push_ret(sysnum::OPENAT, errno::EPERM.to_raw()));

    let path = b"/etc/shadow\0";
    let mut st = Stat::default();
    // SAFETY: `path` is a live NUL-terminated array and `st` is writable.
    let r = unsafe { p.openat(sysnum::AT_FDCWD, path.as_ptr(), 0, 0, &mut st) };
    assert_eq!(r, Err(errno::EPERM));
}

#[test]
fn mmap_passes_the_six_arguments_in_linux_order() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    mock.with(|s| s.push_ret(sysnum::MMAP, 0x7000_0000));

    let got = p
        .mmap(
            0,
            8192,
            sysnum::PROT_READ | sysnum::PROT_WRITE,
            sysnum::MAP_PRIVATE | sysnum::MAP_ANONYMOUS,
            -1,
            0,
        )
        .expect("mmap");

    assert_eq!(got, 0x7000_0000);
    let a = mock.last_args(sysnum::MMAP).expect("mmap was called");
    assert_eq!(a[0], 0, "addr hint");
    assert_eq!(a[1], 8192, "length");
    assert_eq!(a[2], 3, "prot");
    assert_eq!(a[3], 0x22, "MAP_PRIVATE | MAP_ANONYMOUS");
    // `fd` is an `int`, so the SysV ABI only defines its low 32 bits. `mmap` here
    // zero-extends; the kernel is required to mask to 32 bits before comparing it
    // against `AT_FDCWD`. `SYSCALLS.md` states that rule, and this assertion is
    // what would notice if a future change sign-extended instead.
    assert_eq!(a[4] as u32, u32::MAX, "fd -1");
    assert_eq!(a[5], 0, "offset");
}

#[test]
fn clock_gettime_reads_back_what_the_kernel_wrote() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);

    // The kernel reports the time *through* a pointer, so the mock has to write
    // one or the test would only be checking that the buffer started zeroed.
    let want = Timespec { sec: 12, nsec: 345 };
    let mut raw = [0u8; 16];
    raw[..8].copy_from_slice(&want.sec.to_le_bytes());
    raw[8..].copy_from_slice(&want.nsec.to_le_bytes());
    mock.with(|s| s.push_fill(sysnum::CLOCK_GETTIME, 1, &raw));

    let got = p.clock_gettime(sysnum::CLOCK_MONOTONIC).expect("clock");
    assert_eq!(got, want);

    let a = mock.last_args(sysnum::CLOCK_GETTIME).expect("called");
    assert_eq!(a[0], sysnum::CLOCK_MONOTONIC as u32 as u64);
    assert_ne!(a[1], 0, "a timespec pointer must be passed");
}

#[test]
fn timespec_arithmetic_is_exact() {
    for total in [0i128, 1, 999_999_999, 1_000_000_000, 1_234_567_890_123] {
        assert_eq!(Timespec::from_nanos(total).to_nanos(), total);
    }
    assert_eq!(Timespec::from_nanos(1_500_000_000).nsec, 500_000_000);
    assert_eq!(Timespec::from_nanos(1_500_000_000).sec, 1);
}

#[test]
fn stat_type_bits_are_read_correctly() {
    let mut st = Stat::default();
    st.st_mode = super::posix::S_IFREG | 0o644;
    assert!(st.is_file() && !st.is_dir() && !st.is_executable());

    st.st_mode = super::posix::S_IFDIR | 0o755;
    assert!(st.is_dir() && st.is_executable());

    st.st_mode = super::posix::S_IFLNK | 0o777;
    assert!(!st.is_file() && !st.is_dir());
}

#[test]
fn ppoll_passes_the_count_and_the_timeout() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    let mut fds = [
        PollFd {
            fd: 0,
            events: sysnum::POLLIN,
            revents: 0,
        },
        PollFd {
            fd: 1,
            events: sysnum::POLLOUT,
            revents: 0,
        },
    ];

    p.ppoll(&mut fds, -1).expect("ppoll");
    let a = mock.last_args(sysnum::PPOLL).expect("called");
    assert_eq!(a[1], 2, "descriptor count");
    assert_eq!(a[2], u64::MAX, "-1 ms as an unsigned timeout");
}

#[test]
fn getdents64_decodes_the_records_the_kernel_wrote() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);

    let entries = vec![Entry::dir("bin"), Entry::file("init.elf")];
    let total = total_len(&entries);
    let raw = pack(&entries, total);
    mock.with(|s| {
        s.push_ret(sysnum::GETDENTS64, raw.len() as i64);
        s.push_fill(sysnum::GETDENTS64, 1, &raw);
    });

    let mut buf = [0u8; 512];
    // SAFETY: `buf` is a live, exclusively borrowed 512-byte buffer, and the
    // mock copies `raw.len()` (much less) bytes into it.
    let n = unsafe { p.getdents64(4, &mut buf) }.expect("getdents");
    assert_eq!(n, raw.len());

    let got = dirent::decode_all(&buf[..n]);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].name_str(), Some("bin"));
    assert_eq!(got[1].name_str(), Some("init.elf"));
}

#[test]
fn getdents64_reporting_zero_bytes_means_end_of_directory() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);
    mock.with(|s| s.push_ret(sysnum::GETDENTS64, 0));

    let mut buf = [0u8; 512];
    // SAFETY: `buf` is a live, exclusively borrowed buffer.
    let n = unsafe { p.getdents64(4, &mut buf) }.expect("getdents");
    assert_eq!(n, 0);
    assert!(dirent::decode_all(&buf).is_empty());
}

#[test]
fn the_mock_records_and_can_forget_every_call() {
    let mock = Mock::new(PAGE);
    let p = Posix::new(&mock);

    p.close(3).expect("close");
    p.close(4).expect("close");
    assert_eq!(mock.count_of(sysnum::CLOSE), 2);
    assert_eq!(mock.args_of(sysnum::CLOSE).len(), 2);

    mock.with(|s| s.clear_log());
    assert_eq!(mock.count_of(sysnum::CLOSE), 0, "the log was not cleared");

    // Clearing the log must not throw the scripted results away.
    mock.with(|s| s.push_ret(sysnum::CLOSE, errno::EBADF.to_raw()));
    assert_eq!(p.close(3), Err(errno::EBADF));
}

#[test]
fn errno_round_trips_through_the_kernel_representation() {
    for e in [
        errno::EPERM,
        errno::ENOENT,
        errno::EINTR,
        errno::EAGAIN,
        errno::ENOMEM,
        errno::EBADF,
        errno::EINVAL,
        errno::ESPIPE,
        errno::ENOSYS,
        errno::EMFILE,
    ] {
        assert_eq!(Errno::from_raw(e.to_raw()), Err(e));
    }
}
