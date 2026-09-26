//! `gfx-server` — the display server.
//!
//! # What it does
//!
//! It owns a [`compositor::DisplayServer`] and a [`gpu_bridge::GpuBridge`], and
//! runs one loop: take a request off the inbound ring, act on it, answer on the
//! outbound ring. Presenting is a commit: the compositor rasterises into a
//! staging frame and the bridge hands that frame to the scanout engine.
//!
//! # The transport
//!
//! Requests arrive as [`DsMsg`] on the `k2d` ring, which `kapi-syscall` owns.
//! Replies go back the same way, so the wire format is `kapi-abi`'s and not
//! `GfxIpcMsg`'s — the latter is the *user-space* client's view of the same
//! conversation, and duplicating it here is what let the two drift apart.

#![no_std]
#![no_main]

mod compositor;
mod gpu_bridge;

use ds_log::{ds_error, ds_info};
use gfx_protocol::commands::GfxCmd;
use kapi_abi::DsMsg;
use kapi_syscall::{sys_ipc_reply, sys_try_recv, sys_yield};

/// `0` on the wire: the request was carried out.
const OK: i32 = 0;
/// `-1`: the command is not one this server answers.
const UNKNOWN_COMMAND: i32 = -1;

#[unsafe(no_mangle)]
pub extern "C" fn gfx_server_main() {
    ds_info!("GFX: display server starting");

    let mut gpu = gpu_bridge::GpuBridge::new();
    if !gpu.init() {
        ds_error!("GFX: GPU bridge failed to initialise; no display");
        return;
    }

    // The output's real geometry comes from the driver, not from a hard-coded
    // 1920x1080: asking the bridge is the only way to learn what the scanout
    // engine is actually driving.
    let (width, height) = gpu.mode();
    let mut server = match compositor::DisplayServer::new(width, height) {
        Ok(server) => server,
        Err(error) => {
            ds_error!("GFX: compositor allocation failed: {:?}", error);
            return;
        }
    };

    ds_info!("GFX: server ready at {}x{}, entering event loop", width, height);

    loop {
        // A non-blocking receive with a yield between attempts, rather than a
        // blocking pop: the ring is shared with the kernel, and spinning on it
        // would starve the very thing that fills it.
        let Some(request) = next_request() else {
            continue;
        };

        dispatch(&mut server, &gpu, &request);
    }
}

/// The next request, or `None` after yielding.
fn next_request() -> Option<DsMsg> {
    match sys_try_recv() {
        Some(request) => Some(request),
        None => {
            sys_yield();
            None
        }
    }
}

/// Act on one request and answer it.
fn dispatch(server: &mut compositor::DisplayServer, gpu: &gpu_bridge::GpuBridge, msg: &DsMsg) {
    let command = GfxCmd::from_u32(msg.cmd as u32);

    let (status, data) = match command {
        Some(GfxCmd::CreateSurface) => {
            let id = server.create_surface(msg.arg0 as u32, msg.arg1 as u32);
            // A refused creation answers 0 id with a non-zero status, so a
            // client can tell "you got handle 0" from "that failed" even
            // though zero is not a valid handle.
            if id == 0 {
                (-1, 0)
            } else {
                (OK, id as u64)
            }
        }
        Some(GfxCmd::DestroySurface) => {
            if server.destroy_surface(msg.arg0 as u32) {
                (OK, 0)
            } else {
                (-1, 0)
            }
        }
        Some(GfxCmd::SetPosition) => {
            server.set_position(msg.arg0 as u32, msg.arg1 as i32, msg.arg2 as i32);
            (OK, 0)
        }
        Some(GfxCmd::SetZOrder) => {
            server.set_z_order(msg.arg0 as u32, msg.arg1 as i32);
            (OK, 0)
        }
        Some(GfxCmd::Commit) => {
            server.commit(msg.arg0 as u32);
            server.composite();
            gpu.present(server.framebuffer_ptr(), server.fb_size());
            (OK, 0)
        }
        _ => (UNKNOWN_COMMAND, 0),
    };

    reply(msg, status, data);
}

/// Answer a request, echoing its identity back.
///
/// The reply echoes the request's `id` so a client with several requests in
/// flight can match the answer to the right one.
fn reply(request: &DsMsg, status: i32, data: u64) {
    let mut response = *request;
    response.status = status;
    response.arg0 = data;
    // No reply payload: every answer this server sends fits in the header, and
    // an empty slice is how `sys_ipc_reply` is told so.
    sys_ipc_reply(response, &[], 0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // The display server is the thing every other process draws into, so a
    // panic here is not recoverable by unwinding anywhere: report it and stop
    // making the situation worse.
    ds_error!("GFX: display server panicked");
    loop {
        sys_yield();
    }
}