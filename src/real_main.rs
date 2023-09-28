use core::arch::asm;

use ab_os_bel::{
    memory::{self, FrameAllocator},
    vga::WRITER, dbg, serial_dbg,
};

use crate::println;

#[allow(dead_code)]
pub fn main() {
    WRITER.lock().clear();

    println!(
        "Total memory: {:.2} MiB",
        memory::total_mem() as f64 / (1024.0 * 1024.0)
    );

    println!("Areas :");
    for area in unsafe {
        memory::MULTIBOOT2_INFO
            .as_ref()
            .expect("Multiboot info required")
            .memory_map_tag()
            .expect("MemoryMapTag required")
            .memory_areas()
            .iter()
            .filter(|area| area.typ() == multiboot2::MemoryAreaType::Available)
    } {
        println!(
            "    Start: 0x{:x}, Length: {:.2} MiB",
            area.start_address(),
            area.size() as f64 / (1024.0 * 1024.0)
        );
    }

    let mut frame_allocator = memory::frame_allocator();
    for i in 0.. {
        if frame_allocator.allocate_frame().is_none() {
            println!("Allocated {} frames", i);
            println!(
                "Or {:.2} MiB",
                (i * memory::PAGE_SIZE) as f64 / (1024.0 * 1024.0)
            );
            break;
        }
    }

    serial_dbg!(unsafe {
        memory::MULTIBOOT2_INFO
            .as_ref()
            .expect("Multiboot info required")
            .framebuffer_tag()
    });

    serial_dbg!(unsafe {
        memory::MULTIBOOT2_INFO
            .as_ref()
            .expect("Multiboot info required")
            .vbe_info_tag()
    });

    println!("\nEnd of program.");
}
