//! System and navigation commands: the ones that ask about the machine, plus
//! `find` and the path-splitting utilities.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{parse_flags, CmdResult, Ctx};

/// `uname` — the system name, `-a` for the full line.
pub fn uname(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, _) = parse_flags(args);
    let name = ctx.sys.uname();
    if flags.contains('a') {
        let (total, free_bytes) = ctx.sys.memory();
        let mib = 1024 * 1024;
        CmdResult::ok(vec![format!(
            "{name} tgs r1 {}/{} MiB {} cpu(s)",
            free_bytes / mib,
            total / mib,
            ctx.sys.cpus()
        )])
    } else {
        CmdResult::ok(vec![name])
    }
}

/// `free` — memory usage.
pub fn free(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let (total, free_bytes) = ctx.sys.memory();
    let mib = 1024 * 1024;
    let used = total.saturating_sub(free_bytes);
    CmdResult::ok(vec![format!(
        "              total        used        free\nMem:  {:>9} MiB {:>9} MiB {:>9} MiB",
        total / mib,
        used / mib,
        free_bytes / mib
    )])
}

/// `uptime` — time since boot.
pub fn uptime(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let s = ctx.sys.uptime();
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    let cpus = ctx.sys.cpus();
    CmdResult::ok(vec![format!("up {h}h {m}m {sec}s, {cpus} cpu(s)")])
}

/// `df` — filesystem usage, as far as the block device can report it.
pub fn df(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let (total, free_bytes) = ctx.sys.memory();
    let mib = 1024 * 1024;
    let used = total.saturating_sub(free_bytes);
    // Integer division on purpose: a kernel with no FPU still has to produce
    // this, and a truncated percentage is not one a user acts on.
    let pct = if total == 0 { 0 } else { used * 100 / total };
    CmdResult::ok(vec![
        "Filesystem      Size  Used Avail Use%".to_string(),
        format!(
            "/dev/root      {}M {}M {}M {pct}%",
            total / mib,
            used / mib,
            free_bytes / mib
        ),
    ])
}

/// `echo` — print its arguments. `-n` omits the trailing newline.
pub fn echo(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let _ = ctx;
    // `parse_flags` has already stripped `-n`, so the remainder is the text.
    let _ = flags;
    CmdResult::ok(vec![rest.to_string()])
}

/// `seq` — a range of numbers. `seq N` or `seq FIRST LAST`.
pub fn seq(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let _ = ctx;
    let parts: Vec<i64> = args
        .split_whitespace()
        .filter_map(|p| p.parse::<i64>().ok())
        .collect();
    let (first, last) = match parts.len() {
        0 => return CmdResult::err("seq: usage: seq N | seq FIRST LAST"),
        1 => (1, parts[0]),
        2 => (parts[0], parts[1]),
        _ => return CmdResult::err("seq: usage: seq N | seq FIRST LAST"),
    };
    if last < first {
        // Descending is not supported; an empty range would look like a hang.
        return CmdResult::err("seq: LAST must not be below FIRST");
    }
    // Capped so `seq 1 1000000000` cannot exhaust memory in a kernel with no
    // swap. The cap is reported, not silent.
    const MAX: i64 = 100_000;
    if last - first + 1 > MAX {
        return CmdResult::err(format!("seq: refusing to print more than {MAX} lines"));
    }
    CmdResult::ok((first..=last).map(|i| i.to_string()).collect())
}

/// `basename` — the last component of a path.
pub fn basename(_ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let p = args.trim();
    if p.is_empty() {
        return CmdResult::err("basename: usage: basename <path>");
    }
    let p = p.trim_end_matches('/');
    CmdResult::ok(vec![p.rsplit('/').next().unwrap_or(p).to_string()])
}

/// `dirname` — everything but the last component.
pub fn dirname(_ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let p = args.trim();
    if p.is_empty() {
/// `find` — paths under a directory.
///
/// `-name` takes a `*` glob (only `*`; no `?` or character classes, for the
/// reason the module docs give), `-type d|f` restricts by kind. With no `-type`
/// both kinds match, which is why the default is "unfiltered" rather than
/// "files only".
pub fn find(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let mut root = ".";
    let mut name_pattern: Option<&str> = None;
    let mut want_dir = false;
    let mut want_file = false;
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "-name" => {
                i += 1;
                name_pattern = tokens.get(i).copied();
            }
            "-type" => {
                i += 1;
                match tokens.get(i) {
                    Some(&"d") => want_dir = true,
                    Some(&"f") => want_file = true,
                    _ => {}
                }
            }
            other if !other.starts_with('-') && root == "." => root = other,
            _ => {}
        }
        i += 1;
    }

    let start = ctx.resolve(root);
    let mut out = Vec::new();
    walk_find(ctx, &start, name_pattern, want_dir, want_file, &mut out);
    CmdResult::ok(out)
}

fn walk_find(
    ctx: &mut Ctx<'_>,
    path: &str,
    pattern: Option<&str>,
    want_dir: bool,
    want_file: bool,
    out: &mut Vec<String>,
) {
    let Ok(entries) = ctx.fs.read_dir(path) else {
        return;
    };
    for e in entries {
        let type_ok = (want_dir && e.is_dir)
            || (want_file && !e.is_dir)
            || (!want_dir && !want_file);
        if !type_ok {
            continue;
        }
        let full = format!("{}/{}", path.trim_end_matches('/'), e.name);
        if pattern.map_or(true, |p| glob_match(p, &e.name)) {
            out.push(full.clone());
        }
        if e.is_dir {
            walk_find(ctx, &full, pattern, want_dir, want_file, out);
        }
    }
}

