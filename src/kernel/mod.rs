pub mod allocator;
pub mod framebuffer; // TODO : Change this whole mess when we have an allocator
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod x86;

/// Refers to the 8th byte of the GDT, which is the kernel code segment descriptor
pub const KERNEL_CS: u16 = 0x08;