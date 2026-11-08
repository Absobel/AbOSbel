#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(integer_atomics)]
#![allow(clippy::missing_safety_doc)]
// #![allow(dead_code)]

// extern crate alloc;

mod kernel;
pub mod real_main;
mod utils;

use core::arch::asm;

pub use utils::{QemuExitCode, exit_qemu, hlt_loop, panic_report};

#[doc(hidden)]
pub use {framebuffer::_print, io::_serial_print};

use kernel::{allocator, framebuffer, gdt, interrupts, io, pic, x86};
use multiboot2::{BootInformation, BootInformationHeader, LoadError};
use spin::Once;

pub mod init {
    pub use crate::allocator::init as allocator;
    pub use crate::framebuffer::init_graphics as framebuffer;
    pub use crate::gdt::init as gdt;
    pub use crate::interrupts::init_idt as interrupts;
    pub use crate::io::init_serial as serial;
    pub use crate::load_multiboot as multiboot;
    pub use crate::pic::init as pic;
    pub use crate::x86::init_cpuid as cpuid;
    pub use crate::x86::init_msr as msr;
}

// INITIALIZATION

pub(crate) static MULTIBOOT2_INFO: Once<BootInformation<'static>> = Once::new();

/// # Errors
/// This function will return an error if the information can't be parsed as multiboot2 information.
pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), LoadError> {
    let multiboot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }?;
    MULTIBOOT2_INFO.call_once(|| multiboot_info);
    Ok(())
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
    init::interrupts(); // Initialize the interruptions and the handlers
    init::pic();
    unsafe { init::multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    init::allocator(); // Initialize the physical memory allocator
    init::msr();
    init::framebuffer(2);
    println!("ab_os_bel initialized.\n");
}

pub fn event_loop() -> ! {
    loop {
        utils::flush_print_ring();
        pic::flush_key_events();

        // TODO: check if interrupt just before hlt to not keep event queued until next interrupt
        unsafe {
            asm!("hlt");
        }
    }
}
