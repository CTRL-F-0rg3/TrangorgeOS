//! `aplay` - play a raw audio file through the ALSA-compatible stack.
//!
//! A port of `alsa-utils/aplay`. Option spellings keep their C meanings, so
//! existing command lines carry over:
//!
//! ```text
//! aplay -D hw:0,0 -d 0 -f S16_LE -r 48000 -c 2 song.raw
//! aplay -l                 # list PCM endpoints
//! aplay --dump-hw-params   # print the parameters that would be negotiated
//! ```
//!
//! All parsing and arithmetic live in `alsa_trangorgeos::tools`, where they
//! are unit-tested; this binary only wires them to the IPC client and the
//! console. Like every binary in this workspace it is `no_std` / `no_main`:
//! arguments arrive over the OS ABI and output goes through the driver-space
//! log transport, not through libc.

#![no_std]
#![no_main]

extern crate alsa_trangorgeos;
extern crate ds_fw_audio;
extern crate ds_log;
extern crate kapi_abi;

use alsa_trangorgeos::tools::{AplayAction, parse_aplay, PlaybackRequest};
use ds_fw_audio::{AudioClient, CardId, PcmId};
use kapi_abi::Handle;

/// Maximum argv entries this tool accepts.
const MAX_ARGS: usize = 16;

/// Where the arguments land, published by the OS loader before `main`.
#[unsafe(no_mangle)]
pub static mut ARGC: u32 = 0;
#[unsafe(no_mangle)]
pub static mut ARGV: u64 = 0;

const EXIT_OK: i32 = 0;
const EXIT_FAILURE: i32 = 1;

/// The entry point the OS loader calls.
///
/// It is `no_main`, so there is no C runtime and no `main` symbol to link
/// against; the return value is the process exit status, as the kernel ABI
/// expects for a driver-space binary.
#[unsafe(no_mangle)]
pub extern "C" fn alsa_main() -> i32 {
    let (argc, argv) = unsafe { (ARGC as usize, ARGV as *const *const u8) };

    // Copy the vector into fixed-size slots: the loader's buffer is not
    // guaranteed to outlive this call, and the parser takes a slice.
    let mut args: [&str; MAX_ARGS] = [""; MAX_ARGS];
    let mut count = 0usize;
    let mut index = 0usize;
    while index < argc && count < MAX_ARGS {
        // SAFETY: the loader published `argc` NUL-terminated entries.
        args[count] = unsafe { borrow_cstr(*argv.add(index)) };
        count += 1;
        index += 1;
    }

    let client = AudioClient::new(Handle::MANAGER);

    match parse_aplay(&args[..count]) {
        AplayAction::Help => {
            say("usage: aplay [-D name] [-d N] [-f fmt] [-r hz] [-c N] [-p N] FILE");
            say("       aplay -l | --dump-hw-params | -h");
            EXIT_OK
        }
        AplayAction::ListPcms => list(&client),
        AplayAction::DumpParams(request) => dump(request),
        AplayAction::Play(request) => play(&client, request),
        AplayAction::Invalid { option } => {
            // Recover the name for the message; the field is NUL-padded.
            let end = option.iter().position(|&b| b == 0).unwrap_or(0);
            let text = core::str::from_utf8(&option[..end]).unwrap_or("aplay");
            fail(text);
            EXIT_FAILURE
        }
    }
}

/// Print the PCM endpoints, as `aplay -l` does.
fn list(client: &AudioClient) -> i32 {
    let count = match client.pcm_count() {
        Ok(count) => count,
        Err(_) => return fail("cannot enumerate PCMs"),
    };
    say("List of PCMs");
    for index in 0..count {
        let Ok(info) = client.pcm_info(PcmId::new(index)) else {
            continue;
        };
        let mut line = heapless::String::<64>::new();
        let _ = core::fmt::Write::write_fmt(
            &mut line,
            format_args!(
                "  hw:{},{}: {} channels, {} Hz - {} Hz",
                info.card,
                info.device,
                info.channels.count_ones(),
                info.rate_min,
                info.rate_max
            ),
        );
        say(&line);
    }
    EXIT_OK
}

