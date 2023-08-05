use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use crate::*;
use Color4b::*;

// CONSTS

pub const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
        ColorCode::new(Black, Color3b::LightGray, false),
        unsafe { &mut *(VGA_BUFFER as *mut Buffer) }
    ));
}

// MACROS

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
macro_rules! WRITER {
    () => {
        $crate::vga_buffer::WRITER.lock()
    };
}

// ENUMS

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color3b {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color4b {
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

// STRUCTS

#[derive(Debug, Clone, Copy)]
pub struct ColorCode {
    foreground_color: Color4b,
    background_color: Color3b,
    blink: bool,
}

impl ColorCode {
    pub fn new(foreground_color: Color4b, background_color: Color3b, blink: bool) -> Self {
        ColorCode {
            foreground_color,
            background_color,
            blink,
        }
    }

    fn change_foreground_color(&mut self, foreground_color: Color4b) {
        self.foreground_color = foreground_color;
    }

    fn change_background_color(&mut self, background_color: Color3b) {
        self.background_color = background_color;
    }

    fn change_blink(&mut self, blink: bool) {
        self.blink = blink;
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct ColorCode8b(u8);

impl From<ColorCode> for ColorCode8b {
    fn from(color_code: ColorCode) -> Self {
        let blink_bit = (color_code.blink as u8) << 7;
        let background_bits = (color_code.background_color as u8) << 4;
        let forground_bits = color_code.foreground_color as u8;

        ColorCode8b(blink_bit | background_bits | forground_bits)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode8b,
}

impl ScreenChar {
    fn new(ascii_character: u8, color_code: ColorCode) -> Self {
        let color_code_byte = color_code.into();
        ScreenChar {
            ascii_character,
            color_code: color_code_byte,
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    fn write(&mut self, row: usize, col: usize, screen_char: ScreenChar) {
        self.chars[row][col].write(screen_char);
    }

    fn read(&self, row: usize, col: usize) -> ScreenChar {
        self.chars[row][col].read()
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new(color_code: ColorCode, buffer: &'static mut Buffer) -> Self {
        Writer {
            column_position: 0,
            color_code,
            buffer,
        }
    }

    fn write_free(&mut self, row: usize, col: usize, byte: u8) {
        self.buffer
            .write(row, col, ScreenChar::new(byte, self.color_code));
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.write_free(row, col, byte);
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let pxl = self.buffer.read(row, col);
                self.buffer.write(row - 1, col, pxl);
                if row == BUFFER_HEIGHT - 1 {
                    self.write_free(row, col, b' ');
                }
                self.column_position = 0;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.write_free(row, col, b' ');
            }
        }
    }

    #[allow(dead_code)]
    pub fn change_foreground_color(&mut self, foreground_color: Color4b) {
        self.color_code.change_foreground_color(foreground_color);
    }

    #[allow(dead_code)]
    pub fn change_background_color(&mut self, background_color: Color3b) {
        self.color_code.change_background_color(background_color);
    }

    #[allow(dead_code)]
    pub fn change_blink(&mut self, blink: bool) {
        self.color_code.change_blink(blink);
    }

    #[allow(dead_code)]
    pub fn change_column_position(&mut self, column_position: usize) {
        self.column_position = column_position;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// HELPER FUNCTIONS

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER!().write_fmt(args).unwrap();
}

// TESTS

#[cfg(test)]
mod tests {}
