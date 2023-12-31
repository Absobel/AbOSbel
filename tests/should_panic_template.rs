// Purpose: Template for integration test that should panic.
// TODO Current limitations: The test runner does not yet support multiple tests that panic.

#![no_std]
#![no_main]

use ab_os_bel::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;

// CORE

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn main() -> ! {
    serial_print!("should_panic::template...\t");
    should_fail();
    serial_println!("[test did not panic]");
    ab_os_bel::hlt_loop()
}

fn should_fail() {
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    ab_os_bel::hlt_loop()
}
