#![allow(dead_code, unused_variables)]

use ab_os_bel::{
    dbg,
    framebuffer::{self, BUFFER, VGA_TEST_SLICE},
    println, serial_dbg, serial_print, MULTIBOOT2_INFO,
};

pub fn main() {
    let red = framebuffer::Color::new(255, 0, 0); // RGB(255, 0, 0)
    BUFFER.get().expect("Buffer required").lock().clear(red);

    println!("{}\n", VGA_TEST_SLICE);

    let boot_info = MULTIBOOT2_INFO.get().expect("Multiboot info required");
    // log_tag(boot_info.framebuffer_tag());
    // // log_tag(boot_info.efi_memory_map_tag());
    // log_tag(boot_info.memory_map_tag());
    // log_tag(boot_info.elf_sections());

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());


    println!("\nEnd of program.");
}

fn log_tag<T: core::fmt::Debug>(tag: T) {
    serial_dbg!(&tag);
    serial_print!("\n\n");
    dbg!(&tag);
    println!("\n\n");
}
