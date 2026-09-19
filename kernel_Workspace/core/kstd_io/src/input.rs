extern "C" {
    fn k_input_key() -> i32;
    fn k_input_keycode() -> u32;
    fn kbuf_pop() -> i32;
}

pub fn raw_scancode() -> Option<u32> {
    let code = unsafe { k_input_keycode() };
    if code == 0 { None } else { Some(code) }
}

pub fn cooked_char() -> Option<u8> {
    let key = unsafe { k_input_key() };
    if key < 0 { None } else { Some(key as u8) }
}

pub fn buffered_byte() -> Option<u8> {
    let b = unsafe { kbuf_pop() };
    if b < 0 { None } else { Some(b as u8) }
}

pub struct RawInput;
impl crate::traits::Read for RawInput {
    fn read_byte(&mut self) -> Option<u8> {
        raw_scancode().map(|c| c as u8)
    }
}

pub struct CookedInput;
impl crate::traits::Read for CookedInput {
    fn read_byte(&mut self) -> Option<u8> {
        cooked_char()
    }
}