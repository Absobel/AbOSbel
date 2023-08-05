// Tests just after booting (also serves as a template for other integration tests)

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

// CORE

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

// TESTS

use ab_os_bel::println;

#[test_case]
fn no_panic_println() {
    // Tests if println still works right after boot
    println!("test_println output");
}
