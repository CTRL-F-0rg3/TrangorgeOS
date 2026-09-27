//! File-oriented commands: listing, reading, copying, removing.
//!
//! Each one takes a [`Ctx`] and returns a [`CmdResult`], so a test can call
//! `commands::files::ls` directly without going through the shell.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{parse_flags, CmdResult, Ctx};
use crate::apps::shell::DirEntry;

/// `ls` — list a directory.
///
/// Accepts the flags that change what is *shown* rather than how it is paged,
/// which is the subset that makes sense with no pager: `-l` long, `-a` dotfiles,
/// `-h` human sizes, `-R` recurse.
pub fn ls(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let long = flags.contains('l');
    let all = flags.contains('a');
    let human = flags.contains('h');
    let recurse = flags.contains('R');

    let targets: Vec<String> = if rest.is_empty() {
        vec![ctx.session.cwd.clone()]
    } else {
        rest.split_whitespace().map(|s| ctx.resolve(s)).collect()
    };

    // Several targets get a header each, so entries can be attributed; one does
    // not, because a lone header is noise.
    let multi = targets.len() > 1;
    let mut out: Vec<String> = Vec::new();
    let mut failed = false;

    for target in targets {
        if multi {
            out.push(format!("{}:", target));
        }
        match list_one(ctx, &target, long, all, human, recurse, &mut out) {
            Ok(()) => {}
            Err(e) => {
                out.push(format!("ls: {e}"));
                failed = true;
            }
        }
    }

    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

fn list_one(
    ctx: &mut Ctx<'_>,
    path: &str,
    long: bool,
    all: bool,
    human: bool,
    recurse: bool,
    out: &mut Vec<String>,
) -> Result<(), String> {
    let entries = ctx.fs.read_dir(path)?;
    let mut items: Vec<DirEntry> = entries
        .into_iter()
        .filter(|e| all || !e.name.starts_with('.'))
        .collect();
    // Directories first, then by name. A purely name-sorted listing interleaves
    // a directory with the files that happen to sort near it.
    items.sort_by(|a, b| (b.is_dir, &a.name).cmp(&(a.is_dir, &b.name)));

    if items.is_empty() {
        out.push("(empty)".to_string());
    }

    for e in items {
        let full = format!("{}/{}", path.trim_end_matches('/'), e.name);
        if long {
            let kind = if e.is_dir { 'd' } else { '-' };
            let size = if human {
                human_size(e.size)
            } else {
                e.size.to_string()
            };
            out.push(format!("{}rw-r--r-- {size:>8}  {}", kind, e.name));
        } else if e.is_dir {
            out.push(format!("{}/", e.name));
        } else {
            out.push(format!("{}  {} bytes", e.name, e.size));
        }

        if recurse && e.is_dir {
            // A directory that cannot be read is reported, not skipped: a silent
            // skip is indistinguishable from an empty directory.
            if let Err(e) = list_one(ctx, &full, long, all, human, recurse, out) {
                out.push(format!("ls: {full}: {e}"));
            }
        }
    }
    Ok(())
}

/// Render a byte count the way `ls -h` does.
fn human_size(bytes: u32) -> String {
    const UNITS: [&str; 4] = ["B", "K", "M", "G"];
    let mut value = bytes as u64;
    let mut unit = 0;
    while value >= 1024 && unit < UNITS.len() - 1 {
        value /= 1024;
        unit += 1;
    }
    if unit == 0 {
        format!("{value}B")
    } else {
        format!("{value}{}", UNITS[unit])
    }
}

/// `cat` — print files.
pub fn cat(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let targets: Vec<&str> = args.split_whitespace().collect();
    if targets.is_empty() {
        return CmdResult::err("cat: usage: cat <file>...");
    }

    let mut out = Vec::new();
    let mut failed = false;
    for t in targets {
        let path = ctx.resolve(t);
        match ctx.fs.read_file(&path) {
            Ok(data) => {
                // Strip exactly one trailing newline, as cat(1) does; a file
                // ending in one would otherwise print a blank line every time.
                let text = String::from_utf8_lossy(&data);
                let body = text.strip_suffix('\n').unwrap_or(&text);
                for line in body.lines() {
                    out.push(line.to_string());
                }
            }
            Err(e) => {
                out.push(format!("cat: {t}: {e}"));
                failed = true;
            }
        }
    }
    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

/// `cp` — copy a file.
///
/// A destination that is an existing directory receives the copy under the
/// source's name, which is what makes `cp a b/` mean "into b" rather than
/// "overwrite b".
pub fn cp(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (src, dst) = match two_args(args) {
        Some(v) => v,
        None => return CmdResult::err("cp: usage: cp <source> <destination>"),
    };
    let (src, dst) = (ctx.resolve(src), ctx.resolve(dst));

    let data = match ctx.fs.read_file(&src) {
        Ok(d) => d,
        Err(e) => return CmdResult::err(format!("cp: {src}: {e}")),
    };
    let dst = into_dir(ctx, &src, &dst);
    if !ctx.allowed(&dst) {
        return CmdResult::err("cp: outside this session's home");
    }
    match ctx.fs.write_file(&dst, &data) {
        Ok(()) => CmdResult::silent(),
        Err(e) => CmdResult::err(format!("cp: {dst}: {e}")),
    }
}

/// `mv` — move or rename, as copy then delete.
///
/// This is the one place where the userspace filesystem is weaker than a POSIX
/// one: a real rename is atomic, this is not, and a crash between the two leaves
/// both copies. The order is not negotiable — the source is removed only after
/// the destination is safely written, so a failed write cannot lose the file.
pub fn mv(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (src, dst) = match two_args(args) {
        Some(v) => v,
        None => return CmdResult::err("mv: usage: mv <source> <destination>"),
    };
    let (src, dst) = (ctx.resolve(src), ctx.resolve(dst));

    if !ctx.allowed(&src) || !ctx.allowed(&dst) {
        return CmdResult::err("mv: outside this session's home");
    }
    let data = match ctx.fs.read_file(&src) {
        Ok(d) => d,
        Err(e) => return CmdResult::err(format!("mv: {src}: {e}")),
    };
    let dst = into_dir(ctx, &src, &dst);
    if let Err(e) = ctx.fs.write_file(&dst, &data) {
        return CmdResult::err(format!("mv: {dst}: {e}"));
    }
    match ctx.fs.remove(&src) {
        Ok(()) => CmdResult::silent(),
        Err(e) => CmdResult::err(format!("mv: {src}: {e}")),
    }
}

/// If `dst` is a directory, put `src` inside it; otherwise leave it alone.
fn into_dir(ctx: &mut Ctx<'_>, src: &str, dst: &str) -> String {
    if ctx.fs.read_dir(dst).is_ok() {
        let name = src.rsplit('/').next().unwrap_or(src);
        format!("{}/{}", dst.trim_end_matches('/'), name)
    } else {
        dst.to_string()
    }
}

/// `rm` — remove files and directories.
///
/// `-r` recurses, `-f` ignores a missing target. A missing target *without*
/// `-f` is an error, as in rm(1): silence there hides a typo.
pub fn rm(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let recursive = flags.contains('r') || flags.contains('R');
    let force = flags.contains('f');
    if rest.is_empty() {
        return CmdResult::err("rm: usage: rm [-rf] <path>...");
    }

    let mut out = Vec::new();
    let mut failed = false;
    for target in rest.split_whitespace() {
        let path = ctx.resolve(target);
        if path == ctx.session.cwd {
            out.push(format!("rm: refusing to remove the working directory {path}"));
            failed = true;
            continue;
        }
        if !ctx.allowed(&path) {
            out.push(format!("rm: {path}: outside this session's home"));
            failed = true;
            continue;
        }
        if let Err(e) = remove_recursive(ctx, &path, recursive) {
            if !force {
                out.push(format!("rm: {target}: {e}"));
                failed = true;
            }
        }
    }
    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

fn remove_recursive(ctx: &mut Ctx<'_>, path: &str, recursive: bool) -> Result<(), String> {
    // The non-empty check comes before the removal, so a refusal leaves the tree
    // exactly as it was.
    if let Ok(entries) = ctx.fs.read_dir(path) {
        if !entries.is_empty() {
            if !recursive {
                return Err("is a directory (use -r)".to_string());
            }
            for e in entries {
                let child = format!("{}/{}", path.trim_end_matches('/'), e.name);
                remove_recursive(ctx, &child, true)?;
            }
        }
    }
    ctx.fs.remove(path)
}

/// `mkdir` — create directories, `-p` for parents.
pub fn mkdir(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let parents = flags.contains('p');
    if rest.is_empty() {
        return CmdResult::err("mkdir: usage: mkdir [-p] <dir>...");
    }

    let mut out = Vec::new();
    let mut failed = false;
    for target in rest.split_whitespace() {
        let path = ctx.resolve(target);
        if !ctx.allowed(&path) {
            out.push(format!("mkdir: {path}: outside this session's home"));
            failed = true;
            continue;
        }
        match ctx.fs.make_dir(&path) {
            Ok(()) => out.push(format!("created {path}")),
            Err(e) => {
                // With `-p`, an existing directory is the expected outcome, not a
                // failure — that is the point of the flag.
                if parents && e.contains("already exists") {
                    continue;
                }
                out.push(format!("mkdir: {target}: {e}"));
                failed = true;
            }
        }
    }
    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

/// `touch` — create an empty file, or leave an existing one alone.
///
/// The filesystem carries no timestamps, so touching an existing file is a no-op.
/// Said rather than pretended.
pub fn touch(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    if args.trim().is_empty() {
        return CmdResult::err("touch: usage: touch <file>...");
    }
    let mut out = Vec::new();
    let mut failed = false;
    for target in args.split_whitespace() {
        let path = ctx.resolve(target);
        if !ctx.allowed(&path) {
            out.push(format!("touch: {path}: outside this session's home"));
            failed = true;
            continue;
        }
        if ctx.fs.read_file(&path).is_ok() {
            continue;
        }
        if let Err(e) = ctx.fs.write_file(&path, b"") {
            out.push(format!("touch: {target}: {e}"));
            failed = true;
        }
    }
    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

/// Split an argument string into exactly two words.
///
/// A third word is an error rather than something to ignore: the user probably
/// wanted a glob, and silently dropping it would move the wrong file.
pub(super) fn two_args(args: &str) -> Option<(&str, &str)> {
    let mut it = args.split_whitespace();
    let a = it.next()?;
    let b = it.next()?;
    if it.next().is_some() {
        return None;
    }
    Some((a, b))
}
