#![no_std]
#![no_main]

mod real_main;

use ab_os_bel::hlt_loop;

use core::panic::PanicInfo;

// MAIN

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use ab_os_bel::{framebuffer::WRITER, println, serial_println};

    serial_println!("{}", info);
    if WRITER.get().is_some() {
        println!("{}", info);
    }
    hlt_loop()
}

#[unsafe(no_mangle)]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    ab_os_bel::init(multiboot_info_addr);

    real_main::main();

    hlt_loop()
}
