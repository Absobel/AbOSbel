// Tests just after booting (also serves as a template for other integration tests)

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

static mut MULTIBOOT_INFO_ADDR: usize = 0;

// CORE

#[no_mangle]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    unsafe {
        MULTIBOOT_INFO_ADDR = multiboot_info_addr;
    }
    
    test_main();

    ab_os_bel::hlt_loop()
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

// TESTS

use ab_os_bel::{println, memory::load_multiboot};

#[test_case]
fn no_panic_println() {
    // Tests if println still works right after boot
    println!("test_println output");
}

#[test_case]
fn multiboot_info() {
    // Tests if the multiboot info structure is correctly passed
    unsafe{ load_multiboot(MULTIBOOT_INFO_ADDR) }.unwrap(); // Should not panic
}