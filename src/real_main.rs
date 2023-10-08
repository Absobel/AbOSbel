use ab_os_bel::{
    framebuffer::{self, VGA_TEST_SLICE},
    memory, println, serial_dbg, serial_print, dbg,
};

#[allow(dead_code)]
pub fn main() {
    let framebuffer_tag = unsafe {
        memory::MULTIBOOT2_INFO
            .as_ref()
            .expect("Multiboot info required")
            .framebuffer_tag()
            .expect("Framebuffer required")
            .expect("Framebuffer required")
    };

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

    dbg!(framebuffer_tag);

    println!("\nEnd of program.");
}
