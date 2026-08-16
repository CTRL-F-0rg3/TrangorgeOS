const KEY_NORMAL: [u8; 30] = [
    0, 0, 0, 0,
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k',
    b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z',
];

const KEY_SHIFT: [u8; 30] = [
    0, 0, 0, 0,
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K',
    b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z',
];

const NUM_NORMAL: [u8; 10] = *b"1234567890";
const NUM_SHIFT: [u8; 10] = *b"!@#$%^&*()";

pub fn key_to_ascii(key: u8, shift: bool) -> Option<u8> {
    match key {
        0x04..=0x1D => {
            let t = if shift { KEY_SHIFT } else { KEY_NORMAL };
            Some(t[key as usize])
        }
        0x1E..=0x26 => {
            let t = if shift { NUM_SHIFT } else { NUM_NORMAL };
            Some(t[(key - 0x1E) as usize])
        }
        0x27 => {
            let t = if shift { NUM_SHIFT } else { NUM_NORMAL };
            Some(t[9])
        }
        0x28 => Some(b'\n'),
        0x2A => Some(8),
        0x2B => Some(b'\t'),
        0x2C => Some(b' '),
        _ => None,
    }
}

static mut INBUF: [u8; 64] = [0; 64];
static mut IN_HEAD: usize = 0;
static mut IN_TAIL: usize = 0;

pub fn push_char(c: u8) {
    unsafe {
        let next = (IN_HEAD + 1) % 64;

        if next != IN_TAIL {
            INBUF[IN_HEAD] = c;
            IN_HEAD = next;
        }
    }
}

pub fn take_char() -> Option<u8> {
    unsafe {
        if IN_HEAD == IN_TAIL {
            return None;
        }

        let c = INBUF[IN_TAIL];
        IN_TAIL = (IN_TAIL + 1) % 64;
        Some(c)
    }
}