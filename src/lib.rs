#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(clippy::missing_safety_doc)]

mod kernel;
mod utils;

pub use kernel::*;
pub use utils::*;

use lazy_static::lazy_static;
use multiboot2::{BootInformation, BootInformationHeader, LoadError};
use spin::Once;

// INITIALIZATION

lazy_static! {
    pub static ref MULTIBOOT2_INFO: Once<BootInformation<'static>> = Once::new();
}

pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), LoadError> {
    let multiboot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }?;
    MULTIBOOT2_INFO.call_once(|| multiboot_info);
    Ok(())
}