/// Match a `*` glob, anchoring the first and last pieces.
fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !name.starts_with(first) || !name.ends_with(last) {
        return false;
    }
    if name.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &name[first.len()..name.len() - last.len()];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    true
}

/// `test` / `[` — the POSIX conditional, for scripts.
///
/// Compares strings and tests existence. Exits non-zero when the test is false,
/// which is what makes `test -f x && cat x` work.
pub fn test(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let a = args.trim();
    // `[ foo ]` is the shell spelling; the bracket is not part of the test.
    let a = a.strip_prefix('[').unwrap_or(a);
    let a = a.strip_suffix(']').unwrap_or(a);
    let parts: Vec<&str> = a.split_whitespace().collect();

    let ok = match parts.as_slice() {
        [] => false,
        [one] => !one.is_empty(),
        [left, "=", right] | [left, "==", right] => left == right,
        [left, "!=", right] => left != right,
        [flag, path] => match *flag {
            "-e" => ctx.fs.read_file(path).is_ok() || ctx.fs.read_dir(path).is_ok(),
            "-f" => ctx.fs.read_file(path).is_ok(),
            "-d" => ctx.fs.read_dir(path).is_ok(),
            "-z" => path.is_empty(),
            "-n" => !path.is_empty(),
            _ => false,
        },
        _ => false,
    };

    if ok {
        CmdResult::silent()
    } else {
        CmdResult::fail(Vec::new())
    }
}

/// `true` — succeed. For scripts and for `&&` chains.
pub fn true_cmd(_ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::silent()
}

/// `false` — fail.
pub fn false_cmd(_ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::fail(Vec::new())
}

        return CmdResult::err("dirname: usage: dirname <path>");
    }
    match p.trim_end_matches('/').rsplit_once('/') {
        Some(("", _)) => CmdResult::ok(vec!["/".to_string()]),
        Some((d, _)) => CmdResult::ok(vec![d.to_string()]),
        None => CmdResult::ok(vec![".".to_string()]),
    }
}

/// `find` — paths under a directory.
///
/// `-name` takes a `*` glob (only `*`; no `?` or character classes, for the
/// reason the module docs give), `-type d|f` restricts by kind. With no `-type`
/// both kinds match, which is why the default is "unfiltered" rather than
/// "files only".
pub fn find(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let mut root = ".";
    let mut name_pattern: Option<&str> = None;
    let mut want_dir = false;
    let mut want_file = false;
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "-name" => {
                i += 1;
                name_pattern = tokens.get(i).copied();
            }
            "-type" => {
                i += 1;
                match tokens.get(i) {
                    Some(&"d") => want_dir = true,
                    Some(&"f") => want_file = true,
                    _ => {}
                }
            }
            other if !other.starts_with('-') && root == "." => root = other,
            _ => {}
        }
        i += 1;
    }

    let start = ctx.resolve(root);
    let mut out = Vec::new();
    walk_find(ctx, &start, name_pattern, want_dir, want_file, &mut out);
    CmdResult::ok(out)
}

fn walk_find(
    ctx: &mut Ctx<'_>,
    path: &str,
    pattern: Option<&str>,
    want_dir: bool,
    want_file: bool,
    out: &mut Vec<String>,
) {
    let Ok(entries) = ctx.fs.read_dir(path) else {
        return;
    };
    for e in entries {
        let type_ok = (want_dir && e.is_dir)
            || (want_file && !e.is_dir)
            || (!want_dir && !want_file);
        if !type_ok {
            continue;
        }
        let full = format!("{}/{}", path.trim_end_matches('/'), e.name);
        if pattern.map_or(true, |p| glob_match(p, &e.name)) {
            out.push(full.clone());
        }
        if e.is_dir {
            walk_find(ctx, &full, pattern, want_dir, want_file, out);
        }
    }
}

/// Match a `*` glob, anchoring the first and last pieces.
fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !name.starts_with(first) || !name.ends_with(last) {
        return false;
    }
    if name.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &name[first.len()..name.len() - last.len()];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    true
}

/// `test` / `[` — the POSIX conditional, for scripts.
///
/// Compares strings and tests existence. Exits non-zero when the test is false,
/// which is what makes `test -f x && cat x` work.
pub fn test(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let a = args.trim();
    // `[ foo ]` is the shell spelling; the bracket is not part of the test.
    let a = a.strip_prefix('[').unwrap_or(a);
    let a = a.strip_suffix(']').unwrap_or(a);
    let parts: Vec<&str> = a.split_whitespace().collect();

    let ok = match parts.as_slice() {
        [] => false,
        [one] => !one.is_empty(),
        [left, "=", right] | [left, "==", right] => left == right,
        [left, "!=", right] => left != right,
        [flag, path] => match *flag {
            "-e" => ctx.fs.read_file(path).is_ok() || ctx.fs.read_dir(path).is_ok(),
            "-f" => ctx.fs.read_file(path).is_ok(),
            "-d" => ctx.fs.read_dir(path).is_ok(),
            "-z" => path.is_empty(),
            "-n" => !path.is_empty(),
            _ => false,
        },
        _ => false,
    };

    if ok {
        CmdResult::silent()
    } else {
        CmdResult::fail(Vec::new())
    }
}

/// `true` — succeed. For scripts and for `&&` chains.
pub fn true_cmd(_ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::silent()
}

/// `false` — fail.
pub fn false_cmd(_ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::fail(Vec::new())
}
