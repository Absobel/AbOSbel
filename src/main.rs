#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod real_main;

use ab_os_bel::println;
use real_main::*;

// PANIC

use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

// START

#[no_mangle]
pub extern "C" fn _start() -> ! {
    ab_os_bel::init();

    #[cfg(not(test))]
    main();

    //#[cfg(test)]
    //test_main();

    #[allow(clippy::empty_loop)]
    loop {}
}
