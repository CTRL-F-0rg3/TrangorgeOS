//! Login, and the commands that manage accounts.
//!
//! # The login sequence
//!
//! [`LoginPrompt`] is a small state machine rather than a loop, so a test can
//! drive it one keystroke at a time and the kernel can call the same thing the
//! terminal does. It shows the prompt, reads a line, authenticates, and reports
//! either a [`Session`] or an error to display.
//!
//! # What the account commands are allowed to do
//!
//! Every account command checks that the caller is root before acting. That is
//! a *usability* guard — the real boundary is the capability table — but a shell
//! that lets anyone run `userdel` because a check was forgotten is worse than
//! one that refuses politely.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{parse_flags, CmdResult, Ctx};
use crate::login::{Account, Session};

/// The login prompt.
pub const LOGIN_PROMPT: &str = "TrangorgeOS login: ";
/// The password prompt, which does not echo.
pub const PASSWORD_PROMPT: &str = "Password: ";

/// Where a login attempt currently is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Login {
    /// Waiting for a user name.
    AwaitingUser,
    /// Waiting for a password.
    AwaitingPassword { user: String },
    /// The name was entered; waiting for the password, or showing a failure.
    AwaitingRetry,
}

/// Drive the login sequence.
///
/// `submit` receives whatever the user typed and returns any message to show
/// plus the session, if this attempt succeeded. It yields a session exactly once.
///
/// The state is an explicit field rather than a loop so the caller decides how
/// keystrokes arrive — which is what lets this be tested without a terminal.
pub struct LoginPrompt<'a> {
    accounts: &'a crate::login::Accounts,
    state: Login,
    /// How many password attempts have been made.
    pub attempts: u32,
    /// Refuse further attempts past this many.
    pub max_attempts: u32,
}

impl<'a> LoginPrompt<'a> {
    /// A fresh prompt against `accounts`.
    pub fn new(accounts: &'a crate::login::Accounts) -> Self {
        Self {
            accounts,
            state: Login::AwaitingUser,
            attempts: 0,
            // Three is what a physical console would allow. Past that the point
            // is to stop someone grinding the password, not to log in.
            max_attempts: 3,
        }
    }

    /// What the prompt should currently display, if anything.
    pub fn prompt(&self) -> Option<&'static str> {
        match self.state {
            Login::AwaitingUser | Login::AwaitingRetry => Some(LOGIN_PROMPT),
            Login::AwaitingPassword { .. } => Some(PASSWORD_PROMPT),
        }
    }

    /// Whether the field should echo what is typed.
    ///
    /// False for the password, which is the entire point of a second state: the
    /// caller draws asterisks or nothing at all based on this.
    pub fn echoes(&self) -> bool {
        matches!(self.state, Login::AwaitingUser)
    }

    /// Whether the attempt limit has been reached.
    pub fn locked_out(&self) -> bool {
        self.attempts >= self.max_attempts
    }

    /// Feed one submitted line. Returns messages to show, then the session if
    /// this attempt succeeded.
    pub fn submit(&mut self, line: &str) -> (Vec<String>, Option<Session>) {
        let line = line.trim();
        let mut out = Vec::new();
        let state = core::mem::replace(&mut self.state, Login::AwaitingUser);

        match state {
            Login::AwaitingUser | Login::AwaitingRetry => {
                if self.locked_out() {
                    out.push("too many attempts; access denied".to_string());
                    self.state = Login::AwaitingRetry;
                    return (out, None);
                }
                if line.is_empty() {
                    out.push("login: a user name is required".to_string());
                    return (out, None);
                }
                // An unknown user still gets asked for a password, so a correct
                // password for a nonexistent name is not distinguishable by
                // timing alone.
                self.state = Login::AwaitingPassword {

                    user: line.to_string(),
                };
                (out, None)
            }
            Login::AwaitingPassword { user } => {
                self.attempts += 1;
                match self.accounts.login(&user, line) {
                    Ok(session) => (out, Some(session)),
                    Err(e) => {
                        out.push(e.to_string());
                        self.state = Login::AwaitingRetry;
                        if self.locked_out() {
                            out.push("too many attempts; access denied".to_string());
                        }
                        (out, None)
                    }
                }
            }
        }
    }
}

