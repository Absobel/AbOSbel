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

use core::{arch::global_asm, panic::PanicInfo};
use multiboot2::BootInformationHeader;

// MAIN

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

#[no_mangle]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    ab_os_bel::init();

    // TODO: Extract this to a function elsewhere.
    let boot_info = unsafe {
        multiboot2::BootInformation::load(multiboot_info_addr as *const BootInformationHeader)
            .unwrap()
    };

    let elf_sections = boot_info.elf_sections().unwrap();

    let kernel_start = elf_sections.clone().min_by_key(|s| s.start_address()).unwrap().start_address();
    let kernel_end = elf_sections.map(|s| s.end_address()).max().unwrap();
    let kernel_size = kernel_end - kernel_start;

    let multiboot_start = multiboot_info_addr;
    let multiboot_end = multiboot_start + boot_info.total_size();

    println!("Kernel start: {:#x}, Kernel end: {:#x}, Kernel size: {}", kernel_start, kernel_end, kernel_size);
    println!("Multiboot start: {:#x}, Multiboot end: {:#x}", multiboot_start, multiboot_end);

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
