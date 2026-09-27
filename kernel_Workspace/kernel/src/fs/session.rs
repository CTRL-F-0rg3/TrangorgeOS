//! Running userspace: the login prompt, the shell, and the kernel adapters
//! they need.
//!
//! # What this module is
//!
//! Userspace owns the login sequence, the accounts and the shell — all of that
//! lives in `tgs-userspace` and is exercised by that crate's own tests. This
//! module is the *other* half of the handoff: it supplies the things userspace
//! cannot invent for itself, and then gets out of the way.
//!
//! | Userspace asks for | This module gives it |
//! |---|---|
//! | [`Sys`] — uname, memory, uptime, cpus | [`KernelSys`], reading the real kernel |
//! | [`Fs`] — read/write/list | [`DiskFs`], over the real TFS volume |
//! | keystrokes | [`read_line`], off the keyboard driver's queue |
//!
//! # Why the kernel terminal does not run alongside this
//!
//! The keyboard driver pushes every scancode into one shared queue, and both
//! this module and `terminal::run` would read from it. Two consumers on a
//! single queue do not each get half the keys in any useful sense — they get
//! keys at random, so a login would silently lose characters. Therefore [`run`]
//! is entered *instead of* `terminal::init()`/`terminal::run()`, and the kernel
//! terminal's prompt never appears while userspace is up. That is the whole of
//! "hide the kernel terminal", and it follows from there being one keyboard
//! rather than being a special case.
//!
//! # Loop shape
//!
//! ```text
//!   ┌─► login prompt ─(authenticated)─► shell ──(exit/logout)──┐
//!   └──────────────────── (back to login) ─────────────────────┘
//! ```
//!
//! A wrong password returns to the prompt rather than ending the process, and
//! the attempt limit inside `LoginPrompt` is what stops a person from grinding
//! the password. The loop is infinite on purpose: a console whose login gives
//! up after three tries is a machine you cannot log into.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use tgs_userspace::apps::shell::{DirEntry, Fs, Shell, Sys};
use tgs_userspace::commands::accounts::LoginPrompt;
use tgs_userspace::login::{Accounts, Session};

use crate::fs::driver::block::BlockDevice;
use crate::fs::tangfs::tfs;
use crate::vga_buffer::{Color, WRITER};

/// Longest line the console will accept, in bytes.
///
/// Matches what the kernel terminal allows, so a habit learned in one works in
/// the other. Long enough for every path this system can express, short enough
/// that a mistyped paste cannot walk off the buffer.
const MAX_LINE: usize = 256;

/// The real TFS volume, behind the [`Fs`] trait the shell speaks.
pub struct DiskFs {
    dev: &'static dyn BlockDevice,
}

impl DiskFs {
    pub fn new(dev: &'static dyn BlockDevice) -> Self {
        Self { dev }
    }

    /// Walk an absolute directory path to its inode.
    fn resolve_dir(&self, path: &str) -> Result<u32, String> {
        let mut cur = tfs::ROOT_DIR;
        for part in path.split('/').filter(|s| !s.is_empty()) {
            cur = tfs::find_dir(self.dev, cur, part)
                .map_err(|_| format!("{part}: no such directory"))?;
        }
        Ok(cur)
    }

    /// Split an absolute path into the directory holding its last component
    /// and that component's name.
    ///
    /// TFS addresses a file as *(directory, name)*, never by full path, so every
    /// operation here has to split a path the same way. Doing it in one place
    /// is what stops `cat /a/b/c` and `rm /a/b/c` from disagreeing about where
    /// `c` lives.
    fn resolve_parent<'p>(&self, path: &'p str) -> Result<(u32, &'p str), String> {
        let path = path.trim_end_matches('/');
        if path.is_empty() {
            return Err("no such directory".to_string());
        }
        let (dir_path, name) = match path.rfind('/') {
            Some(0) => ("/", &path[1..]),
            Some(i) => (&path[..i], &path[i + 1..]),
            // A relative path has no root to anchor to. The shell resolves those
            // against its working directory before calling, so reaching here is
            // a bug rather than a user error.
            None => return Err("path is not absolute".to_string()),
        };
        if name.is_empty() {
            return Err("no file name given".to_string());
        }
        Ok((self.resolve_dir(dir_path)?, name))
    }

    /// Create a directory and every missing level above it.
    fn mkdir_p(&self, path: &str) -> Result<(), String> {
        let mut cur = tfs::ROOT_DIR;
        for part in path.split('/').filter(|s| !s.is_empty()) {
            // An existing directory is the success case, not a failure: the
            // shell uses this to ensure a home directory, and a second `mkdir`
            // of the same path must not report an error.
            match tfs::find_dir(self.dev, cur, part) {
                Ok(next) => cur = next,
                Err(_) => {
                    tfs::mkdir(self.dev, cur, part).map_err(|e| format!("{part}: {e:?}"))?;
                    cur = tfs::find_dir(self.dev, cur, part)
                        .map_err(|_| format!("{part}: created but not found"))?;
                }
            }
        }
        Ok(())
    }
}


