#![no_std]
#![no_main]

// extern crate alloc;

use ab_os_bel::{hlt_loop, init, println, real_main};

// INITIALIZATION

/// # Panics
/// This function will return an error if the information can't be parsed as multiboot2 information.
pub fn initialization(multiboot_info_addr: usize) {
    // TODO: Make it so that it gets printed on the screen after the framebuffer is initialized. Useless for this specifically though
    let _ = init::serial();
    println!("Initializing ab_os_bel...");

    init::gdt(); // Initialize the segmentation for interruption stacks
    init::interrupts(); // Initialize the interruptions and the handlers
    unsafe { init::multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    init::allocator(); // Initialize the physical memory allocator
    init::framebuffer();
    init::cpuid().unwrap_or_else(|e| {
        println!("{e}");
    });
    println!("ab_os_bel initialized.\n");
}

// MAIN

#[unsafe(no_mangle)]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    initialization(multiboot_info_addr);

    real_main::main();

    hlt_loop()
}
