#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(clippy::missing_safety_doc)]

// extern crate alloc;

mod kernel;
mod utils;

pub use kernel::*;
pub use utils::*;

use multiboot2::{BootInformation, BootInformationHeader, LoadError};
use spin::Once;

use crate::kernel::allocator::PhysicalAddress;

// INITIALIZATION

pub static MULTIBOOT2_INFO: Once<BootInformation<'static>> = Once::new();

unsafe extern "C" {
    static kernel_end: u8;
}
#[must_use]
pub fn kernel_end_addr() -> PhysicalAddress {
    PhysicalAddress(&raw const kernel_end as usize)
}

/// # Errors
/// This function will return an error if the information can't be parsed as multiboot2 information.
pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), LoadError> {
    let multiboot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }?;
    MULTIBOOT2_INFO.call_once(|| multiboot_info);
    Ok(())
}
