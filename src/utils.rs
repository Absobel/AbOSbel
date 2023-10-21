use core::{cell::OnceCell, ops::Deref};

use multiboot2::{BootInformation, BootInformationHeader, MbiLoadError};

pub static MULTIBOOT2_INFO: Multiboot2Info = Multiboot2Info(OnceCell::new());

pub struct Multiboot2Info(OnceCell<BootInformation<'static>>);

/**
 * I Feel like the unsafe impl on the wrapper is justified as it is only intiialised once in a safe context
 * and reading it shouldn't be a problem so isok
 */
unsafe impl Sync for Multiboot2Info {}
impl Deref for Multiboot2Info {
    type Target = OnceCell<BootInformation<'static>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// INITIALIZATION
pub fn init(multiboot_info_addr: usize) {
    crate::interrupts::init_idt(); // Initialize the interruptions and the handlers

    crate::gdt::init(); // Initialize the segmentation for interruption stacks
    unsafe { crate::interrupts::PICS.lock().initialize() }; // Initialize the PIC8259 for hardware interruption

    x86_64::instructions::interrupts::enable(); // Enable hardware interruptions

    unsafe { load_multiboot(multiboot_info_addr).expect("Couldn't load multiboot") };
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
