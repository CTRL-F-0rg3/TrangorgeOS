#[cfg(test)]
mod tests {
    use crate::apps::shell::{Fs, MemoryFs, MockSys};
    use crate::bootstrap::{build_tree, MemoryVolume};
    use crate::apps::shell::PromptStyle;
    use crate::commands::accounts::LoginPrompt;
    use crate::login::Accounts;
    use crate::Shell;

    /// A shell on a filesystem that already holds the userspace tree.
    fn shell_over<'a>(
        fs: &'a mut MemoryFs,
        sys: &'a mut MockSys,
        accounts: &'a mut Accounts,
    ) -> Shell<'a> {
        // The tree the bootstrap would have created.
        for d in crate::layout::required_dirs() {
            fs.mkdir(&d);
        }
        let session = accounts
            .login("root", "root")
            .expect("root account exists");
        Shell::new(session, fs, sys, accounts)
    }

    fn greets_with_hello_in_userspace() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let sh = shell_over(&mut fs, &mut sys, &mut accounts);
        let first = sh.terminal().history().first().copied().unwrap_or("");
        assert_eq!(first, "hello in userspace");
    }

    #[test]
    fn starts_in_the_rung_one_home() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("pwd");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "/kernel/userspace/user/root/home/r1");
    }

    /// A file the shell did not create is still readable.
    ///
    /// `write` is gone: the Linux set has no such command, and adding one would
    /// mean teaching userspace a way to fabricate a file that `cp` cannot. A file
    /// is made with `touch` and filled with `cp` from something real — so this
    /// seeds one directly through the filesystem, before the shell takes the
    /// borrow, and then reads it with `cat`.
    #[test]
    fn a_seeded_file_is_readable_with_cat() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        let cwd = sh.session().cwd.clone();
        // Drop the shell so the filesystem can be seeded, then rebuild it.
        drop(sh);
        fs.write_file(&format!("{cwd}/notes.txt"), b"hello world");
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("cat notes.txt");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "hello world");
    }

    #[test]
    fn grep_filters_loaded_lines() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        let cwd = sh.session().cwd.clone();
        drop(sh);
        fs.write_file(&format!("{cwd}/log"), b"alpha\nbeta\ngamma\n");
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("grep gamma log");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "gamma");
    }

    #[test]
    fn mkdir_then_ls_shows_the_directory() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("mkdir projects");
        sh.run("ls");
        let all = sh.terminal().history().join("\n");
        assert!(all.contains("projects/"), "ls should show the new dir: {all}");
    }

    #[test]
    fn cd_cannot_escape_the_rung_home() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("cd /kernel");
        let all = sh.terminal().history().join("\n");
        assert!(all.contains("outside this session's home"), "{all}");
        // And the working directory did not move.
        sh.run("pwd");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "/kernel/userspace/user/root/home/r1");
    }

    #[test]
    fn tilde_expands_to_the_rung_home() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("mkdir sub");
        sh.run("cd sub");
        sh.run("cd ~");
        sh.run("pwd");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "/kernel/userspace/user/root/home/r1");
    }

    #[test]
    fn rm_refuses_the_working_directory() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("rm .");
        let all = sh.terminal().history().join("\n");
        assert!(all.contains("refusing to remove"), "{all}");
    }

    #[test]
    fn unknown_command_reports_itself() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        sh.run("frobnicate");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert!(out.contains("command not found"), "{out}");
    }

    #[test]
    fn exit_closes_the_shell() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        let mut sh = shell_over(&mut fs, &mut sys, &mut accounts);
        assert!(!sh.terminal().is_closed());
        sh.run("exit");
        assert!(sh.terminal().is_closed());
    }

    #[test]
    fn tree_contains_only_r1() {
        let mut vol = MemoryVolume::new();
        let report = build_tree(&mut vol);
        assert!(report.is_complete(), "{report:?}");
        for rung in 2..=crate::layout::RUNG_MAX {
            let p = crate::layout::rung_home(crate::layout::ROOT_USER, rung);
            assert!(!vol.has(&p), "rung {rung} must not be created");
        }
        assert!(vol.has(&crate::layout::rung_home(crate::layout::ROOT_USER, 1)));
    }

    #[test]
    fn building_the_tree_twice_is_idempotent() {
        let mut vol = MemoryVolume::new();
        let first = build_tree(&mut vol);
        assert!(first.is_fresh_install(), "{first:?}");
        let second = build_tree(&mut vol);
        assert!(second.is_complete(), "{second:?}");
        assert!(second.created.is_empty(), "second run created {:?}", second.created);
    }

    #[test]
    fn shell_path_apps_exist_on_the_volume() {
        let mut vol = MemoryVolume::new();
        build_tree(&mut vol);
        assert!(vol.has(&crate::layout::shell_app_dir("root")));
        assert!(vol.has(&crate::layout::terminal_app_dir("root")));
    }

    /// The login prompt asks for a name, then a password, and only then yields a
    /// session.
    ///
    /// Two `submit` calls is the whole sequence, and asserting on the prompt
    /// between them is what proves the password field does not echo.
    #[test]
    fn login_asks_for_a_name_then_a_password() {
        let accounts = Accounts::with_root();
        let mut login = LoginPrompt::new(&accounts);

        assert_eq!(login.prompt(), Some("TrangorgeOS login: "));
        assert!(login.echoes(), "the user name is visible while being typed");

        let (_, session) = login.submit("root");
        assert!(session.is_none(), "a name alone is not a login");
        assert_eq!(login.prompt(), Some("Password: "));
        assert!(!login.echoes(), "the password must not be echoed");

        let (messages, session) = login.submit("root");
        assert!(messages.is_empty());
        let session = session.expect("correct password logs in");
        assert_eq!(session.user, "root");
        assert_eq!(session.rung, 1);
    }

    /// A wrong password is refused, and the failure does not say *which* half was
    /// wrong — otherwise the message is a way to enumerate accounts.
    #[test]
    fn a_wrong_password_is_refused_without_saying_which_half_was_wrong() {
        let accounts = Accounts::with_root();
        let mut login = LoginPrompt::new(&accounts);
        login.submit("root");
        let (messages, session) = login.submit("not-the-password");
        assert!(session.is_none());
        assert!(!messages.join(" ").contains("password"), "{messages:?}");

        // An account that does not exist gets the same single message, so the
        // two cases cannot be told apart.
        let mut other = LoginPrompt::new(&accounts);
        other.submit("nosuchuser");
        let (messages, _) = other.submit("whatever");
        assert_eq!(messages.len(), 1);
    }

    /// The attempt limit stops a password being ground one guess at a time.
    #[test]
    fn login_locks_out_after_too_many_attempts() {
        let accounts = Accounts::with_root();
        let mut login = LoginPrompt::new(&accounts);
        for _ in 0..3 {
            login.submit("root");
            let (_, session) = login.submit("wrong");
            assert!(session.is_none());
        }
        assert!(login.locked_out());

        // Even the right password is refused once locked out.
        login.submit("root");
        let (_, session) = login.submit("root");
        assert!(session.is_none(), "a locked-out prompt must not accept a login");
    }

    /// The prompt shows the rung, and marks a rung above 1 so a user can tell.
    #[test]
    fn the_prompt_names_the_rung_and_marks_an_elevated_one() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        for d in crate::layout::required_dirs() {
            fs.mkdir(&d);
        }
        let session = accounts.login("root", "root").unwrap();
        let sh = Shell::new(session, &mut fs, &mut sys, &mut accounts);
        assert!(sh.prompt().contains("@r1:"), "{}", sh.prompt());

        // A rung above 1 gets a `!` so it cannot be mistaken for rung 1.
        // `Session` is cloned rather than copied: it owns its paths.
        let mut elevated = sh.session().clone();
        elevated.rung = 7;
        let style = PromptStyle::for_rung(7);
        assert!(
            style.render(&elevated).contains("r7!"),
            "{}",
            style.render(&elevated)
        );
    }

    /// `useradd` is root-only, and the account it makes can log in afterwards.
    #[test]
    fn useradd_is_root_only_and_creates_a_usable_account() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        for d in crate::layout::required_dirs() {
            fs.mkdir(&d);
        }
        let root = accounts.login("root", "root").unwrap();
        let mut sh = Shell::new(root, &mut fs, &mut sys, &mut accounts);

        sh.run("useradd -r 3 alice hunter2");

        // The shell borrows `accounts` for as long as it lives, so the store is
        // inspected after dropping it. Doing it the other way round would not
        // compile, which is the borrow checker saying the ownership is exclusive.
        let sh_session = sh.session().clone();
        drop(sh);
        assert!(
            accounts.get("alice").is_some(),
            "useradd should have created the account"
        );
        assert!(accounts.login("alice", "hunter2").is_ok());

        // A non-root session is refused.
        let mut guest = sh_session;
        guest.user = "guest".to_string();
        let mut sh2 = Shell::new(guest, &mut fs, &mut sys, &mut accounts);
        sh2.run("useradd mallory pw");
        drop(sh2);
        assert!(
            accounts.get("mallory").is_none(),
            "a guest must not add accounts"
        );
    }

    /// `passwd` changes the password, and the old one stops working.
    #[test]
    fn passwd_replaces_the_password() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut accounts = Accounts::with_root();
        for d in crate::layout::required_dirs() {
            fs.mkdir(&d);
        }
        let root = accounts.login("root", "root").unwrap();
        let mut sh = Shell::new(root, &mut fs, &mut sys, &mut accounts);

        sh.run("passwd newsecret");
        drop(sh);
        assert!(accounts.login("root", "newsecret").is_ok());
        assert!(
            accounts.login("root", "root").is_err(),
            "the old password must stop working"
        );
    }
}
