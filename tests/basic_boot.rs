#![no_std]
#![no_main]

use core::panic::PanicInfo;

use ab_os_bel::{QemuExitCode, exit_qemu, initialization, panic_report};

#[unsafe(no_mangle)]
extern "C" fn main(multiboot_info_addr: usize) -> ! {
    initialization(multiboot_info_addr);
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_report(info);
    exit_qemu(QemuExitCode::Failed);
}
