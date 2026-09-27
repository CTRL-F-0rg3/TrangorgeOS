#[cfg(test)]
mod tests {
    use crate::apps::shell::{MemoryFs, MockSys};
    use crate::bootstrap::{build_tree, MemoryVolume};
    use crate::login::Accounts;
    use crate::Shell;

    /// A shell on a filesystem that already holds the userspace tree.
    fn shell_over<'a>(
        fs: &'a mut MemoryFs,
        sys: &'a mut MockSys,
    ) -> Shell<'a> {
        // The tree the bootstrap would have created.
        for d in crate::layout::required_dirs() {
            fs.mkdir(&d);
        }
        let session = Accounts::with_root()
            .login("root", "root")
            .expect("root account exists");
        Shell::new(session, fs, sys)
    }

    #[test]
    fn greets_with_hello_in_userspace() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let sh = shell_over(&mut fs, &mut sys);
        let first = sh.terminal().history().first().copied().unwrap_or("");
        assert_eq!(first, "hello in userspace");
    }

    #[test]
    fn starts_in_the_rung_one_home() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
        sh.run("pwd");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "/kernel/userspace/user/root/home/r1");
    }

    #[test]
    fn write_then_cat_round_trips() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
        sh.run("write notes.txt hello world");
        sh.run("cat notes.txt");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert_eq!(out, "hello world");
    }

    #[test]
    fn mkdir_then_ls_shows_the_directory() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
        sh.run("mkdir projects");
        sh.run("ls");
        let all = sh.terminal().history().join("\n");
        assert!(all.contains("projects/"), "ls should show the new dir: {all}");
    }

    #[test]
    fn cd_cannot_escape_the_rung_home() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
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
        let mut sh = shell_over(&mut fs, &mut sys);
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
        let mut sh = shell_over(&mut fs, &mut sys);
        sh.run("rm .");
        let all = sh.terminal().history().join("\n");
        assert!(all.contains("refusing to remove"), "{all}");
    }

    #[test]
    fn unknown_command_reports_itself() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
        sh.run("frobnicate");
        let out = sh.terminal().history().last().copied().unwrap_or("");
        assert!(out.contains("command not found"), "{out}");
    }

    #[test]
    fn exit_closes_the_shell() {
        let mut fs = MemoryFs::new();
        let mut sys = MockSys::default();
        let mut sh = shell_over(&mut fs, &mut sys);
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
}
