//! Communication layers (privilege domains) and the strict routing matrix.

/// A communication layer (privilege domain) in TrangorgeOS.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// The trusted kernel core.
    Kernel = 0,
    /// Isolated driver execution environment.
    Driverspace = 1,
    /// Untrusted user applications.
    Userspace = 2,
    /// The privileged driver manager (zarządca).
    Manager = 3,
    /// User driver space: high-risk peripherals kept at a safe distance.
    UserDriverSpace = 4,
}

impl Layer {
    /// Convert a raw wire byte into a `Layer`, if it is valid.
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Kernel),
            1 => Some(Self::Driverspace),
            2 => Some(Self::Userspace),
            3 => Some(Self::Manager),
            4 => Some(Self::UserDriverSpace),
            _ => None,
        }
    }

    /// Whether a message may legally travel from `self` to `target`.
    ///
    /// This is the *first* gate of the protocol: an unauthorised routing is
    /// rejected before the capability check is even consulted.
    pub const fn may_target(self, target: Self) -> bool {
        use Layer::*;
        match (self, target) {
            // Drivers may request kernel services and report to the manager.
            (Driverspace, Kernel) | (Driverspace, Manager) => true,
            // User code may call the kernel and talk to the manager only.
            (Userspace, Kernel) | (Userspace, Manager) => true,
            // The manager orchestrates every layer it governs.
            (Manager, Kernel)
            | (Manager, Driverspace)
            | (Manager, Userspace)
            | (Manager, UserDriverSpace) => true,
            // The kernel may emit replies/events down to every layer.
            (Kernel, Driverspace)
            | (Kernel, Userspace)
            | (Kernel, Manager)
            | (Kernel, UserDriverSpace) => true,
            // User driver space may call the kernel and the manager.
            (UserDriverSpace, Kernel) | (UserDriverSpace, Manager) => true,
            // Everything else (e.g. Userspace -> Driverspace) is forbidden.
            _ => false,
        }
    }
}
