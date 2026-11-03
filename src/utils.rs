use core::{
    arch::asm,
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{framebuffer::WRITER, io::EmergencySerialWriter, kernel::io::Port};

static PANICKING: AtomicBool = AtomicBool::new(false);

pub fn panic_report(info: &PanicInfo) {
    // Do not panic again if we are already panicking
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let _ = writeln!(EmergencySerialWriter, "{info}");
        // Only prints the panic message to the framebuffer if it is initialized and available
        if let Some(writer) = WRITER.get()
            && let Some(mut w) = writer.try_lock()
        {
            let _ = w.write_fmt(format_args!("{info}"));
        }
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

/// Qemu exits with (code << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // (0x10 << 1) | 1 = 33
    Failed = 0x11, // (0x11 << 1) | 1 = 35
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    const QEMU_EXIT_PORT: u16 = 0xf4;
    let debug_exit_port = Port::new(QEMU_EXIT_PORT);
    unsafe { debug_exit_port.outl(exit_code as u32) };
    hlt_loop()
}

#[must_use]
pub const fn align_up(addr: usize, align: usize) -> usize {
    usize::next_multiple_of(addr, align)
}

#[must_use]
pub const fn align_down_pow2(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}