/// Print the parameters this invocation would negotiate.
fn dump(request: PlaybackRequest) -> i32 {
    say("HW Params of device \"default\":");
    let mut line = heapless::String::<96>::new();
    let format = request.format.as_str();
    let _ = core::fmt::Write::write_fmt(
        &mut line,
        format_args!(
            "  access: RW_INTERLEAVED\n  format: {format}\n  rate: {} Hz\n  channels: {}",
            request.rate, request.channels
        ),
    );
    say(&line);
    let mut timing = heapless::String::<64>::new();
    let _ = core::fmt::Write::write_fmt(
        &mut timing,
        format_args!(
            "  period size: {} bytes = {} us\n  buffer size: {} frames",
            request.period_bytes(),
            request.period_us(),
            request.effective_buffer_size()
        ),
    );
    say(&timing);
    EXIT_OK
}

/// Open, configure and run one stream.
fn play(client: &AudioClient, request: PlaybackRequest) -> i32 {
    let access = request.direction.access_bit() | 0x100; // plus interleaved
    let handle = match client.open(
        CardId::new(request.device.card),
        PcmId::new(request.device.device),
        request.direction.stream(),
        access,
    ) {
        Ok(handle) => handle,
        Err(_) => return fail("cannot open PCM"),
    };

    // Negotiate every mandatory field before preparing, exactly as the C tool
    // does; a field the hardware refuses aborts the open.
    use kapi_abi::payloads::audio::HwParamsQuery;
    let fields = [
        (HwParamsQuery::FIELD_ACCESS, access as u64),
        (HwParamsQuery::FIELD_FORMAT, request.format.bits() as u64),
        (HwParamsQuery::FIELD_CHANNELS, (1u64 << request.channels) - 1),
        (HwParamsQuery::FIELD_RATE, request.rate as u64),
        (HwParamsQuery::FIELD_PERIOD_SIZE, request.period_size as u64),
    ];
    for (field, value) in fields {
        if client.hw_params_set(handle, field, value).is_err() {
            let _ = client.close(handle);
            return fail("hardware rejected the requested parameters");
        }
    }

    if client.prepare(handle).is_err() || client.start(handle).is_err() {
        let _ = client.close(handle);
        return fail("cannot start the stream");
    }

    // Sample transfer needs a shared buffer granted by `ds-manager`. Until that
    // wiring lands, report what actually happened rather than claiming audio
    // moved: the driver answers a short write with an xrun, which surfaces
    // here as a failed transfer.
    let transfer = client.write(handle, &[0u8; 4], 1);
    let _ = client.drop_stream(handle, ds_fw_audio::DropMode::Drain);
    let _ = client.close(handle);

    match transfer {
        Ok(transfer) => {
            let mut line = heapless::String::<48>::new();
            let _ = core::fmt::Write::write_fmt(
                &mut line,
                format_args!("stream finished ({transfer:?})"),
            );
            say(&line);
            EXIT_OK
        }
        Err(_) => fail("transfer failed"),
    }
}

/// Emit one line through the driver-space log transport.
fn say(text: &str) {
    ds_log::ds_info!("aplay: {text}");
}

/// Report a failure and return the failure code.
fn fail(reason: &str) -> i32 {
    ds_log::ds_error!("aplay: {reason}");
    EXIT_FAILURE
}

/// Borrow a NUL-terminated string published by the loader.
///
/// # Safety
///
/// The pointer must come from the loader's `argv` and be NUL-terminated.
unsafe fn borrow_cstr<'a>(pointer: *const u8) -> &'a str {
    if pointer.is_null() {
        return "";
    }
    let mut length = 0usize;
    // SAFETY: the loader NUL-terminates every argument.
    while unsafe { *pointer.add(length) } != 0 {
        length += 1;
    }
    // SAFETY: `length` counts only non-NUL bytes within a valid allocation.
    let bytes = unsafe { core::slice::from_raw_parts(pointer, length) };
    core::str::from_utf8(bytes).unwrap_or("")
}

/// A `no_main` binary owns its panic behaviour; halting is the only safe
/// response, since unwinding across the driver-space boundary is not wired up.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    ds_log::ds_error!("aplay: PANIC");
    loop {}
}
