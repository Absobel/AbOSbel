use crate::arch::{self, port::Port};

/// Qemu exits with (code << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // (0x10 << 1) | 1 = 33
    Failed = 0x11,  // (0x11 << 1) | 1 = 35
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    const QEMU_EXIT_PORT: u16 = 0xf4;
    let debug_exit_port = Port::new(QEMU_EXIT_PORT);
    unsafe { debug_exit_port.outl(exit_code as u32) };
    arch::hlt_loop()
}
