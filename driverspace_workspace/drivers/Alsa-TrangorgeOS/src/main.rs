//! Driver entry point: the `Alsa-TrangorgeOS` service process.
//!
//! `ds-manager` calls [`driver_main`] after loading the driver. The flow
//! mirrors `iommu-driver`:
//!
//! 1. build the driver on [`SyscallPlatform`], so hardware is reached only
//!    through resources the manager granted;
//! 2. wrap it in an `AudioService` and grant **only** the read capability -
//!    opening a stream or writing a mixer has to be granted explicitly, or the
//!    capability system would be decorative;
//! 3. enter the service loop and drive the card from IPC messages.
//!
//! No unrequested physical address is ever touched.

#![no_std]
#![no_main]

extern crate alsa_trangorgeos;
extern crate ds_fw_audio;
extern crate ds_log;
extern crate kapi_abi;
extern crate kapi_syscall;

use alsa_trangorgeos::{DRIVER_NAME, driver::AlsaDriver, host::SyscallPlatform};
use ds_fw_audio::{AudioService, MAX_PAYLOAD_LEN};
use ds_log::{ds_error, ds_info};
use kapi_abi::CapId;

/// The service instance, in static storage so the loop can hold it for good.
static mut RUNTIME: Option<Runtime> = None;
static mut REQUEST_BUF: [u8; MAX_PAYLOAD_LEN] = [0u8; MAX_PAYLOAD_LEN];
static mut REPLY_BUF: [u8; MAX_PAYLOAD_LEN] = [0u8; MAX_PAYLOAD_LEN];

/// Type alias kept for readability at the storage site.
type Runtime = AudioService<AlsaDriver<SyscallPlatform>>;

/// The entry point `ds-manager` calls after loading the driver.
#[unsafe(no_mangle)]
pub extern "C" fn driver_main() -> ! {
    ds_info!("{}: driver starting", DRIVER_NAME);

    let driver = AlsaDriver::new(SyscallPlatform::new());
    ds_info!(
        "{}: card {} ({}) with {} pcm endpoint(s)",
        DRIVER_NAME,
        driver.card().id_str(),
        driver.card().name_str(),
        driver.pcm_count()
    );

    let mut service = AudioService::new(driver);
    // Read-only at start-up: listing PCMs, mixers and jacks needs nothing
    // else. Playing audio is a decision for `ds-manager`, not for the driver.
    service.grant(CapId::ALSA_ENUMERATE);

    unsafe {
        RUNTIME = Some(service);
    }
    ds_info!("{}: serving", DRIVER_NAME);

    serve()
}

/// The service loop: take a request, dispatch it, send the reply.
fn serve() -> ! {
    loop {
        let Some(msg) = kapi_syscall::sys_try_recv() else {
            // Ring empty: do not spin hot, give the scheduler a turn.
            kapi_syscall::sys_yield();
            continue;
        };
        handle(msg);
    }
}

/// Handle one request and push the reply onto the uplink ring.
fn handle(msg: kapi_abi::DsMsg) {
    let reply = unsafe {
        let Some(runtime) = (*(&raw mut RUNTIME)).as_mut() else {
            return;
        };
        // The caller put the request payload at `msg.arg0`; copy it into a
        // local buffer so the framework needs no raw pointers at all.
        let request_buf = &mut *(&raw mut REQUEST_BUF);
        copy_in(msg, request_buf);
        let request = ds_fw_audio::Request::new(msg, request_buf);

        let reply_buf = &mut *(&raw mut REPLY_BUF);
        let reply = runtime.service.dispatch(&request, reply_buf);
        let len = reply.arg2 as usize;
        let len = len.min(reply_buf.len());

        let reply_msg = reply.into_message(&msg);
        kapi_syscall::sys_ipc_reply(reply_msg, &reply_buf[..len], len);
        reply
    };

    if !reply.is_ok() {
        ds_error!(
            "{}: request 0x{:04x} failed with status {}",
            DRIVER_NAME,
            msg.cmd,
            reply.status
        );
    }
}

/// Copy the request payload that `msg.arg0` points at into a local buffer.
///
/// # Safety
///
/// The caller guarantees that `msg.arg1` bytes are valid in the manager-side
/// buffer; this bounds-checks and never dereferences more than declared.
unsafe fn copy_in(msg: kapi_abi::DsMsg, scratch: &mut [u8]) {
    let len = (msg.arg1 as usize).min(scratch.len());
    if len == 0 || msg.arg0 == 0 {
        return;
    }
    let src = msg.arg0 as *const u8;
    scratch[..len].copy_from_slice(unsafe { core::slice::from_raw_parts(src, len) });
}

/// Halt: on failure we must not return silently, or `ds-manager` would wait
/// for this driver forever.
fn halt() -> ! {
    loop {
        kapi_syscall::sys_yield();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ds_error!("{}: PANIC", DRIVER_NAME);
    halt()
}
