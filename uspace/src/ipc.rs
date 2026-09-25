//! IPC between userspace components and the driver space.
//!
//! Every component (app, userdriver) owns an [`IpcEndpoint`] backed by the
//! shared-memory `tg_comm` protocol. Messages are routed by port id; the
//! driver space and the kernel are just more ports reachable over the same
//! protocol (the `Layer` byte in each message encodes the routing policy).

use tg_comm::{Client, Layer};

/// A routing handle for a destination (a process, userdriver, or the
/// driver-space manager).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PortId(pub u32);

/// The driver-space manager port (routes to `Layer::Manager`).
pub const PORT_MANAGER: PortId = PortId(0);
/// The kernel executive port.
pub const PORT_KERNEL: PortId = PortId(1);

/// A fixed-size message carried over the IPC channel.
#[derive(Debug, Clone, Copy, Default)]
pub struct IpcMessage {
    pub from: PortId,
    pub to: PortId,
    pub opcode: u32,
    pub a0: u64,
    pub a1: u64,
    pub a2: u64,
}

/// An IPC endpoint owned by a userspace component. It sends/receives over a
/// `tg_comm` shared ring; the kernel routes between ports.
pub struct IpcEndpoint {
    port: PortId,
    client: Client,
}

impl IpcEndpoint {
    /// # Safety
    /// `tx_base`/`rx_base` must be shared rings mapped by the kernel.
    pub unsafe fn attach(port: PortId, tx_base: *mut u8, rx_base: *mut u8) -> Self {
        Self {
            port,
            client: Client::attach(Layer::Userspace, tx_base, rx_base, 1 /* default cap */),
        }
    }

    /// Send a request to a destination port. The destination port is encoded
    /// in the message for the kernel router (the `target` layer byte stays
    /// `Userspace`; intra-userspace routing is by port).
    pub fn send(&mut self, to: PortId, opcode: u32, a0: u64, a1: u64) -> bool {
        let id = self.client.request(opcode, a0, a1, to.0 as u64);
        id != 0
    }

    /// Poll the next inbound message, if any.
    pub fn recv(&self) -> Option<IpcMessage> {
        self.client.recv().map(|m| IpcMessage {
            from: PortId(m.seq),
            to: self.port,
            opcode: m.opcode,
            a0: m.a0,
            a1: m.a1,
            a2: m.a2,
        })
    }

    /// Yield to the kernel so it polls the ring.
    pub fn yield_now(&self) {
        uspace_comm::yield_to_kernel();
    }

    /// Convenience: a request that does not expect a reply.
    pub fn notify(&mut self, to: PortId, opcode: u32, a0: u64) {
        let _ = self.send(to, opcode, a0, 0);
    }
}

/// A very small in-memory message queue used to *simulate* the kernel routing
/// for host-side tests (on the real system the kernel routes over shared
/// memory). Kept here so the demo can be exercised without a running kernel.
#[derive(Default)]
pub struct Router {
    pub queues: Vec<Vec<IpcMessage>>,
}

impl Router {
    pub fn new(ports: usize) -> Self {
        Self {
            queues: (0..ports).map(|_| Vec::new()).collect(),
        }
    }

    pub fn deliver(&mut self, msg: IpcMessage) {
        if (msg.to.0 as usize) < self.queues.len() {
            self.queues[msg.to.0 as usize].push(msg);
        }
    }

    pub fn poll(&mut self, port: PortId) -> Option<IpcMessage> {
        if (port.0 as usize) < self.queues.len() {
            if self.queues[port.0 as usize].is_empty() {
                None
            } else {
                Some(self.queues[port.0 as usize].remove(0))
            }
        } else {
            None
        }
    }
}
