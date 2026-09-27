//! The on-disk layout of TrangorgeOS, as it exists **inside the running
//! system**, not on the build host.
//!
//! # Ownership
//!
//! TrangorgeOS is a *distributed* system: different parts are contributed by
//! different people, and this crate owns only its own. `allde` (the niri-like
//! tiling desktop) is one such part and is **not** ours to modify — not this
//! crate, not its `Shell`, not its window manager. If a change here would
//! require touching `allde`, the change is wrong; it belongs in this tree
//! instead. See [`apps`] for what this crate does own.
//!
//! # Who owns which level
//!
//! Each directory is created by exactly one owner:
//!
//! ```text
//! /                      the data volume (TFS)
//! └── kernel/            owned by the kernel: it creates this at first boot
//!     ├── userspace/     owned by *this crate*, not the kernel
//!     │   ├── user/
//!     │   │   └── root/
//!     │   │       ├── shell/     the default shell application
//!     │   │       ├── terminal/  a standalone command terminal
//!     │   │       └── home/
//!     │   │           └── r1/     per-privilege-rung home, r1..r128 reserved
//!     │   └── system/    configuration and logs, not user-writable
//!     └── driverspace/   owned by the driver space
//! ```
//!
//! This crate creates its own subtree the first time it runs, which is why a
//! fresh image containing only `kernel/` still comes up: the tree grows on
//! first boot rather than needing to be baked in.
//!
//! # The `r1`..`r128` rungs
//!
//! `home/rN/` is a per-privilege-rung home. The numbering is reserved up to
//! 128, but **only `r1` is created**. Creating the rest would commit the system
//! to a meaning for rungs 2..128 that nobody has decided yet — a rung that
//! exists with no defined contents is worse than an absent one, because code
//! will treat it as a real privilege level and misbehave when the actual policy
//! for it lands. `r1` exists so the mechanism is proven; the rest appear when
//! their semantics are.

/// Directory names, relative to the volume root.
pub const KERNEL_DIR: &str = "kernel";
pub const USERSPACE_DIR: &str = "userspace";
pub const DRIVERSPACE_DIR: &str = "driverspace";
pub const USER_DIR: &str = "user";
pub const ROOT_USER: &str = "root";
pub const SHELL_DIR: &str = "shell";
pub const TERMINAL_DIR: &str = "terminal";
pub const HOME_DIR: &str = "home";
pub const SYSTEM_DIR: &str = "system";

/// The lowest privilege rung that is actually created.
pub const RUNG_MIN: u32 = 1;
/// The highest rung the numbering reserves. See the module docs: these are
/// *reserved*, not provisioned.
pub const RUNG_MAX: u32 = 128;

// Every path in the system is absolute, so these are built from a leading `/`
// rather than from the previous segment. Formatting `"/{a}/{b}"` with a `a` that
// already starts with `/` is how a path ends up as `//a/b` and then fails to
// compare equal to the same path built the other way — the two must be built
// the same way or they will not be the same string.

/// `/kernel`
pub fn kernel_dir() -> String {
    format!("/{KERNEL_DIR}")
}

/// `/kernel/userspace`
pub fn uspace_dir() -> String {
    format!("/{KERNEL_DIR}/{USERSPACE_DIR}")
}

/// `/kernel/driverspace`
pub fn driverspace_dir() -> String {
    format!("/{KERNEL_DIR}/{DRIVERSPACE_DIR}")
}

/// `/kernel/userspace/user`
pub fn user_dir() -> String {
    format!("/{KERNEL_DIR}/{USERSPACE_DIR}/{USER_DIR}")
}

/// `/kernel/userspace/user/<user>`
pub fn home_of(user: &str) -> String {
    format!("/{KERNEL_DIR}/{USERSPACE_DIR}/{USER_DIR}/{user}")
}

/// `/kernel/userspace/user/<user>/home/r1`
///
/// The rung-1 home. This is the only rung provisioned.
pub fn rung_home(user: &str, rung: u32) -> String {
    format!("{}/{}/r{}", home_of(user), HOME_DIR, rung)
}

/// `/kernel/userspace/user/<user>/shell`
pub fn shell_app_dir(user: &str) -> String {
    format!("{}/{SHELL_DIR}", home_of(user))
}

