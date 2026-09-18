//! Message dispatcher for ds-manager. Routes incoming IPC messages to handlers.

use ds_log::ds_error;
use kapi_abi::opcodes::Opcode;
use kapi_abi::errors::DsError;
use kapi_abi::primitives::Handle;

pub type HandlerFn = fn(sender: Handle, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError>;

pub struct MessageDispatcher {
    handlers: [Option<HandlerFn>; 64], 
}

impl MessageDispatcher {
    pub const fn new() -> Self {
        Self {
            handlers: [None; 64],
        }
    }

    pub fn register(&mut self, opcode: Opcode, handler: HandlerFn) -> Result<(), DsError> {
        let idx = (opcode as u32 & 0x3F) as usize;
        if self.handlers[idx].is_some() {
            return Err(DsError::DeviceBusy);
        }
        self.handlers[idx] = Some(handler);
        Ok(())
    }

    pub fn dispatch(&self, sender: Handle, opcode_raw: u32, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError> {
        let opcode = Opcode::from_u32(opcode_raw).ok_or(DsError::InvalidMessage)?;
        let idx = (opcode_raw & 0x3F) as usize;

        if let Some(handler) = self.handlers[idx] {
            handler(sender, payload_ptr, payload_len)
        } else {
            ds_error!("Unhandled opcode: {:?}", opcode);
            Err(DsError::InvalidMessage)
        }
    }
}