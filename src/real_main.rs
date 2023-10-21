use ab_os_bel::{
    dbg,
    framebuffer::{self, BUFFER, VGA_TEST_SLICE},
    println, serial_dbg, serial_print, MULTIBOOT2_INFO,
};

#[allow(dead_code)]
pub fn main() {
    let framebuffer_tag = MULTIBOOT2_INFO
        .get()
        .expect("Multiboot info required")
        .framebuffer_tag()
        .expect("Framebuffer required")
        .expect("Framebuffer required");

    let red = framebuffer::Color::new(255, 0, 0); // RGB(255, 0, 0)
    BUFFER.get().expect("Buffer required").lock().clear(red);

    println!("{}\n", VGA_TEST_SLICE);

    serial_dbg!(framebuffer_tag);
    serial_print!("\n\n");
    dbg!(framebuffer_tag);
    println!("\n\n");

    // let efi_memory_map_tag = MULTIBOOT2_INFO
    //     .get()
    //     .expect("Multiboot info required")
    //     .efi_memory_map_tag();

    // serial_dbg!(efi_memory_map_tag);
    // serial_print!("\n\n");
    // dbg!(efi_memory_map_tag);
    // println!("\n\n");

    println!("\nEnd of program.");
}
