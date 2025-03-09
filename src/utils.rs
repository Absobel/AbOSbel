use core::{arch::asm, panic::PanicInfo};

use crate::{framebuffer::WRITER, println, serial_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    if WRITER.get().is_some() {
        println!("{}", info);
    }
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

