//! The on-disk hierarchy of TrangorgeOS, as shared by every layer.
//!
//! # Why this lives in `tg-comm`
//!
//! The kernel creates `/kernel/`, userspace creates the tree below it, and the
//! driver space attaches to its own directory. All three are separate crates
//! with separate build targets, so a path defined in any one of them would be
//! a path the other two cannot see. `tg-comm` is the one crate every layer
//! links — and it is `no_std`, so the kernel can use it too. A path therefore
//! has exactly one definition, and changing the hierarchy is a change here that
//! every layer picks up.
//!
//! # The tree
//!
//! ```text
//! /                      the data volume (TFS)
//! └── kernel/            created by the kernel
//!     ├── userspace/     created by userspace on first boot
//!     │   ├── user/<name>/
//!     │   │   ├── shell/     the default shell application
//!     │   │   ├── terminal/  a standalone command terminal
//!     │   │   └── home/rN/   per-privilege-rung home
//!     │   └── system/    configuration and logs
//!     └── driverspace/   owned by the driver space
//! ```
//!
//! # The `r1`..`r128` rungs
//!
//! `home/rN/` is a per-privilege-rung home. The numbering is reserved up to
//! [`RUNG_MAX`], but only [`RUNG_MIN`] is created. Rungs 2..128 are *reserved,
//! not provisioned*: an existing directory with no defined contents is worse
//! than an absent one, because code will treat it as a real privilege level and
//! misbehave once its policy lands. `r1` exists so the mechanism is proven; the
//! rest appear when their semantics are decided.

extern crate alloc;

// `format!` is not in the `no_std` prelude; the `alloc` one has to be named
// explicitly. Every path here is built with it, which is why the import is
// spelled out rather than globbed.
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

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
/// The highest rung the numbering reserves. Reserved, not provisioned — see the
/// module docs.
pub const RUNG_MAX: u32 = 128;

// Paths are built from an explicit leading `/` rather than by appending to the
// previous segment. `format!("{}/{}", a, b)` with an `a` that already starts
// with `/` yields `//a/b`, and the two spellings then fail to compare equal — a
// bug that surfaces only as "no such directory" on a path that visibly exists.

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

/// `/kernel/userspace/user/<user>/home/rN`
pub fn rung_home(user: &str, rung: u32) -> String {
    format!("{}/{HOME_DIR}/r{}", home_of(user), rung)
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
    format!("/{KERNEL_DIR}/{USERSPACE_DIR}/{SYSTEM_DIR}")
}

/// Normalise a path: collapse repeated slashes, resolve `.` and `..`, and make
/// it absolute. `""` and `"/"` both become `"/"`.
///
/// `..` cannot climb past the root, so `"/a/../.."` is `"/"`. A path above the
/// root addresses nothing, and a caller that accepted one would have to
/// re-check it.
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

/// Whether `path` is `ancestor` or lies beneath it.
///
/// Compared per whole component, so `/kernel/user` is *not* inside
/// `/kernel/users`: a trailing `r` must not match on a bare prefix.
pub fn is_under(ancestor: &str, path: &str) -> bool {
    let a: Vec<&str> = ancestor.split('/').filter(|s| !s.is_empty()).collect();
    let p: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if a.len() > p.len() {
        return false;
    }
    a.iter().zip(p.iter()).all(|(x, y)| x == y)
}

/// Every directory that must exist once the system is up, in creation order.
///
/// Parents always precede children, so a caller can create them in one pass
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
        driverspace_dir(),
    ]
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Every path is absolute. A relative one is the bug that made `rm .` fail:
    /// it resolved to `.../r1/.` instead of the working directory.
    #[test]
    fn required_dirs_are_absolute() {
        for p in required_dirs() {
            assert!(p.starts_with('/'), "{p} must be absolute");
        }
    }

    /// Only r1 is provisioned. r2..=r128 are reserved but must not exist, or
    /// code will treat an empty directory as a defined privilege level.
    #[test]
    fn only_r1_is_provisioned() {
        let dirs = required_dirs();
        let prefix = format!("{}/home/r", home_of(ROOT_USER));
        let rungs: Vec<u32> = dirs
            .iter()
            .filter_map(|d| d.strip_prefix(&prefix))
            .filter_map(|t| t.parse().ok())
            .collect();
        assert_eq!(rungs, vec![RUNG_MIN]);
    }

    /// Parents must be created before their children, or a single-pass creator
    /// will fail on the deeper path.
    #[test]
    fn parents_precede_children() {
        let dirs = required_dirs();
        for (i, d) in dirs.iter().enumerate() {
            for e in &dirs[i + 1..] {
                if d != e && is_under(d, e) {
                    let pd = dirs.iter().position(|x| x == d).unwrap();
                    let pe = dirs.iter().position(|x| x == e).unwrap();
                    assert!(pd < pe, "{d} must be created before {e}");
                }
            }
        }
    }

    #[test]
    fn normalize_resolves_dot_and_dotdot() {
        assert_eq!(normalize("/a/./b"), "/a/b");
        assert_eq!(normalize("/a/b/.."), "/a");
        assert_eq!(normalize("a//b"), "/a/b");
        // `..` cannot climb above the root.
        assert_eq!(normalize("/a/../.."), "/");
        assert_eq!(normalize("/.."), "/");
        assert_eq!(normalize(""), "/");
        assert_eq!(normalize("/"), "/");
    }

    #[test]
    fn is_under_compares_whole_components() {
        assert!(is_under("/kernel", "/kernel/userspace"));
        assert!(is_under("/kernel/userspace", "/kernel/userspace/user/root"));
        // A shared prefix is not containment.
        assert!(!is_under("/kernel/user", "/kernel/userspace"));
        assert!(!is_under("/kernel/userspace/user/root", "/kernel"));
    }

    #[test]
    fn the_expected_tree() {
        assert_eq!(kernel_dir(), "/kernel");
        assert_eq!(uspace_dir(), "/kernel/userspace");
        assert_eq!(driverspace_dir(), "/kernel/driverspace");
        assert_eq!(home_of("root"), "/kernel/userspace/user/root");
        assert_eq!(rung_home("root", 1), "/kernel/userspace/user/root/home/r1");
        assert_eq!(shell_app_dir("root"), "/kernel/userspace/user/root/shell");
        assert_eq!(terminal_app_dir("root"), "/kernel/userspace/user/root/terminal");
        assert_eq!(system_dir(), "/kernel/userspace/system");
    }
}
