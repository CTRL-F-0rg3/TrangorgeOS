//! The commands the userspace shell can run.
//!
//! # The Linux set, as the default
//!
//! Userspace ships the commands a Linux user expects, because a shell that has
//! `ls` but not `grep` is not a usable shell — it is a toy with a filesystem.
//! Each one is a separate function rather than a `match` arm, so `help` can list
//! them, a test can call one directly, and a replacement shell can reuse them.
//!
//! # What each command needs
//!
//! Commands take a [`Ctx`] rather than the shell itself. A command that could
//! reach the shell could change the prompt or close the terminal, which is a
//! surprising amount of power for something as innocent as `true`. [`Ctx`]
//! exposes the filesystem, the system, the session and an output sink — and
//! nothing else.
//!
//! # Exit status
//!
//! [`CmdResult`] carries the same convention as a POSIX shell: zero for success,
//! non-zero for failure. `test`, `grep -q` and `[` depend on it to be usable in
//! a conditional, so it is not decorative.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::apps::shell::{Fs, Sys};
use crate::login::{Account, Accounts, LoginError, Session};

pub mod accounts;
pub mod files;
pub mod system;
pub mod text;

/// What a command returns.
///
/// `false` is the POSIX failure status; [`CmdResult::ok`] and
/// [`CmdResult::fail`] are the only constructors, so a command cannot invent a
/// third meaning by accident.
pub struct CmdResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Output to show the user, already formatted.
    pub out: Vec<String>,
    /// Set when the command wants the shell to close.
    pub exit: bool,
}

impl CmdResult {
    /// Succeeded, with output.
    pub fn ok(out: Vec<String>) -> Self {
        Self {
            success: true,
            out,
            exit: false,
        }
    }

    /// Succeeded, quietly.
    pub fn silent() -> Self {
        Self::ok(Vec::new())
    }

    /// Failed, with output explaining why.
    pub fn fail(out: Vec<String>) -> Self {
        Self {
            success: false,
            out,
            exit: false,
        }
    }

    /// Failed, with one line of explanation.
    pub fn err(msg: impl Into<String>) -> Self {
        Self::fail(vec![msg.into()])
    }

    /// Succeeded, and the shell should now close.
    pub fn exit_ok() -> Self {
        Self {
            success: true,
            out: Vec::new(),
            exit: true,
        }
    }
}

/// Everything a command may touch.
///
/// Deliberately not the shell: a command cannot rewrite the prompt, cannot
/// change directory behind the user's back, and cannot close the terminal. Those
/// are the shell's decisions.
pub struct Ctx<'a> {
    pub fs: &'a mut dyn Fs,
    pub sys: &'a mut dyn Sys,
    pub session: &'a mut Session,
    /// The system account store, for the account commands.
    pub accounts: &'a mut Accounts,
    /// Lines the command produced.
    pub out: Vec<String>,
}

impl<'a> Ctx<'a> {
    pub fn new(
        fs: &'a mut dyn Fs,
        sys: &'a mut dyn Sys,
        session: &'a mut Session,
        accounts: &'a mut Accounts,
    ) -> Self {
        Self {
            fs,
            sys,
            session,
            accounts,
            out: Vec::new(),
        }
    }

    /// Add or replace an account.
    pub fn accounts_insert(&mut self, account: Account) -> Result<(), String> {
        self.accounts.insert(account)
    }

