#![no_std]

pub mod levels;
pub mod formatter;
pub mod transport;
#[macro_use]
pub mod macros;

use kapi_abi::primitives::Handle;
use core::sync::atomic::{AtomicU32, Ordering};

static MANAGER_ENDPOINT_RAW: AtomicU32 = AtomicU32::new(0);

pub struct EndpointRef;

impl EndpointRef {
    pub const fn new() -> Self { Self }
    
    pub fn is_valid(&self) -> bool {
        MANAGER_ENDPOINT_RAW.load(Ordering::Acquire) != 0
    }

    /// Handle endpointu Managera (0 = niezarejestrowany).
    pub fn handle(&self) -> u32 {
        MANAGER_ENDPOINT_RAW.load(Ordering::Acquire)
    }

    pub fn set(&self, handle: Handle) {
        MANAGER_ENDPOINT_RAW.store(handle.0, Ordering::Release);
    }
}

pub static MANAGER_ENDPOINT: EndpointRef = EndpointRef::new();