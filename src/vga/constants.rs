use lazy_static::lazy_static;
use spin::Mutex;

use crate::vga::{Color3b, Color4b::*, ColorCode, Writer, Buffer};

pub const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
        ColorCode::new(Black, Color3b::LightGray, false),
        unsafe { &mut *(VGA_BUFFER as *mut Buffer) }
    ));
}


