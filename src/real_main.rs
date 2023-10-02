use core::arch::asm;

use ab_os_bel::{
    memory::{self, FrameAllocator},
    vga::WRITER, serial_dbg,
};

use crate::println;

#[allow(dead_code)]
pub fn main() {
    // WRITER.lock().clear();

    // println!(
    //     "Total memory: {:.2} MiB",
    //     memory::total_mem() as f64 / (1024.0 * 1024.0)
    // );

    // println!("Areas :");
    // for area in unsafe {
    //     memory::MULTIBOOT2_INFO
    //         .as_ref()
    //         .expect("Multiboot info required")
    //         .memory_map_tag()
    //         .expect("MemoryMapTag required")
    //         .memory_areas()
    //         .iter()
    //         .filter(|area| area.typ() == multiboot2::MemoryAreaType::Available)
    // } {
    //     println!(
    //         "    Start: 0x{:x}, Length: {:.2} MiB",
    //         area.start_address(),
    //         area.size() as f64 / (1024.0 * 1024.0)
    //     );
    // }

    // let mut frame_allocator = memory::frame_allocator();
    // for i in 0.. {
    //     if frame_allocator.allocate_frame().is_none() {
    //         println!("Allocated {} frames", i);
    //         println!(
    //             "Or {:.2} MiB",
    //             (i * memory::PAGE_SIZE) as f64 / (1024.0 * 1024.0)
    //         );
    //         break;
    //     }
    // }

    serial_dbg!(unsafe {
        memory::MULTIBOOT2_INFO
            .as_ref()
            .expect("Multiboot info required")
            .framebuffer_tag()
    });

    // serial_dbg!(unsafe {
    //     memory::MULTIBOOT2_INFO
    //         .as_ref()
    //         .expect("Multiboot info required")
    //         .efi_memory_map_tag()
    // });


    unsafe {
        // Assuming the framebuffer_tag is the one you provided
        let framebuffer_tag = memory::MULTIBOOT2_INFO.as_ref().expect("Multiboot info required").framebuffer_tag().expect("Framebuffer tag required").expect("Framebuffer tag required");
        let framebuffer_addr = framebuffer_tag.address() as *mut u8;
        let pitch = framebuffer_tag.pitch() as usize;
        let width = framebuffer_tag.width() as usize;
        let height = framebuffer_tag.height() as usize;
        let bpp = framebuffer_tag.bpp() as usize / 8; // 3 bytes
    
        for y in 0..height {
            for x in 0..width {
                let offset = y * pitch + x * bpp;
                let pixel_addr = framebuffer_addr.add(offset);
    
                if x < width / 2 {
                    // Make it red: RGB(255, 0, 0)
                    *pixel_addr.add(2) = 255; // R at position 16 (2 bytes offset)
                    *pixel_addr.add(1) = 0;   // G at position 8  (1 byte offset)
                    *pixel_addr = 0;          // B at position 0  (0 byte offset)
                } else {
                    // Make it purple: RGB(128, 0, 128)
                    *pixel_addr.add(2) = 128; // R
                    *pixel_addr.add(1) = 0;   // G
                    *pixel_addr = 128;        // B
                }
            }
        }
    }
        
    

    // println!("\nEnd of program.");
}
