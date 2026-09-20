#![no_std]

use gfx_protocol::commands::{GfxCmd, GfxIpcMsg};
use gfx_protocol::surfaces::{SurfaceId, SurfaceConfig, PixelFormat};

pub struct GfxContext {
    server_endpoint: u64,
}

impl GfxContext {
    pub fn init() -> Self {
        Self { server_endpoint: 0x8000 }
    }

    pub fn create_surface(&self, width: u32, height: u32) -> Option<SurfaceId> {
        let config = SurfaceConfig {
            width,
            height,
            format: PixelFormat::XRGB8888,
            flags: gfx_protocol::surfaces::SURFACE_FLAG_VISIBLE,
        };

        let mut msg = GfxIpcMsg::default();
        msg.cmd = GfxCmd::CreateSurface as u32;
        msg.arg0 = &config as *const _ as u64;

        let reply = self.send_ipc(msg);
        if reply.status == 0 {
            Some(SurfaceId(reply.arg0 as u32))
        } else {
            None
        }
    }

    pub fn commit(&self, surface: SurfaceId) {
        let mut msg = GfxIpcMsg::default();
        msg.cmd = GfxCmd::Commit as u32;
        msg.arg0 = surface.0 as u64;
        self.send_ipc(msg);
    }

    fn send_ipc(&self, msg: GfxIpcMsg) -> GfxIpcMsg {
        // Stub: Wywołanie syscalla jądra (np. int 0x80 lub SVC) 
        // przekazujące wiadomość do gfx-server
        GfxIpcMsg::default()
    }
}