/// `/kernel/userspace/user/<user>/terminal`
pub fn terminal_app_dir(user: &str) -> String {
    format!("{}/{TERMINAL_DIR}", home_of(user))
}

/// `/kernel/userspace/system`
pub fn system_dir() -> String {
    format!("{}/{SYSTEM_DIR}", uspace_dir())
}

/// Normalise a path: collapse repeated slashes, resolve `.` and `..`, and make
/// it absolute. `""` and `"/"` both become `"/"`.
///
/// `..` cannot ascend past the root: `"/a/../.."` is `"/"`. Letting it go
/// above the root would produce a path that addresses nothing, and a shell that
/// accepted one would be trusting a caller to check the result.
pub fn normalize(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    if stack.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", stack.join("/"))
    }
}

/// Split an absolute path into its components, dropping the empty head that a
/// leading `/` produces.
///
/// `"/a/b"` yields `["a", "b"]`; `"/"` yields `[]`.
pub fn components(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

/// Whether `path` is `ancestor` or lies beneath it.
///
/// The comparison is on whole components, so `/kernel/user` is *not* inside
/// `/kernel/users`: a trailing `r` must not match on a bare prefix.
pub fn is_under(ancestor: &str, path: &str) -> bool {
    let a = components(ancestor);
    let p = components(path);
    if a.len() > p.len() {
        return false;
    }
    a.iter().zip(p.iter()).all(|(x, y)| x == y)
}

/// Every directory `bootstrap::build_tree` must create, in creation order.
///
/// Parents always precede children, so a caller can create them in order
/// without resolving dependencies itself.
pub fn required_dirs() -> Vec<String> {
    let user = ROOT_USER;
    vec![
        kernel_dir(),
        uspace_dir(),
        user_dir(),
        home_of(user),
        rung_home(user, RUNG_MIN),
        shell_app_dir(user),
        terminal_app_dir(user),
        system_dir(),
        // A sibling of userspace; the driver space owns it, but the path must
        // exist for the driver space to attach to.
        driverspace_dir(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_slashes() {
        assert_eq!(normalize(""), "/");
        assert_eq!(normalize("/"), "/");
        assert_eq!(normalize("a//b"), "/a/b");
        assert_eq!(normalize("/a/b/"), "/a/b");
    }

    #[test]
    fn resolves_dot_and_dotdot() {
        assert_eq!(normalize("/a/./b"), "/a/b");
        assert_eq!(normalize("/a/b/.."), "/a");
        // `..` cannot climb above the root.
        assert_eq!(normalize("/a/../.."), "/");
        assert_eq!(normalize("/.."), "/");
    }

    #[test]
    fn every_path_is_absolute() {
        // A relative path here is the bug that made `rm .` fail: it resolved to
        // ".../r1/." instead of the working directory.
        for p in crate::required_dirs() {
            assert!(p.starts_with('/'), "{p} must be absolute");
        }
    }

    #[test]
    fn components_drops_empty_head() {
        assert_eq!(components("/"), Vec::<&str>::new());
        assert_eq!(components("/a/b"), vec!["a", "b"]);
    }

    #[test]
    fn under_compares_whole_components() {
        assert!(is_under("/kernel/userspace", "/kernel/userspace/user/root"));
        assert!(is_under("/kernel", "/kernel/userspace"));
        // A shared prefix is not containment.
        assert!(!is_under("/kernel/user", "/kernel/userspace"));
        assert!(!is_under("/kernel/userspace/user/root", "/kernel"));
    }

    #[test]
    fn only_r1_is_provisioned() {
        let dirs = required_dirs();
        let rungs: Vec<u32> = dirs
            .iter()
            .filter_map(|d| d.strip_prefix("/kernel/userspace/user/root/home/r"))
            .filter_map(|tail| tail.parse().ok())
            .collect();
        assert_eq!(rungs, vec![RUNG_MIN]);
    }

    #[test]
    fn parents_precede_children() {
        let dirs = required_dirs();
        for (i, d) in dirs.iter().enumerate() {
            for e in &dirs[i + 1..] {
                if is_under(d, e) {
                    assert!(
                        dirs.iter().position(|x| x == d) < dirs.iter().position(|x| x == e),
                        "{d} must be created before {e}"
                    );
                }
            }
        }
    }
}
