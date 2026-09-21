use tg_ipc_abi::wire::{MsgHeader, MsgKind, IpcError};
use tg_ipc_abi::opcodes::SysOpcode;
use tg_cap_table::{Handle, HandleTable};
use tg_cap_core::Rights;

pub trait MessageHandler {
    fn handle_data(&mut self, hdr: &MsgHeader, payload: &[u8]) -> Result<(), IpcError>;
    fn handle_request(&mut self, hdr: &MsgHeader, payload: &[u8]) -> Result<(), IpcError>;
    fn handle_notification(&mut self, hdr: &MsgHeader, payload: &[u8]) -> Result<(), IpcError>;
}

pub struct Dispatcher<H: MessageHandler> {
    handler: H,
}

impl<H: MessageHandler> Dispatcher<H> {
    pub fn new(handler: H) -> Self {
        Self { handler }
    }

    pub fn dispatch(
        &mut self,
        hdr: &MsgHeader,
        payload: &[u8],
        table: &HandleTable,
    ) -> Result<(), IpcError> {
        let target = Handle(hdr.target_handle);
        if target == Handle::INVALID {
            return Err(IpcError::InvalidHandle);
        }

        if !table.check_rights(target, Rights::RECV) {
            return Err(IpcError::PermissionDenied);
        }

        match hdr.kind {
            MsgKind::Data => self.handler.handle_data(hdr, payload),
            MsgKind::Request | MsgKind::Reply => self.handler.handle_request(hdr, payload),
            MsgKind::Notification => self.handler.handle_notification(hdr, payload),
            MsgKind::CapTransfer => self.handle_cap_transfer(hdr, payload, table),
            MsgKind::Control => Ok(()),
        }
    }

    fn handle_cap_transfer(
        &self,
        _hdr: &MsgHeader,
        _payload: &[u8],
        _table: &HandleTable,
    ) -> Result<(), IpcError> {
        Ok(())
    }
}