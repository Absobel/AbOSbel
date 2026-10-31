#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    clippy::missing_panics_doc
)]

use core::arch::asm;

use crate::{
    framebuffer::{self, BUFFER, CP437},
    io::Port,
    print, println,
};

pub fn main() {
    let red = framebuffer::Color::new(255, 0, 0, 255); // RGB(255, 0, 0)
    let lavander = framebuffer::Color::new(191, 148, 228, 255); // RGB(191, 148, 228)

    let buffer = BUFFER.get().expect("Buffer required");
    buffer.lock().clear(lavander);

    for c in CP437 {
        print!("{c}");
    }
    println!("\n");

    // whatever you want
    // let debug_exit_port = Port::new(0xf4);
    // unsafe { debug_exit_port.outl(0x10) }; // Exit QEMU with code 0x10

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
