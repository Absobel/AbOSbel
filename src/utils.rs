use core::arch::asm;

use multiboot2::{BootInformation, BootInformationHeader, LoadError};

use crate::{framebuffer::init_graphics, serial_println};

crate::sync_wrapper!(MULTIBOOT2_INFO, Multiboot2Info, BootInformation<'static>);

// INITIALIZATION
pub fn init(multiboot_info_addr: usize) { 
    serial_println!("Initializing ab_os_bel...");   
    crate::interrupts::init_idt(); // Initialize the interruptions and the handlers
    // crate::gdt::init(); // Initialize the segmentation for interruption stacks
                        // x86_64::instructions::interrupts::enable(); // Enable hardware interruptions

    unsafe { load_multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    init_graphics();
}

// OTHER

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), LoadError> {
    unsafe {
        MULTIBOOT2_INFO
            .set(BootInformation::load(
                multiboot_info_addr as *const BootInformationHeader,
            )?)
            .expect("Shouldn't be initialized");
    }
    Ok(())
}

// MACRO

/* TODO : this feels bad idk why but it will do */
#[macro_export]
macro_rules! sync_wrapper {
    ($namestatic:ident, $namestruct:ident, $type:ty) => {
        pub struct $namestruct(core::cell::OnceCell<$type>);
        unsafe impl Sync for $namestruct {}
        impl core::ops::Deref for $namestruct {
            type Target = core::cell::OnceCell<$type>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        pub static $namestatic: $namestruct = $namestruct(core::cell::OnceCell::new());
    };
}
