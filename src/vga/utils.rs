use core::fmt::{self, Write};

use crate::vga::{ColorCode, ColorCode8b, WRITER};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode8b,
}

impl ScreenChar {
    pub fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        let color_code_byte = color_code.into();
        ScreenChar {
            ascii_character,
            color_code: color_code_byte,
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}