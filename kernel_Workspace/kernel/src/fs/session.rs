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

/// Switch the keyboard driver into keycode mode and report what input there is.
///
/// Must run before anything calls [`next_key`]. The driver has two modes and the
/// default is *character* mode, chosen back when the kernel terminal was the only
/// consumer — and in that mode a scancode becomes a `char` immediately, so the
/// arrow keys are dropped on the floor (`scancode_to_char` has no case for them).
/// The resolution menu needs those keys, so keycode mode is turned on once here
/// and the mapping happens in [`key_from_code`] instead.
///
/// # Both keyboards are live at once
///
/// This is not "USB if present, otherwise PS/2" as an either/or. Both queues are
/// drained on every pass of [`next_key`], so a machine with both stays usable and
/// a keystroke on either device gets through — which is the behaviour you want
/// when the USB keyboard is the one in front of you and the PS/2 port is the one
/// nobody plugged anything into. USB is checked first only because it costs a
/// completed transfer to have anything queued, so it is the one that can still
/// gain a character between two passes.
///
/// The report is for the user rather than for the code: the branch itself needs
/// no choice made in advance, which is the point.
///
/// Calling this more than once is harmless: it sets a flag.
pub fn init_input() {
    crate::terminal::set_keycode_capture(true);

    let usb = crate::drivers::usb::class::hid::keyboard_attached();
    if usb {
        say("input: USB keyboard (PS/2 still active)\n");
    } else {
        // Not a warning. PS/2 is a complete input path on its own, and a machine
        // that never had a USB keyboard plugged in is not broken.
        say("input: PS/2 keyboard (no USB keyboard found)\n");
    }
}

/// A keystroke, from whichever keyboard produced it.
///
/// The two keyboard drivers do not report the same things. PS/2 hands over
/// *keycodes*, so it can name a key that has no character at all — an arrow, an
/// Escape. The USB HID driver translates to characters in the driver and keeps
/// no keycode, so those keys are simply lost. Modelling a keystroke rather than
/// a character is what lets one `read_line` serve both: `Char` is the common
/// case, and the special variants are reported by PS/2 and ignored by USB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    Left,
    Right,
}

/// Block until a keystroke is available on any keyboard.
///
/// This is the join point for the two input paths, and it is why there is only
/// one keyboard queue concern to reason about:
///
/// * **PS/2** — the IRQ handler pushes a keycode; `scancode_to_keycode` maps it
///   and falls through to the character table, so digits and letters arrive here
///   as `Char`.
/// * **USB** — the xHCI transfer completes, `hid::poll` decodes a boot-protocol
///   report and pushes a character into its own ring, which is drained here.
///
/// `usb::poll()` has to be called *before* draining, not after: the characters
/// only exist once the transfer has been serviced, and a driver that is only
/// polled on the way past would report an empty keyboard forever.
pub fn next_key() -> Option<Key> {
    loop {
        // Service USB first so a completed transfer can deposit a character
        // during this same pass. `poll` walks the event ring and is cheap when
        // there is nothing pending, so this is safe to call in a tight loop.
        crate::drivers::usb::poll();

        if let Some(c) = crate::drivers::usb::class::hid::keyboard::take_char() {
            return Some(key_from_char(c));
        }

        if let Some(code) = crate::terminal::pop_keycode() {
            return Some(key_from_code(code));
        }

        // Nothing typed yet. Halting is right here: this is a console, and
        // spinning would keep the boot CPU busy instead of letting the power
        // scheduler see an idle machine.
        x86_64::instructions::hlt();
    }
}

/// Turn a PS/2 keycode into a keystroke.
///
/// The numeric constants are the ones the keyboard driver already uses
/// (`scancode_to_keycode` in `terminal`); they are spelled out here rather than
/// re-exported because that table is the driver's business, not this module's.
fn key_from_code(code: u32) -> Key {
    match code {
        0x100 => Key::Enter,
        0x101 => Key::Backspace,
        0x102 => Key::Esc,
        0x103 => Key::Right,
        0x104 => Key::Left,
        0x105 => Key::Down,
        0x106 => Key::Up,
        // Everything else — letters, digits, and the function keys — falls
        // through as a character, and anything that is not a printable one is
        // dropped by `read_line` on the way past.
        c if (0x20..0x7F).contains(&(c as u8)) => Key::Char(c as u8 as char),
        _ => Key::Char('\0'),
    }
}

/// Turn a USB HID byte into a keystroke.
///
/// The driver only ever stores characters, so this is mostly a matter of
/// recognising the two it encodes as control values: `0x28` (Enter) and `0x2A`
/// (backspace) come out of `key_to_ascii` as those byte values.
fn key_from_char(c: u8) -> Key {
    match c {
        b'\n' | b'\r' => Key::Enter,
        0x08 | 0x7F => Key::Backspace,
        0x1B => Key::Esc,
        _ => Key::Char(c as char),
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
        let key = match next_key() {
            Some(k) => k,
            None => continue,
        };
        match key {
            Key::Enter => {
                say("\n");
                return line;
            }
            Key::Backspace => {
                if line.pop().is_some() && echo {
                    // Rub the character out: step back, overwrite with a space,
                    // step back again. A bare backspace would leave the character
                    // on screen with the cursor parked on top of it.
                    say("\x08 \x08");
                }
            }
            // Arrow keys carry no text; a line editor that cannot move the
            // cursor along the line just ignores them rather than inserting
            // something meaningless.
            Key::Up | Key::Down | Key::Left | Key::Right | Key::Esc => {}
            Key::Char(c) if (c as u32) >= 0x20 && (c as u32) < 0x7F => {
                if line.len() < MAX_LINE {
                    line.push(c);
                    if echo {
                        let mut buf = [0u8; 4];
                        say(c.encode_utf8(&mut buf));
                    }
                }
            }
            Key::Char(_) => {}
        }
    }
}

/// Write `text` in `color` to both outputs.
pub fn say_coloured(text: &str, color: Color) {
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
