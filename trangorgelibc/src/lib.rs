#![no_std]

pub const SYS_YIELD: u64 = 1;
pub const SYS_LOG: u64 = 2;
pub const SYS_EXIT: u64 = 0x1000;
pub const SYS_SPAWN: u64 = 0x1001;
pub const SYS_GETPID: u64 = 0x1002;
pub const SYS_IPC_SEND: u64 = 0x1010;
pub const SYS_IPC_RECV: u64 = 0x1011;
pub const SYS_KEY: u64 = 0x1040;

fn sc0(n: u64) -> u64 {
    let r: u64;
    unsafe { asm!("int 0x80", in("rax") n, lateout("rax") r); }
    r
}

fn sc1(n: u64, a0: u64) -> u64 {
    let r: u64;
    unsafe { asm!("int 0x80", in("rax") n, in("rdi") a0, lateout("rax") r); }
    r
}

fn sc3(n: u64, a0: u64, a1: u64, a2: u64) -> u64 {
    let r: u64;
    unsafe {
        asm!("int 0x80", in("rax") n, in("rdi") a0,
             in("rsi") a1, in("rdx") a2, lateout("rax") r);
    }
    r
}

pub fn log(s: &str) { sc1(SYS_LOG, s.as_ptr() as u64); }
pub fn yield_cpu() { sc0(SYS_YIELD); }
pub fn exit(code: i32) -> ! { sc1(SYS_EXIT, code as u64); loop {} }
pub fn getpid() -> u32 { sc0(SYS_GETPID) as u32 }
pub fn spawn(path: &str) -> i32 { sc1(SYS_SPAWN, path.as_ptr() as u64) as i32 }
pub fn key() -> Option<u8> {
    match sc0(SYS_KEY) { 0 => None, v => Some(v as u8) }
}

pub fn ipc_send(pid: u32, a0: u64, a1: u64) -> bool {
    sc3(SYS_IPC_SEND, pid as u64, a0, a1) != u64::MAX
}

pub struct Mail { pub from: u32, pub a0: u64, pub a1: u64 }

pub fn ipc_recv() -> Option<Mail> {
    let from: u64;
    let a0: u64;
    let a1: u64;
    unsafe {
        asm!("int 0x80", in("rax") SYS_IPC_RECV,
             lateout("rax") from, lateout("rdi") a0, lateout("rsi") a1);
    }
    if from == u64::MAX { None } else {
        Some(Mail { from: from as u32, a0, a1 })
    }
}

/* bump heap */
static mut HEAP: [u8; 256 * 1024] = [0; 256 * 1024];
static mut HEAP_POS: usize = 0;

pub fn malloc(n: usize) -> *mut u8 {
    unsafe {
        let p = HEAP_POS;
        HEAP_POS = (HEAP_POS + n + 15) & !15;
        HEAP.as_mut_ptr().add(p)
    }
}

/* liczby */
pub fn put_u32(v: u32) {
    let mut buf = [0u8; 10];
    let mut n = 0usize;
    let mut x = v;

    if x == 0 { buf[0] = b'0'; n = 1; }
    else {
        while x > 0 { buf[n] = b'0' + (x % 10) as u8; x /= 10; n += 1; }
        for i in 0..n / 2 { buf.swap(i, n - 1 - i); }
    }

    log(core::str::from_utf8(&buf[..n]).unwrap());
}