//! Message kinds.

/// The kind (intent) of a message.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    /// A request that expects a reply.
    Request = 0,
    /// A reply to a previous request.
    Reply = 1,
    /// An asynchronous event emitted by the kernel or a driver.
    Event = 2,
    /// A fire-and-forget notification.
    Notification = 3,
    /// A message carrying a capability transfer body.
    CapTransfer = 4,
}
