use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[cfg(target_arch = "x86_64")]
mod x86_impl {
    use super::Color;
    use core::fmt;
    use lazy_static::lazy_static;
    use spin::Mutex;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    struct ColorCode(u8);

    impl ColorCode {
        fn new(foreground: Color, background: Color) -> ColorCode {
            ColorCode((background as u8) << 4 | (foreground as u8))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    struct ScreenChar {
        ascii_character: u8,
        color_code: ColorCode,
    }

    const BUFFER_HEIGHT: usize = 25;
    const BUFFER_WIDTH: usize = 80;

    #[repr(transparent)]
    struct Buffer {
        chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    }

    pub struct Writer {
        column_position: usize,
        color_code: ColorCode,
        buffer: &'static mut Buffer,
    }

    impl Writer {
        pub fn write_byte(&mut self, byte: u8) {
            match byte {
                b'\n' => self.new_line(),
                byte => {
                    if self.column_position >= BUFFER_WIDTH {
                        self.new_line();
                    }
                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position;
                    let color_code = self.color_code;
                    self.buffer.chars[row][col] = ScreenChar {
                        ascii_character: byte,
                        color_code,
                    };
                    self.column_position += 1;
                }
            }
        }

        fn new_line(&mut self) {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col];
                    self.buffer.chars[row - 1][col] = character;
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.column_position = 0;
        }

        fn clear_row(&mut self, row: usize) {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = blank;
            }
        }

        pub fn write_string(&mut self, s: &str) {
            for byte in s.bytes() {
                match byte {
                    0x20..=0x7e | b'\n' => self.write_byte(byte),
                    _ => self.write_byte(0xfe),
                }
            }
        }

        pub fn set_color(&mut self, foreground: Color) {
            self.color_code = ColorCode::new(foreground, Color::Black);
        }

        pub fn set_column(&mut self, col: usize) {
            self.column_position = col.min(BUFFER_WIDTH);
        }

        pub fn clear_to_end(&mut self) {
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            let color_code = self.color_code;
            for c in col..BUFFER_WIDTH {
                self.buffer.chars[row][c] = ScreenChar {
                    ascii_character: b' ',
                    color_code,
                };
            }
        }

        pub fn write_byte_colored(&mut self, byte: u8, fg: Color) {
            let prev = self.color_code;
            self.set_color(fg);
            self.write_byte(byte);
            self.color_code = prev;
        }

        pub fn column(&self) -> usize {
            self.column_position
        }

        pub fn clear_screen(&mut self) {
            for row in 0..BUFFER_HEIGHT {
                self.clear_row(row);
            }
            self.column_position = 0;
        }
    }

    impl fmt::Write for Writer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.write_string(s);
            Ok(())
        }
    }

    const BLANK: ScreenChar = ScreenChar {
        ascii_character: b' ',
        color_code: ColorCode(0x07),
    };

    static mut TEXT_BUFFER: Buffer = Buffer {
        chars: [[BLANK; BUFFER_WIDTH]; BUFFER_HEIGHT],
    };

    lazy_static! {
        pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut TEXT_BUFFER },
        });
    }

    pub fn text_cell(row: usize, col: usize) -> (u8, u8) {
        unsafe {
            let cell = TEXT_BUFFER.chars[row][col];
            (cell.ascii_character, cell.color_code.0)
        }
    }

    pub(super) fn write(args: fmt::Arguments) {
        use core::fmt::Write as _;
        WRITER.lock().write_fmt(args).unwrap();
    }

    pub(super) fn write_colored(color: Color, args: fmt::Arguments) {
        use core::fmt::Write as _;
        let mut writer = WRITER.lock();
        let previous = writer.color_code;
        writer.set_color(color);
        writer.write_fmt(args).unwrap();
        writer.color_code = previous;
    }

    pub(super) fn self_test() -> crate::testing::TestResult {
        let s = "roundtrip";
        let expected_color = ColorCode::new(Color::Yellow, Color::Black);
        {
            let mut w = WRITER.lock();
            w.column_position = 0;
            w.set_color(Color::Yellow);
            w.write_string(s);
        }
        for (i, expected_byte) in s.bytes().enumerate() {
            let cell = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][i];
            if cell.ascii_character != expected_byte {
                return Err("VGA buffer ascii readback mismatch");
            }
            if cell.color_code != expected_color {
                return Err("VGA buffer color readback mismatch");
            }
        }
        Ok("ascii + color roundtrip")
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86_impl::{WRITER, Writer, text_cell};
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    #[cfg(target_arch = "x86_64")]
    x86_impl::write(args);
    crate::serial::print_args(args);
}

#[doc(hidden)]
pub fn _print_colored(_color: Color, args: fmt::Arguments) {
    #[cfg(target_arch = "x86_64")]
    x86_impl::write_colored(_color, args);
    crate::serial::print_args(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) => (
        $crate::vga_buffer::_print_colored($color, format_args!($($arg)*))
    );
}

#[cfg(target_arch = "x86_64")]
crate::test_module!({ x86_impl::self_test() });

#[cfg(target_arch = "riscv64")]
crate::test_module!({
    Ok("serial console (RISC-V)")
});
