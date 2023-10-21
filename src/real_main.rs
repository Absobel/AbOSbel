use ab_os_bel::{
    dbg,
    framebuffer::{self, VGA_TEST_SLICE},
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

    serial_dbg!(framebuffer_tag);
    serial_print!("\n\n");

    // serial_dbg!(unsafe {
    //     memory::MULTIBOOT2_INFO
    //         .as_ref()
    //         .expect("Multiboot info required")
    //         .efi_memory_map_tag()
    // });

    let mut buffer = framebuffer::Buffer::new(framebuffer_tag);
    let red = framebuffer::Color::new(255, 0, 0); // RGB(255, 0, 0)
    buffer.clear(red);

    println!("{}\n", VGA_TEST_SLICE);

    dbg!(framebuffer_tag);

    println!("\nEnd of program.");
}
