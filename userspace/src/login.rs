//! Users, passwords and sessions.
//!
//! # Why this is its own module
//!
//! The shell needs to know *who* it is running as — the prompt shows it, and
//! `~` resolves to that user's rung home. That makes identity a property of the
//! session, not of the shell, so the shell takes a [`Session`] and never
//! touches an account store.
//!
//! # Passwords
//!
//! [`Account::verify`] compares a stored string and is **not** a password hash.
//! It stays a method rather than an `==` at the call site precisely so that
//! swapping in a KDF with a per-account salt is a change in one function. The
//! weakness is documented at the definition instead of hidden.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// What went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginError {
    /// No such account, or wrong password — indistinguishable to the caller.
    BadPassword,
    /// The account is locked, or otherwise not permitted to log in.
    Denied,
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Both cases read the same to the user. Distinguishing them turns
            // the login form into a user-enumeration oracle, and there is
            // nothing to gain from telling an attacker which half failed.
            Self::BadPassword => write!(f, "login failed"),
            Self::Denied => write!(f, "account is not permitted to log in"),
        }
    }
}

/// One account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub name: String,
    /// Stored verifier. See [`Self::verify`].
    pub password_hash: String,
    /// The privilege rung this account logs in at, in `1..=RUNG_MAX`.
    pub rung: u32,
    /// May this account log in at all?
    pub enabled: bool,
}

impl Account {
    /// Check a password against the stored verifier.
    ///
    /// Plain string comparison. Not a hash, and not to be mistaken for one: a
    /// real deployment needs a KDF, a per-account salt and a work factor. It
    /// stays a method so that swap is a change in one function.
    pub fn verify(&self, password: &str) -> bool {
        self.password_hash == password
    }

    /// The rung home this account's session starts in.
    pub fn home(&self) -> String {
        crate::layout::rung_home(&self.name, self.rung)
    }
}

/// The account store.
#[derive(Debug, Default, Clone)]
pub struct Accounts {
    by_name: BTreeMap<String, Account>,
}

impl Accounts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace an account.
    ///
    /// A rung outside `RUNG_MIN..=RUNG_MAX` is rejected rather than clamped: a
    /// typo in an account file should fail loudly, not silently become rung 128.
    pub fn insert(&mut self, account: Account) -> Result<(), String> {
        if account.rung < crate::layout::RUNG_MIN || account.rung > crate::layout::RUNG_MAX {
            return Err(format!(
                "rung {} outside {}..={}",
                account.rung,
                crate::layout::RUNG_MIN,
                crate::layout::RUNG_MAX
            ));
        }
        self.by_name.insert(account.name.clone(), account);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Account> {
        self.by_name.get(name)
    }

    /// Every account, in name order.
    pub fn all(&self) -> Vec<Account> {
        self.by_name.values().cloned().collect()
    }

    /// Remove an account, returning it if it existed.
    pub fn remove(&mut self, name: &str) -> Option<Account> {
        self.by_name.remove(name)
    }

    /// Replace an account's password.
    ///
    /// The stored value is a plain comparison, not a hash — see
    /// [`Account::verify`]. This is the one function that would change when a
    /// KDF lands, which is why it is a method rather than a field write.
    pub fn set_password(&mut self, name: &str, password: &str) -> Result<(), String> {
        let account = self
            .by_name
            .get_mut(name)
            .ok_or_else(|| format!("no such account: {name}"))?;
        account.password_hash = password.to_string();
        Ok(())
    }

    pub fn names(&self) -> Vec<String> {
        self.by_name.keys().cloned().collect()
    }

    /// The account every fresh install starts with.
    ///
    /// Password is the literal `root` and the account is enabled, because a
    /// fresh image has to be usable without an install step. It exists to be
    /// changed, not to be trusted.
    pub fn with_root() -> Self {
        let mut a = Accounts::new();
        let _ = a.insert(Account {
            name: crate::layout::ROOT_USER.to_string(),
            password_hash: "root".to_string(),
            rung: crate::layout::RUNG_MIN,
            enabled: true,
        });
        a
    }

    /// Attempt a login.
    ///
    /// An unknown user and a wrong password produce the same error, so a caller
    /// cannot use the message to enumerate accounts.
    pub fn login(&self, name: &str, password: &str) -> Result<Session, LoginError> {
        let account = self
            .by_name
            .get(name)
            .ok_or(LoginError::BadPassword)?;
        if !account.enabled {
            return Err(LoginError::Denied);
        }
        if !account.verify(password) {
            return Err(LoginError::BadPassword);
        }
        Ok(Session::from(account))
    }
}

