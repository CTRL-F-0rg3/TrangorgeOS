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

use std::collections::BTreeMap;

use crate::apps::terminal::Terminal;
use crate::login::Session;

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
    dirs: std::collections::BTreeSet<String>,
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
pub struct Shell<'a> {
    session: Session,
    fs: &'a mut dyn Fs,
    sys: &'a mut dyn Sys,
    term: Terminal,
    /// What the user last typed, for the `history` command.
    history: Vec<String>,
}

impl<'a> Shell<'a> {
    /// A shell for `session`, reading and writing through `fs` and `sys`.
    ///
    /// The greeting is the first thing a user sees, and it names the
    /// privilege rung they ended up at — a login that silently drops you into
    /// a different rung than the account asked for would be invisible.
    pub fn new(session: Session, fs: &'a mut dyn Fs, sys: &'a mut dyn Sys) -> Self {
        let mut term = Terminal::default();
        term.say("hello in userspace");
        term.say(format!(
            "TrangorgeOS userspace — {} at rung {}",
            session.user, session.rung
        ));
        term.say(format!("home: {}", session.cwd));
        term.say("type 'help' for commands");
        Self {
            session,
            fs,
            sys,
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

    /// The prompt, e.g. `root@r1 /kernel/…/r1$ `.
    pub fn prompt(&self) -> String {
        format!("{} {}$ ", self.session.whoami(), self.session.cwd)
    }

    /// Feed one keystroke. Returns the line when the user pressed Enter.
    pub fn feed_char(&mut self, c: char) -> Option<String> {
        self.term.feed_char(c)
    }

    /// Run one command line, writing its output to the terminal.
    pub fn run(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        self.history.push(line.to_string());
        self.term.print_prompt(format!("{}{}", self.prompt(), line));

        let (cmd, rest) = split_command(line);
        match cmd {
            "help" => self.cmd_help(),
            "echo" => self.term.say(rest),
            "clear" => self.term.clear(),
            "exit" | "logout" => self.term.close(),
            "whoami" => self.term.say(self.session.whoami()),
            "pwd" => self.term.say(&self.session.cwd),
            "cd" => self.cmd_cd(rest),
            "ls" => self.cmd_ls(rest),
            "cat" => self.cmd_cat(rest),
            "mkdir" => self.cmd_mkdir(rest),
            "write" => self.cmd_write(rest),
            "rm" => self.cmd_rm(rest),
            "uname" => self.term.say(self.sys.uname()),
            "free" => self.cmd_free(),
            "uptime" => self.cmd_uptime(),
            "history" => {
                for (i, h) in self.history.iter().enumerate() {
                    self.term.say(format!("{:>4}  {h}", i + 1));
                }
            }
            // A bare path is a guess, not an error: users type `notes.txt` and
            // mean to read it. Saying so beats "command not found".
            other if other.contains('/') || self.fs.read_file(other).is_ok() => {
                self.cmd_cat(other)
            }
            other => self.term.say(format!("{other}: command not found (try 'help')")),
        }
    }

    fn cmd_help(&mut self) {
        let rows: &[(&str, &str)] = &[
            ("help", "list these commands"),
            ("echo <text>", "print text"),
            ("clear", "clear the screen"),
            ("exit", "close this shell"),
            ("whoami", "print user and rung"),
            ("pwd", "print the working directory"),
            ("cd <dir>", "change directory"),
            ("ls [dir]", "list a directory"),
            ("cat <file>", "print a file"),
            ("mkdir <dir>", "create a directory"),
            ("write <file> <text>", "write a text file"),
            ("rm <path>", "remove a file or directory"),
            ("uname", "system name"),
            ("free", "memory usage"),
            ("uptime", "seconds since boot"),
            ("history", "commands already run"),
        ];
        for (c, d) in rows {
            self.term.say(format!("  {c:<22} {d}"));
        }
    }

    /// Resolve a possibly-relative path against the working directory.
    ///
    /// `~` and `~/…` expand to the session's rung *home*, not to the current
    /// directory — that is the whole point of the tilde, and expanding it to
    /// `cwd` would make `cd sub; cd ~` a no-op.
    fn resolve(&self, arg: &str) -> String {
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

    fn cmd_cd(&mut self, arg: &str) {
        let target = self.resolve(arg);
        if self.fs.read_dir(&target).is_err() {
            self.term.say(format!("cd: {target}: no such directory"));
            return;
        }
        // A session stays inside its own rung. Refusing `..` at the rung home,
        // rather than silently doing nothing, is what tells the user why.
        if !self.session.owns(&target) {
            self.term.say(format!(
                "cd: {target}: outside this session's home (rung {})",
                self.session.rung
            ));
            return;
        }
        self.session.cwd = target;
    }

    fn cmd_ls(&mut self, arg: &str) {
        let path = self.resolve(arg);
        match self.fs.read_dir(&path) {
            Ok(entries) if entries.is_empty() => self.term.say("(empty)"),
            Ok(entries) => {
                for e in entries {
                    if e.is_dir {
                        self.term.say(format!("{}/", e.name));
                    } else {
                        self.term.say(format!("{}  {} bytes", e.name, e.size));
                    }
                }
            }
            Err(e) => self.term.say(format!("ls: {e}")),
        }
    }

    fn cmd_cat(&mut self, arg: &str) {
        if arg.is_empty() {
            self.term.say("cat: usage: cat <file>");
            return;
        }
        let path = self.resolve(arg);
        match self.fs.read_file(&path) {
            Ok(data) => {
                // Strip exactly one trailing newline, so a file that ends in
                // one does not print a blank line on every `cat`.
                let text = String::from_utf8_lossy(&data);
                for line in text.trim_end_matches('\n').lines() {
                    self.term.say(line);
                }
            }
            Err(e) => self.term.say(format!("cat: {e}")),
        }
    }

    fn cmd_mkdir(&mut self, arg: &str) {
        if arg.is_empty() {
            self.term.say("mkdir: usage: mkdir <dir>");
            return;
        }
        let path = self.resolve(arg);
        if !self.session.owns(&path) {
            self.term.say("mkdir: outside this session's home");
            return;
        }
        match self.fs.make_dir(&path) {
            Ok(()) => self.term.say(format!("created {path}")),
            Err(e) => self.term.say(format!("mkdir: {e}")),
        }
    }

    fn cmd_write(&mut self, arg: &str) {
        let (name, text) = split_command(arg);
        if name.is_empty() {
            self.term.say("write: usage: write <file> <text>");
            return;
        }
        let path = self.resolve(name);
        if !self.session.owns(&path) {
            self.term.say("write: outside this session's home");
            return;
        }
        match self.fs.write_file(&path, text.as_bytes()) {
            Ok(()) => self.term.say(format!("wrote {} bytes to {name}", text.len())),
            Err(e) => self.term.say(format!("write: {e}")),
        }
    }

    fn cmd_rm(&mut self, arg: &str) {
        if arg.is_empty() {
            self.term.say("rm: usage: rm <path>");
            return;
        }
        let path = self.resolve(arg);
        if path == self.session.cwd {
            self.term.say("rm: refusing to remove the working directory");
            return;
        }
        if !self.session.owns(&path) {
            self.term.say("rm: outside this session's home");
            return;
        }
        match self.fs.remove(&path) {
            Ok(()) => self.term.say(format!("removed {path}")),
            Err(e) => self.term.say(format!("rm: {e}")),
        }
    }

    fn cmd_free(&mut self) {
        let (total, free) = self.sys.memory();
        let mib = 1024 * 1024;
        self.term.say(format!(
            "memory: {} MiB total, {} MiB free",
            total / mib,
            free / mib
        ));
    }

    fn cmd_uptime(&mut self) {
        let s = self.sys.uptime();
        self.term.say(format!(
            "up {}h {}m {}s, {} cpu(s)",
            s / 3600,
            (s % 3600) / 60,
            s % 60,
            self.sys.cpus()
        ));
    }
}
