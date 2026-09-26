//! `tgs-init` — a TrangorgeOS userspace program that uses the whole `std`.
//!
//! This is the proof that the sysroot works, and it is written the way an ordinary
//! Rust program is written: `Vec`, `String`, `HashMap`, `format!`, `println!`,
//! `std::fs`, `std::env`, `std::time`, `std::process`. There is no `unsafe` in this
//! file, and no `no_std`.
//!
//! Every section checks something the sysroot is actually responsible for, and
//! prints a verdict rather than assuming:
//!
//! | Section | Exercises |
//! |---------|-----------|
//! | collections, `format!` | the allocator, through `Vec` and `String` |
//! | `HashMap` | `getrandom`, without which a map cannot even be built |
//! | `std::env` | `getauxval`, the environment block, `argv` |
//! | `std::time` | `clock_gettime` |
//! | `std::fs` | `openat`, `fstat`, `read`, `close`, `getdents64` |
//! | `std::thread` | `pthread_create` — expected to refuse, and to say why |
//!
//! A section that cannot work says so and carries on. A demo that stopped at the
//! first `ENOSYS` would not tell anyone which parts of the sysroot are done.

use std::collections::HashMap;
use std::fs;
use std::time::{Instant, SystemTime};

fn main() {
    println!("TrangorgeOS userspace — real Rust std");
    println!("=====================================");

    // `std::env::args` reads the image `_start` found on the stack, or the
    // synthetic one it built when the kernel had not pushed one.
    let args: Vec<String> = std::env::args().collect();
    println!("args        : {args:?}");
    println!("os          : {}", std::env::consts::OS);
    println!("arch        : {}", std::env::consts::ARCH);
    println!(
        "PATH        : {}",
        std::env::var("PATH").unwrap_or_else(|_| "<unset>".into())
    );

    collections();
    hashing();
    timing();
    filesystem();
    threads();

    println!();
    println!("done — every section above ran the real standard library");
}

/// `Vec`, `String` and `format!`.
///
/// The point here is the allocator underneath: every one of these allocates, and
/// `std` has no other way to get memory.
fn collections() {
    let mut v: Vec<u32> = (1..=10_000).map(|i| i * 3).filter(|x| x % 2 == 0).collect();
    v.sort_unstable();
    v.dedup();
    let total: u64 = v.iter().map(|&x| x as u64).sum();

    let mut s = String::new();
    for chunk in v.chunks(3) {
        s.push_str(&format!("{chunk:?}"));
        s.push(' ');
    }

    println!();
    println!("--- collections ---");
    println!("vec         : {} elements, sum {total}", v.len());
    println!(
        "string      : {} bytes, starts {:?}",
        s.len(),
        &s[..40.min(s.len())]
    );
    println!(
        "capacity    : {} for {} elements (realloc exercised: {})",
        v.capacity(),
        v.len(),
        v.capacity() > v.len()
    );
}

/// `HashMap`, which cannot be built at all unless `getrandom` works.
fn hashing() {
    let words = ["kernel", "driver", "userspace", "capability", "gateway"];
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for w in words.iter().cycle().take(1_000) {
        *counts.entry(w).or_insert(0) += 1;
    }
    let mut sorted: Vec<(&str, u32)> = counts.into_iter().collect();
    sorted.sort_unstable_by_key(|(k, _)| *k);

    println!();
    println!("--- HashMap --- (needs getrandom)");
    for (k, v) in sorted {
        println!("  {k:<12} {v}");
    }
}

/// `std::time`, which is `clock_gettime` and nothing else.
fn timing() {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let t0 = Instant::now();
    let mut acc = 0u64;
    for i in 0..1_000_000u64 {
        acc = acc.wrapping_add(i);
    }
    let dt = t0.elapsed();

    println!();
    println!("--- time ---");
    println!("unix epoch  : {}.{:09}s", now.as_secs(), now.subsec_nanos());
    println!("monotonic   : {dt:?} for 1e6 adds (acc {acc})");
}

/// `std::fs`: the working directory, metadata, reading a file, and listing a
/// directory.
///
/// This is the section that depends most on the kernel, so each operation is
/// reported separately and a failure is a fact rather than a crash.
fn filesystem() {
    println!();
    println!("--- filesystem ---");

    match std::env::current_dir() {
        Ok(cwd) => println!("cwd         : {}", cwd.display()),
        Err(e) => println!("cwd         : unavailable ({e})"),
    }

    let path = std::env::var("TGS_DEMO_FILE").unwrap_or_else(|_| "/etc/motd".into());
    match fs::metadata(&path) {
        Ok(md) => {
            let kind = if md.is_dir() {
                "dir"
            } else if md.is_file() {
                "file"
            } else {
                "other"
            };
            println!("metadata    : {path} — {kind}, {} bytes", md.len());
        }
        Err(e) => println!("metadata    : {path} — {e}"),
    }

    match fs::read_to_string(&path) {
        Ok(text) => {
            let first = text.lines().next().unwrap_or("<empty>");
            println!("read        : {} bytes, first line {first:?}", text.len());
        }
        Err(e) => println!("read        : {path} — {e}"),
    }

    let dir = std::env::var("TGS_DEMO_DIR").unwrap_or_else(|_| "/".into());
    match fs::read_dir(&dir) {
        Ok(entries) => {
            // `read_dir` parses `linux_dirent64` records itself, so this is the
            // one place the dirent codec in `tgs-hal` is actually exercised.
            let mut names: Vec<String> = entries
                .filter_map(Result::ok)
                .map(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if e.path().is_dir() {
                        format!("{n}/")
                    } else {
                        n
                    }
                })
                .collect();
            names.sort();
            let shown = names.len().min(12);
            names.truncate(shown);
            println!("read_dir    : {dir} — {} entries", shown);
            for n in &names {
                println!("              {n}");
            }
        }
        Err(e) => println!("read_dir    : {dir} — {e}"),
    }
}

/// `std::thread::spawn`.
///
/// It does not work, and it *should* fail loudly: a second task would corrupt the
/// allocator, because the kernel programs no `fs_base`. Reporting that is more
/// useful than letting a program's first `spawn` quietly corrupt its heap.
fn threads() {
    println!();
    println!("--- threads ---");
    match std::thread::Builder::new().spawn(|| 1u32) {
        Ok(_h) => println!("spawn       : ok (the kernel grew fs_base)"),
        Err(e) => println!("spawn       : refused — {e}"),
    }
    println!(
        "parallelism : {:?}",
        std::thread::available_parallelism().map(|n| n.get())
    );
}
