#![allow(dead_code, unused_variables, unused_imports)]

use core::arch::asm;

use ab_os_bel::{
    MULTIBOOT2_INFO,
    allocator::FRAME_ALLOCATOR,
    dbg,
    framebuffer::{self, BUFFER, VGA_TEST_SLICE},
    kernel_end_addr, println, serial_dbg, serial_print, serial_println,
};

pub fn main() {
    let red = framebuffer::Color::new(255, 0, 0, 255); // RGB(255, 0, 0)
    let lavander = framebuffer::Color::new(191, 148, 228, 255); // RGB(191, 148, 228)

    let buffer = BUFFER.get().expect("Buffer required");
    buffer.lock().clear(lavander);

    println!("{}\n", VGA_TEST_SLICE);

    // let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");
    // log_tag(boot_info.framebuffer_tag());
    // log_tag(boot_info.memory_map_tag());
    // log_tag(boot_info.efi_memory_map_tag());

    // division_by_zero();

    println!("\nEnd of program.");
}

fn log_tag<T: core::fmt::Debug>(tag: T) {
    serial_dbg!(&tag);
    serial_print!("\n\n");
    dbg!(&tag);
    println!("\n\n");
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
