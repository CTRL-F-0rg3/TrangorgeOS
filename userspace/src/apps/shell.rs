//! The shell: it interprets command lines and owns what they *mean*.
//!
//! The shell is a replaceable application. It lives at
//! `user/root/shell/` on the volume, and what runs is whatever is in that
//! directory — so a different shell is a different directory, not an edit here.
//! That is also why this type takes its filesystem as a trait rather than
//! reaching for TFS directly: a replacement shell is written against the same
//! interface.
//!
//! # Commands
//!
//! Split into three groups by where they run:
//!
//! * *shell builtins* — `help`, `echo`, `clear`, `cd`, `pwd`, `exit`,
//!   `whoami`. They need the shell's own state, so they cannot be an external
//!   program.
//! * *filesystem* — `ls`, `cat`, `mkdir`, `rm`, `touch`, `write`. They go
//!   through [`Fs`], which the shell does not implement.
//! * *system* — `uname`, `free`, `uptime`. They need the kernel, and go
//!   through [`Sys`].
//!
//! The split is the point: a command that only needs a filesystem works against
//! any [`Fs`], including one backed by a RAM disk in a test.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::apps::terminal::Terminal;
use crate::commands::builtins;
use crate::login::{Accounts, Session};

/// One entry in a directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u32,
}

/// The filesystem a shell runs against.
///
/// `&mut self` on the read methods: a real implementation caches, and a shell
/// that cannot hold a cache will re-read the same directory on every `ls`.
pub trait Fs {
    fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, String>;
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String>;
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String>;
    fn make_dir(&mut self, path: &str) -> Result<(), String>;
    fn remove(&mut self, path: &str) -> Result<(), String>;
}

/// What a shell can ask the kernel.
///
/// Separate from [`Fs`] on purpose: a filesystem command works against any
/// [`Fs`], including a RAM disk, while these need a live kernel. Merging them
/// would force every test to invent a kernel.
pub trait Sys {
    fn uname(&mut self) -> String;
    /// `(total_bytes, free_bytes)`.
    fn memory(&mut self) -> (u64, u64);
    /// Uptime in seconds.
    fn uptime(&mut self) -> u64;
    /// How many CPUs.
    fn cpus(&mut self) -> u32;
}

/// A filesystem backed by a map, for tests and headless runs.
#[derive(Debug, Default)]
pub struct MemoryFs {
    /// Absolute path to the data it holds. `/a/b` implies `/a`.
    files: BTreeMap<String, Vec<u8>>,
    dirs: BTreeSet<String>,
}

impl MemoryFs {
    pub fn new() -> Self {
        let mut fs = Self::default();
        fs.dirs.insert("/".to_string());
        fs
    }

    /// Create a directory and its parents.
    pub fn mkdir(&mut self, path: &str) {
        let p = crate::layout::normalize(path);
        let mut acc = String::new();
        self.dirs.insert("/".to_string());
        for part in p.split('/').filter(|s| !s.is_empty()) {
            acc.push('/');
            acc.push_str(part);
            self.dirs.insert(acc.clone());
        }
    }
}

impl Fs for MemoryFs {
    fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, String> {
        let p = crate::layout::normalize(path);
        if !self.dirs.contains(&p) {
            return Err(format!("no such directory: {p}"));
        }
        let prefix = if p == "/" { "/".to_string() } else { format!("{p}/") };
        let mut out = Vec::new();
        for d in &self.dirs {
            if d == &p || !d.starts_with(&prefix) {
                continue;
            }
            let rest = &d[prefix.len()..];
            if !rest.contains('/') {
                out.push(DirEntry {
                    name: rest.to_string(),
                    is_dir: true,
                    size: 0,
                });
            }
        }
        for (f, data) in &self.files {
            if let Some(rest) = f.strip_prefix(&prefix) {
                if !rest.contains('/') {
                    out.push(DirEntry {
                        name: rest.to_string(),
                        is_dir: false,
                        size: data.len() as u32,
                    });
                }
            }
        }
        out.sort_by(|a, b| (b.is_dir, &a.name).cmp(&(a.is_dir, &b.name)));
        Ok(out)
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let p = crate::layout::normalize(path);
        self.files
            .get(&p)
            .cloned()
            .ok_or_else(|| format!("no such file: {p}"))
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        let p = crate::layout::normalize(path);
        let (dir, _) = split_path(&p);
        if !self.dirs.contains(&dir) {
            return Err(format!("no such directory: {dir}"));
        }
        self.files.insert(p, data.to_vec());
        Ok(())
    }

