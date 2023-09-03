#![no_std]
#![no_main]

use ab_os_bel::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;

// CORE

#[no_mangle]
pub extern "C" fn main(multiboot_info_addr: usize) -> ! {
    ab_os_bel::init(multiboot_info_addr);

    serial_print!("page_fault::should_fail...\t");
    should_fail();
    serial_println!("[test did not panic]");

    ab_os_bel::hlt_loop()
}

fn should_fail() {
    let ptr = 0xdeadbeaf as *mut u32;
    unsafe {
        *ptr = 42;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    ab_os_bel::hlt_loop()
}