    /// Remove an account by name.
    pub fn accounts_remove(&mut self, name: &str) -> Result<(), String> {
        // The account is returned by `Accounts::remove`; the caller here only
        // needs to know whether it existed, so the value is dropped here rather
        // than propagating a return type the commands do not use.
        self.accounts
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("no such account: {name}"))
    }

    /// Set an account's password.
    pub fn accounts_set_password(&mut self, name: &str, password: &str) -> Result<(), String> {
        self.accounts.set_password(name, password)
    }

    /// Authenticate, for the `login` command.
    pub fn accounts_login(&mut self, name: &str, password: &str) -> Result<Session, LoginError> {
        self.accounts.login(name, password)
    }

    /// Every account, for the `users` command.
    pub fn accounts_list(&self) -> Vec<Account> {
        self.accounts.all()
    }

    /// Print one line of output.
    pub fn say(&mut self, line: impl Into<String>) {
        self.out.push(line.into());
    }

    /// Finish with these lines.
    pub fn done(self) -> CmdResult {
        CmdResult::ok(self.out)
    }

    /// Finish, failing, with one more line of explanation.
    pub fn fail(mut self, msg: impl Into<String>) -> CmdResult {
        self.out.push(msg.into());
        CmdResult::fail(self.out)
    }

    /// Resolve a path against the session's working directory.
    ///
    /// `~` expands to the rung *home*, not the working directory — the tilde
    /// means "home" everywhere else, and a shell where it means "here" is a
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

    /// Whether the session may touch `path`.
    ///
    /// A usability guard, not a security boundary — that is the capability
    /// table's job. Its purpose is to make `cd ..` at a rung home a refusal the
    /// user can see, rather than a silent no-op.
    pub fn allowed(&self, path: &str) -> bool {
        self.session.owns(path)
    }

    /// Whether the session is root.
    ///
    /// The gate the account commands use. A usability guard: the real boundary
    /// is the capability table, and a caller that bypasses this still cannot
    /// reach anything the table does not grant.
    pub fn is_root(&self) -> bool {
        self.session.user == crate::layout::ROOT_USER
    }
}

/// Pull leading flags out of an argument string.
///
/// Stops at the first non-flag word, so `ls -l myfile` keeps `myfile` as a
/// target and `ls myfile -l` does not treat `-l` as a flag — which is what a
/// user who typed it last means.
pub fn parse_flags(args: &str) -> (String, &str) {
    let mut flags = String::new();
    let mut rest = args.trim_start();
    loop {
        let Some(after) = rest.strip_prefix('-') else {
            break;
        };
        if after.is_empty() || after.starts_with('-') {
            // `--` ends the flags, and a bare `-` is a filename.
            break;
        }
        let end = after.find(char::is_whitespace).unwrap_or(after.len());
        flags.push_str(&after[..end]);
        rest = after[end..].trim_start();
    }
    (flags, rest)
}

/// The shell's own commands, which need the shell state rather than a `Ctx`.
///
/// Kept separate from [`dispatch`] because `cd`, `exit` and `history` change
/// the session or the terminal, which is exactly what `Ctx` deliberately does
/// not expose.
pub mod builtins {
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    use super::CmdResult;
    use crate::apps::shell::Shell;

    /// `cd` — change directory.
    pub fn cd(shell: &mut Shell<'_>, args: &str) -> CmdResult {
        let target = shell.resolve(args);
        if shell.fs().read_dir(&target).is_err() {
            return CmdResult::err(format!("cd: {target}: no such directory"));
        }
        // A session stays inside its own rung. Refusing `..` at the home, rather
        // than silently doing nothing, is what tells the user why.
        if !shell.session().owns(&target) {
            return CmdResult::err(format!(
                "cd: {target}: outside this session's home (rung {})",
                shell.session().rung
            ));
        }
        shell.set_cwd(target);
        CmdResult::silent()
    }

    /// `pwd` — the working directory.
    pub fn pwd(shell: &mut Shell<'_>) -> CmdResult {
        CmdResult::ok(vec![shell.session().cwd.clone()])
    }

    /// `exit` — close the shell.
    pub fn exit(_shell: &mut Shell<'_>) -> CmdResult {
        CmdResult::exit_ok()
    }

    /// `history` — commands already run.
    pub fn history(shell: &mut Shell<'_>) -> CmdResult {
        let mut out = Vec::new();
        for (i, h) in shell.history().iter().enumerate() {
            out.push(format!("{:>4}  {h}", i + 1));
        }
        CmdResult::ok(out)
    }

    /// `help` — every command, with its one-line description.
    pub fn help() -> CmdResult {
        let mut out = Vec::new();
        for c in super::registry() {
            out.push(c.about.to_string());
        }
        out.push(String::new());
        out.push("Shell builtins:".to_string());
        out.push("  cd <dir>              change directory".to_string());
        out.push("  pwd                   print the working directory".to_string());
        out.push("  history               commands already run".to_string());
        out.push("  exit                  close this shell".to_string());
        CmdResult::ok(out)
    }
}

