use crate::vga_buffer::{Buffer, Color3b, Color4b, ColorCode, Writer};

use core::fmt::Write;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

pub fn main() {
    let mut writer = Writer::new(
        ColorCode::new(Color4b::Black, Color3b::Blue, false),
        unsafe { &mut *(VGA_BUFFER as *mut Buffer) },
    );

    writer.clear();


    write!(writer, "Hello\nWorld").unwrap();
}

// DEBUG
#[allow(dead_code)]
pub fn hello_world() {
    let hello = b"Hello World!";
    for (i, &byte) in hello.iter().enumerate() {
        unsafe {
            *VGA_BUFFER.offset(i as isize * 2) = byte;
            *VGA_BUFFER.offset(i as isize * 2 + 1) = 0xb;
        }
    }
}
