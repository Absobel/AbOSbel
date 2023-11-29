#![allow(dead_code, unused_variables, unused_imports)]

use ab_os_bel::{
    dbg,
    framebuffer::{self, BUFFER, VGA_TEST_SLICE},
    io::{inb, outb, PS2_KEYBOARD_IN, PS2_KEYBOARD_OUT},
    memory::{FrameAllocator, FRAME_ALLOCATOR},
    println, serial_dbg, serial_print, MULTIBOOT2_INFO,
};

pub fn main() {
    let red = framebuffer::Color::new(255, 0, 0, 255); // RGB(255, 0, 0)
    let lavander = framebuffer::Color::new(191, 148, 228, 255); // RGB(191, 148, 228)
    let buffer = BUFFER.get().expect("Buffer required");

    buffer.lock().clear(lavander);

    println!("{}\n", VGA_TEST_SLICE);

    let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");

    // log_tag(boot_info.framebuffer_tag());
    // log_tag(boot_info.memory_map_tag());
    // log_tag(boot_info.elf_sections());
    // log_tag(boot_info.efi_memory_map_tag()); // None idk why
    // log_tag(boot_info.efi_bs_not_exited_tag());
    // log_tag(boot_info.efi_sdt64_tag());

    let mut frame_allocator = FRAME_ALLOCATOR
        .get()
        .expect("Frame allocator required")
        .lock();

    for i in 0.. {
        if frame_allocator.allocate_frame().is_none() {
            println!("allocated {} frames", i);
            break;
        }
    }

    println!("\nEnd of program.");
}

fn log_tag<T: core::fmt::Debug>(tag: T) {
    serial_dbg!(&tag);
    serial_print!("\n\n");
    dbg!(&tag);
    println!("\n\n");
}
