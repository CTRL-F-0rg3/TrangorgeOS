#![no_std]
#![no_main]

use trangorgelibc as tr;

fn load_autostart() -> bool {
    let fd = tr::open("/sys/autostart.cfg");

    if fd < 0 {
        return false;
    }

    let mut buf = [0u8; 1024];
    let n = tr::read(fd, &mut buf);

    tr::close(fd);

    if n <= 0 {
        return false;
    }

    let mut start = 0usize;
    let mut spawned = 0;

    for i in 0..=n as usize {
        if i == n as usize || buf[i] == b'\n' {
            if i > start && buf[start] == b'/' {
                let mut j = i;

                while j > start && (buf[j - 1] == b'\r' || buf[j - 1] == b' ') {
                    j -= 1;
                }

                let s = core::str::from_utf8(&buf[start..j]).unwrap_or("");

                if !s.is_empty() {
                    tr::spawn(s);
                    spawned += 1;
                }
            }

            start = i + 1;
        }
    }

    spawned > 0
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tr::log("tr-init: session manager + loader");

    if !load_autostart() {
        tr::log("tr-init: brak autostart.cfg, fallback");
        tr::spawn("/bin/shell.elf");
        tr::spawn("/bin/terminal.elf");
    }

    loop {
        if let Some(m) = tr::ipc_recv() {
            tr::log("tr-init: ipc from");
            tr::put_u32(m.from);
        }

        /* reap dzieci, żeby nie zbierać zombie */
        let _ = tr::wait();

        tr::yield_cpu();
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! {
    loop {}
}