/// What the kernel tells the shell about the machine.
pub struct KernelSys;

impl Sys for KernelSys {
    fn uname(&mut self) -> String {
        String::from("TrangorgeOS")
    }

    fn memory(&mut self) -> (u64, u64) {
        // `phys` owns the frame bitmap, so it is the only thing here that knows
        // how much memory exists. The pair is (total, free) in that order
        // because that is the order the trait asks for, and swapping them would
        // make `free` report usage.
        (
            crate::mm::phys::total_bytes(),
            crate::mm::phys::free_bytes(),
        )
    }

    fn uptime(&mut self) -> u64 {
        // The PIT is programmed at 1000 Hz, so a tick is a millisecond. Reported
        // in seconds because the trait asks for seconds, and `uptime` printing
        // "86400000" is not a number a person reads. The tick counter itself is
        // the public atomic the IRQ handler bumps.
        crate::interrupts::TIMER_TICKS.load(core::sync::atomic::Ordering::Relaxed) / 1000
    }

    fn cpus(&mut self) -> u32 {
        crate::cpu::total_cpus()
    }
}

impl Fs for DiskFs {
    fn read_dir(&mut self, path: &str) -> Result<Vec<DirEntry>, String> {
        let dir = self.resolve_dir(path)?;
        let entries = tfs::entries(self.dev, dir).map_err(|e| format!("{path}: {e:?}"))?;
        Ok(entries
            .into_iter()
            .map(|(name, size, kind)| DirEntry {
                name,
                // TFS marks a directory with kind 2. The constant is private to
                // that module, so it is compared here rather than by a name
                // that does not exist outside it.
                is_dir: kind == 2,
                size,
            })
            .collect())
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let (dir, name) = self.resolve_parent(path)?;
        tfs::read_file(self.dev, dir, name).map_err(|e| format!("{path}: {e:?}"))
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        let (dir, name) = self.resolve_parent(path)?;
        tfs::write_file(self.dev, dir, name, data).map_err(|e| format!("{path}: {e:?}"))
    }

    fn make_dir(&mut self, path: &str) -> Result<(), String> {
        self.mkdir_p(path)
    }

    fn remove(&mut self, path: &str) -> Result<(), String> {
        let (dir, name) = self.resolve_parent(path)?;
        tfs::remove(self.dev, dir, name).map_err(|e| format!("{path}: {e:?}"))
    }
}

/// Write `text` to the screen *and* the serial port.
///
/// Both, always. The serial port is what `just run` shows and what a log is
/// read from; the framebuffer is what a person at the machine sees. Writing to
/// one and not the other produces a system that looks broken in exactly the way
/// you are not looking.
pub fn say(text: &str) {
    {
        let mut w = WRITER.lock();
        w.set_color(Color::White);
        w.write_string(text);
    }
    crate::serial::write_str(text);
    crate::gfx::refresh();
}

/// Block until a keystroke is available.
fn next_key() -> Option<char> {
    loop {
        if let Some(c) = crate::terminal::pop_char() {
            return Some(c);
        }
        // Nothing typed yet. Halting is right here: this is a console, and
        // spinning would keep the boot CPU busy instead of letting the power
        // scheduler see an idle machine.
        x86_64::instructions::hlt();
    }
}

