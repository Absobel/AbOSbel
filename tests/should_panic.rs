// Purpose: Template for integration test that should panic.
// Current limitations: The test runner does not yet support multiple tests that panic.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use ab_os_bel::{exit_qemu, QemuExitCode, serial_print, serial_println};

// CORE

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    serial_print!("should_panic::should_fail...\t");
    should_fail();
    serial_println!("[test did not panic");
    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
