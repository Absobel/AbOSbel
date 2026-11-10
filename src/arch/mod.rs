//! Module to talk with the CPU, never to a device

pub mod cpuid;
pub mod exceptions;
pub mod gdt;
pub mod idt;
pub mod msr;
pub mod port;

use core::arch::asm;

pub fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let r: R;
    unsafe {
        let rflags: usize;
        asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        let enabled = rflags & (1 << 9) != 0;
        if enabled {
            asm!("cli", options(nostack, preserves_flags));
        }
        r = f();
        if enabled {
            asm!("sti", options(nostack, preserves_flags));
        }
    }
    r
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}
