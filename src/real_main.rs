use ab_os_bel::{vga::WRITER, memory::{self, FrameAllocator}};

use crate::println;

#[allow(dead_code)]
pub fn main(multiboot_info_addr: usize) {
    WRITER.lock().clear();
    println!("Hello World{}", "!");

    let mut frame_allocator = memory::frame_allocator(multiboot_info_addr);
    for i in 0.. {
        if frame_allocator.allocate_frame().is_none() {
            println!("Allocated {} frames", i);
            break;
        }
    }

    println!("It did not crash!");
}
