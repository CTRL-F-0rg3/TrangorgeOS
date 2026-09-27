//! Text-processing commands: reading, filtering, sorting.
//!
//! # Patterns are literal
//!
//! `grep` matches literal substrings and `find -name` matches `*` globs, not
//! regular expressions. A regex engine is a large dependency for a feature most
//! use does not need, and a pattern that *looks* like a regex but silently is
//! not one is worse than one that documents what it is.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{parse_flags, CmdResult, Ctx};

/// `head` — the first lines of a file. Ten by default, `-n` to choose.
pub fn head(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    ends(ctx, args, false)
}

/// `tail` — the last lines of a file.
pub fn tail(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    ends(ctx, args, true)
}

fn ends(ctx: &mut Ctx<'_>, args: &str, from_end: bool) -> CmdResult {
    let name = if from_end { "tail" } else { "head" };
    let (count, rest) = line_count_flag(args);
    if rest.trim().is_empty() {
        return CmdResult::err(format!("{name}: usage: {name} [-n N] <file>..."));
    }

    let mut out = Vec::new();
    let mut failed = false;
    for target in rest.split_whitespace() {
        let path = ctx.resolve(target);
        match ctx.fs.read_file(&path) {
            Ok(data) => {
                let text = String::from_utf8_lossy(&data);
                let lines: Vec<&str> = text.lines().collect();
                let slice = if from_end {
                    // `saturating_sub` so a count larger than the file yields
                    // the whole file rather than a panicking range.
                    &lines[lines.len().saturating_sub(count)..]
                } else {
                    &lines[..lines.len().min(count)]
                };
                for l in slice {
                    out.push(l.to_string());
                }
            }
            Err(e) => {
                out.push(format!("{name}: {target}: {e}"));
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

/// Read a leading `-n N`, returning the count and the rest.
fn line_count_flag(args: &str) -> (usize, &str) {
    let trimmed = args.trim_start();
    if let Some(after) = trimmed.strip_prefix("-n") {
        let after = after.trim_start();
        let end = after.find(char::is_whitespace).unwrap_or(after.len());
        if let Ok(n) = after[..end].parse::<usize>() {
            return (n, &after[end..]);
        }
    }
    (10, args)
}

/// `wc` — count lines, words and bytes.
pub fn wc(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    if args.trim().is_empty() {
        return CmdResult::err("wc: usage: wc <file>...");
    }
    let mut out = Vec::new();
    let mut failed = false;
    for target in args.split_whitespace() {
        let path = ctx.resolve(target);
        match ctx.fs.read_file(&path) {
            Ok(data) => {
                let text = String::from_utf8_lossy(&data);
                out.push(format!(
                    "{} {} {} {target}",
                    text.lines().count(),
                    text.split_whitespace().count(),
                    data.len()
                ));
            }
            Err(e) => {
                out.push(format!("wc: {target}: {e}"));
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

/// `grep` — lines containing a literal pattern.
///
/// `-i` ignores case, `-v` inverts, `-n` prefixes line numbers. With several
/// files each line is prefixed with its filename, because a bare line from
/// `grep -n pat a b` cannot be attributed to either.
pub fn grep(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let ignore_case = flags.contains('i');
    let invert = flags.contains('v');
    let numbered = flags.contains('n');

    let (pattern, files) = match rest.find(char::is_whitespace) {
        Some(i) => (&rest[..i], rest[i..].trim()),
        None => (rest, ""),
    };
    if pattern.is_empty() {
        return CmdResult::err("grep: usage: grep [-inv] <pattern> [file...]");
    }

    let needle = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };
    // Two different questions, kept apart because they answer different things.
    //
    // `any_file` picks the branch: with no file named, grep(1) reads the pattern
    // *as* a filename. `multi` only decides whether to prefix output with the
    // filename, and one file is not "several". Collapsing them into one flag is
    // what made `grep pat log` search for a file called `pat`.
    let any_file = !files.is_empty();
    let multi = any_file && files.split_whitespace().count() > 1;
    let mut out = Vec::new();
    let mut failed = false;

    // The label prefixes a line only when it cannot be attributed otherwise:
    // several files, or `-n` making the number ambiguous. A single file without
    // `-n` prints bare lines, as grep(1) does.
    let mut scan = |data: Vec<u8>, label: &str, out: &mut Vec<String>| {
        let text = String::from_utf8_lossy(&data);
        for (i, line) in text.lines().enumerate() {
            let hay = if ignore_case {
                line.to_lowercase()
            } else {
                line.to_string()
            };
            // A match prints unless `-v` asked for the opposite.
            if hay.contains(&needle) != invert {
                out.push(match (numbered, multi) {
                    (true, true) => format!("{label}:{i}: {line}"),
                    (true, false) => format!("{i}:{line}"),
                    (false, true) => format!("{label}:{line}"),
                    (false, false) => line.to_string(),
                });
            }
        }
    };

    if any_file {
        for target in files.split_whitespace() {
            let path = ctx.resolve(target);
            match ctx.fs.read_file(&path) {
                Ok(data) => scan(data, target, &mut out),
                Err(e) => {
                    out.push(format!("grep: {target}: {e}"));
                    failed = true;
                }
            }
        }
    } else {
        // No file named: the pattern is the file, exactly as grep(1) reads it.
        let path = ctx.resolve(pattern);
        match ctx.fs.read_file(&path) {
            Ok(data) => scan(data, pattern, &mut out),
            Err(e) => return CmdResult::err(format!("grep: {e}")),
        }
    }

    if failed {
        CmdResult::fail(out)
    } else {
        CmdResult::ok(out)
    }
}

/// `sort` — sort lines. `-r` reversed, `-u` unique.
pub fn sort(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (flags, rest) = parse_flags(args);
    let mut lines = match read_lines(ctx, rest) {
        Ok(l) => l,
        Err(e) => return CmdResult::err(format!("sort: {e}")),
    };
    lines.sort();
    if flags.contains('r') {
        lines.reverse();
    }
    if flags.contains('u') {
        // After sorting, duplicates are adjacent, so one pass suffices.
        lines.dedup();
    }
    CmdResult::ok(lines)
}
/// `uniq` — drop adjacent duplicate lines.
///
/// Adjacent only, as in uniq(1): it is normally fed sorted input, and collapsing
/// non-adjacent duplicates would silently reorder the file.
pub fn uniq(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let lines = match read_lines(ctx, args.trim()) {
        Ok(l) => l,
        Err(e) => return CmdResult::err(format!("uniq: {e}")),
    };
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for line in lines {
        if out.last() != Some(&line) {
            out.push(line);
        }
    }
    CmdResult::ok(out)
}

/// `rev` — reverse each line's characters.
pub fn rev(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let lines = match read_lines(ctx, args.trim()) {
        Ok(l) => l,
        Err(e) => return CmdResult::err(format!("rev: {e}")),
    };
    CmdResult::ok(lines.into_iter().map(|l| l.chars().rev().collect()).collect())
}

/// `cut` — select fields. `-d` delimiter, `-f` field list like `1,3-5`.
pub fn cut(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let mut delim = '\t';
    let mut selector: Option<String> = None;
    let mut rest = args;

    loop {
        let t = rest.trim_start();
        if let Some(after) = t.strip_prefix("-d") {
            let after = after.trim_start();
            match after.chars().next() {
                Some(c) => {
                    delim = c;
                    rest = &after[c.len_utf8()..];
                    continue;
                }
                None => return CmdResult::err("cut: -d needs a delimiter"),
            }
        }
        if let Some(after) = t.strip_prefix("-f") {
            let after = after.trim_start();
            let end = after.find(char::is_whitespace).unwrap_or(after.len());
            selector = Some(after[..end].to_string());
            rest = &after[end..];
            continue;
        }
        rest = t;
        break;
    }

    let Some(selector) = selector else {
        return CmdResult::err("cut: usage: cut -d <delim> -f <list> <file>");
    };
    let wanted = parse_field_list(&selector);
    if wanted.is_empty() {
        return CmdResult::err("cut: -f needs at least one field number");
    }

    let lines = match read_lines(ctx, rest) {
        Ok(l) => l,
        Err(e) => return CmdResult::err(format!("cut: {e}")),
    };
    let out = lines
        .into_iter()
        .map(|line| {
            line.split(delim)
                .enumerate()
                // Fields are 1-based, as everywhere else: a 0-based selector is
                // the kind of thing only discoverable by getting it wrong.
                .filter(|(i, _)| wanted.contains(&(*i + 1)))
                .map(|(_, f)| f.to_string())
                .collect::<Vec<_>>()
                .join(&delim.to_string())
        })
        .collect();
    CmdResult::ok(out)
}

/// Parse `1,3-5` into `{1, 3, 4, 5}`.
///
/// A reversed range yields nothing rather than looping: `cut -f 5-1` is a user
/// error, not something to crash on.
fn parse_field_list(spec: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        match part.split_once('-') {
            Some((a, b)) => {
                if let (Ok(a), Ok(b)) = (a.trim().parse::<usize>(), b.trim().parse::<usize>()) {
                    for i in a..=b {
                        out.push(i);
                    }
                }
            }
            None => {
                if let Ok(n) = part.trim().parse::<usize>() {
                    out.push(n);
                }
            }
        }
    }
    out
}

/// Read a file's lines, or the previous command's output when `path` is empty.
///
/// Reading back what the last command produced is what makes a pipe work without
/// pipes: the shell feeds each command the previous one's lines.
pub(super) fn read_lines(ctx: &mut Ctx<'_>, path: &str) -> Result<Vec<String>, String> {
    if path.trim().is_empty() {
        return Ok(core::mem::take(&mut ctx.out));
    }
    let full = ctx.resolve(path);
    let data = ctx.fs.read_file(&full)?;
    let text = String::from_utf8_lossy(&data);
    Ok(text.lines().map(|s| s.to_string()).collect())
}
