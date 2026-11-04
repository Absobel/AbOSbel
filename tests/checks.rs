#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

use ab_os_bel::{QemuExitCode, exit_qemu, initialization, panic_report, println};

#[unsafe(no_mangle)]
extern "C" fn main(multiboot_info_addr: usize) -> ! {
    initialization(multiboot_info_addr);

    println!("-- Testing breakpoint interrupt --");
    unsafe { asm!("int3") };

    println!("-- Testing framebuffer limits --");
    for i in 0..200 {
        println!("Line {i}");
    }

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_report(info);
    exit_qemu(QemuExitCode::Failed);
}
