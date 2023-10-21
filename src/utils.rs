use core::cell::OnceCell;

use multiboot2::{BootInformation, BootInformationHeader, MbiLoadError};

use crate::framebuffer::init_buffer;

pub static MULTIBOOT2_INFO: Multiboot2Info = Multiboot2Info(OnceCell::new());
crate::sync_wrapper!(Multiboot2Info, BootInformation<'static>);

// INITIALIZATION
pub fn init(multiboot_info_addr: usize) {
    crate::interrupts::init_idt(); // Initialize the interruptions and the handlers

    crate::gdt::init(); // Initialize the segmentation for interruption stacks
    unsafe { crate::interrupts::PICS.lock().initialize() }; // Initialize the PIC8259 for hardware interruption

    x86_64::instructions::interrupts::enable(); // Enable hardware interruptions

    unsafe { load_multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
    init_buffer();
}

// QEMU EXIT CODE

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0xf4);

    unsafe {
        port.write(exit_code as u32);
    }
}

// OTHER

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn load_multiboot(multiboot_info_addr: usize) -> Result<(), MbiLoadError> {
    MULTIBOOT2_INFO
        .set(BootInformation::load(
            multiboot_info_addr as *const BootInformationHeader,
        )?)
        .expect("Shouldn't be initialized");
    Ok(())
}

// MACRO

/* TODO : this feels bad idk why but it will do */
#[macro_export]
macro_rules! sync_wrapper {
    ($name:ident, $type:ty) => {
        pub struct $name(OnceCell<$type>);
        unsafe impl Sync for $name {}
        use core::ops::Deref;
        impl Deref for $name {
            type Target = OnceCell<$type>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