    fn make_dir(&mut self, path: &str) -> Result<(), String> {
        let p = crate::layout::normalize(path);
        if self.dirs.contains(&p) {
            return Err(format!("already exists: {p}"));
        }
        self.mkdir(&p);
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<(), String> {
        let p = crate::layout::normalize(path);
        if self.dirs.remove(&p) {
            return Ok(());
        }
        if self.files.remove(&p).is_some() {
            return Ok(());
        }
        Err(format!("no such file or directory: {p}"))
    }
}

/// A [`Sys`] with fixed answers, for tests.
#[derive(Debug, Default)]
pub struct MockSys {
    pub boot_uname: String,
    pub total_mem: u64,
    pub free_mem: u64,
    pub uptime_s: u64,
    pub cpu_count: u32,
}

impl Sys for MockSys {
    fn uname(&mut self) -> String {
        if self.boot_uname.is_empty() {
            "TrangorgeOS".to_string()
        } else {
            self.boot_uname.clone()
        }
    }
    fn memory(&mut self) -> (u64, u64) {
        (self.total_mem, self.free_mem)
    }
    fn uptime(&mut self) -> u64 {
        self.uptime_s
    }
    fn cpus(&mut self) -> u32 {
        if self.cpu_count == 0 {
            1
        } else {
            self.cpu_count
        }
    }
}

/// Split `/a/b/c` into `("/a/b", "c")`.
fn split_path(path: &str) -> (String, String) {
    match path.rfind('/') {
        Some(0) => ("/".to_string(), path[1..].to_string()),
        Some(i) => (path[..i].to_string(), path[i + 1..].to_string()),
        None => ("/".to_string(), path.to_string()),
    }
}

/// Split a command line into its first word and the rest.
///
/// Whitespace-separated rather than space-only, so `ls  -l` does not leave a
/// leading space in `rest` that would become part of a filename.
pub fn split_command(line: &str) -> (&str, &str) {
    match line.find(char::is_whitespace) {
        Some(i) => (line[..i].trim(), line[i..].trim_start()),
        None => (line.trim(), ""),
    }
}

/// The shell: a session, a working directory and a command interpreter.
///
/// The shell owns three things and delegates everything else:
///
/// * the prompt,
/// * the line editor and the scrollback,
/// * the working directory.
///
/// A command's *meaning* lives in [`crate::commands`], which is what makes the
/// shell replaceable: a different interpreter reuses them, and a new command does
/// not need a new `match` arm here.
pub struct Shell<'a> {
    session: Session,
    fs: &'a mut dyn Fs,
    sys: &'a mut dyn Sys,
    accounts: &'a mut Accounts,
    term: Terminal,
    /// What the user last typed, for `history`.
    history: Vec<String>,
    /// The prompt style, so a different rung is visibly a different rung.
    style: PromptStyle,
}

/// How the prompt looks.
///
/// A rung is worth showing: a user who cannot tell which rung they are at cannot
/// tell what a command is about to be allowed to do. This corrects for that; it
/// is not a security control, and the capability table is that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    /// rung 1: the ordinary userspace prompt.
    Standard,
    /// A rung above 1, marked with `!` so it cannot be mistaken for rung 1.
    Elevated,
    /// The login prompt, shown before a session exists.
    Login,
}

impl PromptStyle {
    /// `user@rN:/path$ ` — the Linux form, because users arrive knowing it and
    /// re-learning a prompt is a tax paid on every session.
    pub fn render(&self, session: &Session) -> String {
        match self {
            PromptStyle::Standard => {
                format!("{}@r{}:{}$ ", session.user, session.rung, session.cwd)
            }
            PromptStyle::Elevated => {
                format!("{}@r{}!:{}$ ", session.user, session.rung, session.cwd)
            }
            PromptStyle::Login => "login: ".to_string(),
        }
    }

    /// The style a session's rung calls for.
    pub fn for_rung(rung: u32) -> Self {
        if rung <= crate::layout::RUNG_MIN {
            PromptStyle::Standard
        } else {
            PromptStyle::Elevated
        }
    }
}

