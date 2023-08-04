#![no_std]
#![no_main]

mod macros;
mod prelude;
mod real_main;
mod vga_buffer;

use real_main::*;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();
    panic!();
}
