#![no_std]
#![no_main]

// extern crate alloc;

use core::panic::PanicInfo;

use ab_os_bel::{hlt_loop, initialization, panic_report, real_main};

// MAIN

#[unsafe(no_mangle)]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    initialization(multiboot_info_addr);

    real_main::main();

    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_report(info);
    hlt_loop()
}
