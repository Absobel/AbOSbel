#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(integer_atomics)]
#![allow(clippy::missing_safety_doc)]

// extern crate alloc;

mod arch;
mod boot;
mod console;
mod drivers;
mod memory;
pub mod real_main;
mod ring;

use core::{
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

pub use arch::hlt_loop;

use crate::drivers::{
    framebuffer::WRITER, keyboard::flush_key_events, serial::EmergencySerialWriter,
};

#[doc(hidden)]
pub use {
    console::_print,
    console::_serial_print,
    drivers::qemu::{QemuExitCode, exit_qemu},
};

pub mod init {
    pub use crate::arch::cpuid::init as cpuid;
    pub use crate::arch::gdt::init as gdt;
    pub use crate::arch::idt::init as idt;
    pub use crate::arch::msr::init as msr;
    pub use crate::boot::init as multiboot;
    pub use crate::drivers::framebuffer::init as framebuffer;
    pub use crate::drivers::pic::init as pic;
    pub use crate::drivers::serial::init as serial;
    pub use crate::memory::frame::init as allocator;
}

/// # Panics
/// This function will panic if the information can't be parsed as multiboot2 information.
pub fn initialization(multiboot_info_addr: usize) {
    let _ = init::serial();
    println!("Initializing ab_os_bel...");

    init::cpuid().unwrap_or_else(|e| {
        println!("{e}");
    });
    init::gdt(); // Initialize the segmentation for interruption stacks
    init::idt(); // Initialize the interruptions and the handlers
    init::pic();
    unsafe { init::multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    init::allocator(); // Initialize the physical memory allocator
    init::msr();
    init::framebuffer(2);

    arch::enable_interrupts();

    println!("ab_os_bel initialized.\n");
}

static PANICKING: AtomicBool = AtomicBool::new(false);

pub fn panic_report(info: &PanicInfo) {
    // Do not panic again if we are already panicking
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let _ = writeln!(EmergencySerialWriter, "{info}");
        // Only prints the panic message to the framebuffer if it is initialized and available
        if let Some(writer) = WRITER.get()
            && let Some(mut w) = writer.try_lock()
        {
            let _ = writeln!(w, "{info}");
        }
    }
}

pub fn flush_print_ring() {
    print!();
}

pub fn event_loop() -> ! {
    loop {
        flush_print_ring();
        flush_key_events();

        // TODO: check if interrupt just before hlt to not keep event queued until next interrupt
        arch::hlt();
    }
}