/// One entry in the command table: the handler and its one-line description.
pub struct Command {
    pub name: &'static str,
    pub about: &'static str,
    pub run: fn(&mut Ctx<'_>, &str) -> CmdResult,
}

/// Every command, in the order `help` lists them.
///
/// Grouped the way a user thinks about them — files, then text, then the machine
/// — rather than alphabetically, because `help` is read by a person deciding
/// what to type next, not scanned for a name.
pub fn registry() -> Vec<Command> {
    use crate::commands::{accounts, files, system, text};

    vec![
        // Files.
        Command { name: "ls", about: "ls [-lahR] [path...]        list a directory", run: files::ls },
        Command { name: "cat", about: "cat <file>...             print files", run: files::cat },
        Command { name: "cp", about: "cp <src> <dst>            copy a file", run: files::cp },
        Command { name: "mv", about: "mv <src> <dst>            move or rename", run: files::mv },
        Command { name: "rm", about: "rm [-rf] <path>...        remove files or directories", run: files::rm },
        Command { name: "mkdir", about: "mkdir [-p] <dir>...       create directories", run: files::mkdir },
        Command { name: "touch", about: "touch <file>...          create empty files", run: files::touch },
        // Text.
        Command { name: "head", about: "head [-n N] <file>...     the first lines", run: text::head },
        Command { name: "tail", about: "tail [-n N] <file>...     the last lines", run: text::tail },
        Command { name: "grep", about: "grep [-inv] <pat> [file]  lines containing a literal", run: text::grep },
        Command { name: "sort", about: "sort [-ru] <file>        sort lines", run: text::sort },
        Command { name: "uniq", about: "uniq <file>              drop adjacent duplicate lines", run: text::uniq },
        Command { name: "cut", about: "cut -d D -f LIST <file>   select fields", run: text::cut },
        Command { name: "rev", about: "rev <file>                reverse each line", run: text::rev },
        Command { name: "wc", about: "wc <file>...              count lines, words, bytes", run: text::wc },
        // Navigation and the machine.
        Command { name: "echo", about: "echo [-n] <text>         print text", run: system::echo },
        Command { name: "find", about: "find [path] [-name P] [-type d|f]  search", run: system::find },
        Command { name: "seq", about: "seq N | seq FIRST LAST   print a number range", run: system::seq },
        Command { name: "basename", about: "basename <path>          the last path component", run: system::basename },
        Command { name: "dirname", about: "dirname <path>           everything but the last component", run: system::dirname },
        Command { name: "uname", about: "uname [-a]               system name", run: system::uname },
        Command { name: "free", about: "free                     memory usage", run: system::free },
        Command { name: "uptime", about: "uptime                   time since boot", run: system::uptime },
        Command { name: "df", about: "df                       filesystem usage", run: system::df },
        // Accounts.
        Command { name: "whoami", about: "whoami                  current user and rung", run: accounts::whoami },
        Command { name: "id", about: "id                       user, rung and home", run: accounts::id },
        Command { name: "who", about: "who                      who is logged in", run: accounts::who },
        Command { name: "users", about: "users                    list every account", run: accounts::users },
        Command { name: "useradd", about: "useradd [-r N] <name>    create an account (root)", run: accounts::useradd },
        Command { name: "userdel", about: "userdel [-r] <name>      remove an account (root)", run: accounts::userdel },
        Command { name: "passwd", about: "passwd [user] <password>  change a password", run: accounts::passwd },
        Command { name: "login", about: "login <name> <password>  switch account", run: accounts::login },
        Command { name: "logout", about: "logout                  end the session", run: accounts::logout },
        // Tests and constants.
        Command { name: "test", about: "test [-efdnz] A B        the POSIX conditional", run: system::test },
        Command { name: "[", about: "[ ... ]                  the same test, shell spelling", run: system::test },
        Command { name: "true", about: "true                     succeed", run: system::true_cmd },
        Command { name: "false", about: "false                    fail", run: system::false_cmd },
    ]
}

/// Run one command by name.
///
/// Returns `None` for an unknown name, so the caller decides how loud to be: an
/// unknown command is a message, not a failure, because the user typed it.
pub fn dispatch(ctx: &mut Ctx<'_>, name: &str, args: &str) -> Option<CmdResult> {
    registry()
        .into_iter()
        // `[` is a distinct name for the same implementation as `test`, which
        // is why the lookup is by name rather than by handler.
        .find(|c| c.name == name)
        .map(|c| (c.run)(ctx, args))
}
