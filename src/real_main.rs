#![allow(dead_code, unused_variables)]

use ab_os_bel::{
    dbg,
    framebuffer::{self, BUFFER, VGA_TEST_SLICE},
    println, serial_dbg, serial_print, MULTIBOOT2_INFO,
};
use x86_64::registers::control::Cr0;

pub fn main() {
    let red = framebuffer::Color::new(255, 0, 0, 255); // RGB(255, 0, 0)
    let lavander = framebuffer::Color::new(191, 148, 228, 255); // RGB(191, 148, 228)
    let buffer = BUFFER.get().expect("Buffer required");

    buffer.lock().clear(lavander);

    println!("{}\n", VGA_TEST_SLICE);

    let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");
    // log_tag(boot_info.framebuffer_tag());
    // log_tag(boot_info.efi_memory_map_tag());
    // log_tag(boot_info.memory_map_tag());
    // log_tag(boot_info.elf_sections());

    let smth = Cr0::read();
    dbg!(smth);

    println!();
    x86_64::instructions::interrupts::int3();

    buffer.lock().clear(lavander);

    println!("\nEnd of program.");
}

fn log_tag<T: core::fmt::Debug>(tag: T) {
    serial_dbg!(&tag);
    serial_print!("\n\n");
    dbg!(&tag);
    println!("\n\n");
}