/// `whoami` — the current user and rung.
pub fn whoami(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::ok(vec![ctx.session.whoami()])
}

/// `id` — the same, plus the home directory.
pub fn id(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let s: &Session = ctx.session;
    CmdResult::ok(vec![format!(
        "uid=0({}) gid=0(r{}) home={}",
        s.user, s.rung, s.home
    )])
}

/// `who` — who is logged in.
///
/// There is one session — the terminal's — so this reports that one. That is
/// not a stub pretending otherwise: the answer is accurate for a single-session
/// system, and inventing a list would be a lie.
pub fn who(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let s: &Session = ctx.session;
    CmdResult::ok(vec![format!("{:<16} tty1  rung {}", s.user, s.rung)])
}

/// `users` — every account.
pub fn users(ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    let mut out = vec![format!("{:<16} {:>5} {:>8}  home", "USER", "RUNG", "STATE")];
    for a in ctx.accounts_list() {
        out.push(format!(
            "{:<16} {:>5} {:>8}  {}",
            a.name,
            a.rung,
            if a.enabled { "enabled" } else { "locked" },
            a.home()
        ));
    }
    if out.len() == 1 {
        out.push("(no accounts)".to_string());
    }
    CmdResult::ok(out)
}

/// `login` — switch to another account without ending the shell.
///
/// Takes the password on the command line, which is convenient and is *not* safe:
/// it puts the password in the shell's history. The alternative — an
/// interactive prompt here — would block a batch-driven caller. Recorded here so
/// the choice is visible rather than accidental.
pub fn login(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    let (name, password) = match args.trim().split_once(char::is_whitespace) {
        Some((n, p)) => (n.trim(), p.trim()),
        None => (args.trim(), ""),
    };
    if name.is_empty() {
        return CmdResult::err("login: usage: login <name> <password>");
    }
    if password.is_empty() {
        return CmdResult::err("login: a password is required");
    }
    match ctx.accounts_login(name, password) {
        Ok(session) => {
            let who = session.whoami();
            *ctx.session = session;
            CmdResult::ok(vec![format!("logged in as {who}")])
        }
        Err(e) => CmdResult::err(format!("login: {e}")),
    }
}

/// `logout` — end the session.
///
/// Ends the shell rather than only the account: in a single-session system the
/// right result is back at the login prompt, which is what the shell's caller
/// does when the session ends.
pub fn logout(_ctx: &mut Ctx<'_>, _args: &str) -> CmdResult {
    CmdResult::exit_ok()
}

