use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::vga_buffer::*;
use crate::vga_buffer::Color4b::*;

pub const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
        ColorCode::new(Black, Color3b::LightGray, false),
        unsafe { &mut *(VGA_BUFFER as *mut Buffer) }
    ));
}

#[macro_export]
macro_rules! WRITER {
    () => {
        WRITER.lock()
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER!().write_fmt(args).unwrap();
}