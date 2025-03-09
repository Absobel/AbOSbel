use core::{arch::asm, panic::PanicInfo};

use lazy_static::lazy_static;
use multiboot2::{BootInformation, BootInformationHeader, LoadError};
use spin::Once;

use crate::{framebuffer::WRITER, println, serial_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    if WRITER.get().is_some() {
        println!("{}", info);
    }
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

lazy_static! {
    pub static ref MULTIBOOT2_INFO: Once<BootInformation<'static>> = Once::new();
}

pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), LoadError> {
    let multiboot_info =
        unsafe { BootInformation::load(multiboot_info_addr as *const BootInformationHeader) }?;
    MULTIBOOT2_INFO.call_once(|| multiboot_info);
    Ok(())
}