/// An authenticated user, and the identity a shell runs as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub user: String,
    /// The privilege rung this session holds.
    pub rung: u32,
    /// The user's rung home. This is what `~` expands to.
    pub home: String,
    /// The working directory. Starts at [`Self::home`], and `cd` moves it.
    ///
    /// Kept separate from `home` because a shell that treats them as the same
    /// breaks the moment the user enters a subdirectory: `cd sub` followed by
    /// `cd ~` has to return to the home, not to `sub`.
    pub cwd: String,
}

impl Session {
    /// Start a session for `account`, rooted at its rung home.
    pub fn from(account: &Account) -> Self {
        let home = account.home();
        Self {
            user: account.name.clone(),
            rung: account.rung,
            cwd: home.clone(),
            home,
        }
    }

    /// The prompt's user part, e.g. `root@r1`.
    pub fn whoami(&self) -> String {
        format!("{}@r{}", self.user, self.rung)
    }

    /// Whether `path` lies inside this session's rung home.
    ///
    /// Checked against `home`, never against `cwd`: the boundary is the rung, and
    /// `cwd` moves. A guard that compared against the current directory would
    /// stop constraining the session the moment it entered a subdirectory —
    /// `cd sub` would make everything outside `sub` suddenly "not owned", and
    /// `cd ~` would fail.
    ///
    /// A usability guard, not a security boundary; that is the capability
    /// table's job.
    pub fn owns(&self, path: &str) -> bool {
        crate::layout::is_under(&self.home, &crate::layout::normalize(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(name: &str, pass: &str, rung: u32, enabled: bool) -> Account {
        Account {
            name: name.into(),
            password_hash: pass.into(),
            rung,
            enabled,
        }
    }

    #[test]
    fn root_session_starts_in_its_rung_home() {
        let s = Accounts::with_root().login("root", "root").unwrap();
        assert_eq!(s.user, "root");
        assert_eq!(s.rung, 1);
        assert_eq!(s.cwd, "/kernel/userspace/user/root/home/r1");
        assert_eq!(s.whoami(), "root@r1");
    }

    #[test]
    fn wrong_password_and_unknown_user_look_identical() {
        let a = Accounts::with_root();
        let unknown = a.login("nobody", "root").unwrap_err();
        let wrong = a.login("root", "nope").unwrap_err();
        assert_eq!(unknown, wrong);
        assert_eq!(unknown.to_string(), wrong.to_string());
    }

    #[test]
    fn disabled_account_is_denied_even_with_the_right_password() {
        let mut a = Accounts::with_root();
        a.insert(account("guest", "guest", 1, false)).unwrap();
        assert_eq!(
            a.login("guest", "guest").unwrap_err(),
            LoginError::Denied
        );
    }

    #[test]
    fn out_of_range_rung_is_rejected_not_clamped() {
        let mut a = Accounts::new();
        assert!(a.insert(account("x", "x", 0, true)).is_err());
        assert!(a.insert(account("x", "x", 129, true)).is_err());
        // The boundary values are accepted.
        assert!(a.insert(account("x", "x", 1, true)).is_ok());
        assert!(a.insert(account("y", "y", 128, true)).is_ok());
    }

    #[test]
    fn session_owns_only_its_own_rung_home() {
        let s = Accounts::with_root().login("root", "root").unwrap();
        assert!(s.owns(&s.cwd));
        assert!(s.owns("/kernel/userspace/user/root/home/r1/sub"));
        assert!(!s.owns("/kernel/userspace/user/root/home/r2"));
        assert!(!s.owns("/kernel"));
    }
}
