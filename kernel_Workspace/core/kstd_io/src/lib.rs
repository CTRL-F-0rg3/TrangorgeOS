#![no_std]

#[macro_use]
pub mod console;

pub mod traits;
pub mod buffer;
pub mod input;

pub use traits::{Write, Read};
pub use buffer::RingBuffer;
pub use console::{Serial, VgaConsole};
pub use input::{RawInput, CookedInput, raw_scancode, cooked_char, buffered_byte};