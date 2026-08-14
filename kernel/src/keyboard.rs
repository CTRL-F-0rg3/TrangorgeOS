use core::sync::atomic::{AtomicBool, Ordering};

pub static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Tab,
    None,
}

crate::test_module!({
    match process_scancode(0x03) {
        KeyEvent::Char('2') => {}
        _ => return Err("scancode 0x03 without shift should produce '2'"),
    }

    match process_scancode(0x2A) {
        KeyEvent::None => {}
        _ => return Err("shift press should not itself produce a character"),
    }
    if !SHIFT_PRESSED.load(Ordering::Relaxed) {
        return Err("shift press did not set SHIFT_PRESSED");
    }

    match process_scancode(0x03) {
        KeyEvent::Char('@') => {}
        _ => return Err("scancode 0x03 with shift held should produce '@'"),
    }

    match process_scancode(0xAA) {
        KeyEvent::None => {}
        _ => return Err("shift release should not itself produce a character"),
    }
    if SHIFT_PRESSED.load(Ordering::Relaxed) {
        return Err("shift release did not clear SHIFT_PRESSED");
    }

    match process_scancode(0x83) {
        KeyEvent::None => {}
        _ => return Err("a key release event should not produce a character"),
    }

    match process_scancode(0x1C) {
        KeyEvent::Enter => {}
        _ => return Err("scancode 0x1C should produce Enter"),
    }

    Ok("scancode to key event translation verified, including shift state")
});

pub fn process_scancode(scancode: u8) -> KeyEvent {
    let released = scancode & 0x80 != 0;
    let code = scancode & 0x7F;

    match code {
        0x2A | 0x36 => {
            SHIFT_PRESSED.store(!released, Ordering::Relaxed);
            KeyEvent::None
        }
        _ if released => KeyEvent::None,
        0x1C => KeyEvent::Enter,
        0x0E => KeyEvent::Backspace,
        0x0F => KeyEvent::Tab,
        0x39 => KeyEvent::Char(' '),
        _ => match scancode_to_char(code) {
            Some(c) => KeyEvent::Char(c),
            None => KeyEvent::None,
        },
    }
}

fn scancode_to_char(code: u8) -> Option<char> {
    let shift = SHIFT_PRESSED.load(Ordering::Relaxed);
    let (lower, upper) = match code {
        0x02 => ('1', '!'),
        0x03 => ('2', '@'),
        0x04 => ('3', '#'),
        0x05 => ('4', '$'),
        0x06 => ('5', '%'),
        0x07 => ('6', '^'),
        0x08 => ('7', '&'),
        0x09 => ('8', '*'),
        0x0A => ('9', '('),
        0x0B => ('0', ')'),
        0x0C => ('-', '_'),
        0x0D => ('=', '+'),
        0x10 => ('q', 'Q'),
        0x11 => ('w', 'W'),
        0x12 => ('e', 'E'),
        0x13 => ('r', 'R'),
        0x14 => ('t', 'T'),
        0x15 => ('y', 'Y'),
        0x16 => ('u', 'U'),
        0x17 => ('i', 'I'),
        0x18 => ('o', 'O'),
        0x19 => ('p', 'P'),
        0x1A => ('[', '{'),
        0x1B => (']', '}'),
        0x1E => ('a', 'A'),
        0x1F => ('s', 'S'),
        0x20 => ('d', 'D'),
        0x21 => ('f', 'F'),
        0x22 => ('g', 'G'),
        0x23 => ('h', 'H'),
        0x24 => ('j', 'J'),
        0x25 => ('k', 'K'),
        0x26 => ('l', 'L'),
        0x27 => (';', ':'),
        0x28 => ('\'', '"'),
        0x29 => ('`', '~'),
        0x2B => ('\\', '|'),
        0x2C => ('z', 'Z'),
        0x2D => ('x', 'X'),
        0x2E => ('c', 'C'),
        0x2F => ('v', 'V'),
        0x30 => ('b', 'B'),
        0x31 => ('n', 'N'),
        0x32 => ('m', 'M'),
        0x33 => (',', '<'),
        0x34 => ('.', '>'),
        0x35 => ('/', '?'),
        _ => return None,
    };
    Some(if shift { upper } else { lower })
}
