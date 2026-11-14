#![no_std]
#![no_main]

use core::panic::PanicInfo;

use ab_os_bel::{event_loop, hlt_loop, initialization, panic_report, real_main};

// MAIN

#[unsafe(no_mangle)]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    initialization(multiboot_info_addr);

    real_main::main();

    event_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_report(info);
    hlt_loop()
}
