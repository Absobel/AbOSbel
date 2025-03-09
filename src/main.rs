#![no_std]
#![no_main]

mod real_main;

use ab_os_bel::{hlt_loop, serial_println};

// INITIALIZATION
pub fn init(multiboot_info_addr: usize) {
    serial_println!("Initializing ab_os_bel...");

    ab_os_bel::gdt::init(); // Initialize the segmentation for interruption stacks
    ab_os_bel::interrupts::init_idt(); // Initialize the interruptions and the handlers
    unsafe { ab_os_bel::load_multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    ab_os_bel::framebuffer::init_graphics();

    serial_println!("ab_os_bel initialized.");
}

// MAIN

#[unsafe(no_mangle)]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    init(multiboot_info_addr);

    real_main::main();

    hlt_loop()
}
