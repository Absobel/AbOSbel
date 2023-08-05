// Purpose: Template for integration test that should panic.
// Current limitations: The test runner does not yet support multiple tests that panic.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use ab_os_bel::{exit_qemu, QemuExitCode, serial_print, serial_println};

// CORE

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

// TO RUN TESTS

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} test", tests.len());
    for test in tests {
        test();
        serial_println!("[test did not panic]");
        exit_qemu(QemuExitCode::Failed);
    }
    unreachable!();
}

// TESTS

#[test_case]
fn should_fail() {
    serial_print!("should_panic::should_fail... ");
    assert_eq!(0, 1);
}