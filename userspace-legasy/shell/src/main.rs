#![no_std]
#![no_main]
// W tej malej, jednowatkowej aplikacji odwołania do `static mut` są bezpieczne;
// w edycji 2024 lint `static_mut_refs` jest domyślnie błędem.
#![allow(static_mut_refs)]

use trangorgelib as tr;

static mut LINE: [u8; 128] = [0; 128];
static mut LEN: usize = 0;

/// Zamienia bufor zakonczony NUL na `&str` (nazwy z `readdir`).
fn cstr(buf: &[u8]) -> &str {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    core::str::from_utf8(&buf[..end]).unwrap_or("")
}

#[unsafe(no_mangle)]
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

fn run(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    if cmd == "help" {
        tr::print("shell: help | ver | pid | echo | cat <path> | run <path> | wait\n");
    } else if cmd == "ver" {
        tr::print("TrangorgeOS 0.5 (trojpodzial dzielny)\n");
    } else if cmd == "pid" {
        tr::put_u32(tr::getpid());
        tr::print("\n");
    } else if let Some(p) = cmd.strip_prefix("echo ") {
        tr::print(p);
        tr::print("\n");
    } else if let Some(p) = cmd.strip_prefix("cat ") {
        let fd = tr::open(p);

        if fd < 0 {
            tr::print("cat: brak pliku\n");
        } else {
            let mut buf = [0u8; 256];

            loop {
                let n = tr::read(fd, &mut buf);

                if n <= 0 {
                    break;
                }

                tr::write(1, &buf[..n as usize]);
            }

            tr::close(fd);
        }
    } else if let Some(p) = cmd.strip_prefix("run ") {
        match tr::spawn(p) {
            -1 => tr::print("shell: spawn failed\n"),
            pid => {
                tr::print("shell: pid ");
                tr::put_u32(pid as u32);
                tr::print("\n");
            }
        }
    } else if cmd == "wait" {
        match tr::wait() {
            Some((pid, code)) => {
                tr::print("reaped pid ");
                tr::put_u32(pid);
                tr::print(" code ");
                tr::put_u32(code as u32);
                tr::print("\n");
            }
            None => tr::print("wait: brak\n"),
        }
    } else if cmd == "ls" {
        let mut idx = 0u64;
        let mut name = [0u8; 128];

        loop {
            match tr::readdir(idx, &mut name) {
                Some(t) => {
                    let s = cstr(&name);

                    if t == 2 {
                        tr::print("d ");
                    } else {
                        tr::print("- ");
                    }

                    tr::print(s);
                    tr::print("\n");
                }
                None => break,
            }

            idx += 1;
        }
    } else if let Some(p) = cmd.strip_prefix("cl ") {
        match tr::runcl(p) {
            0 => tr::print("cl: ok\n"),
            e => {
                tr::print("cl: error ");
                tr::put_u32((-e) as u32);
                tr::print("\n");
            }
        }
    } else {
        tr::print("shell: nieznana komenda\n");
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }