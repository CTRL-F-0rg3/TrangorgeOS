extern "C" {
    fn nim_u64_to_str(v: u64, base: u8, buf: *mut u8, cap: u32) -> u32;
    fn nim_parse_u64(s: *const u8, len: u32, base: u8, out: *mut u64) -> u8;
    fn nim_rb_push(b: u8) -> u8;
    fn nim_rb_pop() -> i32;
    fn nim_shell_register(name: *const u8, nlen: u32,
                          h: extern "C" fn(*mut u8, u32) -> i32) -> u8;
    fn nim_shell_run(line: *const u8, len: u32) -> i32;
    fn nim_banner(buf: *mut u8, cap: u32) -> u32;
}

pub type ShellHandler = extern "C" fn(*mut u8, u32) -> i32;

pub fn banner(buf: &mut [u8]) -> usize {
    unsafe { nim_banner(buf.as_mut_ptr(), buf.len() as u32) as usize }
}

pub fn shell_register(name: &str, h: ShellHandler) -> bool {
    unsafe { nim_shell_register(name.as_ptr(), name.len() as u32, h) == 1 }
}

pub fn shell_run(line: &str) -> i32 {
    unsafe { nim_shell_run(line.as_ptr(), line.len() as u32) }
}

pub fn key_push(c: u8) {
    unsafe { let _ = nim_rb_push(c); }
}

pub fn key_pop() -> Option<u8> {
    let v = unsafe { nim_rb_pop() };
    if v < 0 { None } else { Some(v as u8) }
}

pub fn parse_u64(s: &str, base: u8) -> Option<u64> {
    let mut v = 0u64;
    let ok = unsafe { nim_parse_u64(s.as_ptr(), s.len() as u32, base, &mut v) };
    if ok == 1 { Some(v) } else { None }
}