impl<'a> Shell<'a> {
    /// A shell for `session`, reading and writing through `fs` and `sys`.
    ///
    /// The greeting is the first thing a user sees, and it names the
    /// privilege rung they ended up at — a login that silently drops you into
    /// a different rung than the account asked for would be invisible.
    pub fn new(
        session: Session,
        fs: &'a mut dyn Fs,
        sys: &'a mut dyn Sys,
        accounts: &'a mut Accounts,
    ) -> Self {
        let mut term = Terminal::default();
        term.say("hello in userspace");
        term.say(format!(
            "TrangorgeOS userspace — {} at rung {}",
            session.user, session.rung
        ));
        term.say(format!("home: {}", session.home));
        term.say("type 'help' for commands");
        Self {
            style: PromptStyle::for_rung(session.rung),
            session,
            fs,
            sys,
            accounts,
            term,
            history: Vec::new(),
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn terminal(&self) -> &Terminal {
        &self.term
    }

    /// Commands already run, oldest first.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Change the working directory, unchecked.
    ///
    /// Public because `cd` lives in [`crate::commands::builtins`] and needs it.
    /// Unchecked on purpose: the caller applies the checks that depend on the
    /// session, and duplicating them here would be one more place to forget one.
    pub fn set_cwd(&mut self, path: String) {
        self.session.cwd = path;
    }

    /// The filesystem this shell reads and writes.
    pub fn fs(&mut self) -> &mut dyn Fs {
        self.fs
    }

    /// The prompt, for the current rung.
    pub fn prompt(&self) -> String {
        self.style.render(&self.session)
    }

    /// Resolve a path against the working directory.
    ///
    /// `~` expands to the session's rung *home*, not the working directory — the
    /// tilde means "home" everywhere else, and a shell where it means "here" is a
    /// shell users will fight.
    pub fn resolve(&self, arg: &str) -> String {
        let arg = arg.trim();
        if arg == "~" {
            return self.session.home.clone();
        }
        if let Some(rest) = arg.strip_prefix("~/") {
            return crate::layout::normalize(&format!("{}/{}", self.session.home, rest));
        }
        if arg.starts_with('/') {
            crate::layout::normalize(arg)
        } else if arg.is_empty() {
            self.session.cwd.clone()
        } else {
            crate::layout::normalize(&format!("{}/{}", self.session.cwd, arg))
        }
    }

    /// Feed one keystroke. Returns the line when the user pressed Enter.
    pub fn feed_char(&mut self, c: char) -> Option<String> {
        self.term.feed_char(c)
    }

    /// Run one command line, writing its output to the terminal.
    ///
    /// Returns `true` when the shell should close — `exit`, or a logout that
    /// ended the session.
    pub fn run(&mut self, line: &str) -> bool {
        let line = line.trim();
        if line.is_empty() {
            return false;
        }
        self.history.push(line.to_string());
        self.term.print_prompt(format!("{}{}", self.prompt(), line));

        let (cmd, rest) = split_command(line);

        // The shell's own commands come first: `cd` and `history` need the
        // shell, and a table entry for them would have to reach back into it —
        // the thing `Ctx` exists to prevent.
        let builtin = match cmd {
            "cd" => Some(builtins::cd(self, rest)),
            "pwd" => Some(builtins::pwd(self)),
            "history" => Some(builtins::history(self)),
            "help" | "?" => Some(builtins::help()),
            "exit" => Some(builtins::exit(self)),
            "clear" => {
                self.term.clear();
                None
            }
            _ => None,
        };
        if let Some(result) = builtin {
            return self.apply(result);
        }

        // Everything else goes through the table, with a `Ctx` that cannot reach
        // the prompt, the terminal or the working directory.
        let result = {
            let mut ctx =
                crate::commands::Ctx::new(self.fs, self.sys, &mut self.session, self.accounts);
            match crate::commands::dispatch(&mut ctx, cmd, rest) {
                Some(r) => r,
                // A bare path is a guess, not an error: users type `notes.txt`
                // and mean to read it.
                None if cmd.contains('/') => crate::commands::files::cat(&mut ctx, cmd),
                None => crate::commands::CmdResult::err(format!(
                    "{cmd}: command not found (try 'help')"
                )),
            }
        };
        self.apply(result)
    }

    /// Show a command's output and act on its exit status.
    fn apply(&mut self, result: crate::commands::CmdResult) -> bool {
        for line in result.out {
            self.term.say(line);
        }
        if result.exit {
            self.term.close();
            return true;
        }
        false
    }
}
