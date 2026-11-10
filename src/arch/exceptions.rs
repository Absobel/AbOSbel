use crate::arch::idt::InterruptStackFrame;

macro_rules! exception {
    ($name:ident, $txt:literal) => {
        pub(super) extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            panic!("{} at address {:#x}", $txt, stack_frame.rip)
        }
    };
    ($name:ident, $txt:literal, $err:ident) => {
        pub(super) extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, $err: u64) {
            panic!(
                "{} at address {:#x} with error code: {:#x}",
                $txt, stack_frame.rip, $err
            )
        }
    };
}

use core::arch::asm;

use crate::println;

exception!(divide_error, "DIVISION ERROR");
exception!(debug, "DEBUG EXCEPTION");
exception!(nmi, "NON-MASKABLE INTERRUPT");
exception!(overflow, "OVERFLOW");
exception!(bound_range_exceeded, "BOUND RANGE EXCEEDED");
exception!(invalid_opcode, "INVALID OPCODE");
exception!(device_not_available, "DEVICE NOT AVAILABLE");
exception!(double_fault, "DOUBLE FAULT", error_code);
exception!(invalid_tss, "INVALID TSS", error_code);
exception!(segment_not_present, "SEGMENT NOT PRESENT", error_code);
exception!(stack_segment_fault, "STACK SEGMENT FAULT", error_code);
exception!(
    general_protection_fault,
    "GENERAL PROTECTION FAULT",
    error_code
);
exception!(x87_floating_point, "x87 FLOATING POINT EXCEPTION");
exception!(alignment_check, "ALIGNMENT CHECK", error_code);
exception!(machine_check, "MACHINE CHECK");
exception!(simd_floating_point, "SIMD FLOATING POINT EXCEPTION");
exception!(virtualization, "VIRTUALIZATION EXCEPTION");
exception!(
    control_protection,
    "CONTROL PROTECTION EXCEPTION",
    error_code
);

pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("BREAKPOINT at address {:#x}", stack_frame.rip);
}

// TODO: decode error code
pub(super) extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, error_code: u64) {
    let cr2: u64;
    unsafe { asm!("mov {}, cr2", out(reg) cr2) };
    panic!(
        "PAGE FAULT at address {:#x} with error code: {}\n\
                Trying to access address: {:#x}",
        stack_frame.rip, error_code, cr2
    );
}
