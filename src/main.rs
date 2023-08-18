#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]


// BOOTSTRAP

global_asm!(include_str!("preliminary/multiboot.s"), options(raw));
global_asm!(include_str!("preliminary/boot.s"), options(raw));

// ACTUAL START

mod real_main;

use ab_os_bel::{hlt_loop, println};

use core::{panic::PanicInfo, arch::global_asm};

// MAIN

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    ab_os_bel::init();
    real_main::main();
    hlt_loop()
}

// TESTS

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn main() {
    ab_os_bel::init();
    test_main();
    hlt_loop()
}