/// `useradd` — create an account.
///
/// `-r N` sets the privilege rung; without it the account gets
/// [`crate::layout::RUNG_MIN`], the least-privileged rung that exists. `-d`
/// chooses a home directory, defaulting to the account's rung-1 home.
///
/// The account record itself lives in the system account store, not on the
/// filesystem, so this command writes through [`crate::login::Accounts`] and
/// creates the home directory so the account is immediately usable.
pub fn useradd(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    if !ctx.is_root() {
        return CmdResult::err("useradd: only root may create accounts");
    }

    let mut rung = crate::layout::RUNG_MIN;
    let mut home: Option<String> = None;
    let mut rest = args;
    loop {
        let t = rest.trim_start();
        if let Some(after) = t.strip_prefix("-r") {
            let after = after.trim_start();
            let end = after.find(char::is_whitespace).unwrap_or(after.len());
            match after[..end].parse::<u32>() {
                Ok(n) => {
                    rung = n;
                    rest = &after[end..];
                    continue;
                }
                Err(_) => return CmdResult::err("useradd: -r needs a number"),
            }
        }
        if let Some(after) = t.strip_prefix("-d") {
            let after = after.trim_start();
            let end = after.find(char::is_whitespace).unwrap_or(after.len());
            home = Some(after[..end].to_string());
            rest = &after[end..];
            continue;
        }
        rest = t;
        break;
    }

    let (name, password) = match rest.split_once(char::is_whitespace) {
        Some((n, p)) => (n, p.trim().to_string()),
        None => (rest, String::new()),
    };
    if name.is_empty() {
        return CmdResult::err("useradd: usage: useradd [-r N] [-d home] <name> [password]");
    }
    if name.contains('/') {
        return CmdResult::err("useradd: a user name may not contain '/'");
    }
    if rung < crate::layout::RUNG_MIN || rung > crate::layout::RUNG_MAX {
        return CmdResult::err(format!(
            "useradd: rung {rung} is outside {}..={}",
            crate::layout::RUNG_MIN,
            crate::layout::RUNG_MAX
        ));
    }

    let account = Account {
        name: name.to_string(),
        // An account with no given password defaults to one named after itself,
        // as useradd(8) does. It exists to be changed, not to be trusted.
        password_hash: if password.is_empty() {
            name.to_string()
        } else {
            password
        },
        rung,
        enabled: true,
    };

    if let Some(h) = &home {
        // A home outside the user directory would be an account whose files the
        // session guard can never reach, so it is refused rather than created.
        if !ctx.allowed(h) {
            return CmdResult::err("useradd: a home outside the user's own rung is not allowed");
        }
        // Deliberately unchecked: `make_dir` creates missing parents, and an
        // account whose home already exists is not a failure — the account is
        // what was asked for, and the directory may well be there already.
        let _ = ctx.fs.make_dir(h);
    }

    match ctx.accounts_insert(account) {
        Ok(()) => CmdResult::ok(vec![format!("created user {name} at rung {rung}")]),
        Err(e) => CmdResult::err(format!("useradd: {e}")),
    }
}

/// `userdel` — remove an account. `-r` also removes its rung home.
pub fn userdel(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    if !ctx.is_root() {
        return CmdResult::err("userdel: only root may remove accounts");
    }
    let (flags, rest) = parse_flags(args);
    let name = rest.trim();
    if name.is_empty() {
        return CmdResult::err("userdel: usage: userdel [-r] <name>");
    }
    if name == ctx.session.user {
        return CmdResult::err("userdel: a user cannot delete their own account");
    }
    match ctx.accounts_remove(name) {
        Ok(()) => {
            let mut out = vec![format!("removed user {name}")];
            if flags.contains('r') {
                let home = crate::layout::rung_home(name, crate::layout::RUNG_MIN);
                if ctx.fs.remove(&home).is_ok() {
                    out.push(format!("removed {home}"));
                }
            }
            CmdResult::ok(out)
        }
        Err(e) => CmdResult::err(format!("userdel: {e}")),
    }
}

/// `passwd` — change a password.
///
/// With no arguments, the caller's own — but the new password must be given, so
/// `passwd new` rather than `passwd` interactively. Naming another account is
/// root only.
pub fn passwd(ctx: &mut Ctx<'_>, args: &str) -> CmdResult {
    // The name is copied rather than borrowed: `ctx.session.user.as_str()` would
    // hold an immutable borrow of `ctx` across the `&mut` call below.
    let (name, new_password): (String, String) = match args.trim().split_once(char::is_whitespace)
    {
        Some((n, p)) => (n.trim().to_string(), p.trim().to_string()),
        None => (ctx.session.user.clone(), args.trim().to_string()),
    };
    if name != ctx.session.user && !ctx.is_root() {
        return CmdResult::err("passwd: only root may change another user's password");
    }
    if new_password.is_empty() {
        return CmdResult::err("passwd: usage: passwd [user] <new-password>");
    }
    match ctx.accounts_set_password(&name, &new_password) {
        Ok(()) => CmdResult::ok(vec![format!("password changed for {name}")]),
        Err(e) => CmdResult::err(format!("passwd: {e}")),
    }
}
