//! A userspace demonstration: log in, build the tree, run some commands.
//!
//! This is the "what does it look like" program. It is not the shell — the
//! shell is whatever lives in `user/root/shell/` on the volume. Running this
//! just shows that the pieces fit together.
//!
//! ```sh
//! cd userspace && cargo run --example demo
//! ```

use tgs_userspace::apps::shell::{MemoryFs, MockSys};
use tgs_userspace::bootstrap::{build_tree, MemoryVolume, Volume};
use tgs_userspace::commands::accounts::LoginPrompt;
use tgs_userspace::login::Accounts;
use tgs_userspace::Shell;

fn main() {
    // What the kernel would have created.
    let mut vol = MemoryVolume::new();
    let _ = vol.mkdir("/kernel");

    // What userspace adds on first boot.
    let report = build_tree(&mut vol);
    println!("userspace tree: {} created, {} already present",
        report.created.len(), report.existed.len());
    for d in &vol.dirs() {
        println!("  {d}");
    }

    // Log in, the way the terminal does: prompt, then password, then a session.
    let mut accounts = Accounts::with_root();
    let mut login = LoginPrompt::new(&accounts);
    println!("\n--- login ---");
    println!("{}", login.prompt().unwrap_or(""));
    let (messages, _) = login.submit("root");
    for m in messages {
        println!("{m}");
    }
    println!("{}", login.prompt().unwrap_or(""));
    let (messages, session) = login.submit("root");
    for m in messages {
        println!("{m}");
    }
    let Some(session) = session else {
        eprintln!("login failed");
        return;
    };
    println!("logged in as {}", session.whoami());

    // A filesystem holding the same tree, so the shell has somewhere to look.
    let mut fs = MemoryFs::new();
    for d in vol.dirs() {
        fs.mkdir(&d);
    }
    let mut sys = MockSys {
        boot_uname: "TrangorgeOS".to_string(),
        total_mem: 511 * 1024 * 1024,
        free_mem: 489 * 1024 * 1024,
        uptime_s: 3725,
        cpu_count: 4,
        ..Default::default()
    };

    let mut shell = Shell::new(session, &mut fs, &mut sys, &mut accounts);
    println!("\n--- shell ---");
    for cmd in [
        "pwd", "id", "mkdir projects", "cd projects",
        "ls -la", "users", "uname -a", "free", "df", "seq 3", "basename /a/b/c",
    ] {
        shell.run(cmd);
    }

    println!("\n--- terminal ---");
    for line in shell.terminal().history() {
        println!("{line}");
    }
}
