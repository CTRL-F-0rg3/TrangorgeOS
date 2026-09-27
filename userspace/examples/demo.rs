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

    // Log in.
    let accounts = Accounts::with_root();
    let session = match accounts.login("root", "root") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("login failed: {e}");
            return;
        }
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

    let mut shell = Shell::new(session, &mut fs, &mut sys);
    for cmd in [
        "whoami", "pwd", "mkdir projects", "cd projects", "write todo.txt ship the shell",
        "ls", "cat todo.txt", "cd ~", "pwd", "uname", "free", "uptime", "help",
    ] {
        shell.run(cmd);
    }

    println!("\n--- terminal ---");
    for line in shell.terminal().history() {
        println!("{line}");
    }
}
