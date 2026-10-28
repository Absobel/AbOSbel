use core::{arch::asm, panic::PanicInfo};

use crate::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[must_use]
pub const fn align_up(addr: usize, align: usize) -> usize {
    usize::next_multiple_of(addr, align)
}

#[must_use]
pub const fn align_down_pow2(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}
