#![no_std]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use kapi_abi::primitives::Handle;

pub const MAX_PORTS: usize = 256;
pub const PORT_NAME_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PortState {
    Closed = 0,
    Listening = 1,
    Connected = 2,
    Error = 3,
}

#[repr(C)]
pub struct Port {
    pub handle: Handle,
    pub state: AtomicU32,
    pub name: [u8; PORT_NAME_LEN],
    pub owner: Handle,
    pub max_queue: u32,
    pub pending: AtomicU32,
    pub peer: Handle,
    pub flags: u32,
}

impl Port {
    pub const fn new(handle: Handle, owner: Handle) -> Self {
        Self {
            handle,
            state: AtomicU32::new(PortState::Closed as u32),
            name: [0; PORT_NAME_LEN],
            owner,
            max_queue: 64,
            pending: AtomicU32::new(0),
            peer: Handle::INVALID,
            flags: 0,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = [0; PORT_NAME_LEN];
        let bytes = name.as_bytes();
        let len = if bytes.len() < PORT_NAME_LEN { bytes.len() } else { PORT_NAME_LEN - 1 };
        self.name[..len].copy_from_slice(&bytes[..len]);
    }

    pub fn state(&self) -> PortState {
        match self.state.load(Ordering::Acquire) {
            0 => PortState::Closed,
            1 => PortState::Listening,
            2 => PortState::Connected,
            3 => PortState::Error,
            _ => PortState::Error,
        }
    }

    pub fn set_state(&self, state: PortState) {
        self.state.store(state as u32, Ordering::Release);
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state(), PortState::Listening | PortState::Connected)
    }

    pub fn try_acquire(&self) -> bool {
        self.state
            .compare_exchange(
                PortState::Listening as u32,
                PortState::Connected as u32,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}

pub struct PortRegistry {
    ports: [Port; MAX_PORTS],
    next_handle: AtomicU32,
}

impl PortRegistry {
    pub const fn new() -> Self {
        const EMPTY_PORT: Port = Port::new(Handle::INVALID, Handle::INVALID);
        Self {
            ports: [EMPTY_PORT; MAX_PORTS],
            next_handle: AtomicU32::new(2),
        }
    }

    pub fn alloc_port(&self, owner: Handle) -> Option<Handle> {
        let handle_val = self.next_handle.fetch_add(1, Ordering::Relaxed);
        if handle_val as usize >= MAX_PORTS {
            return None;
        }
        let handle = Handle(handle_val);
        self.ports[handle_val as usize] = Port::new(handle, owner);
        Some(handle)
    }

    pub fn get(&self, handle: Handle) -> Option<&Port> {
        let idx = handle.raw() as usize;
        if idx >= MAX_PORTS { return None; }
        let port = &self.ports[idx];
        if port.handle != handle { return None; }
        Some(port)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut Port> {
        let idx = handle.raw() as usize;
        if idx >= MAX_PORTS { return None; }
        let port = &mut self.ports[idx];
        if port.handle != handle { return None; }
        Some(port)
    }

    pub fn close(&mut self, handle: Handle) {
        if let Some(port) = self.get_mut(handle) {
            port.set_state(PortState::Closed);
            port.peer = Handle::INVALID;
        }
    }
}