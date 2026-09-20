#![no_std]
#![no_main]

mod compositor;
mod surface_manager;
mod gpu_bridge;

use gfx_protocol::commands::{GfxCmd, GfxIpcMsg};
use ds_log::{ds_info, ds_error};
use kapi_abi::DsMsg;

#[no_mangle]
pub extern "C" fn gfx_server_main() {
    ds_info!("GFX", "Display server starting...");

    let mut server = compositor::DisplayServer::new(1920, 1080);
    let mut gpu = gpu_bridge::GpuBridge::new();

    if !gpu.init() {
        ds_error!("GFX", "Failed to initialize GPU bridge via vgpu");
        return;
    }

    ds_info!("GFX", "GPU bridge ready, entering event loop");

    loop {
        let msg = wait_for_ipc();

        match GfxCmd::from_u32(msg.cmd) {
            Some(GfxCmd::CreateSurface) => {
                let id = server.create_surface(msg.arg0 as u32, msg.arg1 as u32);
                reply(msg.cmd, 0, id as u64);
            }
            Some(GfxCmd::Commit) => {
                server.commit(msg.arg0 as u32);
                server.composite();
                gpu.present(server.framebuffer_ptr(), server.fb_size());
                reply(msg.cmd, 0, 0);
            }
            Some(GfxCmd::SetPosition) => {
                server.set_position(msg.arg0 as u32, msg.arg1 as i32, msg.arg2 as i32);
                reply(msg.cmd, 0, 0);
            }
            _ => reply(msg.cmd, -1, 0),
        }
    }
}

fn wait_for_ipc() -> GfxIpcMsg {
    // Block on kernel IPC ring (k2d) for messages from user-space apps
    GfxIpcMsg::default()
}

fn reply(cmd: u32, status: i32, data: u64) {
    // Push reply to kernel IPC ring (d2k)
}