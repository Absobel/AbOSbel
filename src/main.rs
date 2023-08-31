#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod real_main;

use ab_os_bel::{hlt_loop, println};

use core::panic::PanicInfo;

// MAIN

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

#[no_mangle]
pub extern "C" fn main(_multiboot_info_addr: usize) -> ! {
    ab_os_bel::init();

    #[cfg(not(test))]
    real_main::main(_multiboot_info_addr);
    
    #[cfg(test)]
    test_main();
    
    hlt_loop()
}

// TESTS

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}