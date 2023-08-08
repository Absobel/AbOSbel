#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod real_main;

use ab_os_bel::println;
use real_main::main;

use core::panic::PanicInfo;

// MAIN

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    ab_os_bel::init();
    main();

    #[allow(clippy::empty_loop)]
    loop {}
}

// TESTS

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() {
    ab_os_bel::init();
    test_main();
    loop {}
}
