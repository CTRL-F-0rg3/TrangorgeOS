#![no_std]
#![no_main]

use trangorgelibc as tr;

static mut LINE: [u8; 128] = [0; 128];
static mut LEN: usize = 0;

fn run(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    if cmd == "help" {
        tr::log("shell: help | echo | ver | pid | run <path>");
    } else if cmd == "ver" {
        tr::log("TrangorgeOS 0.4 (trójpodział dzielny)");
    } else if cmd == "pid" {
        tr::put_u32(tr::getpid());
    } else if cmd == "echo" {
        tr::log(cmd);
    } else if let Some(p) = cmd.strip_prefix("run ") {
        match tr::spawn(p) {
            -1 => tr::log("shell: spawn failed"),
            pid => {
                tr::log("shell: spawned pid");
                tr::put_u32(pid as u32);
            }
        }
    } else {
        tr::log("shell: unknown command");
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tr::log("shell: ready $");

    loop {
        if let Some(c) = tr::key() {
            unsafe {
                match c {
                    b'\n' => {
                        let s = core::str::from_utf8(&LINE[..LEN]).unwrap_or("");
                        run(s);
                        LINE = [0; 128];
                        LEN = 0;
                        tr::log("$");
                    }
                    8 => { if LEN > 0 { LEN -= 1; } }
                    _ => {
                        if LEN < 127 {
                            LINE[LEN] = c;
                            LEN += 1;
                        }
                    }
                }
            }
        }

        tr::yield_cpu();
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }