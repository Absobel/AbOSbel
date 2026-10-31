#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(clippy::missing_safety_doc)]

// extern crate alloc;

mod kernel;
pub mod real_main;
mod utils;

#[doc(hidden)]
pub use framebuffer::_print;
pub use utils::hlt_loop;

use kernel::{allocator, framebuffer, gdt, interrupts, io, x86};
use multiboot2::{BootInformation, BootInformationHeader, LoadError};
use spin::Once;

pub mod init {
    pub use crate::allocator::init as allocator;
    pub use crate::framebuffer::init_graphics as framebuffer;
    pub use crate::gdt::init as gdt;
    pub use crate::interrupts::init_idt as interrupts;
    pub use crate::io::init_serial as serial;
    pub use crate::load_multiboot as multiboot;
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
