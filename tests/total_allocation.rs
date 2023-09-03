// TODO : write a better test

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ab_os_bel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use ab_os_bel::{
    memory::{self, FrameAllocator},
    println,
};

// CORE

#[no_mangle]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    ab_os_bel::init(multiboot_info_addr);
    test_main();

    ab_os_bel::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ab_os_bel::test_panic_handler(info)
}

// TESTS

#[test_case]
fn no_panic_println() {
    let mut frame_allocator = memory::frame_allocator();
    let mut total_allocated_memory = 0;
    for nb_frames in 0.. {
        if frame_allocator.allocate_frame().is_none() {
            total_allocated_memory = nb_frames * memory::PAGE_SIZE;
            break;
        }
    }

    let total_mem = memory::total_mem();
    if (1..total_mem).contains(&total_allocated_memory) {
        println!("ok");
    } else {
        panic!(
            "failed: total_mem = {}, total_allocated_memory = {}",
            total_mem, total_allocated_memory
        );
    }
}
