#![no_std]
#![no_main]

mod real_main;
mod vga_buffer;

use real_main::*;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    //hello_world();
    main();
    panic!();
}
