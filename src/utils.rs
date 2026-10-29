use core::{
    arch::asm,
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{framebuffer::WRITER, io::EmergencySerialWriter};

static PANICKING: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Do not panic again if we are already panicking
    if !PANICKING.swap(true, Ordering::Relaxed) {
        let _ = writeln!(EmergencySerialWriter, "{info}");
        // Only prints the panic message to the framebuffer if it is initialized and available
        if let Some(writer) = WRITER.get()
            && let Some(mut w) = writer.try_lock()
        {
            let _ = w.write_fmt(format_args!("{info}"));
        }
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

#[must_use]
pub const fn align_up(addr: usize, align: usize) -> usize {
    usize::next_multiple_of(addr, align)
}

#[must_use]
pub const fn align_down_pow2(addr: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}
