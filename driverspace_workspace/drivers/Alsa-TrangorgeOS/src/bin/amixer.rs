//! `amixer` - inspect and change mixer controls.
//!
//! A port of `alsa-utils/amixer`. The subcommands keep their C spellings:
//!
//! ```text
//! amixer                      # list every simple element
//! amixer get Master           # read one element
//! amixer set Master 80%       # set a level
//! amixer set Master unmute    # or `mute`
//! ```
//!
//! As with `aplay`, the parsing lives in `alsa_trangorgeos::tools` where it is
//! unit-tested, and this binary only wires it to the IPC client.

#![no_std]
#![no_main]

extern crate alsa_trangorgeos;
extern crate ds_fw_audio;
extern crate ds_log;
extern crate kapi_abi;

use alsa_trangorgeos::tools::{AmixerAction, parse_amixer};
use ds_fw_audio::{AudioClient, MixerId};
use kapi_abi::Handle;

/// Maximum argv entries this tool accepts.
const MAX_ARGS: usize = 12;

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
/// against; the return value is the process exit status.
#[unsafe(no_mangle)]
pub extern "C" fn alsa_main() -> i32 {
    let (argc, argv) = unsafe { (ARGC as usize, ARGV as *const *const u8) };

    // Copy the argument vector into fixed-size slots: the loader's buffer is
    // not guaranteed to outlive this call, and the parser takes a slice.
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

    match parse_amixer(&args[..count]) {
        AmixerAction::Help => {
            say("usage: amixer [get|set] [element] [value]");
            EXIT_OK
        }
        AmixerAction::List => list(&client),
        AmixerAction::Get { index: element } => read_one(&client, element),
        AmixerAction::Set { index: element, percent } => write_one(&client, element, percent),
        AmixerAction::Invalid { message } => {
            // Recover the text for the log; the field is NUL-padded.
            let end = message.iter().position(|&b| b == 0).unwrap_or(0);
            let text = core::str::from_utf8(&message[..end]).unwrap_or("amixer");
            say(text);
            EXIT_FAILURE
        }
    }
}

/// Print every mixer element the card exposes.
fn list(client: &AudioClient) -> i32 {
    // Walk a bounded range: the framework reports the count, and a card with
    // more elements than the bound is truncated rather than unbounded.
    for index in 0..32u32 {
        let Ok(selem) = client.mixer_info(MixerId::new(index)) else {
            // The first missing element ends the list.
            break;
        };
        let Ok(value) = client.mixer_read(MixerId::new(index)) else {
            continue;
        };
        let mut line = heapless::String::<96>::new();
        let name = selem.name_str();
        let level = value.min().unwrap_or(0);
        let _ = core::fmt::Write::write_fmt(
            &mut line,
            format_args!("Simple mixer control '{name}',0\n  Capabilities: pvolume pswitch\n  Playback channels: {level}"),
        );
        say(&line);
    }
    EXIT_OK
}

/// Print the current value of one element.
fn read_one(client: &AudioClient, index: u32) -> i32 {
    let Ok(selem) = client.mixer_info(MixerId::new(index)) else {
        say("amixer: no such control");
        return EXIT_FAILURE;
    };
    let Ok(value) = client.mixer_read(MixerId::new(index)) else {
        say("amixer: cannot read control");
        return EXIT_FAILURE;
    };
    let mut line = heapless::String::<64>::new();
    let name = selem.name_str();
    let level = value.min().unwrap_or(0);
    let _ = core::fmt::Write::write_fmt(&mut line, format_args!("{name}: {level}"));
    say(&line);
    EXIT_OK
}

/// Set one element to `percent` of full scale.
fn write_one(client: &AudioClient, index: u32, percent: i32) -> i32 {
    let Ok(mut value) = client.mixer_read(MixerId::new(index)) else {
        say("amixer: no such control");
        return EXIT_FAILURE;
    };
    // A percent of 0..=100 maps onto the 0..=100 element range this card uses.
    if !value.set(0, percent) {
        say("amixer: control has no channels");
        return EXIT_FAILURE;
    }
    match client.mixer_write(&value) {
        Ok(stored) => {
            let mut line = heapless::String::<64>::new();
            let level = stored.min().unwrap_or(0);
            let _ = core::fmt::Write::write_fmt(&mut line, format_args!("set to {level}"));
            say(&line);
            EXIT_OK
        }
        Err(_) => {
            say("amixer: write refused");
            EXIT_FAILURE
        }
    }
}

/// Emit one line through the driver-space log transport.
fn say(text: &str) {
    ds_log::ds_info!("amixer: {text}");
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
    ds_log::ds_error!("amixer: PANIC");
    loop {}
}
