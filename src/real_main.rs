#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    clippy::missing_panics_doc
)]

use core::arch::asm;

use crate::{
    framebuffer::{self, BUFFER, CP437}, io::Port, kernel::framebuffer::WRITER, print, println,
};

pub fn main() {
    // let red = framebuffer::Color::new(255, 0, 0, 255);
    let lavender = framebuffer::Color::new(191, 148, 228, 255);

    let buffer = BUFFER.get().expect("Buffer required");
    buffer.lock().clear(lavender);
    // let writer = WRITER.get().expect("Writer required");
    // writer.lock().change_background_color(red);
    // writer.lock().change_foreground_color(framebuffer::Color::new(255, 0, 0, 255));

    for c in CP437 {
        print!("{c}");
    }
    println!("\n");

    // whatever you want

    println!("\nEnd of program.");
}

fn division_by_zero() {
    unsafe {
        asm!(
            "xor edx, edx",
            "xor ecx, ecx",
            "div ecx",
            out("eax") _, out("edx") _, out("ecx") _,
            options(nomem, nostack),
        );
    }
}
