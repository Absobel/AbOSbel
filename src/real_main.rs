#![allow(dead_code, unused_variables, unused_imports)]

use core::arch::asm;

use ab_os_bel::{
    MULTIBOOT2_INFO,
    allocator::FRAME_ALLOCATOR,
    dbg,
    framebuffer::{self, BUFFER, CP437},
    kernel_end_addr, print, println, serial_dbg, serial_print, serial_println,
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