/// Read one line from the keyboard.
///
/// `echo` is what makes this a login rather than a terminal: the user name is
/// typed back, the password is not. A password echoed to the screen has been
/// shown to everyone in the room and to every framebuffer capture since.
///
/// Backspace is honoured by rewriting the line, so a typo gets corrected rather
/// than submitted.
pub fn read_line(prompt: &str, echo: bool) -> String {
    say_coloured(prompt, Color::LightGreen);

    let mut line = String::new();
    loop {
        let c = match next_key() {
            Some(c) => c,
            None => continue,
        };
        match c {
            '\n' => {
                say("\n");
                return line;
            }
            '\x08' => {
                if line.pop().is_some() && echo {
                    // Rub the character out: step back, overwrite with a space,
                    // step back again. A bare backspace would leave the character
                    // on screen with the cursor parked on top of it.
                    say("\x08 \x08");
                }
            }
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => {
                if line.len() < MAX_LINE {
                    line.push(c);
                    if echo {
                        let mut buf = [0u8; 4];
                        say(c.encode_utf8(&mut buf));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Write `text` in `color` to both outputs.
fn say_coloured(text: &str, color: Color) {
    {
        let mut w = WRITER.lock();
        w.set_color(color);
        w.write_string(text);
    }
    crate::serial::write_str(text);
    crate::gfx::refresh();
}

/// The login sequence. Returns `None` when the caller should ask again.
///
/// The prompt is rebuilt on every call, which is what resets the attempt
/// counter: three tries per *login*, not three for the lifetime of the
/// machine. A counter that survived to the next boot would turn a wrong
/// password at the console into a permanent lockout with no way back but a
/// reboot — and the same three wrong tries again.
fn login(accounts: &Accounts) -> Option<Session> {
    let mut prompt = LoginPrompt::new(accounts);

    loop {
        let text = read_line(prompt.prompt().unwrap_or(""), prompt.echoes());
        let (messages, session) = prompt.submit(&text);

        for m in &messages {
            say(m);
            say("\n");
        }

        if let Some(s) = session {
            return Some(s);
        }

        // A lockout prints its own message; the extra blank line keeps the next
        // prompt from sitting directly under it.
        if prompt.locked_out() {
            say("\n");
        }
    }
}

/// Run the shell for `session` until it closes.
fn run_shell(session: &Session, accounts: &mut Accounts, fs: &mut DiskFs) {
    let mut sys = KernelSys;
    let mut shell = Shell::new(session.clone(), fs, &mut sys, accounts);

    // Everything the shell wrote while being constructed — greeting, home
    // directory, the hint — is shown now, in order. The greeting is the
    // *shell's* text, not something the kernel prints on the way in: a greeting
    // shown before login is a greeting shown to someone who has not yet proved
    // who they are.
    let mut shown = 0usize;
    drain(&mut shell, &mut shown);

    loop {
        say_coloured(&shell.prompt(), Color::LightCyan);
        let line = read_line("", true);

        if shell.run(&line) {
            say("logout\n");
            return;
        }

        // `run` has already echoed the command line into the terminal, and the
        // user's typing was echoed by `read_line` above, so only the command's
        // output still needs printing.
        drain(&mut shell, &mut shown);
    }
}

/// Print terminal lines that have not been printed yet.
///
/// The terminal is cumulative and the screen is not: reprinting everything on
/// every command would scroll earlier output away, so the count of already
/// printed lines is what separates "the command's output" from "the scrollback".
fn drain(shell: &mut Shell<'_>, shown: &mut usize) {
    let history = shell.terminal().history();
    if history.len() < *shown {
        // The terminal dropped old lines to stay within its capacity, so the
        // indexes have shifted. Start over rather than print the wrong range.
        *shown = 0;
    }
    for text in &history[*shown..] {
        say(text);
        say("\n");
    }
    *shown = history.len();
}

/// Log in, run the shell, and return to the login prompt when it ends.
///
/// This is the loop the machine boots into.
pub fn run(dev: &'static dyn BlockDevice) {
    let mut accounts = Accounts::with_root();
    let mut fs = DiskFs::new(dev);

    // The accounts table is in-memory and the disk has no format for it yet, so
    // every boot starts with exactly one account. Said once, plainly, rather
    // than discovered by a user who cannot get in.
    say("first boot: the only account is 'root' / 'root'\n\n");

    loop {
        let session = match login(&accounts) {
            Some(s) => s,
            // `login` only returns `None` in a form it does not currently have;
            // going round again is the correct reaction to it either way.
            None => continue,
        };

        run_shell(&session, &mut accounts, &mut fs);
    }
}
