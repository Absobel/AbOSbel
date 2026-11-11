// INITIALIZATION

use multiboot2::{BootInformation, LoadError};
use spin::Once;

use crate::memory::VirtualAddress;

pub static MULTIBOOT2_INFO: Once<BootInformation<'static>> = Once::new();

/// # Errors
/// This function will return an error if the information can't be parsed as multiboot2 information.
pub unsafe fn init(multiboot_info_addr: VirtualAddress) -> Result<(), LoadError> {
    let multiboot_info = unsafe { BootInformation::load(multiboot_info_addr.as_ptr()) }?;
    MULTIBOOT2_INFO.call_once(|| multiboot_info);
    Ok(())
}
