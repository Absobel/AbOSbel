// INIT INTERRUPTS

pub fn init() {
    crate::interrupts::init_idt(); // Initialize the interruptions and the handlers
    crate::gdt::init(); // Initialize the segmentation for interruption stacks
    unsafe { crate::interrupts::PICS.lock().initialize() }; // Initialize the PIC8259 for hardware interruptions
    x86_64::instructions::interrupts::enable(); // Enable hardware interruptions
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
