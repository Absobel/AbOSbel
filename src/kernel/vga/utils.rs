use core::fmt::{self, Write};

use crate::vga::WRITER;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Deactivating interrupts to avoid deadlocks
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
