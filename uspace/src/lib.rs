//! TrangorgeOS **userspace** — a nested system with its own capability-based
//! permissions, a connection to the kernel for executive actions, and IPC to
//! the driver space and other user components.
//!
//! # Layering
//!
//! ```text
//! +--------------------------------------------------------------+
//! | apps (shell, terminal, ...)                                  |
//! | userdrivers (specific user-space drivers: input, gpu, ...)   |
//! +--------------------------------------------------------------+
//! | uspace::caps   nested capability permissions (kernel-consistent)
//! | uspace::ipc    ports / channels (app <-> userdriver <-> ds)  |
//! | uspace::kernel executive actions (framebuffer, process, ...) |
//! +--------------------------------------------------------------+
//! | tg-comm shared-memory transport (driverspace / kernel)       |
//! +--------------------------------------------------------------+
//!
//! The permission model is *monotone*: a child process can only hold a subset
//! of its parent's capabilities, which is exactly the property proven in
//! `lib-ada` (capability revocation never grants new authorization). This is
//! what makes it safe for the nested user space to be granted capabilities
//! without risking the kernel.

pub mod caps;
pub mod ipc;
pub mod kernel;
pub mod gfx;

pub use caps::{CapEntry, CapId, CapTable, Process, Rights, Session};
pub use ipc::{IpcEndpoint, IpcMessage, PortId};
pub use kernel::{Framebuffer, KernelClient};

#[cfg(test)]
mod tests;
